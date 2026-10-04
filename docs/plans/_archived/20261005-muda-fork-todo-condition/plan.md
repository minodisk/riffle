<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Refresh the muda template-icon fork exit condition in todo.md

## Purpose

The "App: the muda template-icon fork is a temporary bridge" section of
`todo.md` still states the fork's exit condition as "once muda#413 ships in a
muda release that Tauri resolves under its `muda = "^0.19"`". That is stale:
muda#413 merged on 2026-09-29 and shipped in muda v0.21.0 (2026-09-30), and
the newest Tauri v2 (tauri-v2.12.1) depends on `muda = "0.20"`, which cannot
resolve 0.21, while its menu-item API (`crates/tauri/src/menu/icon.rs`,
`submenu.rs` on the dev branch) still exposes no template flag (only tray
icons have one). Rewriting the condition to what actually remains (a Tauri v2
release on muda >= 0.21 **and** a Tauri template flag for menu items) keeps
the todo actionable, so the fork is dropped at the right moment and not
checked against a condition that can never become true. The fork removal
itself is not part of this work.

## Steps

- [x] Step 1: Rewrite the exit condition and background of the muda fork todo section
  - Done when:
    - In `todo.md`, section "App: the muda template-icon fork is a temporary
      bridge", the first TODO bullet reads to the effect of: once a Tauri v2
      release depends on muda >= 0.21 (where muda#413's
      `set_icon_as_template` lives) **and** Tauri exposes the template flag
      for menu items, drop the `[patch.crates-io]` entry and switch to the
      upstream opt-in API. No reference to `muda = "^0.19"` remains in the
      section.
    - The section's intro paragraph (or a short note next to the bullet)
      records the verified facts, dated 2026-10-05: muda#413 merged
      2026-09-29 and shipped in muda v0.21.0 (2026-09-30); Tauri v2.12.1
      (the latest v2) depends on `muda = "0.20"` so does not resolve 0.21;
      Tauri's menu-item API still has no template flag (template exists only
      for tray icons); the workspace is still on tauri 2.11.6 / fork muda
      0.19.3.
    - The second bullet (re-verify tinting after the switch) and the
      "Related:" line are unchanged in meaning.
    - No other `todo.md` section, `Cargo.toml`, `Cargo.lock`,
      `docs/agents/tauri-app.md` or code changes (the user chose to keep the
      matching stale prose in `docs/agents/tauri-app.md` out of scope).
    - `mise run lint` passes (lychee checks the links in `todo.md`; the
      muda#413 URL stays a valid link).
  - Implementation approach:
    - Edit only the section between
      `### App: the muda template-icon fork is a temporary bridge` and
      `### App: real-device checks for the View menu`.
    - Keep the section structure (intro paragraph, `#### TODO`, two bullets,
      `Related:` line) and the existing wrapped prose style.
    - The mention "against muda `dev`" in the intro describes the PR as open;
      rewrite it to say it merged and shipped in v0.21.0.
    - Everything in English.

## Trade-offs and risks

- The same stale condition exists in the comment above `[patch.crates-io]` in
  the workspace `Cargo.toml` and in `docs/agents/tauri-app.md`. Both stay as-is
  in this work (user decision); the todo is the source of truth for the
  condition.

## Progress

- (2026-10-05) Step 1 complete
