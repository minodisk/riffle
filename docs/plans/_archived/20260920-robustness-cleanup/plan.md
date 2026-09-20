<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Robustness and cleanup: sidecar error keys, XMP prefix collisions, stale shortcut entries, format-switch race test

## Purpose

Four small items from `todo.md`, each independent, each closing one heading:

1. A sidecar that fails to read on open and then fails to write shows two
   sticky errors instead of one superseding the other, because the read side
   keys the error by the sidecar path and the write side by the RAW path.
2. `xmp_prefix` in `crates/core/src/xmp.rs` can rebind a sidecar's `xmp`
   prefix to the XMP namespace when the sidecar had bound it to something
   else, corrupting the meaning of that tag's other attributes.
3. Modifier-only shortcut entries (`control`, `shift`, `alt`, `meta`) that an
   earlier bug let into the `shortcuts` store are never cleaned up.
4. The deferred `switch_sidecar_format` race test (a rating set between the
   format swap and `reset_sidecars`) was blocked on the function taking a
   `tauri::AppHandle`. Investigation at planning time found that extracting
   the body into a plain function is enough; no `tauri::test` mock-app
   harness is needed (see "Background" below).

Every step's wrap-up drops the corresponding `todo.md` heading.

## Background (from investigation)

- Frontend keying: `crates/app/ui/src/errors.ts` is a `Map<key, message>`;
  `crates/app/ui/src/main.ts` calls ``errors.add(path, `${baseName(path)}: ${message}`)``
  for both `ScanStarted.sidecar_errors` (line ~1057) and the `sidecar-error`
  event (line ~1188). Only the `path` field decides whether two errors
  coexist. The `sidecar-error` payload in `crates/app/src/main.rs` (~line
  390) is emitted with the RAW path handed to the writer's `on_error`; the
  message from `sidecar::write` already contains the sidecar's full path
  (`"{target.display()}: {e}"`).
- `reconcile_sidecars_of` (`crates/app/src/commands.rs:222`) iterates
  `to_parse` as `(path, (sidecar, size, mtime_ns), dirty)` where `path` is
  the RAW path and `sidecar` the sidecar `PathBuf`; the `problem` closure
  (line 255) uses `sidecar`. Two tests assert `problems[0].path` equals the
  sidecar path: `an_oversize_dop_is_reported_and_left_unread` (~line 1847)
  and `an_unparseable_sidecar_is_reported_on_every_open_and_leaves_the_rating`
  (~line 1881).
- `xmp_prefix` (`crates/core/src/xmp.rs:255`) calls
  `reader.resolver().resolve_attribute(QName("xmp:Rating"))`; quick-xml 0.42's
  `ResolveResult` is `Bound(Namespace)` (prefix declared), `Unbound` (no
  prefix) or `Unknown(String)` (prefix not declared). The current code
  returns `(candidate, false)` on `Bound(XMP_NS)` and otherwise falls through
  to `("xmp", true)`, which `set` turns into ` xmlns:xmp="..."` on the first
  `rdf:Description` start tag. Existing tests covering this:
  `declares_the_namespace_when_the_description_has_none`, `handles_the_xap_prefix`.
- `Keymap::from_overrides` (`crates/app/src/shortcuts.rs:226`) parses each
  action's stored list through `parse_keys` (line 403: non-empty array of
  non-empty strings, else `None` -> default kept), rejects `forbidden` keys,
  then applies overrides order-independently. A conflicting override is kept
  verbatim in `Keymap::inactive` and re-emitted by `overrides()`. The
  frontend's guard is `MODIFIER_KEYS = ["control", "alt", "shift", "meta"]`
  in `crates/app/ui/src/keys.ts:5`.
- `switch_sidecar_format` (`crates/app/src/commands.rs:387`) takes
  `AppSwitchLock`, early-returns if `AppSidecarFormat` already matches,
  writes the new format, `writer.flush(DRAIN_TIMEOUT)`, saves `sidecarFormat`
  to the store, then `index.reset_sidecars()`. Every dependency except the
  store save is a plain type tests already build (`sidecar.rs` tests have
  `index()`, `writer()`, `eventually()`; `commands.rs` tests have
  `temp_dir`, `sidecar_index`, `rating_of`).
