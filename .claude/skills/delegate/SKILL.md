---
name: delegate
description: Hands a task to a fresh Claude session in a new herdr worktree based on
  origin/main, which runs `/develop` on it, without interrupting the current session.
argument-hint: "[task description]"
allowed-tools: Bash
disable-model-invocation: true
---

```
/delegate <task description>   → new herdr worktree + Claude session running /develop <task>
```

Run every command below with the Bash tool (the checks assume a POSIX shell).

`herdr --skill` prints herdr's official skill text. It and
`herdr <group> --help` (e.g. `herdr worktree --help`, `herdr agent --help`) are
the source of truth for command syntax; if a command fails, re-check them
rather than guessing. **Never run a mutating herdr command with its arguments
omitted to "see the help"**: `herdr worktree create` runs with defaults even
with no arguments. Use `herdr worktree create --help` instead.

Read JSON field names from the actual response, not from memory. If a field
named below is missing, inspect the response and use the field that carries the
same value.

The new session picks its own branch through `/develop`, so this skill does not
pass `--branch`.

Follow these steps in order.

## 1. Check that the session runs inside herdr

```bash
test "${HERDR_ENV:-}" = 1
```

If it fails, tell the user the session is not running inside Herdr and stop.

## 2. Create the worktree from the latest main

```bash
herdr worktree list
git -C <main checkout> fetch origin main
herdr worktree create --cwd <main checkout> --base origin/main --no-focus
```

`<main checkout>` is `.result.source.source_checkout_path` of
`herdr worktree list`. The fetch only updates remote refs, so the main checkout
itself is left alone and `origin/main` is current.

From the JSON `herdr worktree create` prints, read:

- `.result.root_pane.pane_id`: the pane the agent starts in
- `.result.workspace.worktree.checkout_path`: the worktree directory (its last
  component is the worktree name, e.g. `worktree-brave-forest-9d98`)
- the workspace ID, for the report

## 3. Start Claude in the new pane

```bash
herdr agent list
herdr agent start <name> --kind claude --pane <pane_id> --timeout 120000
```

`<name>` is a short name describing the task (e.g. `burst-badge`). It must match
`[a-z][a-z0-9_-]{0,31}` and be unique among the live agents `herdr agent list`
shows. The default 30000 ms timeout can be too short for Claude to start in a
fresh worktree.

## 4. Hand over the task

```bash
herdr agent prompt <name> "/develop <task description>"
```

Do **not** pass `--wait`: `/develop` runs for a long time.

The new session shares none of this conversation's context, so the description
must stand alone: the symptom, its cause if known, the related files, and the
evidence (e.g. "the user confirmed it on real hardware"). Expand the
`$ARGUMENTS` the user gave with what this conversation already established.

## 5. Report

```bash
echo "worktree: <worktree name>, workspace: <workspace ID>, agent: <name>"
```

Tell the user the worktree name, the workspace ID, and the agent name.

Report the result of each step to the user.
