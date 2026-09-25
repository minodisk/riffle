# Learnings

## Step 1

- The plan's implementation approach said `#position` "comes after `#tools`",
  but in `crates/app/ui/index.html` it came first. With `margin-left: auto`
  on a first child, the counter and the tools would both have been pushed to
  the right. In line with the "order from markup" decision, `#position` was
  also moved after `#tools` in the markup (the only markup change beyond the
  planned `#sort` / `#filter` swap). `main.ts` looks it up by id, so nothing
  else depends on the order.
- The menus now use a shared `left: 8px` (the `#sort` wrapper's left padding)
  with a `#filter-menu { left: 0 }` override, since `#filter` has
  `padding-left: 0`.
- `mise run ci` failed locally on Windows only in `lychee`: it reported a
  missing fragment in
  `docs/plans/review-history/thumbnail-cache-filmstrip-step-8/review-20260918-0253.md`,
  a file the `--exclude-path docs/plans/review-history` flag is meant to skip.
  On Windows lychee sees the path with backslashes, so the forward-slash
  exclude does not match. Running lychee with `--exclude-path review-history`
  passed (0 errors), as did the rest of `lint` and all of `mise run test`.
  The worktree also needed `pnpm install` before `mise run fmt` could find
  `vp`.

## Deferred issues (todo candidates)

- `mise run lint`'s lychee `--exclude-path docs/plans/review-history` does not
  match on Windows (the path is reported with backslashes), so local
  `mise run ci` fails on Windows on
  `docs/plans/review-history/thumbnail-cache-filmstrip-step-8/review-20260918-0253.md`
  (`#running-the-app` fragment). Found while running the step 1 local checks.
  Related file: `mise.toml` (`[tasks.lint]`).
