# Configure Dev Team

Paths:

```bash
herdr plugin config-dir dev-team
```

Typical layout:

```text
settings.toml          # Full Config (+ role prompts)
secrets.json           # PATs for Lazy Git
git/accounts.toml
session.toml           # under state dir — Master session
```

## Full Config (guided)

Open **Full Config** from the main menu. Prefer the **Setup wizard** the first time:

1. Project name + default clone dir  
2. Agent kind + preferred git provider  
3. Board auto-open + nudge/stuck timeouts + Master always-confirm  

Then drill into sections for deeper edits:

| Section | Edits |
|---|---|
| Session defaults | agent kind, clone dir, workspace label prefix, which roles launch |
| Loop / nudge | idle nudge secs, stuck timeout, max inflight tickets, auto card tags |
| Board | plugin id/entrypoint, open on launch |
| Git | preferred provider + clone dir |
| Role prompt studio | long-form master prompts per role |
| Safety | `master_always_confirm`, destructive confirm, notes |
| Paths | config/state locations + Herdr keybind sample |

## Master session

State file (plugin state dir): `session.toml`

Fields of interest:

- `working_dir` / `project_name` / `git_remote` — set by Lazy Git clone or Master → Set environment  
- `pending` — queued human prompts awaiting confirmation  
- `loop_state` — `stopped` \| `running` \| `paused`  
- `history` — confirmed / rejected summaries  

Master keys inside the Master pane: `y` confirm · `n` reject.

## Board conventions

Orchestrator should title cards like:

```text
[Coder][order:3][depends:1,2] Implement auth middleware
```

Suggested columns: Backlog · Todo · Doing · Review · Test · Done · Blocked

## Agent kinds

Set `default_agent_kind` / session `agent_kind` to whatever Herdr supports on your machine (`codex`, `cursor`, `claude`, …).
