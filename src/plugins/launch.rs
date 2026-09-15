//! Launch the full Dev Team workspace (Master + roles + optional board).

use crate::herdr::run_herdr_ok;
use crate::registry::{NavAction, PluginCtx, SubPlugin};
use crate::roles::RoleId;
use crate::session::{expand_home, Session, TeamSettings};
use crate::ui::draw_select_list;
use crate::ui::ScrollList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub struct LaunchPlugin {
    list: ScrollList,
    log: String,
}

impl LaunchPlugin {
    pub fn new() -> Self {
        Self {
            list: ScrollList::new(vec![
                "Launch Dev Team workspace (from session cwd)".into(),
                "Preview panes that will start".into(),
                "Open kanban board only".into(),
                "Install tips".into(),
            ]),
            log: String::new(),
        }
    }

    fn preview(&mut self, ctx: &mut PluginCtx) {
        let settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
        let session = Session::load(&ctx.paths).unwrap_or_default();
        let mut lines = vec![
            format!("cwd: {}", blank(&session.working_dir, "(set via Master or Lazy Git)")),
            format!("project: {}", blank(&session.project_name, "(unnamed)")),
            format!("agent_kind: {}", blank(&session.agent_kind, &settings.default_agent_kind)),
            format!("open_board: {}", settings.open_board_on_launch),
            String::new(),
            "Panes:".into(),
        ];
        for role in settings.launch_roles() {
            lines.push(format!(
                "  - {} [{}] → {} / {}",
                role.id.as_str(),
                role.direction,
                role.primary_skill,
                role.output
            ));
        }
        self.log = lines.join("\n");
        ctx.set_status("preview ready");
    }

    fn launch(&mut self, ctx: &mut PluginCtx) {
        let settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
        let mut session = Session::load(&ctx.paths).unwrap_or_default();
        if session.working_dir.trim().is_empty() {
            self.log = "No working_dir in session.\nUse Master → Set working environment, or Lazy Git to clone first.".into();
            ctx.set_error("working_dir required");
            return;
        }
        let cwd = expand_home(&session.working_dir);
        let kind = if session.agent_kind.trim().is_empty() {
            settings.default_agent_kind.clone()
        } else {
            session.agent_kind.clone()
        };
        let label = if session.project_name.trim().is_empty() {
            format!(
                "{}-session",
                blank(&settings.workspace_label_prefix, "dev-team")
            )
        } else {
            format!(
                "{}-{}",
                blank(&settings.workspace_label_prefix, "dev-team"),
                sanitize(&session.project_name)
            )
        };

        let created = match run_herdr_ok(&[
            "workspace",
            "create",
            "--cwd",
            &cwd,
            "--label",
            &label,
            "--focus",
        ]) {
            Ok(out) => out,
            Err(e) => {
                self.log = format!("workspace create failed:\n{e}");
                ctx.set_error("workspace create failed");
                return;
            }
        };

        let mut log = format!("Created workspace `{label}` @ {cwd}\n");
        let mut last_pane = extract_json_str(&created, &["result", "root_pane", "pane_id"])
            .or_else(|| extract_json_str(&created, &["root_pane", "pane_id"]));

        let roles = settings.launch_roles();
        for (idx, role) in roles.iter().enumerate() {
            let pane_id = if idx == 0 {
                match last_pane.clone() {
                    Some(id) => id,
                    None => {
                        log.push_str("Missing root pane id — stop.\n");
                        break;
                    }
                }
            } else {
                let from = last_pane.clone().unwrap_or_default();
                let dir = if role.direction == "down" {
                    "down"
                } else {
                    "right"
                };
                match run_herdr_ok(&[
                    "pane",
                    "split",
                    &from,
                    "--direction",
                    dir,
                    "--cwd",
                    &cwd,
                    "--no-focus",
                ]) {
                    Ok(out) => {
                        if let Some(id) = extract_json_str(&out, &["result", "pane", "pane_id"])
                            .or_else(|| extract_json_str(&out, &["pane", "pane_id"]))
                        {
                            id
                        } else {
                            log.push_str(&format!("split ok but no pane id: {out}\n"));
                            break;
                        }
                    }
                    Err(e) => {
                        log.push_str(&format!("split {} failed: {e}\n", role.id.as_str()));
                        break;
                    }
                }
            };

            let name = role.id.as_str();
            let _ = run_herdr_ok(&["pane", "rename", &pane_id, name]);
            last_pane = Some(pane_id.clone());

            match run_herdr_ok(&["agent", "start", name, "--kind", &kind, "--pane", &pane_id]) {
                Ok(_) => {
                    log.push_str(&format!("Pane {name} ({pane_id}) started as {kind}\n"));
                    let mut prompt = role.master_prompt.clone();
                    prompt.push_str(&format!(
                        "\n\n---\nSession cwd: {cwd}\nProject: {}\nYou report status to Orchestrator (Master for Master role).\n",
                        blank(&session.project_name, label.as_str())
                    ));
                    if role.id == RoleId::Master {
                        prompt.push_str(
                            "\nHARD RULE remains: ALWAYS ask the human for confirmation before dispatching.\n",
                        );
                    }
                    match run_herdr_ok(&["agent", "prompt", &prompt]) {
                        Ok(_) => log.push_str("  master prompt sent\n"),
                        Err(e) => log.push_str(&format!("  prompt failed: {e}\n")),
                    }
                }
                Err(e) => {
                    log.push_str(&format!("Pane {name} ready but agent start failed: {e}\n"));
                }
            }
        }

        if settings.open_board_on_launch {
            match run_herdr_ok(&[
                "plugin",
                "pane",
                "open",
                "--plugin",
                &settings.board_plugin_id,
                "--entrypoint",
                &settings.board_entrypoint,
            ]) {
                Ok(_) => {
                    log.push_str("Kanban board opened\n");
                    session.board_open = true;
                }
                Err(e) => log.push_str(&format!("board open failed: {e}\n")),
            }
        }

        session.working_dir = cwd;
        let _ = session.save(&ctx.paths);
        self.log = log;
        ctx.set_status(format!("launched `{label}`"));
    }

