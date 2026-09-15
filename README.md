# Dev Team

### One [Herdr](https://herdr.dev) plugin for a Master-gated multi-agent coding loop

## Open **Dev Team** and drive Lazy Git + live kanban + role agents from a split-pane menu.

#### **Plugin id:** `dev-team`
#### **Requires:** Herdr ≥ 0.9.0 · Rust 1.88+ / `cargo` for install builds · optional `herdr-board` for kanban

Inspired by [lazy-herd](https://github.com/pve-homelab/lazy-herd), but focused entirely on:

1. **Master** — human interface, project cwd via lazy git, **always confirms before dispatch**
2. **Orchestrator** — live kanban updates + nudge stuck agents
3. **Planner / Coder / Reviewer / Tester / Architect** — role agents that work tickets in a loop

## Install

```bash
herdr plugin install pve-homelab/Development-Team --yes
```

**Local / linked:**

```bash
bash scripts/build.sh              # Linux/macOS
# powershell -File scripts/build.ps1   # Windows
herdr plugin link .
```

## Run

```bash
# Linux / macOS
herdr plugin action invoke dev-team.open

# Windows
herdr plugin action invoke dev-team.open-windows
```

Optional keybind (`~/.config/herdr/config.toml`):

```toml
[[keys.command]]
key = "prefix+d"
type = "plugin_action"
command = "dev-team.open"
```

## Menu keys

| Key | Do |
|---|---|
| `↑` `↓` / `j` `k` | Move |
| `Enter` | Open |
| `?` | Docs |
| `Esc` | Back (or quit on main menu) |

## Control surface

| Feature | What it does |
|---|---|
| **Master** | Queue human prompts, set cwd/project, **show confirmation block**, `y`/`n` confirm/reject before any Dev Team dispatch, start/stop loop |
| **Launch Team** | Create multi-pane workspace (Master + Orchestrator + roles), seed prompts, optionally open kanban |
| **Agent Loop** | Start / pause / stop loop; mirror nudge + stuck timeouts to Orchestrator |
| **Team Roles** | Skill cards + editable master prompts for every role |
| **Lazy Git** | Forge accounts + clone → writes session cwd for Master/Launch |
| **Kanban Board** | Opens `herdr-board` with Dev Team card conventions (`agent:` / `order:` / `depends:`) |
| **Secrets** | Masked PATs for Lazy Git `secret_ref` |
| **Full Config** | Deep guided menu + multi-step setup wizard (session, loop, board, git, role prompts, safety) |
| **Doctor** | Health check (herdr, git, board, session) |

Config dir: `herdr plugin config-dir dev-team`

## Recommended flow

1. **Full Config → Setup wizard** (agent kind, clone dir, board, nudge policy, Master gate)
2. **Secrets** → store forge PAT
3. **Lazy Git** → account + clone (sets session `working_dir` / project)
4. **Master** → queue goals → review confirmation block → `y` to dispatch
5. **Launch Team** → panes + board
6. Watch **Orchestrator** move kanban cards while role agents loop

### Master confirmation (hard rule)

Whenever you promote instructions, Master must ask first:

```text
Hey — you have told me:
  1) …
Working environment:
  - repo/path: …
  - project: …
  - loop: …
Do you want me to implement / dispatch this to the Dev Team? (yes / no)
```

Only after **yes** does Master relay work to Orchestrator / the team.

## Role starting point

| Agent | Primary | Secondary | Output |
|---|---|---|---|
| Master | Session control | Human gate | Confirmed directives |
| Orchestrator | Kanban + loop | Unstick agents | Live board + nudges |
| Planner | Task breakdown | Requirements | Tickets, roadmap |
| Coder | Code writing | Docs | Code modules |
| Reviewer | Code review | Security | Review reports |
| Tester | Test creation | Debugging | Test suites, logs |
| Architect | System design | Standards | Diagrams, guidelines |

## CLI

```bash
dev-team version
dev-team doctor
dev-team doctor --json
```
