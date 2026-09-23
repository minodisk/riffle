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
- Why this follow-up exists: #383 shipped a 12 MP limit on a secondhand
  threshold ("12.3 MP drew") that was never measured on the shipped code path.
  The user's manual check still showed a blank main view. A MiniBrowser bisect
  (a worker `createImageBitmap` with resize options, the bitmap transferred,
  `drawImage` to a canvas, the center pixel read, results POSTed to a local
  Python HTTP server) put the real threshold at ~6.87 MP by pixel count, and
  3000x4097 did not draw either.

## Deferred issues (todo candidates)

- Verify a platform workaround's numeric threshold on the exact production
  code path before shipping it, instead of taking a quoted number. Change:
  decide whether this belongs as a note in the `docs/agents/tauri-app.md`
  WebKitGTK Hit item (with the MiniBrowser + POST-logging harness as the
  method) or as a broader practice. Why: #383's 12 MP limit came from an
  unverified claim and needed this follow-up. Done when: the practice is
  written into the chosen guide.
