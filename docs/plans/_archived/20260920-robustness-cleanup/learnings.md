# Learnings

## Step 1

- `SidecarError.path` now carries the RAW path; the sidecar's file name moved
  to the front of `message`, so a read error reads `a.ARW: a.xmp: ...` in the
  pane (the write side keeps the sidecar's full path in its message, as the
  plan's trade-off note says).
- Verified no frontend change is needed: `crates/app/ui/src/main.ts` uses the
  field only as the `errors.add` key and for `baseName(path)` (lines ~1057 and
  ~1188), and the `sidecar-error` handler additionally filters on
  `allFiles.includes(payload.path)` — which the RAW path satisfies, whereas
  the sidecar path never did on the read side, so the shared key is strictly
  an improvement. `crates/app/ui/src/errors.test.ts` treats keys as opaque
  strings.

## Step 2

- quick-xml 0.42's `ResolveResult::Bound(Namespace)`: `Namespace::as_ref()`
  yields `&str`, not `&[u8]` (compare against `XMP_NS` directly).
- `resolve_attribute` needs a `QName` borrowed from a local `String`, so the
  helper returning `ResolveResult<'a>` must tie its lifetime to the reader,
  not to the name buffer. That works because the only borrowing variant
  (`Bound`) points into the reader's namespace buffer.
- Both new tests pass on the first run; the existing xmp tests needed no
  change.

## Step 3

- Filtering inside `parse_keys` means the empty check has to run twice: once
  on the raw array (so `[]` stays "not a non-empty list of keys") and once
  after the retain (so `["control"]` also falls back to the default).
- `pending` no longer needs to carry the raw `Value`: the only consumer was
  `inactive.insert`, which now stores `Value::from(keys.clone())`, so the
  tuple shrank to `(index, keys)`.
- No frontend change: `MODIFIER_KEYS` in `crates/app/ui/src/keys.ts` already
  rejects lone modifiers at capture time; `MODIFIER_ONLY` in
  `crates/app/src/shortcuts.rs` mirrors it and carries a sync comment.

## Step 4

- No `tauri::test` harness was needed: extracting the body into
  `switch_format(current, writer, index, format, persist)` in
  `crates/app/src/commands.rs` makes the race directly testable, with the
  `persist` closure standing in for "a rating set while the switch runs".
- `rating_of` joins `files`, which only a scan fills in, so the test has to
  call `write_batch` with `index::stat` before reading the rating back (the
  same trick `a_foreign_sidecar_is_read_on_the_first_open_without_any_files_row`
  uses).
- The rating-only case passes: the `.dop` holds the rating, no `.xmp` is
  written, and the row ends clean.
- The `pick = true` variant **fails**, exactly as the plan's Background
  predicted. It was run locally and then removed from the PR per the
  decision recorded at planning time; see the deferred issue below.

## Deferred issues (todo candidates)

- **A pick made during a sidecar format switch is lost on the next open.**
  Basis: Step 4's `pick = true` run of the new race test (plan Background
  and "Trade-offs and risks"; the user chose at planning time to record it
  rather than fix it here).
  Reproduction: start in `Xmp`, call `switch_format(.., Dop, persist)` where
  `persist` does `index.set_rating(dir, path, Some(4), true, None, true)` and
  `writer.set(path, Some(4), true, None, true, Dop)`, then
  `writer.flush(DRAIN_TIMEOUT)`. The `.dop` sidecar correctly holds both the
  rating and the pick, but `assert!(index.dirty_rows(dir).is_empty())` fails.
  Cause: `Index::reset_sidecars` (`crates/app/src/index.rs`) runs after
  `persist` and executes
  `UPDATE ratings SET xmp_size = NULL, xmp_mtime_ns = NULL, pick = 0 WHERE dirty = 1`,
  zeroing the pick of the row the writer has not landed yet. `Index::mark_written`
  then clears `dirty` only
  `WHERE path = ?1 AND rating IS ?4 AND pick = ?5 AND label IS ?6`, which no
  longer matches (`pick = 0` in the row vs `pick = 1` in the write), so the
  row stays dirty with `pick = 0` and the next folder open replays it over the
  sidecar, stripping the pick.
  Files: `crates/app/src/index.rs` (`reset_sidecars`, `mark_written`),
  `crates/app/src/commands.rs` (`switch_format`).

