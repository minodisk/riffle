# Learnings

## Step 1

- Same-offset ordering: `write_rating` pushes `Rating`, `ShouldProcess`,
  `Settings`, then `Orientation`, so when an item lacks all of them the
  inserts at the closing brace come out `Orientation`, `Settings`,
  `ShouldProcess`, `Rating`. Only `Rating` breaks alphabetical order there,
  and that is the pre-existing push order the plan said not to touch. With a
  `Rating` line present (every Riffle-made file), `Orientation` goes at the
  start of that line and shares no offset with another insert.
- `Settings` is inserted at the start of the `ShouldProcess` line, not at the
  brace, whenever `ShouldProcess` exists; a "no Rating line" test that wants
  `ColorLabel`, `Orientation`, `Settings` together at the brace has to drop
  `ShouldProcess` too.
- The test helpers `patched` / `labeled` / `fresh` now use `Some(6)`, so every
  existing byte-for-byte PhotoLab fixture test also proves that an existing
  `Orientation = 8` is left alone; `old_template` keeps the orientation line
  and `pre_orientation` strips it for the insert tests.
- The writer only looks the orientation up for a `Dop` or `Both` entry, so an
  XMP-only setup never pays for a `read_metadata` on a file with no index row.
- Tooling: `cargo` is not on the Git Bash `PATH` here; `mise exec -- cargo`
  works. Python heredocs through the Bash tool turned `\\n` into real
  newlines in the written Rust source; writing the script to the scratchpad
  (or using the Edit tool) avoids it.
- CI failure (attempt 1): the new `orientation` parameter pushed
  `sidecar::write_kind` to 8 arguments and tripped
  `clippy::too_many_arguments`; fixed with the same
  `#[allow(clippy::too_many_arguments)]` the file already uses on
  `Writer::set` / `set_now`.

## Deferred issues (todo candidates)

- An explicit `Orientation = 1` line for landscape files has not been
  verified in PhotoLab in isolation (plan "Trade-offs and risks"); the user is
  checking one file from the bulk-repaired folder. Related:
  `crates/core/src/dop.rs` (`template`, `Doc::insert_orientation`).
