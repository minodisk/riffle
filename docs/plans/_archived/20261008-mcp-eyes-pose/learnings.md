# Learnings: mcp-eyes-pose

## Step 1

- Parallel session check: `git log origin/main` (fetched 2026-10-08, head
  `ac6f6ade`) has no `dlg-store-eyes-pose` commit; the index still does not
  store the EAR or the pose, so `ViewApi.eyes` in `main.ts` reads
  `eyesCache.get`. If that work lands later, that getter is the one line to
  repoint.
- `current.eyes` is set on the object only when `view.eyes(path)` is not
  `undefined`, so `"eyes" in current` is false for a file not judged yet both
  in `getView`'s return and after the JSON round trip `respond` goes through
  (the test asserts both).
- A view-only (JPEG) folder needs no special case: `eyes_of` never runs
  there, so `EyesCache.get` answers `undefined` and the key is absent.
- A Python `str.replace` with a backslash-continued Rust string literal in a
  heredoc failed to match for no obvious reason; the Edit tool matched it.
- The fresh worktree had no `node_modules`, so `mise run fmt` failed on the
  missing `vite-plus` entry point (environmental); `mise exec -- pnpm install
  --frozen-lockfile` fixed it (plain `pnpm` is not on the Bash tool's PATH).

## Deferred issues (todo candidates)

- **Repoint `ViewApi.eyes` when the index stores the EAR and pose.**
  `ViewApi.eyes` in `crates/app/ui/src/main.ts` reads `eyesCache.get` because
  the index does not store the eye judgment or the head pose yet (no
  `dlg-store-eyes-pose` commit on `origin/main` at `ac6f6ade`, 2026-10-08).
  When that lands, repoint this one getter to the stored value, keeping the
  `eyes` key absent from `get_view` until a file is judged. Done when
  `get_view` answers from the stored values and the existing `get_view` test
  (key absent before and after the JSON round trip) still passes. Conditional
  follow-up: merge into the "Store the scan's mesh-derived eye state and head
  pose in the index" todo item rather than filing a new one.
