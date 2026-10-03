# Learnings: dop-orientation-landscape-verified

## Step 1

- Verification (by hand, 2026-10-04): a Riffle-made `.dop` written by
  `riffle-core`'s `dop::write_rating`, carrying the explicit
  `Orientation = 1,` line, for a landscape ILCE-7M4 uncompressed ARW with EXIF
  Orientation 1, was opened in PhotoLab 10 at
  `D:\Photos\tests\2026-10-03-dop-orientation-landscape`. The image displayed
  upright, and the `.dop`'s `Rating = 3` was read, confirming PhotoLab used
  the sidecar. So the explicit landscape line is safe, and the `todo.md` item
  "Core: an explicit `Orientation = 1` line for landscape `.dop` files is
  unverified in PhotoLab" was removed.
- Incidental observation: an extra item appeared in PhotoLab during the test.
  The ARW had been copied into a folder PhotoLab was already viewing before
  the `.dop` existed, so PhotoLab registered the image with its own UUIDs, and
  the fresh `.dop` then carried random ones. This is a race in the test
  procedure, not a Riffle bug; the app avoids it through
  `crates/app/src/photolab.rs`'s UUID lookup (see
  [`docs/agents/photolab.md`](../../agents/photolab.md)). For future hand
  checks, write the `.dop` before PhotoLab sees the RAW, or through the app.
- The plan's location hints were stale by the time the step ran: the section
  sat at `todo.md` lines 615–628 (not 607–620), and the preceding section was
  "Agents: fix the inaccurate \"subagent exits the moment its turn ends\"
  rationale in the other agent files", not the "Release: ..." one, because
  `todo.md` gained items after the plan was written. The section was found by
  its heading; the neighbors stay separated by one blank line.
