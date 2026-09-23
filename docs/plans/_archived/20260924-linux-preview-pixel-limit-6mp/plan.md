<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Lower the Linux preview pixel limit to 6 MP

## Purpose

Follow-up to #383 (`docs/plans/_archived/20260924-linux-preview-pixel-limit/`).
That PR resized previews over `PREVIEW_PIXEL_LIMIT` (12 MP on Linux) in the
decode worker, on the belief that WebKitGTK drew a transferred `ImageBitmap`
transparent only from about 16 MP (12.3 MP drew, 4000x4000 did not). A new
measurement (WebKitGTK 2.50.4 MiniBrowser under WSLg, worker
`createImageBitmap` with resize options, bitmap transferred to the main
thread, drawn to a canvas, center pixel read) shows the threshold is a pixel
count between 6,840,000 and 6,900,000 px, independent of shape:

- Transparent: 3000x2300, 2000x3500, 3276x2177, 3300x2193, 3600x2393,
  4000x2659, 4096x2722, 4249x2824 (the current 12 MP fit), 3000x4097
  (12.3 MP; the earlier "drew fine" did not reproduce), 3200x3200.
- Drawn: 3000x2280 (6,840,000), 2000x3400 (6,800,000), 3200x2127
  (6,806,400), and wide-short ones such as 4096x500 and 3584x1000.

So the 12 MP fit still lands above the threshold and SIGMA fp L DNGs keep a
blank main view on Linux. The fix, approved by the user, lowers the Linux
limit to 6 MP (a margin under ~6.87 MP, since the threshold may depend on the
environment) and corrects the documentation that cites 12.3 MP / 16 MP.
Non-Linux stays `None`; no other behavior changes.

## Steps

- [x] Step 1: Lower `PREVIEW_PIXEL_LIMIT` to 6 MP on Linux and correct the docs
  - Done when:
    - `crates/app/src/commands.rs` (~line 1443-1452): `PREVIEW_PIXEL_LIMIT`
      is `Some(6_000_000)` on Linux, `None` elsewhere. Its doc comment no
      longer says "12.3 MP drew, 16 MP did not"; it states that the failure is
      by pixel count regardless of shape, measured at roughly 6.87 MP
      (6,840,000 px drew, 6,900,000 px did not; WebKitGTK 2.50.4 under WSLg),
      and that 6 MP keeps a margin because the threshold may vary by
      environment.
    - The unit test `the_preview_pixel_limit_applies_on_linux_only` in the
      same file (~line 1820) asserts `Some(6_000_000)` on Linux.
    - `docs/agents/tauri-app.md`, the Hit item "WebKitGTK draws a large
      transferred `ImageBitmap` transparent" (~line 1065-1074): replace the
      "12.3 MP drew, 16 MP did not" sentence with the pixel-count threshold
      (~6.87 MP, shape-independent) and change "Linux only, 12 MP" to
      "Linux only, 6 MP". Keep the 60 MP SIGMA fp L example and the "Why"
      bullet.
    - `crates/app/ui/src/decode.test.ts` (`fitWithin` tests, lines 60-77):
      12_000_000 there is an arbitrary fixture for a pure function, not the
      Linux limit. Leave it as is unless the implementer judges the test
      title "fits a 60 MP frame under 12 MP" misleading; if changed, keep
      the assertions equivalent (e.g. switch the fixture to 6_000_000 and
      the "at or under the limit" case to 3000x2000).
    - `crates/app/ui/src/worker.ts`, `decode.ts`, `main.ts` and the
      `preview` payload are untouched: the limit already flows from the
      backend command, so nothing else needs to change.
    - `mise run ci` passes.
    - Manual check on Linux (`mise run dev`, WSLg): a SIGMA fp L DNG shows in
      the main view.
  - Implementation approach:
    - Do NOT touch `crates/app/src/index.rs` (edited concurrently elsewhere).
    - Only the constant, its doc comment, the Rust test, and the
      `tauri-app.md` Hit item change. Do not restructure the comment or the
      docs section beyond replacing the stale numbers.
    - Conventional Commit:
      `fix(app): lower the Linux preview pixel limit to 6 MP`.

## Trade-offs and risks

- **Margin size.** 6 MP is about 13% under the measured ~6.87 MP threshold.
  A larger margin (e.g. 5 MP) would tolerate more environment variance at the
  cost of detail; the user chose 6 MP. A fitted main view is still far smaller
  than 6 MP and the focus check uses `focus_crop`, so nothing visible changes.
- **The `decode.test.ts` fixture.** Leaving 12_000_000 there keeps the diff
  minimal; the test is for `fitWithin`, whose behavior does not depend on the
  Linux constant. Changing it only buys a less confusing test title.
- **Archived plan text.** The archived plan and learnings under
  `docs/plans/_archived/20260924-linux-preview-pixel-limit/` cite the old
  16 MP belief. Archives are historical records and are not corrected here;
  this plan's Purpose records the corrected measurement.

## Progress

- (2026-09-24) Step 1 complete
