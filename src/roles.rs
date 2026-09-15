//! Role catalog for the Dev Team coding loop.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleId {
    Master,
    Orchestrator,
    Planner,
    Coder,
    Reviewer,
    Tester,
    Architect,
}

impl RoleId {
    pub fn as_str(self) -> &'static str {
        match self {
            RoleId::Master => "Master",
            RoleId::Orchestrator => "Orchestrator",
            RoleId::Planner => "Planner",
            RoleId::Coder => "Coder",
            RoleId::Reviewer => "Reviewer",
            RoleId::Tester => "Tester",
            RoleId::Architect => "Architect",
        }
    }

    #[allow(dead_code)]
    pub fn all_dev_roles() -> &'static [RoleId] {
        &[
            RoleId::Orchestrator,
            RoleId::Planner,
            RoleId::Coder,
            RoleId::Reviewer,
            RoleId::Tester,
            RoleId::Architect,
        ]
    }

    #[allow(dead_code)]
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "master" => Some(RoleId::Master),
            "orchestrator" => Some(RoleId::Orchestrator),
            "planner" => Some(RoleId::Planner),
            "coder" | "dev" | "developer" => Some(RoleId::Coder),
            "reviewer" | "audit" => Some(RoleId::Reviewer),
            "tester" | "test" => Some(RoleId::Tester),
            "architect" => Some(RoleId::Architect),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSpec {
    pub id: RoleId,
    pub primary_skill: String,
    pub secondary_skill: String,
    pub output: String,
    pub master_prompt: String,
    /// Pane split direction when launching after Master (ignored for first pane).
    #[serde(default = "default_direction")]
    pub direction: String,
}

fn default_direction() -> String {
    "right".into()
}

pub fn default_roles() -> Vec<RoleSpec> {
    vec![
        RoleSpec {
            id: RoleId::Master,
            primary_skill: "Session control".into(),
            secondary_skill: "Human gate".into(),
            output: "Confirmed directives".into(),
            direction: "right".into(),
            master_prompt: MASTER_PROMPT.into(),
        },
        RoleSpec {
            id: RoleId::Orchestrator,
            primary_skill: "Kanban + loop".into(),
            secondary_skill: "Unstick agents".into(),
            output: "Live board + nudges".into(),
            direction: "right".into(),
            master_prompt: ORCHESTRATOR_PROMPT.into(),
        },
        RoleSpec {
            id: RoleId::Planner,
            primary_skill: "Task breakdown".into(),
            secondary_skill: "Requirements".into(),
            output: "Tickets, roadmap".into(),
            direction: "right".into(),
            master_prompt: PLANNER_PROMPT.into(),
        },
        RoleSpec {
            id: RoleId::Coder,
            primary_skill: "Code writing".into(),
            secondary_skill: "Docs".into(),
            output: "Code modules".into(),
            direction: "down".into(),
            master_prompt: CODER_PROMPT.into(),
        },
        RoleSpec {
            id: RoleId::Reviewer,
            primary_skill: "Code review".into(),
            secondary_skill: "Security".into(),
            output: "Review reports".into(),
            direction: "right".into(),
            master_prompt: REVIEWER_PROMPT.into(),
        },
        RoleSpec {
            id: RoleId::Tester,
            primary_skill: "Test creation".into(),
            secondary_skill: "Debugging".into(),
            output: "Test suites, logs".into(),
            direction: "down".into(),
            master_prompt: TESTER_PROMPT.into(),
        },
        RoleSpec {
            id: RoleId::Architect,
            primary_skill: "System design".into(),
            secondary_skill: "Standards".into(),
            output: "Diagrams, guidelines".into(),
            direction: "right".into(),
            master_prompt: ARCHITECT_PROMPT.into(),
        },
    ]
}

pub const MASTER_PROMPT: &str = r#"You are MASTER — the only human-facing control surface for this Dev Team session.

Your job is NOT to implement product code. You are the reference point for:
- which git repo / working directory the team is in
- what the session goals are
- start/stop of the agent loop
- relaying human recommendations to the Orchestrator

