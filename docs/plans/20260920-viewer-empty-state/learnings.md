# Learnings

## Step 1

- `empty.ts` exports `emptyState`, `openHint`, `displayKey` and the two
  zero-file sentences (`NO_FILES_TEXT`, `FILTERED_TEXT`) so step 2 can set the
  overlay text without composing wording in `main.ts`.
- `openHint` treats an empty `Binding[]` (keymap not resolved) and an `open`
  action with no keys the same way: the key clause is dropped.
- `displayKey` only maps `space` -> `Space`, mirroring the one-off formatter in
  `settings.ts` (left untouched on purpose).

## Deferred issues (todo candidates)

- The key formatter is duplicated between `crates/app/ui/src/empty.ts`
  (`displayKey`) and `crates/app/ui/src/settings.ts` (the local `display`);
  sharing one via `keys.ts` is out of scope here (plan "Trade-offs and risks" 3).
