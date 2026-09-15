//! Deep guided configuration menu for Dev Team.

use crate::registry::{NavAction, PluginCtx, SubPlugin};
use crate::roles::RoleId;
use crate::session::TeamSettings;
use crate::ui::draw_select_list;
use crate::ui::{FormField, FormResult, FormState};
use crate::ui::ScrollList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Clone, Copy)]
enum Section {
    Root,
    Session,
    Loop,
    Board,
    Git,
    Roles,
    Safety,
    Paths,
    Wizard,
}

enum Mode {
    Browse(Section),
    Form {
        section: Section,
        kind: FormKind,
    },
}

#[derive(Clone, Copy)]
enum FormKind {
    SessionDefaults,
    LoopPolicy,
    BoardIntegration,
    GitDefaults,
    Safety,
    RolePrompt(RoleId),
    WizardStep(u8),
}

pub struct ConfigPlugin {
    mode: Mode,
    list: ScrollList,
    settings: TeamSettings,
    form: Option<FormState>,
    info: String,
    wizard_scratch: WizardScratch,
}

#[derive(Default, Clone)]
struct WizardScratch {
    project: String,
    clone_dir: String,
    agent_kind: String,
    provider: String,
    open_board: String,
}

impl ConfigPlugin {
    pub fn new() -> Self {
        Self {
            mode: Mode::Browse(Section::Root),
            list: ScrollList::default(),
            settings: TeamSettings::default(),
            form: None,
            info: String::new(),
            wizard_scratch: WizardScratch::default(),
        }
    }

    fn reload(&mut self, ctx: &mut PluginCtx) {
        match TeamSettings::load(&ctx.paths) {
            Ok(s) => self.settings = s,
            Err(e) => ctx.set_error(format!("settings: {e}")),
        }
    }

    fn persist(&mut self, ctx: &mut PluginCtx) {
        if let Err(e) = self.settings.save(&ctx.paths) {
            ctx.set_error(format!("save: {e}"));
        } else {
            ctx.set_status("settings saved");
        }
    }

    fn enter_section(&mut self, section: Section, ctx: &mut PluginCtx) {
        self.mode = Mode::Browse(section);
        self.list.set_items(section_items(section));
        self.info = section_blurb(section, &self.settings, ctx);
        ctx.set_status(match section {
            Section::Root => "Enter open section · Esc back to Dev Team menu",
            _ => "Enter edit / open · Esc parent section",
        });
    }