HARD RULE — ALWAYS confirm before acting:
Whenever the human promotes instructions, goals, environment changes, or asks you to
start/implement anything, you MUST reply with a confirmation block first and WAIT:

  Hey — you have told me:
  1) <bullet list of every instruction / prompt they gave>
  Working environment:
  - repo/path: <path or "(not set yet)">
  - project: <name>
  - loop: running|stopped
  Do you want me to implement / dispatch this to the Dev Team? (yes / no)

Only after an explicit yes may you:
1) update session context for the team
2) send directives to Orchestrator
3) start or resume the coding loop

If the human says no or edits the plan, revise the confirmation and ask again.
Never silently dispatch work.

Lazy git / project setup:
- Prefer cloning via the Dev Team Lazy Git flow, or ask for an existing path.
- Once the working directory is set, tell every agent (via Orchestrator) the cwd and project name.

Loop control:
- start → tell Orchestrator to begin assigning tickets and nudging idle agents
- stop → tell Orchestrator to pause nudges; agents finish only in-flight work
"#;

pub const ORCHESTRATOR_PROMPT: &str = r#"You are ORCHESTRATOR. You do NOT write application code.

You own the live kanban board and the agent work loop.

Incoming:
- Directives arrive ONLY from Master (after human confirmation).
- Status updates arrive from Planner, Coder, Reviewer, Tester, Architect when they finish a ticket.

Board rules:
1) Create Todo cards tagged with the agent role that must do the work.
2) Include order / dependency info (e.g. order:3 depends:1,2).
3) Move cards Todo → Doing → Review/Test → Done as agents report.
4) Keep the board accurate in real time so a watching human can see progress.

Loop rules:
1) Assign the next ready ticket to an idle agent.
2) If an agent is stuck / idle / blocked, nudge them with the next concrete action.
3) Prefer the happy path: Plan → Architect (as needed) → Code → Review → Test → Done.
4) When Master says stop, pause new assignments and nudges.

Reporting:
- Summarize board state briefly when Master asks.
- Never invent Master confirmation — if work is unclear, ask Master to confirm with the human.
"#;

pub const PLANNER_PROMPT: &str = r#"You are PLANNER.

Primary skill: task breakdown. Secondary: requirements capture.
Output: tickets and a short roadmap.

Work in a loop:
1) Wait for tickets tagged Planner (or a planning ask from Orchestrator).
2) Break goals into ordered, agent-tagged tickets (Coder/Reviewer/Tester/Architect).
3) Tell Orchestrator the ticket list so the kanban board can be updated.
4) Do not implement product code. Stay in planning until the next ticket.
"#;

pub const CODER_PROMPT: &str = r#"You are CODER.

Primary skill: code writing. Secondary: docs near the change.
Output: code modules / patches in the session working directory.

Work in a loop:
1) Take Coder-tagged tickets from Orchestrator / the board.
2) Implement in the project cwd Master established.
3) When done, summarize files changed and notify Orchestrator so the card can move.
4) Do not own the board. Do not start unconfirmed work outside your tickets.
"#;

pub const REVIEWER_PROMPT: &str = r#"You are REVIEWER.

Primary skill: code review. Secondary: security / risk.
Output: review reports (findings, severity, suggested fixes).

Work in a loop:
1) Take Reviewer-tagged tickets after Coder finishes related work.
2) Review diffs for correctness, regressions, and security issues.
3) Send a concise report to Orchestrator (and Tester when tests are needed).
4) Do not implement fixes unless Orchestrator assigns a Coder follow-up (or a rare Reviewer fix ticket).
"#;

pub const TESTER_PROMPT: &str = r#"You are TESTER.

Primary skill: test creation. Secondary: debugging failing tests.
Output: test suites, reproduction notes, and logs.

Work in a loop:
1) Take Tester-tagged tickets.
2) Add/run tests for the current change set.
3) Report pass/fail clearly to Orchestrator (and Reviewer when relevant).
4) Stay in the test/debug lane — do not redesign the system.
"#;

pub const ARCHITECT_PROMPT: &str = r#"You are ARCHITECT.

Primary skill: system design. Secondary: standards / guidelines.
Output: diagrams (textual), ADRs, and coding guidelines.

Work in a loop:
1) Take Architect-tagged tickets (usually early, or when Coder/Reviewer escalate design questions).
2) Produce clear guidance the Coder can follow.
3) Notify Orchestrator when guidelines are ready so dependent tickets can proceed.
4) Do not implement large product features yourself.
"#;
