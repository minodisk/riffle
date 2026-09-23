# Learnings

## Step 1

- The limit flows from `PREVIEW_PIXEL_LIMIT` through the `preview_pixel_limit`
  command to the decode worker, so lowering it only touched the constant, its
  doc comment, the Rust unit test and the `tauri-app.md` Hit item.
- `crates/app/ui/src/decode.test.ts` was left at its 12_000_000 fixture: it
  tests the pure `fitWithin` function, and its titles describe the fixture,
  not the Linux constant.
- The manual Linux GUI check (`mise run dev` under WSLg, a SIGMA fp L DNG in
  the main view) was not performed by the implementation agent.

## Deferred issues (todo candidates)

- (none)
