# Learnings

## Step 1

- `choose_format` (`crates/app/src/commands.rs`) calls `switch_format` with a
  no-op `persist` and then persists itself, so the format is saved exactly
  once whether or not it changed, and a save failure is returned (the dialog
  stays up) instead of being logged as `switch_format` does. `switch_format`'s
  early return is untouched.
- The "is it saved" decision is a pure `format_saved(Result<bool, String>)`
  so the broken-store case (counts as saved) is unit-testable without a
  Tauri app; `sidecar_format_saved` only feeds it `store.has("sidecarFormat")`.
- The frontend gate is `FormatGate` (`crates/app/ui/src/firstrun.ts`): closed
  at startup, it holds the one deferred action (`reopenLastFolder`, chained
  after `sort_order`) and runs it on `open()`. The `sidecar_format_saved`
  invoke and the `sort_order` chain resolve in either order; `whenOpen`
  covers both.
- The main window's `keydown` handler returns early (without
  `preventDefault`) while the dialog is up, so Tab / Enter / Space still reach
  the dialog's buttons natively and no app shortcut acts. Tab can still move
  focus to the `Open folder` button behind the overlay (no focus trap / `inert`,
  given the `safari13` build target), but the gate makes it a no-op.
- The `(default)` marker on Lightroom (XMP) was dropped in README "Working
  with other software" and `docs/usage.md`: with the dialog there is no
  format a user gets without choosing, and the fallback for an unknown stored
  value is an implementation detail.
