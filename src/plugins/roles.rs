//! Browse and edit role skill cards + master prompts.

use crate::registry::{NavAction, PluginCtx, SubPlugin};
use crate::roles::{default_roles, RoleId};
use crate::session::TeamSettings;
use crate::ui::draw_select_list;
use crate::ui::{FormField, FormResult, FormState};
use crate::ui::ScrollList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

enum Mode {
    List,
    EditPrompt { role: RoleId },
    EditMeta { role: RoleId },
}

pub struct RolesPlugin {
    list: ScrollList,
    settings: TeamSettings,
    mode: Mode,
    form: Option<FormState>,
    detail: String,
}

impl RolesPlugin {
    pub fn new() -> Self {
        Self {
            list: ScrollList::default(),
            settings: TeamSettings::default(),
            mode: Mode::List,
            form: None,
            detail: String::new(),
        }
    }

    fn reload(&mut self, ctx: &mut PluginCtx) {
        self.settings = TeamSettings::load(&ctx.paths).unwrap_or_default();
        self.list.set_items(
            self.settings
                .roles
                .iter()
                .map(|r| {
                    format!(
                        "{} — {} / {} → {}",
                        r.id.as_str(),
                        r.primary_skill,
                        r.secondary_skill,
                        r.output
                    )
                })
                .collect(),
        );
        self.show_selected();
    }

    fn show_selected(&mut self) {
        let Some(i) = self.list.selected() else {
            self.detail.clear();
            return;
        };
        let Some(role) = self.settings.roles.get(i) else {
            return;
        };
        self.detail = format!(
            "Role: {}\nPrimary: {}\nSecondary: {}\nOutput: {}\nDirection: {}\n\nMaster prompt:\n{}\n",
            role.id.as_str(),
            role.primary_skill,
            role.secondary_skill,
            role.output,
            role.direction,
            role.master_prompt
        );
    }

    fn persist(&mut self, ctx: &mut PluginCtx) {
        if let Err(e) = self.settings.save(&ctx.paths) {
            ctx.set_error(format!("save: {e}"));
        } else {
            ctx.set_status("roles saved");
        }
    }
}

impl SubPlugin for RolesPlugin {
    fn id(&self) -> &'static str {
        "roles"
    }
    fn title(&self) -> &'static str {
        "Team Roles"
    }
    fn description(&self) -> &'static str {
        "Planner / Coder / Reviewer / Tester / Architect (+ Master & Orchestrator) skill cards and editable master prompts."
    }

    fn on_enter(&mut self, ctx: &mut PluginCtx) {
        self.mode = Mode::List;
        self.reload(ctx);
        ctx.set_status("e edit prompt · m edit meta · r reset role · Esc back");
    }

    fn handle(&mut self, ctx: &mut PluginCtx, key: KeyEvent) -> NavAction {
        match &mut self.mode {
            Mode::EditPrompt { role } | Mode::EditMeta { role } => {
                let role = *role;
                let is_prompt = matches!(self.mode, Mode::EditPrompt { .. });
                let Some(form) = self.form.as_mut() else {
                    self.mode = Mode::List;
                    return NavAction::None;
                };
                match form.handle(key) {
                    FormResult::Cancel => {
                        self.form = None;
                        self.mode = Mode::List;
                    }
                    FormResult::Submit => {
                        let vals = form.values();
                        if let Some(spec) = self.settings.role_mut(role) {
                            if is_prompt {
                                spec.master_prompt = vals.first().cloned().unwrap_or_default();
                            } else {
                                spec.primary_skill = vals.first().cloned().unwrap_or_default();
                                spec.secondary_skill = vals.get(1).cloned().unwrap_or_default();
                                spec.output = vals.get(2).cloned().unwrap_or_default();
                                spec.direction = vals
                                    .get(3)
                                    .cloned()
                                    .filter(|s| !s.is_empty())
                                    .unwrap_or_else(|| "right".into());
                            }
                        }
                        self.persist(ctx);
                        self.form = None;
                        self.mode = Mode::List;
                        self.reload(ctx);
                    }
                    FormResult::Continue => {}
                }
                NavAction::None
            }
            Mode::List => {
                if self.list.handle_nav(key) {
                    self.show_selected();
                    return NavAction::None;
                }
                match key.code {
                    KeyCode::Esc => NavAction::Back,
                    KeyCode::Char('e') => {
                        if let Some(i) = self.list.selected() {
                            if let Some(role) = self.settings.roles.get(i).cloned() {
                                self.form = Some(FormState::new(
                                    format!("Edit {} prompt", role.id.as_str()),
                                    vec![FormField::new("master_prompt")
                                        .with_value(&role.master_prompt)],
                                ));
                                self.mode = Mode::EditPrompt { role: role.id };
                            }
                        }
                        NavAction::None
                    }
                    KeyCode::Char('m') => {
                        if let Some(i) = self.list.selected() {
                            if let Some(role) = self.settings.roles.get(i).cloned() {
                                self.form = Some(FormState::new(
                                    format!("Edit {} meta", role.id.as_str()),
                                    vec![
                                        FormField::new("primary_skill")
                                            .with_value(&role.primary_skill),
                                        FormField::new("secondary_skill")
                                            .with_value(&role.secondary_skill),
                                        FormField::new("output").with_value(&role.output),
                                        FormField::new("direction").with_value(&role.direction),
                                    ],
                                ));
                                self.mode = Mode::EditMeta { role: role.id };
                            }
                        }
                        NavAction::None
                    }
                    KeyCode::Char('r') => {
                        if let Some(i) = self.list.selected() {
                            if let Some(id) = self.settings.roles.get(i).map(|r| r.id) {
                                if let Some(stock) = default_roles().into_iter().find(|r| r.id == id)
                                {
                                    if let Some(slot) = self.settings.role_mut(id) {
                                        *slot = stock;
                                    }
                                    self.persist(ctx);
                                    self.reload(ctx);
                                }
                            }
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
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);
        draw_select_list(
            frame,
            chunks[0],
            "Team Roles · e prompt · m meta · r reset",
            &self.list.items,
            self.list.selected(),
        );
        frame.render_widget(
            Paragraph::new(self.detail.as_str())
                .wrap(Wrap { trim: false })
                .block(Block::default().title("Role card").borders(Borders::ALL)),
            chunks[1],
        );
        if let Some(form) = &self.form {
            form.draw(frame, area);
        }
    }
}
