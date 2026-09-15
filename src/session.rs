//! Durable session state shared by Master and the Dev Team loop.

use crate::roles::{default_roles, RoleId, RoleSpec};
use crate::storage::{read_toml, write_toml, Paths};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopState {
    Stopped,
    Running,
    Paused,
}

impl Default for LoopState {
    fn default() -> Self {
        Self::Stopped
    }
}

impl LoopState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoopState::Stopped => "stopped",
            LoopState::Running => "running",
            LoopState::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PendingDirective {
    /// Human-facing bullet lines Master must confirm.
    pub prompts: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default)]
    pub git_remote: String,
    #[serde(default)]
    pub loop_state: LoopState,
    /// Queued human instructions awaiting explicit Master confirmation.
    #[serde(default)]
    pub pending: Option<PendingDirective>,
    /// Last directive Master actually dispatched after confirmation.
    #[serde(default)]
    pub last_confirmed: Option<PendingDirective>,
    #[serde(default)]
    pub board_open: bool,
    #[serde(default)]
    pub agent_kind: String,
    #[serde(default)]
    pub history: Vec<String>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            working_dir: String::new(),
            git_remote: String::new(),
            loop_state: LoopState::Stopped,
            pending: None,
            last_confirmed: None,
            board_open: false,
            agent_kind: "codex".into(),
            history: Vec::new(),
        }
    }
}

impl Session {
    pub fn path(paths: &Paths) -> PathBuf {
        paths.join_state("session.toml")
    }

    pub fn load(paths: &Paths) -> Result<Self> {
        Ok(read_toml(&Self::path(paths))?.unwrap_or_default())
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        write_toml(&Self::path(paths), self)
    }

    pub fn queue_prompts(&mut self, prompts: Vec<String>, notes: impl Into<String>) {
        self.pending = Some(PendingDirective {
            prompts,
            notes: notes.into(),
        });
    }

    pub fn confirm_pending(&mut self) -> Option<PendingDirective> {
        let pending = self.pending.take()?;
        let summary = format!(
            "CONFIRMED: {}",
            pending.prompts.join(" | ")
        );
        self.history.push(summary);
        if self.history.len() > 40 {
            let drain = self.history.len() - 40;
            self.history.drain(0..drain);
        }
        self.last_confirmed = Some(pending.clone());
        Some(pending)
    }

    pub fn reject_pending(&mut self) {
        if let Some(p) = self.pending.take() {
            self.history.push(format!("REJECTED: {}", p.prompts.join(" | ")));
        }
    }

    pub fn confirmation_block(&self) -> String {
        let mut lines = Vec::new();
        lines.push("Hey — you have told me:".into());
        match &self.pending {
            Some(p) if !p.prompts.is_empty() => {
                for (i, prompt) in p.prompts.iter().enumerate() {
                    lines.push(format!("  {}) {}", i + 1, prompt));
                }
                if !p.notes.trim().is_empty() {
                    lines.push(format!("  notes: {}", p.notes.trim()));
                }
            }
            _ => lines.push("  (nothing queued yet)".into()),
        }
        lines.push("Working environment:".into());
        lines.push(format!(
            "  - repo/path: {}",
            if self.working_dir.is_empty() {
                "(not set yet)"
            } else {
                &self.working_dir
            }
        ));
        lines.push(format!(
            "  - project: {}",
            if self.project_name.is_empty() {
                "(unnamed)"
            } else {
                &self.project_name
            }
        ));
        lines.push(format!("  - loop: {}", self.loop_state.as_str()));
        lines.push(format!("  - agent_kind: {}", self.agent_kind));
        lines.push(
            "Do you want me to implement / dispatch this to the Dev Team? (yes / no)".into(),
        );
        lines.join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub master_always_confirm: bool,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
    #[serde(default = "default_clone_dir")]
    pub default_clone_dir: String,
    #[serde(default = "default_agent_kind")]
    pub default_agent_kind: String,
    #[serde(default = "default_true")]
    pub open_board_on_launch: bool,
    #[serde(default = "default_nudge_secs")]
    pub nudge_idle_secs: u64,
    #[serde(default = "default_stuck_secs")]
    pub stuck_timeout_secs: u64,
    #[serde(default = "default_max_inflight")]
    pub max_inflight_tickets: u32,
    #[serde(default = "default_true")]
    pub auto_tag_agent_on_cards: bool,
    #[serde(default = "default_board_plugin")]
    pub board_plugin_id: String,
    #[serde(default = "default_board_entry")]
    pub board_entrypoint: String,
    #[serde(default)]
    pub preferred_git_provider: String,
    #[serde(default)]
    pub workspace_label_prefix: String,
    #[serde(default)]
    pub include_architect_on_launch: bool,
    #[serde(default = "default_true")]
    pub include_reviewer_on_launch: bool,
    #[serde(default = "default_true")]
    pub include_tester_on_launch: bool,
    #[serde(default)]
    pub notes: String,
    /// Editable role prompts (persisted separately-ish inside settings for guided config).
    #[serde(default = "default_roles")]
    pub roles: Vec<RoleSpec>,
}

fn default_theme() -> String {
    "cyan".into()
}
fn default_true() -> bool {
    true
}
fn default_clone_dir() -> String {
    "~/repos".into()
}
fn default_agent_kind() -> String {
    "codex".into()
}
fn default_nudge_secs() -> u64 {
    120
}
fn default_stuck_secs() -> u64 {
    600
}
fn default_max_inflight() -> u32 {
    3
}
fn default_board_plugin() -> String {
    "herdr-board".into()
}
fn default_board_entry() -> String {
    "board".into()
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            master_always_confirm: true,
            confirm_destructive: true,
            default_clone_dir: default_clone_dir(),
            default_agent_kind: default_agent_kind(),
            open_board_on_launch: true,
            nudge_idle_secs: default_nudge_secs(),
            stuck_timeout_secs: default_stuck_secs(),
            max_inflight_tickets: default_max_inflight(),
            auto_tag_agent_on_cards: true,
            board_plugin_id: default_board_plugin(),
            board_entrypoint: default_board_entry(),
            preferred_git_provider: "github".into(),
            workspace_label_prefix: "dev-team".into(),
            include_architect_on_launch: true,
            include_reviewer_on_launch: true,
            include_tester_on_launch: true,
            notes: String::new(),
            roles: default_roles(),
        }
    }
}