    fn open_board(&mut self, ctx: &mut PluginCtx) {
        let settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
        match run_herdr_ok(&[
            "plugin",
            "pane",
            "open",
            "--plugin",
            &settings.board_plugin_id,
            "--entrypoint",
            &settings.board_entrypoint,
        ]) {
            Ok(out) => {
                self.log = format!("Opened board.\n{out}");
                ctx.set_status("board opened");
            }
            Err(e) => {
                self.log = format!("Board open failed:\n{e}\n\nInstall herdr-board (Linux/macOS).");
                ctx.set_error("board unavailable");
            }
        }
    }
}

fn blank<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn extract_json_str(raw: &str, path: &[&str]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    let mut cur = &v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str().map(str::to_string)
}

impl SubPlugin for LaunchPlugin {
    fn id(&self) -> &'static str {
        "launch"
    }
    fn title(&self) -> &'static str {
        "Launch Team"
    }
    fn description(&self) -> &'static str {
        "Spin up Master + Orchestrator + Planner/Coder/Reviewer/Tester/Architect panes, seed prompts, and open the kanban board."
    }

    fn on_enter(&mut self, ctx: &mut PluginCtx) {
        self.preview(ctx);
        ctx.set_status("Enter launch · Esc back");
    }

    fn handle(&mut self, ctx: &mut PluginCtx, key: KeyEvent) -> NavAction {
        if self.list.handle_nav(key) {
            return NavAction::None;
        }
        match key.code {
            KeyCode::Esc => NavAction::Back,
            KeyCode::Enter => {
                match self.list.selected() {
                    Some(0) => self.launch(ctx),
                    Some(1) => self.preview(ctx),
                    Some(2) => self.open_board(ctx),
                    Some(3) => {
                        self.log = r#"Tips:
1) Lazy Secrets → store forge PAT
2) Lazy Git → account + clone (sets session cwd via Master env, or set manually)
3) Master → queue goals → confirm (y)
4) Launch Team → start panes + board
5) Watch Orchestrator move kanban cards while agents loop

herdr plugin install <owner>/herdr-board
herdr plugin link .
"#
                        .into();
                        ctx.set_status("tips");
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
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);
        draw_select_list(
            frame,
            chunks[0],
            "Launch Team",
            &self.list.items,
            self.list.selected(),
        );
        frame.render_widget(
            Paragraph::new(self.log.as_str())
                .wrap(Wrap { trim: false })
                .block(Block::default().title("Plan / output").borders(Borders::ALL)),
            chunks[1],
        );
    }
}