    fn open_form(&mut self, section: Section, kind: FormKind) {
        let fields = match kind {
            FormKind::SessionDefaults => vec![
                FormField::new("default_agent_kind")
                    .with_value(&self.settings.default_agent_kind),
                FormField::new("default_clone_dir").with_value(&self.settings.default_clone_dir),
                FormField::new("workspace_label_prefix")
                    .with_value(&self.settings.workspace_label_prefix),
                FormField::new("theme").with_value(&self.settings.theme),
                FormField::new("include_architect_on_launch")
                    .with_value(self.settings.include_architect_on_launch.to_string()),
                FormField::new("include_reviewer_on_launch")
                    .with_value(self.settings.include_reviewer_on_launch.to_string()),
                FormField::new("include_tester_on_launch")
                    .with_value(self.settings.include_tester_on_launch.to_string()),
            ],
            FormKind::LoopPolicy => vec![
                FormField::new("nudge_idle_secs")
                    .with_value(self.settings.nudge_idle_secs.to_string()),
                FormField::new("stuck_timeout_secs")
                    .with_value(self.settings.stuck_timeout_secs.to_string()),
                FormField::new("max_inflight_tickets")
                    .with_value(self.settings.max_inflight_tickets.to_string()),
                FormField::new("auto_tag_agent_on_cards")
                    .with_value(self.settings.auto_tag_agent_on_cards.to_string()),
            ],
            FormKind::BoardIntegration => vec![
                FormField::new("open_board_on_launch")
                    .with_value(self.settings.open_board_on_launch.to_string()),
                FormField::new("board_plugin_id").with_value(&self.settings.board_plugin_id),
                FormField::new("board_entrypoint").with_value(&self.settings.board_entrypoint),
            ],
            FormKind::GitDefaults => vec![
                FormField::new("preferred_git_provider")
                    .with_value(&self.settings.preferred_git_provider),
                FormField::new("default_clone_dir").with_value(&self.settings.default_clone_dir),
            ],
            FormKind::Safety => vec![
                FormField::new("master_always_confirm")
                    .with_value(self.settings.master_always_confirm.to_string()),
                FormField::new("confirm_destructive")
                    .with_value(self.settings.confirm_destructive.to_string()),
                FormField::new("notes").with_value(&self.settings.notes),
            ],
            FormKind::RolePrompt(id) => {
                let prompt = self
                    .settings
                    .role(id)
                    .map(|r| r.master_prompt.clone())
                    .unwrap_or_default();
                vec![FormField::new(format!("{} master_prompt", id.as_str())).with_value(prompt)]
            }
            FormKind::WizardStep(0) => vec![
                FormField::new("project_name").with_value(&self.wizard_scratch.project),
                FormField::new("default_clone_dir").with_value(
                    if self.wizard_scratch.clone_dir.is_empty() {
                        &self.settings.default_clone_dir
                    } else {
                        &self.wizard_scratch.clone_dir
                    },
                ),
            ],
            FormKind::WizardStep(1) => vec![
                FormField::new("agent_kind").with_value(if self.wizard_scratch.agent_kind.is_empty()
                {
                    &self.settings.default_agent_kind
                } else {
                    &self.wizard_scratch.agent_kind
                }),
                FormField::new("preferred_git_provider").with_value(
                    if self.wizard_scratch.provider.is_empty() {
                        &self.settings.preferred_git_provider
                    } else {
                        &self.wizard_scratch.provider
                    },
                ),
            ],
            FormKind::WizardStep(_) => vec![
                FormField::new("open_board_on_launch").with_value(
                    if self.wizard_scratch.open_board.is_empty() {
                        self.settings.open_board_on_launch.to_string()
                    } else {
                        self.wizard_scratch.open_board.clone()
                    },
                ),
                FormField::new("nudge_idle_secs")
                    .with_value(self.settings.nudge_idle_secs.to_string()),
                FormField::new("stuck_timeout_secs")
                    .with_value(self.settings.stuck_timeout_secs.to_string()),
                FormField::new("master_always_confirm")
                    .with_value(self.settings.master_always_confirm.to_string()),
            ],
        };
        let title = match kind {
            FormKind::SessionDefaults => "Guided: session defaults".into(),
            FormKind::LoopPolicy => "Guided: loop / nudge policy".into(),
            FormKind::BoardIntegration => "Guided: board integration".into(),
            FormKind::GitDefaults => "Guided: git defaults".into(),
            FormKind::Safety => "Guided: safety / Master gate".into(),
            FormKind::RolePrompt(id) => format!("Guided: {} prompt", id.as_str()),
            FormKind::WizardStep(n) => format!("Setup wizard · step {}", n + 1),
        };
        self.form = Some(FormState::new(title, fields));
        self.mode = Mode::Form { section, kind };
    }

