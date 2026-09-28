# Learnings

## Step 1

- The helper is `settle(selection, files, focused, resumed)` in
  `crates/app/ui/src/selection.ts`; `refilter` passes its existing `force`
  flag as `resumed`, so `main.ts` changed by one line plus the import and
  `refreshEntries` / `openDirectory` stayed untouched.
- `prune` stays in use in `refilter`'s `files.length === 0` branch, so its
  import in `main.ts` remains.
- The manual desktop check listed in the plan (resume in a folder, with a
  hiding filter, and with no index cache) was not run by the implementation
  agent; only the unit tests cover the helper.
