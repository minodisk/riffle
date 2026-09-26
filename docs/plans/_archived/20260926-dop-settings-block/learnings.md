# Learnings

## Step 1: Minimal `Settings` block

- The user's PhotoLab 10 bisection (2026-09-26): an `Items[0]` without a
  `Settings` table makes PhotoLab show "no images in this folder". A CRLF
  tail, `ProcessingStatus`, a PhotoLab `Software` string, `Albums` / `IPTC` /
  `Keywords` / `OutputItems`, or `CafId` / `ShotDate` do not help; a
  brace-balanced `Settings = {\nVersion = "21.0",\n}\n,\n` between `Rating`
  and `ShouldProcess` does, and keeps the pick and the rotation.
- The archived claim in
  `docs/plans/_archived/20260918-photolab-dop-sidecar/learnings.md` (Step 5)
  that the Settings-less template was accepted was wrong (most likely the
  folder was already in PhotoLab's database); a one-line correction note was
  added there.
- Edits at the same offset end up in the reverse of their push order: they
  are applied in a stable descending sort, so each later insertion lands in
  front of the earlier one. `write_rating` therefore pushes the `Settings`
  edit after `ShouldProcess` (so `Settings` precedes an inserted
  `ShouldProcess`), and `write_label` pushes it before `ColorLabel` (so an
  inserted `ColorLabel` precedes it, alphabetically).
- Repairing Riffle's old template with `write_rating` yields exactly the new
  template, which the test asserts byte-for-byte.
- The fresh-template tests that pinned `Rating = n,\nShouldProcess = m,\n`
  adjacency had to be updated to include the block.