    fn apply_form(&mut self, ctx: &mut PluginCtx, kind: FormKind, vals: &[String]) {
        let parse_bool = |s: &str| {
            matches!(
                s.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "y"
            )
        };
        let parse_u64 = |s: &str, default: u64| s.trim().parse().unwrap_or(default);
        let parse_u32 = |s: &str, default: u32| s.trim().parse().unwrap_or(default);

        match kind {
            FormKind::SessionDefaults => {
                self.settings.default_agent_kind = vals.first().cloned().unwrap_or_default();
                self.settings.default_clone_dir = vals.get(1).cloned().unwrap_or_default();
                self.settings.workspace_label_prefix = vals.get(2).cloned().unwrap_or_default();
                self.settings.theme = vals.get(3).cloned().unwrap_or_else(|| "cyan".into());
                self.settings.include_architect_on_launch =
                    vals.get(4).map(|s| parse_bool(s)).unwrap_or(true);
                self.settings.include_reviewer_on_launch =
                    vals.get(5).map(|s| parse_bool(s)).unwrap_or(true);
                self.settings.include_tester_on_launch =
                    vals.get(6).map(|s| parse_bool(s)).unwrap_or(true);
            }
            FormKind::LoopPolicy => {
                self.settings.nudge_idle_secs =
                    vals.first().map(|s| parse_u64(s, 120)).unwrap_or(120);
                self.settings.stuck_timeout_secs =
                    vals.get(1).map(|s| parse_u64(s, 600)).unwrap_or(600);
                self.settings.max_inflight_tickets =
                    vals.get(2).map(|s| parse_u32(s, 3)).unwrap_or(3);
                self.settings.auto_tag_agent_on_cards =
                    vals.get(3).map(|s| parse_bool(s)).unwrap_or(true);
            }
            FormKind::BoardIntegration => {
                self.settings.open_board_on_launch =
                    vals.first().map(|s| parse_bool(s)).unwrap_or(true);
                self.settings.board_plugin_id = vals
                    .get(1)
                    .cloned()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "herdr-board".into());
                self.settings.board_entrypoint = vals
                    .get(2)
                    .cloned()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "board".into());
            }
            FormKind::GitDefaults => {
                self.settings.preferred_git_provider =
                    vals.first().cloned().unwrap_or_else(|| "github".into());
                self.settings.default_clone_dir = vals.get(1).cloned().unwrap_or_default();
            }
            FormKind::Safety => {
                self.settings.master_always_confirm =
                    vals.first().map(|s| parse_bool(s)).unwrap_or(true);
                self.settings.confirm_destructive =
                    vals.get(1).map(|s| parse_bool(s)).unwrap_or(true);
                self.settings.notes = vals.get(2).cloned().unwrap_or_default();
            }
            FormKind::RolePrompt(id) => {
                if let Some(role) = self.settings.role_mut(id) {
                    role.master_prompt = vals.first().cloned().unwrap_or_default();
                }
            }
            FormKind::WizardStep(0) => {
                self.wizard_scratch.project = vals.first().cloned().unwrap_or_default();
                self.wizard_scratch.clone_dir = vals.get(1).cloned().unwrap_or_default();
                self.settings.default_clone_dir = self.wizard_scratch.clone_dir.clone();
                self.persist(ctx);
                self.open_form(Section::Wizard, FormKind::WizardStep(1));
                return;
            }
            FormKind::WizardStep(1) => {
                self.wizard_scratch.agent_kind = vals.first().cloned().unwrap_or_default();
                self.wizard_scratch.provider = vals.get(1).cloned().unwrap_or_default();
                self.settings.default_agent_kind = self.wizard_scratch.agent_kind.clone();
                self.settings.preferred_git_provider = self.wizard_scratch.provider.clone();
                self.persist(ctx);
                self.open_form(Section::Wizard, FormKind::WizardStep(2));
                return;
            }
            FormKind::WizardStep(_) => {
                self.wizard_scratch.open_board = vals.first().cloned().unwrap_or_default();
                self.settings.open_board_on_launch =
                    vals.first().map(|s| parse_bool(s)).unwrap_or(true);
                self.settings.nudge_idle_secs =
                    vals.get(1).map(|s| parse_u64(s, 120)).unwrap_or(120);
                self.settings.stuck_timeout_secs =
                    vals.get(2).map(|s| parse_u64(s, 600)).unwrap_or(600);
                self.settings.master_always_confirm =
                    vals.get(3).map(|s| parse_bool(s)).unwrap_or(true);
                self.persist(ctx);
                self.info = format!(
                    "Wizard complete.\n\nproject hint: {}\nclone_dir: {}\nagent_kind: {}\nprovider: {}\nopen_board: {}\nnudge_idle_secs: {}\n\nNext:\n1) Lazy Secrets → PAT\n2) Lazy Git → account + clone\n3) Master → queue goals → confirm\n4) Launch Team",
                    self.wizard_scratch.project,
                    self.settings.default_clone_dir,
                    self.settings.default_agent_kind,
                    self.settings.preferred_git_provider,
                    self.settings.open_board_on_launch,
                    self.settings.nudge_idle_secs
                );
                self.enter_section(Section::Root, ctx);
                return;
            }
        }
        self.persist(ctx);
        self.enter_section(
            match kind {
                FormKind::SessionDefaults => Section::Session,
                FormKind::LoopPolicy => Section::Loop,
                FormKind::BoardIntegration => Section::Board,
                FormKind::GitDefaults => Section::Git,
                FormKind::Safety => Section::Safety,
                FormKind::RolePrompt(_) => Section::Roles,
                FormKind::WizardStep(_) => Section::Wizard,
            },
            ctx,
        );
    }
}