- `Index::reset_sidecars` (`crates/app/src/index.rs:726`) deletes clean rows
  and runs `UPDATE ratings SET xmp_size = NULL, xmp_mtime_ns = NULL, pick = 0
  WHERE dirty = 1`. `Index::mark_written` (line 690) clears `dirty` only
  `WHERE ... rating IS ?4 AND pick = ?5 ...`. So a judgement with `pick =
  true` made during a switch to `.dop` would have its row's pick zeroed before
  the writer lands it, `mark_written` would not match, the row stays dirty
  with `pick = 0`, and the next open replays it over the sidecar's pick.
  Rating-only judgements are unaffected. This is unverified; Step 4's test
  is where it gets checked.

## Steps

- [x] Step 1: Key sidecar read errors by the RAW path so they supersede write errors for the same file
  - Done when:
    - `reconcile_sidecars_of`'s `SidecarError.path` is the RAW path (the
      `path` of the `to_parse` tuple), and its `message` starts with the
      sidecar's file name (e.g. `a.xmp: larger than 4 MiB, not read`,
      `a.ARW.dop: <parse error>`), so the pane still says which sidecar
      failed.
    - The two existing tests that assert `problems[0].path` now assert the
      RAW path (`listed[0]`) and additionally assert the message contains the
      sidecar's file name.
    - A comment on `SidecarError` in `commands.rs` states that `path` is the
      RAW path and why (it must match the `sidecar-error` event's key so the
      frontend's `ErrorList` supersedes one with the other).
    - Verified, and stated in the PR description, that no frontend change is
      needed: `main.ts` uses `payload.path` / `path` only as the key and for
      `baseName(path)`, and `errors.test.ts` keys are opaque strings.
    - `mise run ci` passes.
    - Wrap-up drops the `todo.md` heading "App: sidecar read and write errors
      for the same file show as two separate entries".
  - Implementation approach:
    - Change only the `problem` closure in `reconcile_sidecars_of`: `path:
      path.clone()` and prefix the message with
      `sidecar.file_name().map(|n| n.to_string_lossy())` (fall back to the
      whole sidecar path if there is no file name). Do not touch
      `crates/app/src/main.rs`'s `sidecar-error` payload or the writer's
      messages.
    - Do not rename the `path` field: the frontend and the
      `scan_started_serialises_the_fields_the_frontend_reads` test read it.

- [x] Step 2: Make `xmp_prefix` skip a prefix bound to another namespace and fall back to a generated one
  - Done when:
    - For each candidate in `["xmp", "xap"]`, `xmp_prefix` returns
      `(candidate, false)` on `Bound(XMP_NS)`, `(candidate, true)` on
      `Unknown` (undeclared, safe to declare), and skips the candidate on
      `Bound(other)`. When both are skipped it returns `(generated, true)`
      where `generated` is the first of `xmp1`, `xmp2`, ... whose
      `resolve_attribute` result is `Unknown`.
    - New test: a sidecar whose `rdf:Description` binds `xmlns:xmp` to a
      non-XMP namespace and binds nothing to the XMP namespace. After
      `write_rating(.., Some(n))`: the original `xmlns:xmp="..."` is still
      present and unchanged, the output declares `xmlns:xap="http://ns.adobe.com/xap/1.0/"`
      and writes `xap:Rating="n"`, and `read_rating` reads `n` back.
    - New test: both `xmp` and `xap` bound to other namespaces. After the
      patch, both original bindings are unchanged, a generated prefix (assert
      `xmp1` since nothing else is bound) is declared and used, and
      `read_rating` reads the value back. Optionally also assert the second
      patch of the same output is a pure value replacement (no second
      declaration).
    - Existing xmp tests (`declares_the_namespace_when_the_description_has_none`,
      `handles_the_xap_prefix`, the label tests) still pass unchanged.
    - `xmp_prefix`'s doc comment describes the three cases.
    - `mise run ci` passes.
    - Wrap-up drops the `todo.md` heading "Core: `xmp_prefix` can rebind a
      namespace prefix already used for something else".
  - Implementation approach:
    - Stay inside `xmp_prefix`; `locate` and `set` need no change since they
      already take `(prefix, declare)`.
    - Keep using `reader.resolver().resolve_attribute(QName(..))` (the
      established way to ask the resolver at the current element; see the
      `docs/agents/core.md` todo note about quick-xml 0.42 facts). Match on
      `ResolveResult` directly instead of only `bound_to`.
    - Bound the generated-prefix loop (e.g. `1..=99`) and fall back to the
      last candidate rather than looping forever; a sidecar with 99 `xmpN`
      bindings is not a real input.

- [x] Step 3: Drop modifier-only shortcut entries when the `shortcuts` setting is read
  - Done when:
    - `Keymap::from_overrides` ignores any key that is exactly `control`,
      `shift`, `alt` or `meta` in a stored override list before the list is
      validated: `["control"]` leaves the action on its default; `["meta",
      "q"]` applies as `["q"]`; a list that equals the default after
      filtering is treated as no override.
    - `Keymap::overrides()` after such a load no longer contains the filtered
      key, so the next save (any `update_keymap`) drops it from the store
      without a migration. Re-loading the saved value gives the same keymap
      (idempotent).
    - Tests in `shortcuts.rs`'s test module cover: modifier-only list ->
      default kept; mixed list -> only the real key applied and `overrides()`
      emits only the real key; a conflicting override with a bogus key ->
      the value kept in `inactive`/`overrides()` is the filtered list (see
      trade-off below).
    - `mise run ci` passes.
    - Wrap-up drops the `todo.md` heading "App: clean up bogus modifier-only
      shortcut entries left in the store".
  - Implementation approach:
    - Add a `const MODIFIER_ONLY: [&str; 4] = ["control", "alt", "shift", "meta"]`
      next to `forbidden` with a comment that it mirrors `MODIFIER_KEYS` in
      `crates/app/ui/src/keys.ts` and the two must stay in sync.
    - Filter inside `parse_keys` (retain non-modifier keys, then apply the
      existing empty check) so every consumer sees the cleaned list, and log
      at `warn` when something was dropped, matching the other
      `log::warn!("ignoring the shortcut for ...")` lines.
    - Because `from_overrides` stores `value.clone()` (the raw JSON) in
      `inactive` for a conflicting override, store `Value::from(keys.clone())`
      (the filtered list) there instead, or the bogus key survives in that
      one case. Update the `overrides()` doc comment ("kept as it was") to
      say it is kept minus modifier-only keys.
    - No one-time migration, no store write at load time.

- [x] Step 4: Extract `switch_sidecar_format`'s body into a testable function and add the deferred race test
  - Done when:
    - `commands.rs` has a plain (non-`AppHandle`) function, e.g.
      `fn switch_format(current: &Mutex<SidecarFormat>, writer: Option<&Writer>, index: Option<&Arc<Mutex<Index>>>, format: SidecarFormat, persist: impl FnOnce(SidecarFormat) -> Result<(), String>) -> Result<(), String>`,
      that does the early-return check, the state write, the drain, calls
      `persist` (logging a failure as today), then `reset_sidecars`.
      `switch_sidecar_format(app, format)` becomes: take `AppSwitchLock`,
      look up the three states, call `switch_format` with a closure that
      saves `sidecarFormat` to the store. Behaviour is unchanged.
    - New test (in `commands.rs`'s test module, reusing `temp_dir`,
      `sidecar_index`, `list_arw_in`, `rating_of`; spawn a `Writer` as
      `sidecar.rs`'s tests do): start in `Xmp`, call `switch_format(..,
      Dop, persist)` where `persist` performs what `set_rating` does (read
      the current format from the `Mutex`, `index.set_rating(...)`,
      `writer.set(..., format)`) for one file with rating `Some(4)`. After
      the switch returns, `writer.flush(DRAIN_TIMEOUT)` and assert: the
      `.dop` sidecar exists and `SidecarFormat::Dop.read_rating` returns
      `Some(4)`; no `.xmp` was written; the row's rating is `Some(4)` and it
      is no longer dirty (`dirty_rows(dir)` is empty). Also assert the
      format observed inside `persist` was already `Dop` (the swap happens
      before the drain and save, as the doc comment promises).
    - The same scenario is run once with `pick = true` (the `.dop`-only
      field) and the outcome is recorded in `learnings.md`. If it passes,
      keep it as a second test. If it fails because of `reset_sidecars`'s
      `pick = 0` (see Background), **keep the PR to the rating-only test and
      add a new `todo.md` item** describing the pick loss with the exact SQL
      and the reproduction (the user chose this option at planning time; do
      not fix `reset_sidecars` in this PR).
    - `mise run ci` passes (the writer test uses `eventually`-style waiting
      or an explicit `flush`, never a fixed sleep).
    - Wrap-up drops the `todo.md` heading "App: no test harness for
      `tauri::AppHandle`-taking commands" (its premise, that a harness is
      needed, is no longer true; do not build one).
  - Implementation approach:
    - No `tauri::test`, no mock runtime, no `tauri_plugin_store` wiring. The
      `persist` closure is the injection point for the race; do not add a
      test-only hook or `#[cfg(test)]` branch to `switch_format`.
    - Keep the `AppSwitchLock` acquisition in the `AppHandle` wrapper; the
      extracted function assumes its caller serialises switches (say so in
      its doc comment).
    - The tests in `commands.rs` do not currently spawn a `Writer`; copy the
      pattern from `sidecar.rs` (`Writer::spawn(index, |path, message| eprintln!(..))`)
      rather than sharing helpers across modules.

## Trade-offs and risks

- Step 1 message asymmetry: the write-side message already includes the
  sidecar's full path (`sidecar::write`), so after this step a read error
  displays as `a.ARW: a.xmp: ...` and a write error as `a.ARW: /dir/a.xmp:
  ...`. Left as is (the todo item asks only for a shared key); trimming the
  write-side path to a file name would be a separate, unrelated change.
- Step 3 `inactive` value: the doc says a conflicting override is "kept as
  it was". Storing the filtered list instead is a small semantic change but
  is the only way the bogus key ever leaves the store in that case.
  Alternative: keep the raw value and accept that a bogus key inside a
  conflicting override lingers. The plan takes the filtered list.
- Step 4 pick case: the `reset_sidecars` `pick = 0` interaction may make the
  `pick = true` variant fail. Decided at planning time: keep the PR to the
  rating-only test the review asked for and record the pick finding as a new
  `todo.md` item, rather than widening this PR into sidecar semantics.
- Step 2 touches `crates/core/src/xmp.rs`, which is the trigger named in the
  `todo.md` item "Docs: consider a `docs/agents/core.md` guide". Not planned
  here to keep the step surgical; decide at wrap-up whether to create the
  guide (capturing the `ResolveResult` three-way match) or drop that item.
- Step 2 generated prefix choice: `xmp1`, `xmp2`, ... is arbitrary; any
  undeclared prefix is valid XML. Readers that only look for a literal `xmp:`
  prefix (not namespace-aware) would miss the value, but such readers also
  miss `xap:` today and the sidecar in question already had `xmp` bound
  elsewhere, so there is no better option.

## Progress

- Step 1: Sidecar read errors are now keyed by the RAW path in `crates/app/src/commands.rs`, so they supersede write errors for the same file (commit `a26aa38`, PR TBD).
- (2026-09-20) Step 1 complete
- (2026-09-20) Step 2 complete
- (2026-09-20) Step 3 complete
- (2026-09-20) Step 4 complete
