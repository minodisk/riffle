# Learnings

## Step 1

- A diff of the README checklist against `docs/cameras.md` found no missing
  body: all 76 already had a row, so no `–` placeholder rows were needed.
- The README section kept both "the cameras verified ... are in cameras.md" and
  the existing "What each camera records ..." pointer as two adjacent
  sentences pointing at the same file; they were merged into one paragraph
  instead ("The cameras verified on a real file, what each records, and which
  features that affects are listed in docs/cameras.md").
- `todo.md` had two more references than the three the plan named: the tier
  intro's `(README.md "RAW formats and cameras")` (renamed to the new heading)
  and the `sony-arw` item's file list (`README.md`, `README.ja.md` dropped,
  since adding those bodies now touches only `docs/cameras.md`). The tier
  item's `Files:` list naming `README.md` was left as is.
- The `## Compatibility` intro "Anything unchecked has not been verified yet"
  now applies only to the OS checklist; it was kept, as the plan did not
  cover it.
- `mise run lint` failed on the plan itself: it quoted the old and new link
  text as live Markdown links (`[Compatibility](../README.md#compatibility)`),
  which lychee resolves from the plan folder. They were wrapped in backticks.
- The worktree had no `node_modules`, so `mise run fmt` failed until
  `mise exec -- pnpm install --frozen-lockfile` was run (`pnpm` is only on the
  PATH through mise).
