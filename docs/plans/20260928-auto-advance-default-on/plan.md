<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Auto-advance on by default

## Purpose

`Auto-advance after a star, reject or pick` (settings modal, `autoAdvance`
key in the settings store) is off unless the user turns it on. It is the
convenient way to cull, so a fresh install should start with it on. A user who
explicitly saved `false` keeps it off; only an absent (or non-boolean) value
changes meaning.

## Steps

- [x] Step 1: Flip the `autoAdvance` default to on
  - Done when:
    - With no `autoAdvance` key in the settings store the app launches with
      auto-advance on: `auto_advance` returns `true`, the settings checkbox is
      checked, and a `1`-`5` / reject / pick that changes the current file
      advances.
    - A stored `autoAdvance: false` still yields off; a stored `true` still
      yields on.
    - The Rust unit test asserts the new default; `docs/usage.md` says "on by
      default"; `mise run ci` passes.
  - Implementation approach:
    - The default is defined in exactly one place: `auto_advance_setting` in
      `crates/app/src/commands.rs` (~line 540), `value.and_then(Value::as_bool).unwrap_or(false)`.
      Change `unwrap_or(false)` to `unwrap_or(true)` and update its doc
      comment ("missing or non-boolean means off" -> "on"). `load_settings`
      already calls it with `None` on a store-open failure and with
      `store.get("autoAdvance")` otherwise, so both the absent-key and the
      broken-store paths pick up the new default; `main.rs` seeds
      `AppAutoAdvance` from that value and the frontend (`main.ts` ~line 2648,
      `settings.ts` ~line 207) reads it through `invoke("auto_advance")`, so
      nothing else needs to change for the behavior.
    - Rename and update the test `auto_advance_setting_reads_a_boolean_or_defaults_to_off`
      (`commands.rs` ~line 2424) to `..._defaults_to_on`: `None` and the
      non-boolean `json!("true")` now assert `true`; `json!(false)` still
      asserts `false`.
    - Docs: `docs/usage.md` line 264, "(off by default)" -> "(on by
      default)". `README.md` / `README.ja.md` do not mention auto-advance
      (the happy-path README rewrite removed it), so they stay untouched.
      `docs/agents/tauri-app.md` describes the mechanism, not the default,
      so it also stays untouched.
    - Leave `let autoAdvance = false;` in `crates/app/ui/src/main.ts`
      (~line 2647) and the unchecked `<input id="auto-advance">` in
      `index.html` as they are: both are pre-`invoke` placeholders that the
      backend value overwrites within the first tick (see Trade-offs).
    - Commit: `feat(app): turn auto-advance on by default`.

## Trade-offs and risks

- **Frontend placeholder before `invoke("auto_advance")` resolves.**
  `main.ts` initializes `autoAdvance = false` and the HTML checkbox is
  unchecked until the backend answers. Option A (chosen, surgical): leave
  them; a key press in the few milliseconds before the promise resolves
  would not advance, which is unobservable in practice, and the checkbox is
  only visible once the modal opens. Option B: flip the placeholder to `true`
  / add `checked` so the pre-resolution state matches the new default; it
  duplicates the default in a second place.
- **Non-boolean stored values** (e.g. `"true"` as a string) now mean on
  rather than off. The only writer is `set_auto_advance`, which stores a
  real boolean, so this only affects hand-edited `settings.json`.
- **Existing users who never touched the setting** will see auto-advance
  start firing after the update. That is the requested behavior; the
  CHANGELOG entry (generated from the `feat` commit) is the notice.

## Progress

- (2026-09-28) Step 1 complete
