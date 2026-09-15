//! Built-in Dev Team documentation reader.

use crate::registry::{NavAction, PluginCtx, SubPlugin};
use crate::ui::ScrollList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

struct DocPage {
    title: &'static str,
    body: &'static str,
}

const PAGES: &[DocPage] = &[
    DocPage {
        title: "Overview",
        body: r#"Dev Team is a Herdr plugin focused on:
- Lazy Git project setup (clone → cwd)
- Live kanban (herdr-board)
- A Master-gated multi-agent coding loop

Agents:
  Master        human interface + confirm gate + project cwd
  Orchestrator  kanban updates + nudge stuck agents
  Planner       tickets / roadmap
  Coder         code modules
  Reviewer      review reports
  Tester        tests + logs
  Architect     design guidelines

Flow: you talk to Master → Master ALWAYS confirms → Orchestrator updates board
→ role agents work tickets in a loop → report back → board moves in real time."#,
    },
    DocPage {
        title: "Master gate",
        body: r#"Master never silently dispatches work.

Every time you promote goals / environment / start-loop instructions, Master must show:

  Hey — you have told me:
    1) …
  Working environment:
    - repo/path: …
    - project: …
    - loop: …
  Do you want me to implement / dispatch this to the Dev Team? (yes / no)

Only after yes does Master relay to Orchestrator and (optionally) start the loop.
Use Master → Queue prompt, then Confirm / Reject."#,
    },
    DocPage {
        title: "Launch + Board",
        body: r#"Launch Team creates a Herdr workspace with panes for Master + Orchestrator + roles,
sends each role its master prompt, and optionally opens herdr-board.

Requires:
  herdr plugin install <owner>/herdr-board   # Linux/macOS today

Kanban cards should be tagged with the agent role and an order/dependency hint
so Orchestrator can keep the pipeline moving while you watch."#,
    },
    DocPage {
        title: "Keys",
        body: r#"Main menu: ↑/↓ j/k · Enter open · ? docs · Esc/q quit
Sub-menus: Esc back
Forms: Tab fields · Enter save · Esc cancel
Deletes: type the item name to confirm
Master: y confirm · n reject · q queue · s start/stop loop"#,
    },
];

pub struct DocsPlugin {
    list: ScrollList,
}

impl DocsPlugin {
    pub fn new() -> Self {
        Self {
            list: ScrollList::new(PAGES.iter().map(|p| p.title.to_string()).collect()),
        }
    }
}

impl SubPlugin for DocsPlugin {
    fn id(&self) -> &'static str {
        "docs"
    }
    fn title(&self) -> &'static str {
        "Docs"
    }
    fn description(&self) -> &'static str {
        "How Master confirmation, the Orchestrator loop, roles, kanban, and lazy git fit together."
    }

    fn on_enter(&mut self, ctx: &mut PluginCtx) {
        ctx.set_status("j/k pages · Esc back");
    }

    fn handle(&mut self, _ctx: &mut PluginCtx, key: KeyEvent) -> NavAction {
        if self.list.handle_nav(key) {
            return NavAction::None;
        }
        match key.code {
            KeyCode::Esc => NavAction::Back,
            _ => NavAction::None,
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &PluginCtx) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
            .split(area);
        crate::ui::draw_select_list(
            frame,
            chunks[0],
            "Docs",
            &self.list.items,
            self.list.selected(),
        );
        let body = self
            .list
            .selected()
            .and_then(|i| PAGES.get(i))
            .map(|p| p.body)
            .unwrap_or("");
        frame.render_widget(
            Paragraph::new(body)
                .wrap(Wrap { trim: false })
                .block(Block::default().title("Guide").borders(Borders::ALL)),
            chunks[1],
        );
    }
}
