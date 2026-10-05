# Learnings

## Step 1

- `resync` needed no change: it switches on nothing but logs the trigger, so
  adding `"refresh"` to `RescanTrigger` and calling `resync("refresh")` from
  the `refreshFolder` branch of the folder menu in
  `crates/app/ui/src/main.ts` was enough.
- The same-folder decision is `opensTarget(openDir, [path], false,
  folders.ignoreCase)`. The existing `opensTarget` / `relation` tests already
  covered a trailing separator, `\` vs `/`, drive-letter case, ignored path
  case and an ancestor of the open folder; the only gap was a subfolder of the
  open folder with `recursive` off, now covered in
  `crates/app/ui/src/trash.test.ts`.

## Deferred issues (todo candidates)

- Pending manual check (on the user; Step 1's checkbox was ticked on the
  automated criteria): on Windows, open a folder on a share (or any volume)
  whose watch does not fire, add a RAW to it and edit a sidecar outside
  Riffle, then right-click the open folder in the tree and choose `Refresh`.
  Expected: the new file appears in the strip, the edited judgment is picked
  up, the tree's RAW count updates, and `Riffle.log` shows
  `rescan: trigger=refresh`. Also check that `Refresh` during a running scan
  logs `rescan deferred: trigger=refresh` and runs when the scan ends, and
  that `Refresh` on a folder that is not open (including its parent or a
  subfolder) leaves the strip untouched. Basis: plan Step 1 Done-when;
  files `crates/app/ui/src/main.ts`, `crates/app/ui/src/refresh.ts`.