fn section_items(section: Section) -> Vec<String> {
    match section {
        Section::Root => vec![
            "Setup wizard (guided, multi-step)".into(),
            "Session defaults".into(),
            "Loop / nudge policy".into(),
            "Board integration".into(),
            "Git defaults".into(),
            "Role prompt studio".into(),
            "Safety & Master gate".into(),
            "Paths & Herdr hints".into(),
            "Dump full settings summary".into(),
        ],
        Section::Session => vec![
            "Edit session defaults (guided form)".into(),
            "Explain each field".into(),
            "Back".into(),
        ],
        Section::Loop => vec![
            "Edit loop / nudge policy".into(),
            "Explain nudge vs stuck timeouts".into(),
            "Back".into(),
        ],
        Section::Board => vec![
            "Edit board integration".into(),
            "Explain card tagging conventions".into(),
            "Back".into(),
        ],
        Section::Git => vec![
            "Edit git defaults".into(),
            "Provider notes (github/gitlab/forgejo)".into(),
            "Back".into(),
        ],
        Section::Roles => vec![
            "Edit Master prompt".into(),
            "Edit Orchestrator prompt".into(),
            "Edit Planner prompt".into(),
            "Edit Coder prompt".into(),
            "Edit Reviewer prompt".into(),
            "Edit Tester prompt".into(),
            "Edit Architect prompt".into(),
            "Back".into(),
        ],
        Section::Safety => vec![
            "Edit safety / confirmation policy".into(),
            "Why Master must always confirm".into(),
            "Back".into(),
        ],
        Section::Paths => vec![
            "Show config & state paths".into(),
            "Herdr keybind example".into(),
            "Back".into(),
        ],
        Section::Wizard => vec![
            "Start / resume wizard".into(),
            "Back".into(),
        ],
    }
}

fn section_blurb(section: Section, settings: &TeamSettings, ctx: &PluginCtx) -> String {
    match section {
        Section::Root => format!(
            "Guided configuration for Dev Team.\n\nCurrent snapshot:\n  agent_kind: {}\n  clone_dir: {}\n  open_board_on_launch: {}\n  nudge_idle_secs: {}\n  master_always_confirm: {}\n  roles: {}\n\nPick a section. The setup wizard walks defaults end-to-end.",
            settings.default_agent_kind,
            settings.default_clone_dir,
            settings.open_board_on_launch,
            settings.nudge_idle_secs,
            settings.master_always_confirm,
            settings.roles.len()
        ),
        Section::Session => "Session defaults control agent kind, clone parent dir, workspace label prefix, theme, and which roles Launch Team includes.".into(),
        Section::Loop => "Loop policy tells Orchestrator how aggressive to be about nudging idle agents and escalating stuck tickets.".into(),
        Section::Board => "Board integration controls auto-open on launch and which herdr-board plugin id/entrypoint to call.".into(),
        Section::Git => "Git defaults seed Lazy Git clone parent and preferred forge provider.".into(),
        Section::Roles => "Role prompt studio edits the long-form master prompts used when Launch Team starts each agent.".into(),
        Section::Safety => "Safety keeps Master from dispatching without an explicit yes, and keeps destructive deletes confirmed.".into(),
        Section::Paths => format!(
            "Config dir:\n  {}\nState dir:\n  {}\n\nTip: herdr plugin config-dir dev-team",
            ctx.paths.config_dir.display(),
            ctx.paths.state_dir.display()
        ),
        Section::Wizard => "Multi-step wizard: project + clone dir → agent/provider → board + nudge + Master gate.".into(),
    }
}

