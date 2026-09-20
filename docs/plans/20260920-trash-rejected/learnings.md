# Learnings

## Step 1

- `trash` 5.2.9's macOS default delete method is `DeleteMethod::Finder`
  (`osascript` driving the Finder), which needs Automation permission and
  plays the Finder delete sound. `commands::trash_context` therefore pins
  `DeleteMethod::NsFileManager` through `trash::macos::TrashContextExtMacos`.
  The crate's own docs note the trade-off: `NsFileManager` needs no extra
  permission and is faster, but on some systems the Trash's "Put Back" entry
  is missing for files it moved (a macOS bug, trash-rs#14); the file can still
  be dragged out of the Trash, which restores the sidecar and so the
  judgement.
- `mod trash;` in `main.rs` shadows the `trash` crate inside `main.rs`, so the
  external crate is reached as `::trash::` (`commands.rs` uses
  `use crate::trash;` for the module and `::trash::TrashContext` for the
  crate).
- The case-insensitive-`exists()` pitfall (`docs/agents/tauri-app.md`) shows
  up in the test for it: on macOS `existing_sidecar` returns the minted
  `a.ARW.dop` for a file actually named `A.ARW.DOP`, because `exists()`
  already matches. The test therefore compares the sidecar name
  case-insensitively rather than expecting the on-disk spelling.

## Step 2

- `History` already had `remove(entry)` (by identity, used to undo a failed
  `set_rating`), but nothing to drop entries by path, so `removeWhere(match)`
  was added with a `undo.test.ts` case. Entries for a RAW that failed to move
  are kept: the file is still there and still judgeable.
- The pure parts live in `crates/app/ui/src/trash.ts` (`rejectedPaths`,
  `trashedStatus`) with `trash.test.ts`, as the plan asks; the event handler in
  `main.ts` stays a thin wiring layer.
- `trash_rejected` returns `Option<Summary>`, so the frontend's invoke is typed
  `invoke<TrashSummary | null>` and the `null` cancel is a plain early return.
- `NativeIcon::TrashFull` is the macOS menu icon; it renders as a template
  image like `FollowLinkFreestanding` does for `Open in DxO PhotoLab`. Not
  verifiable without a GUI run, so it stays part of Step 3's manual checks.

## Deferred issues (todo candidates)

- The "Put Back" promise in the plan's Purpose may not hold on macOS with
  `DeleteMethod::NsFileManager` (see above). Worth checking by hand during
  Step 3's manual verification, and wording `README.md` accordingly
  (`crates/app/src/commands.rs` `trash_context`, `README.md`).

## Step 3

- `README.md` promises only what the implementation guarantees: the files go to
  the OS Trash and nothing is unlinked, so restoring one from the Trash brings
  its sidecar and judgement back. It deliberately does not name Finder's "Put
  Back", because `DeleteMethod::NsFileManager` may leave no such entry
  (trash-rs#14); that uncertainty lives in the new `todo.md` manual-check item
  instead.
- No `docs/agents/tauri-app.md` entry was added: Steps 1 and 2 hit no pitfall
  beyond the `trash` crate's own macOS delete-method trade-off, which is
  specific to this command rather than a rule for the next feature, and the
  `mod trash;` shadowing note is a one-off naming collision.
