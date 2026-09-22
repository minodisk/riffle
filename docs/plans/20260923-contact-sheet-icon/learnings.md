# Learnings

## Step 1

- `pnpm exec tauri icon` still creates `ios/` and `android/` under
  `crates/app/icons/`; deleting them afterwards is required. It left
  `menu/` and `tauri.conf.json` untouched, as in the previous icon plans.
- `cp` of the scratchpad image kept the bytes identical (sha256 matched).

## Deferred issues (todo candidates)

- (none)
