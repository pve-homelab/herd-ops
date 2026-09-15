//! Kanban board opener (herdr-board) tuned for Dev Team.

use crate::herdr::{run_herdr_ok, which_exists};
use crate::registry::{NavAction, PluginCtx, SubPlugin};
use crate::session::TeamSettings;
use crate::ui::draw_select_list;
use crate::ui::ScrollList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub struct BoardPlugin {
    list: ScrollList,
    log: String,
}

impl BoardPlugin {
    pub fn new() -> Self {
        Self {
            list: ScrollList::new(vec![
                "Open kanban board pane".into(),
                "Invoke board open action".into(),
                "Show Dev Team board conventions".into(),
            ]),
            log: String::new(),
        }
    }

    fn conventions() -> String {
        r#"Dev Team board conventions (Orchestrator owns these):

Columns (suggested): Backlog · Todo · Doing · Review · Test · Done · Blocked

Every Todo/Doing card MUST include:
  agent:<Planner|Coder|Reviewer|Tester|Architect>
  order:<n>
  depends:<comma ids optional>

Example title:
  [Coder][order:3][depends:1,2] Implement auth middleware

Flow:
  Planner creates ordered tickets → Orchestrator writes cards
  → agents pull their tags → report done → Orchestrator moves cards
  → Orchestrator nudges idle/stuck agents so the loop continues

Install: herdr plugin install <owner>/herdr-board  (Linux/macOS)"#
            .into()
    }

    fn open_pane(&mut self, ctx: &mut PluginCtx) {
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
                self.log = format!("Opened board pane.\n{out}");
                ctx.set_status("board opened");
            }
            Err(e) => {
                self.log = format!("{}\n\nError:\n{e}", Self::conventions());
                ctx.set_error("board not available");
            }
        }
    }

    fn open_action(&mut self, ctx: &mut PluginCtx) {
        let settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
        let action = format!("{}.open-board", settings.board_plugin_id);
        match run_herdr_ok(&["plugin", "action", "invoke", &action]) {
            Ok(out) => {
                self.log = format!("Invoked {action}.\n{out}");
                ctx.set_status("board action ok");
            }
            Err(e) => {
                self.log = format!("{}\n\nError:\n{e}", Self::conventions());
                ctx.set_error("board action failed");
            }
        }
    }
}

impl SubPlugin for BoardPlugin {
    fn id(&self) -> &'static str {
        "board"
    }
    fn title(&self) -> &'static str {
        "Kanban Board"
    }
    fn description(&self) -> &'static str {
        "Open herdr-board and follow Dev Team card tagging (agent + order + depends) so Orchestrator can keep the loop visible."
    }

    fn on_enter(&mut self, ctx: &mut PluginCtx) {
        self.log = Self::conventions();
        let ready = which_exists("board") || which_exists("herdr-board");
        ctx.set_status(if ready {
            "board CLI on PATH · Enter to open"
        } else {
            "Enter open · install herdr-board if this fails"
        });
    }

    fn handle(&mut self, ctx: &mut PluginCtx, key: KeyEvent) -> NavAction {
        if self.list.handle_nav(key) {
            return NavAction::None;
        }
        match key.code {
            KeyCode::Esc => NavAction::Back,
            KeyCode::Enter => {
                match self.list.selected() {
                    Some(0) => self.open_pane(ctx),
                    Some(1) => self.open_action(ctx),
                    Some(2) => {
                        self.log = Self::conventions();
                        ctx.set_status("conventions");
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
            "Kanban Board",
            &self.list.items,
            self.list.selected(),
        );
        frame.render_widget(
            Paragraph::new(self.log.as_str())
                .wrap(Wrap { trim: false })
                .block(Block::default().title("Notes").borders(Borders::ALL)),
            chunks[1],
        );
    }
}
