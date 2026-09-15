//! Loop control surface — start/stop + nudge policy mirror for Orchestrator.

use crate::herdr::run_herdr_ok;
use crate::registry::{NavAction, PluginCtx, SubPlugin};
use crate::session::{LoopState, Session, TeamSettings};
use crate::ui::draw_select_list;
use crate::ui::ScrollList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub struct LoopPlugin {
    list: ScrollList,
    panel: String,
}

impl LoopPlugin {
    pub fn new() -> Self {
        Self {
            list: ScrollList::new(vec![
                "Show loop status + nudge policy".into(),
                "Start loop (after Master confirm preferred)".into(),
                "Pause loop".into(),
                "Stop loop".into(),
                "Send nudge-all reminder to Orchestrator".into(),
            ]),
            panel: String::new(),
        }
    }

    fn refresh(&mut self, ctx: &mut PluginCtx) {
        let session = Session::load(&ctx.paths).unwrap_or_default();
        let settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
        self.panel = format!(
            "loop_state: {}\ncwd: {}\nproject: {}\n\nNudge policy (from Config):\n  nudge_idle_secs: {}\n  stuck_timeout_secs: {}\n  max_inflight_tickets: {}\n  auto_tag_agent_on_cards: {}\n\nOrchestrator should:\n  - assign ready tickets up to max_inflight\n  - nudge agents idle longer than nudge_idle_secs\n  - escalate tickets stuck longer than stuck_timeout_secs\n  - keep kanban tags agent/order/depends accurate\n",
            session.loop_state.as_str(),
            if session.working_dir.is_empty() {
                "(unset)"
            } else {
                &session.working_dir
            },
            if session.project_name.is_empty() {
                "(unnamed)"
            } else {
                &session.project_name
            },
            settings.nudge_idle_secs,
            settings.stuck_timeout_secs,
            settings.max_inflight_tickets,
            settings.auto_tag_agent_on_cards,
        );
    }

    fn set_state(&mut self, ctx: &mut PluginCtx, state: LoopState) {
        let mut session = Session::load(&ctx.paths).unwrap_or_default();
        session.loop_state = state.clone();
        if let Err(e) = session.save(&ctx.paths) {
            ctx.set_error(format!("save: {e}"));
            return;
        }
        let settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
        let msg = match state {
            LoopState::Running => format!(
                "LOOP START. nudge_idle_secs={} stuck_timeout_secs={} max_inflight={}. Keep board live; nudge stuck agents.",
                settings.nudge_idle_secs, settings.stuck_timeout_secs, settings.max_inflight_tickets
            ),
            LoopState::Paused => {
                "LOOP PAUSE. Hold new assignments; allow in-flight tickets to finish.".into()
            }
            LoopState::Stopped => {
                "LOOP STOP. No nudges. Wait for Master confirmation before resuming.".into()
            }
        };
        let _ = run_herdr_ok(&["agent", "prompt", &msg]);
        self.refresh(ctx);
        self.panel = format!("{msg}\n\n{}", self.panel);
        ctx.set_status(format!("loop {}", state.as_str()));
    }
}

impl SubPlugin for LoopPlugin {
    fn id(&self) -> &'static str {
        "loop"
    }
    fn title(&self) -> &'static str {
        "Agent Loop"
    }
    fn description(&self) -> &'static str {
        "Start/pause/stop the Orchestrator work loop and mirror nudge/stuck timeouts from Config."
    }

    fn on_enter(&mut self, ctx: &mut PluginCtx) {
        self.refresh(ctx);
        ctx.set_status("Enter select · Esc back");
    }

    fn handle(&mut self, ctx: &mut PluginCtx, key: KeyEvent) -> NavAction {
        if self.list.handle_nav(key) {
            return NavAction::None;
        }
        match key.code {
            KeyCode::Esc => NavAction::Back,
            KeyCode::Enter => {
                match self.list.selected() {
                    Some(0) => {
                        self.refresh(ctx);
                        ctx.set_status("status refreshed");
                    }
                    Some(1) => self.set_state(ctx, LoopState::Running),
                    Some(2) => self.set_state(ctx, LoopState::Paused),
                    Some(3) => self.set_state(ctx, LoopState::Stopped),
                    Some(4) => {
                        let settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
                        let msg = format!(
                            "NUDGE CHECK: scan for idle agents (>{}s) and stuck tickets (>{}s). Re-prompt each with their next board card.",
                            settings.nudge_idle_secs, settings.stuck_timeout_secs
                        );
                        match run_herdr_ok(&["agent", "prompt", &msg]) {
                            Ok(_) => {
                                self.panel = format!("{msg}\n\n(sent)");
                                ctx.set_status("nudge reminder sent");
                            }
                            Err(e) => {
                                self.panel = format!("{msg}\n\nherdr prompt failed: {e}");
                                ctx.set_error("prompt failed");
                            }
                        }
                    }
                    _ => {}
                }
                NavAction::None
            }
            _ => NavAction::None,
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &PluginCtx) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);
        draw_select_list(
            frame,
            chunks[0],
            "Agent Loop",
            &self.list.items,
            self.list.selected(),
        );
        frame.render_widget(
            Paragraph::new(self.panel.as_str())
                .wrap(Wrap { trim: false })
                .block(Block::default().title("Policy / status").borders(Borders::ALL)),
            chunks[1],
        );
    }
}