impl SubPlugin for ConfigPlugin {
    fn id(&self) -> &'static str {
        "config"
    }
    fn title(&self) -> &'static str {
        "Full Config"
    }
    fn description(&self) -> &'static str {
        "Deep guided configuration: setup wizard, session/loop/board/git defaults, role prompt studio, safety gate, and path hints."
    }

    fn on_enter(&mut self, ctx: &mut PluginCtx) {
        self.reload(ctx);
        self.enter_section(Section::Root, ctx);
    }

    fn handle(&mut self, ctx: &mut PluginCtx, key: KeyEvent) -> NavAction {
        if let Mode::Form { section, kind } = self.mode {
            let Some(form) = self.form.as_mut() else {
                self.enter_section(section, ctx);
                return NavAction::None;
            };
            match form.handle(key) {
                FormResult::Cancel => {
                    self.form = None;
                    self.enter_section(section, ctx);
                }
                FormResult::Submit => {
                    let vals = form.values();
                    self.form = None;
                    self.apply_form(ctx, kind, &vals);
                }
                FormResult::Continue => {}
            }
            return NavAction::None;
        }

        if self.list.handle_nav(key) {
            return NavAction::None;
        }

        let Mode::Browse(section) = self.mode else {
            return NavAction::None;
        };

        match key.code {
            KeyCode::Esc => match section {
                Section::Root => NavAction::Back,
                _ => {
                    self.enter_section(Section::Root, ctx);
                    NavAction::None
                }
            },
            KeyCode::Enter => {
                let sel = self.list.selected();
                match section {
                    Section::Root => match sel {
                        Some(0) => {
                            self.enter_section(Section::Wizard, ctx);
                            self.open_form(Section::Wizard, FormKind::WizardStep(0));
                        }
                        Some(1) => self.enter_section(Section::Session, ctx),
                        Some(2) => self.enter_section(Section::Loop, ctx),
                        Some(3) => self.enter_section(Section::Board, ctx),
                        Some(4) => self.enter_section(Section::Git, ctx),
                        Some(5) => self.enter_section(Section::Roles, ctx),
                        Some(6) => self.enter_section(Section::Safety, ctx),
                        Some(7) => self.enter_section(Section::Paths, ctx),
                        Some(8) => {
                            self.info = format!("{:#?}", self.settings);
                            ctx.set_status("summary dumped");
                        }
                        _ => {}
                    },
                    Section::Session => match sel {
                        Some(0) => self.open_form(section, FormKind::SessionDefaults),
                        Some(1) => {
                            self.info = "default_agent_kind: Herdr agent backend (codex/cursor/claude/…)\ndefault_clone_dir: parent folder for Lazy Git clones\nworkspace_label_prefix: prefix for launched workspace labels\ntheme: TUI accent hint\ninclude_*: toggles which role panes Launch Team creates".into();
                        }
                        _ => self.enter_section(Section::Root, ctx),
                    },
                    Section::Loop => match sel {
                        Some(0) => self.open_form(section, FormKind::LoopPolicy),
                        Some(1) => {
                            self.info = "nudge_idle_secs: Orchestrator re-prompts agents idle this long\nstuck_timeout_secs: escalate tickets with no progress\nmax_inflight_tickets: cap concurrent Doing cards\nauto_tag_agent_on_cards: require agent:/order:/depends: on cards".into();
                        }
                        _ => self.enter_section(Section::Root, ctx),
                    },
                    Section::Board => match sel {
                        Some(0) => self.open_form(section, FormKind::BoardIntegration),
                        Some(1) => {
                            self.info = "Card title pattern:\n  [Coder][order:3][depends:1,2] Implement X\nOrchestrator creates/moves cards; humans watch the board for live progress.".into();
                        }
                        _ => self.enter_section(Section::Root, ctx),
                    },
                    Section::Git => match sel {
                        Some(0) => self.open_form(section, FormKind::GitDefaults),
                        Some(1) => {
                            self.info = "github: prefers `gh` then API+PAT\ngitlab: prefers `glab` then API+PAT\nforgejo/gitea: API via host + PAT from Secrets\nSet secret_ref on the Lazy Git account to a Secrets entry name.".into();
                        }
                        _ => self.enter_section(Section::Root, ctx),
                    },
                    Section::Roles => match sel {
                        Some(0) => self.open_form(section, FormKind::RolePrompt(RoleId::Master)),
                        Some(1) => {
                            self.open_form(section, FormKind::RolePrompt(RoleId::Orchestrator))
                        }
                        Some(2) => self.open_form(section, FormKind::RolePrompt(RoleId::Planner)),
                        Some(3) => self.open_form(section, FormKind::RolePrompt(RoleId::Coder)),
                        Some(4) => self.open_form(section, FormKind::RolePrompt(RoleId::Reviewer)),
                        Some(5) => self.open_form(section, FormKind::RolePrompt(RoleId::Tester)),
                        Some(6) => {
                            self.open_form(section, FormKind::RolePrompt(RoleId::Architect))
                        }
                        _ => self.enter_section(Section::Root, ctx),
                    },
                    Section::Safety => match sel {
                        Some(0) => self.open_form(section, FormKind::Safety),
                        Some(1) => {
                            self.info = "Master is the only human interface. Forcing confirmation prevents accidental mass-agent work when you are still brainstorming. Keep master_always_confirm=true unless you fully accept auto-dispatch risk.".into();
                        }
                        _ => self.enter_section(Section::Root, ctx),
                    },
                    Section::Paths => match sel {
                        Some(0) => {
                            self.info = section_blurb(Section::Paths, &self.settings, ctx);
                        }
                        Some(1) => {
                            self.info = r#"~/.config/herdr/config.toml

[[keys.command]]
key = "prefix+d"
type = "plugin_action"
command = "dev-team.open"
description = "open Dev Team"

Plugin mgmt: herdr plugin list | link | enable
"#
                            .into();
                        }
                        _ => self.enter_section(Section::Root, ctx),
                    },
                    Section::Wizard => match sel {
                        Some(0) => self.open_form(Section::Wizard, FormKind::WizardStep(0)),
                        _ => self.enter_section(Section::Root, ctx),
                    },
                }
                NavAction::None
            }
            _ => NavAction::None,
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &PluginCtx) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area);
        let title = match self.mode {
            Mode::Browse(Section::Root) | Mode::Form { section: Section::Root, .. } => {
                "Full Config"
            }
            Mode::Browse(s) | Mode::Form { section: s, .. } => match s {
                Section::Session => "Config · Session",
                Section::Loop => "Config · Loop",
                Section::Board => "Config · Board",
                Section::Git => "Config · Git",
                Section::Roles => "Config · Roles",
                Section::Safety => "Config · Safety",
                Section::Paths => "Config · Paths",
                Section::Wizard => "Config · Wizard",
                Section::Root => "Full Config",
            },
        };
        draw_select_list(frame, chunks[0], title, &self.list.items, self.list.selected());
        frame.render_widget(
            Paragraph::new(self.info.as_str())
                .wrap(Wrap { trim: false })
                .block(Block::default().title("Guidance").borders(Borders::ALL)),
            chunks[1],
        );
        if let Some(form) = &self.form {
            form.draw(frame, area);
        }
    }
}
