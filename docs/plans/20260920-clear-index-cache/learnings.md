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

## Deferred issues (todo candidates)

- (none)

### `clear` has no caller until Step 2

`crates/app` is a bin crate, so `pub fn clear` tripped `-D dead-code`. It
carries `#[cfg_attr(not(test), expect(dead_code))]`; **Step 2 must remove that
attribute** when `clear_index` calls it, or `unfulfilled_lint_expectations`
fails the build.