impl TeamSettings {
    pub fn path(paths: &Paths) -> PathBuf {
        paths.join_config("settings.toml")
    }

    pub fn load(paths: &Paths) -> Result<Self> {
        let mut s: Self = read_toml(&Self::path(paths))?.unwrap_or_default();
        // Ensure all stock roles exist even if an older settings file omitted some.
        for stock in default_roles() {
            if !s.roles.iter().any(|r| r.id == stock.id) {
                s.roles.push(stock);
            }
        }
        Ok(s)
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        write_toml(&Self::path(paths), self)
    }

    pub fn role_mut(&mut self, id: RoleId) -> Option<&mut RoleSpec> {
        self.roles.iter_mut().find(|r| r.id == id)
    }

    pub fn role(&self, id: RoleId) -> Option<&RoleSpec> {
        self.roles.iter().find(|r| r.id == id)
    }

    pub fn launch_roles(&self) -> Vec<&RoleSpec> {
        self.roles
            .iter()
            .filter(|r| match r.id {
                RoleId::Master => true,
                RoleId::Orchestrator | RoleId::Planner | RoleId::Coder => true,
                RoleId::Architect => self.include_architect_on_launch,
                RoleId::Reviewer => self.include_reviewer_on_launch,
                RoleId::Tester => self.include_tester_on_launch,
            })
            .collect()
    }
}

pub fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            return format!("{home}/{rest}");
        }
    }
    if path == "~" {
        return std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| path.to_string());
    }
    path.to_string()
}

#[allow(dead_code)]
pub fn sanitize_name(name: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn confirmation_requires_pending() {
        let mut s = Session::default();
        s.working_dir = "/tmp/demo".into();
        s.project_name = "demo".into();
        let block = s.confirmation_block();
        assert!(block.contains("(nothing queued yet)"));
        s.queue_prompts(vec!["add auth".into(), "write tests".into()], "prio high");
        let block = s.confirmation_block();
        assert!(block.contains("1) add auth"));
        assert!(block.contains("Do you want me to implement"));
        let confirmed = s.confirm_pending().unwrap();
        assert_eq!(confirmed.prompts.len(), 2);
        assert!(s.pending.is_none());
    }

    #[test]
    fn settings_round_trip() {
        let dir = tempdir().unwrap();
        let paths = Paths {
            config_dir: dir.path().join("cfg"),
            state_dir: dir.path().join("state"),
        };
        std::fs::create_dir_all(&paths.config_dir).unwrap();
        let mut s = TeamSettings::default();
        s.nudge_idle_secs = 42;
        s.save(&paths).unwrap();
        let loaded = TeamSettings::load(&paths).unwrap();
        assert_eq!(loaded.nudge_idle_secs, 42);
        assert!(loaded.role(RoleId::Master).is_some());
    }
}
