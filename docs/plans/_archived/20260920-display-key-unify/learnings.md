# Learnings

## Step 1

- `displayKey` now lives at the end of `crates/app/ui/src/keys.ts`, right after
  `keyName`, and both `empty.ts` and `settings.ts` import it from `./keys.js`.
- The `settings.ts` local `display` closure had the identical body, so removing
  it and renaming the two call sites was enough; no behavior change.
- `empty.test.ts` needed no change: it only exercises `openHint`.

## Deferred issues (todo candidates)

- (none)
