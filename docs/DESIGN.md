# Dev Team — Design Spec

**Date:** 2026-09-15  
**Status:** Implemented (v0.1.0)

## Problem

Lazy Herd bundles many workflows. This plugin narrows to a **Master-gated coding loop** with live kanban visualization and lazy-git project setup so a human can watch role agents work tickets without accidentally launching unconfirmed work.

## Goals

1. Herdr plugin `dev-team` with split-pane TUI (lazy-herd style).
2. Master always confirms before dispatching instructions to the Dev Team.
3. Orchestrator owns kanban + idle/stuck nudges.
4. Role agents: Planner, Coder, Reviewer, Tester, Architect (+ Master, Orchestrator).
5. Lazy Git clone writes session cwd used by Launch / Master.
6. Full Config is deeper and more guided than Lazy Herd’s config (wizard + section studio).

## Architecture

```
herdr-plugin.toml → popup pane → ./bin/dev-team
App → PluginRegistry of SubPlugins
Session (state) + TeamSettings (config) under Herdr plugin dirs
```

## Master gate

Pending directives live in `session.toml`. Master renders a confirmation block listing prompts + working environment and only calls `herdr agent prompt` for Orchestrator after explicit yes.

## Success criteria

- `cargo test` + `cargo build --release` produce `bin/dev-team`
- Menu exposes Master, Launch, Loop, Roles, Git, Board, Secrets, Full Config, Doctor, Docs
- Confirmation block copy matches the hard-rule wording in README
