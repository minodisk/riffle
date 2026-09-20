# Learnings

## Step 1: `Index::clear`

### The WAL does keep the old size — the checkpoint is required

Measured in `clear_empties_the_index_and_gives_the_space_back` (2 folders x 5
rows of a 200 KB thumbnail each), summing `index.sqlite` + `index.sqlite-wal`
on disk right after `clear`:

| | before | after |
| --- | --- | --- |
| without `wal_checkpoint(TRUNCATE)` | 2,121,808 B | 2,212,448 B (grew) |
| with `wal_checkpoint(TRUNCATE)` | 2,121,808 B | 40,960 B |

So `VACUUM` alone gives nothing back to the file system: the rebuilt database
is written through the WAL, so the footprint is slightly *larger* than before
even though `page_count` dropped by more than 10x. `clear` therefore ends with
`PRAGMA wal_checkpoint(TRUNCATE)`, and the test pins the on-disk figure, not
only the page count.

Step 2's size display and Step 4's documentation can rely on "the figure drops
to a few tens of KB after a clear".

### `pragma_query` cannot take a pragma argument

`self.conn.pragma_query(None, "wal_checkpoint(TRUNCATE)", ..)` compiles but
does not truncate (rusqlite quotes the pragma name, so the argument is lost);
the first run of the test failed with the unchanged size. `query_row("PRAGMA
wal_checkpoint(TRUNCATE)", ..)` works. A `SQLITE_BUSY` from it is logged and
the clear still succeeds, as the plan requires.

### Timing

`VACUUM` on this ~2 MB test index is instant; no large index was at hand, so
the "seconds at ~1 GB" note in the plan is still unmeasured.

## Step 2: `index_size` and `clear_index`

### The dialog is awaited with no lock held, so the scan check happens twice

`clear_index` checks `Scans.running` before showing the dialog (so the user is
not asked to confirm something that then fails) and again inside the
`spawn_blocking`, under the `Scans` lock it then holds for the clear. The first
check's guard is dropped at the end of its block, before any `.await`; holding
it across the dialog would be both a `Send` problem and a way to freeze every
scan for as long as the dialog is up.

### `format_bytes` keeps one decimal from KB up, including `312.0 KB`

The plan's example reads `312 KB`, but "one decimal from `KB` up" is the rule
that was implemented, so the test pins `312.0 KB`. Mixing the two would need a
per-unit rule for no benefit.

### `SIZE_BASE` is pinned by a test that branches on `cfg!`

`the_size_base_follows_the_platforms_file_manager` asserts 1000 on macOS and
1024 elsewhere, so every OS in the CI matrix checks its own branch.

### Removing `expect(dead_code)` from `Index::clear`

Done as Step 1's note required; `with_suffix` in `index.rs` became
`pub(crate)` so `on_disk_bytes` can reuse it.

### Manual verification

Not run: the Clear Cache button does not exist until Step 3, and no other UI
invokes `clear_index` or `index_size`, so there is no reachable path to the
dialog, the cancel case, the mid-scan refusal or the `index-cleared` reopen in
a running build. GUI automation is unavailable on this Mac. All of that has to
be verified by hand after Step 3.

## Deferred issues (todo candidates)

- Manual GUI verification of `clear_index` (dialog confirm/cancel, the
  mid-scan refusal, and the main window's `index-cleared` reopen) is deferred
  to Step 3, when the settings UI can actually invoke it. Basis: Step 2's
  manual-verification requirement with no UI caller yet. Files:
  `crates/app/src/commands.rs`, `crates/app/ui/src/main.ts`,
  `crates/app/ui/settings.html`.

### `clear` has no caller until Step 2

`crates/app` is a bin crate, so `pub fn clear` tripped `-D dead-code`. It
carries `#[cfg_attr(not(test), expect(dead_code))]`; **Step 2 must remove that
attribute** when `clear_index` calls it, or `unfulfilled_lint_expectations`
fails the build.
