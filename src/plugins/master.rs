//! Master — human gate, session cwd, confirm-before-dispatch.

use crate::herdr::run_herdr_ok;
use crate::registry::{NavAction, PluginCtx, SubPlugin};
use crate::roles::RoleId;
use crate::session::{expand_home, LoopState, Session, TeamSettings};
use crate::ui::draw_select_list;
use crate::ui::{FormField, FormResult, FormState};
use crate::ui::ScrollList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

enum Mode {
    Menu,
    QueueForm,
    EnvForm,
}

pub struct MasterPlugin {
    list: ScrollList,
    session: Session,
    settings: TeamSettings,
    mode: Mode,
    form: Option<FormState>,
    panel: String,
}

impl MasterPlugin {
    pub fn new() -> Self {
        Self {
            list: ScrollList::new(vec![
                "Show confirmation block".into(),
                "Queue human prompt(s)".into(),
                "Confirm & dispatch (yes)".into(),
                "Reject pending (no)".into(),
                "Set working environment".into(),
                "Start agent loop".into(),
                "Stop / pause agent loop".into(),
                "Relay recommendation only".into(),
                "Refresh session from disk".into(),
            ]),
            session: Session::default(),
            settings: TeamSettings::default(),
            mode: Mode::Menu,
            form: None,
            panel: String::new(),
        }
    }

    fn reload(&mut self, ctx: &mut PluginCtx) {
        match Session::load(&ctx.paths) {
            Ok(s) => self.session = s,
            Err(e) => ctx.set_error(format!("session: {e}")),
        }
        match TeamSettings::load(&ctx.paths) {
            Ok(s) => self.settings = s,
            Err(e) => ctx.set_error(format!("settings: {e}")),
        }
        if self.session.agent_kind.is_empty() {
            self.session.agent_kind = self.settings.default_agent_kind.clone();
        }
        self.panel = self.session.confirmation_block();
        if let Some(hist) = self.session.history.last() {
            self.panel.push_str("\n\nLast history:\n");
            self.panel.push_str(hist);
        }
    }

    fn persist(&mut self, ctx: &mut PluginCtx) {
        if let Err(e) = self.session.save(&ctx.paths) {
            ctx.set_error(format!("save session: {e}"));
        }
    }

    fn dispatch_confirmed(&mut self, ctx: &mut PluginCtx) {
        if self.settings.master_always_confirm && self.session.pending.is_none() {
            ctx.set_error("nothing pending — queue prompts first, then confirm");
            self.panel = self.session.confirmation_block();
            return;
        }
        let Some(directive) = self.session.confirm_pending() else {
            ctx.set_error("no pending directive");
            return;
        };

        let prompt = format!(
            "{}\n\n---\nMaster CONFIRMED directive for Orchestrator.\nWorking dir: {}\nProject: {}\nLoop: {}\n\nDo this now:\n{}\n\nUpdate the kanban board with Todo cards tagged by agent role and order.\nNudge idle agents. Report status back to Master when the board changes.\n",
            self.settings
                .role(RoleId::Orchestrator)
                .map(|r| r.master_prompt.as_str())
                .unwrap_or("You are Orchestrator."),
            self.session.working_dir,
            self.session.project_name,
            self.session.loop_state.as_str(),
            directive
                .prompts
                .iter()
                .enumerate()
                .map(|(i, p)| format!("{}. {p}", i + 1))
                .collect::<Vec<_>>()
                .join("\n")
        );

        match run_herdr_ok(&["agent", "prompt", &prompt]) {
            Ok(_) => {
                self.session.loop_state = LoopState::Running;
                self.persist(ctx);
                self.panel = format!(
                    "Dispatched to Orchestrator.\n\n{}",
                    self.session.confirmation_block()
                );
                ctx.set_status("confirmed + dispatched");
            }
            Err(e) => {
                // Keep confirmed locally even if herdr prompt fails (offline / no agent).
                self.persist(ctx);
                self.panel = format!(
                    "Saved confirmation locally, but herdr agent prompt failed:\n{e}\n\nPaste this into the Orchestrator pane manually:\n\n{prompt}"
                );
                ctx.set_error("dispatch prompt failed — see panel");
            }
        }
    }

    fn set_loop(&mut self, ctx: &mut PluginCtx, state: LoopState) {
        self.session.loop_state = state.clone();
        self.persist(ctx);
        let msg = match state {
            LoopState::Running => {
                "Master says: START the loop. Assign ready tickets, nudge idle agents, keep the board live."
            }
            LoopState::Paused | LoopState::Stopped => {
                "Master says: STOP/PAUSE the loop. Finish in-flight work only; no new nudges."
            }
        };
        let _ = run_herdr_ok(&["agent", "prompt", msg]);
        self.panel = format!("{msg}\n\n{}", self.session.confirmation_block());
        ctx.set_status(format!("loop {}", state.as_str()));
    }
}

impl SubPlugin for MasterPlugin {
    fn id(&self) -> &'static str {
        "master"
    }
    fn title(&self) -> &'static str {
        "Master"
    }
    fn description(&self) -> &'static str {
        "Human interface + ALWAYS-confirm gate. Queue prompts, set cwd/project, confirm before any Dev Team dispatch, start/stop the loop."
    }

    fn on_enter(&mut self, ctx: &mut PluginCtx) {
        self.mode = Mode::Menu;
        self.reload(ctx);
        ctx.set_status("y confirm · n reject · Enter action · Esc back");
    }

    fn handle(&mut self, ctx: &mut PluginCtx, key: KeyEvent) -> NavAction {
        match self.mode {
            Mode::QueueForm | Mode::EnvForm => {
                let Some(form) = self.form.as_mut() else {
                    self.mode = Mode::Menu;
                    return NavAction::None;
                };
                match form.handle(key) {
                    FormResult::Cancel => {
                        self.form = None;
                        self.mode = Mode::Menu;
                    }
                    FormResult::Submit => {
                        let vals = form.values();
                        match self.mode {
                            Mode::QueueForm => {
                                let raw = vals.first().cloned().unwrap_or_default();
                                let prompts: Vec<String> = raw
                                    .split(|c| c == ';' || c == '\n')
                                    .map(str::trim)
                                    .filter(|s| !s.is_empty())
                                    .map(str::to_string)
                                    .collect();
                                let notes = vals.get(1).cloned().unwrap_or_default();
                                if prompts.is_empty() {
                                    ctx.set_error("enter at least one prompt (separate with ;)");
                                } else {
                                    self.session.queue_prompts(prompts, notes);
                                    self.persist(ctx);
                                    self.panel = self.session.confirmation_block();
                                    ctx.set_status("queued — review confirmation, then Confirm");
                                }
                            }
                            Mode::EnvForm => {
                                self.session.project_name =
                                    vals.first().cloned().unwrap_or_default();
                                self.session.working_dir =
                                    expand_home(&vals.get(1).cloned().unwrap_or_default());
                                self.session.git_remote =
                                    vals.get(2).cloned().unwrap_or_default();
                                self.session.agent_kind = vals
                                    .get(3)
                                    .cloned()
                                    .filter(|s| !s.trim().is_empty())
                                    .unwrap_or_else(|| self.settings.default_agent_kind.clone());
                                self.persist(ctx);
                                self.panel = self.session.confirmation_block();
                                ctx.set_status("environment updated");
                            }
                            Mode::Menu => {}
                        }
                        self.form = None;
                        self.mode = Mode::Menu;
                    }
                    FormResult::Continue => {}
                }
                NavAction::None
            }
            Mode::Menu => {
                // Global confirm / reject shortcuts.
                match key.code {
                    KeyCode::Char('y') => {
                        self.dispatch_confirmed(ctx);
                        return NavAction::None;
                    }
                    KeyCode::Char('n') => {
                        self.session.reject_pending();
                        self.persist(ctx);
                        self.panel = format!(
                            "Rejected pending directive.\n\n{}",
                            self.session.confirmation_block()
                        );
                        ctx.set_status("rejected");
                        return NavAction::None;
                    }
                    _ => {}
                }

                if self.list.handle_nav(key) {
                    return NavAction::None;
                }
                match key.code {
                    KeyCode::Esc => NavAction::Back,
                    KeyCode::Enter => {
                        match self.list.selected() {
                            Some(0) => {
                                self.panel = self.session.confirmation_block();
                                ctx.set_status("confirmation block");
                            }
                            Some(1) => {
                                self.form = Some(FormState::new(
                                    "Queue prompts (separate multiple with ;)",
                                    vec![
                                        FormField::new("prompts"),
                                        FormField::new("notes"),
                                    ],
                                ));
                                self.mode = Mode::QueueForm;
                            }
                            Some(2) => self.dispatch_confirmed(ctx),
                            Some(3) => {
                                self.session.reject_pending();
                                self.persist(ctx);
                                self.panel = self.session.confirmation_block();
                                ctx.set_status("rejected");
                            }
                            Some(4) => {
                                self.form = Some(FormState::new(
                                    "Working environment",
                                    vec![
                                        FormField::new("project_name")
                                            .with_value(&self.session.project_name),
                                        FormField::new("working_dir")
                                            .with_value(&self.session.working_dir),
                                        FormField::new("git_remote")
                                            .with_value(&self.session.git_remote),
                                        FormField::new("agent_kind")
                                            .with_value(&self.session.agent_kind),
                                    ],
                                ));
                                self.mode = Mode::EnvForm;
                            }
                            Some(5) => self.set_loop(ctx, LoopState::Running),
                            Some(6) => self.set_loop(ctx, LoopState::Stopped),
                            Some(7) => {
                                self.form = Some(FormState::new(
                                    "Recommendation (still requires Confirm)",
                                    vec![
                                        FormField::new("prompts"),
                                        FormField::new("notes")
                                            .with_value("recommendation"),
                                    ],
                                ));
                                self.mode = Mode::QueueForm;
                            }
                            Some(8) => {
                                self.reload(ctx);
                                ctx.set_status("reloaded");
                            }
                            _ => {}
                        }
                        NavAction::None
                    }
                    _ => NavAction::None,
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &PluginCtx) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area);
        draw_select_list(
            frame,
            chunks[0],
            "Master · y=confirm n=reject",
            &self.list.items,
            self.list.selected(),
        );
        frame.render_widget(
            Paragraph::new(self.panel.as_str())
                .wrap(Wrap { trim: false })
                .block(
                    Block::default()
                        .title("Confirmation / dispatch")
                        .borders(Borders::ALL),
                ),
            chunks[1],
        );
        if let Some(form) = &self.form {
            form.draw(frame, area);
        }
    }
}
