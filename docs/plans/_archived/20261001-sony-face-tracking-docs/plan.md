<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Document Sony face tracking on the verified bodies and how it helps culling

## Purpose

`docs/cameras.md` shows `✓` under `Face tracking` for the Sony α7 V only and
says the other Sony bodies' `–` means the sample held no face. Local samples
in `D:\photos\samples\ARW` (checked with `reader::read_metadata` and
`sharpness::eye_af_frame`, and by drawing the frame on the preview) show
`AFTracking` 1 with a valid `FocusFrameSize` on the α7 IV, α7R V, α7S III,
α6700 and ZV-E1 too, with the frame on a face or eye, so the table and the
paragraph are stale. The same file's `AF frame size and face tracking`
bullet says sharpness is scored on the eye-AF frame but not why that helps
culling, and nothing else in the docs does. `docs/agents/raw-metadata-parsing.md`
still says face tracking is unverified on every body because the
raw.pixls.us samples hold no `AFTracking` 1.

After this work the table shows the five bodies as verified, the paragraph
states what the local samples proved and their limit, cameras.md explains
what the reader gains from face tracking, and the agent guide reflects the
local sample set. No code changes.

Verified samples (for `learnings.md`): `ILCE-7M4_valeqvisuals-1-signature_edits_free_raw_photos.ARW`
(eye), `ILCE-7RM5_sony_a7r_v_37.arw` and `_69.arw` (eyes),
`ILCE-7SM3_tag_signatureeditsco_DSC02439.ARW` and `DSC04841.ARW` (face),
`ILCE-6700_sony_a6700_01.arw` (head), `ZV-E1_sony_zv_e1_01.arw` (eye of a
face on a poster). Off-face with `AFTracking` 1: `ILCE-6700_sony_a6700_70.arw`
(empty background) and `_71.arw` (bread on a market stall).

## Steps

- [x] Step 1: Update `docs/cameras.md` and `docs/agents/raw-metadata-parsing.md`
  - Done when:
    - The `Face tracking` column in `../../cameras.md` shows `✓` for Sony
      α7 IV, α7R V, α7S III, α6700 and ZV-E1; α1, α9 III, α7C II and α7CR
      stay `–`; no rows are added (the older bodies with `AFTracking` 1 but
      no `FocusFrameSize` are not in the table).
    - The paragraph beginning "On the Sony bodies other than the α7 V, `–`
      under Face tracking means ..." is rewritten: `–` on the remaining
      Sony bodies means no sample of that body recorded face tracking, not
      that the body lacks it; and on the α6700 two samples record
      `AFTracking` 1 with the frame off any face (empty background, bread
      on a market stall), so the value does not guarantee a face under the
      frame.
    - The `AF frame size and face tracking` bullet says why it helps
      culling: the sharpness cue (the bar beside each thumbnail marking the
      sharpest frame of a burst) is measured on the eye the camera focused
      on, so the frame whose eye is sharp comes out sharpest rather than one
      whose background or clothing is sharper, and this does not depend on
      Riffle's own face detection finding the face; face tracking without
      an AF frame size gives no such benefit (sharpness falls back to the
      usual AF-point window); and it does not change the focus mark's color
      (the AF eye in-focus probability comes from Riffle's face detection).
      The existing α7 V back-of-head note and the `AF tracking` meta row
      sentence are kept.
    - The `Sample limits` bullet in `../../agents/raw-metadata-parsing.md`
      (the `FocusLocation` frame section) says the raw.pixls.us samples hold
      no `AFTracking` 1, but the local sample set has `AFTracking` 1 with a
      valid frame on the α7 IV, α7R V, α7S III, α6700 and ZV-E1, with the
      frame on a face or eye except two α6700 samples; portrait frames remain
      unverified; `AFTracking` 2 is still noted as lock-on AF.
    - `README.md`, `README.ja.md` and `docs/usage.md` are unchanged (they
      already say "when the camera recorded face tracking" without naming a
      body).
    - `mise run ci` passes.
  - Implementation approach:
    - All three claims of the new bullet are already confirmed against
      `crates/core/src/sharpness.rs` (`eye_af_frame` needs a trusted AF
      point, `AFTracking` 1, a valid frame and a point off the exact sensor
      center; `score` then windows on the AF point at the frame's long side
      and ignores faces; without a frame it falls to the plain AF-point
      window) and `crates/core/src/candidate.rs` (a Sony eye-AF frame gets
      the same face detection as any other frame). Do not re-derive them;
      keep the wording consistent with the sharpness cue bullet in
      `README.md` and `docs/usage.md` (`eye-AF frame`, `sharpness cue`,
      `focus mark`).
    - Keep the bullet short and in the file's style (one bullet, bold
      lead-in, plain sentences); the sample file names belong in
      `learnings.md` of this plan, not in cameras.md.
    - Optionally note in the paragraph that Riffle also ignores the frame
      when the AF point sits at the exact sensor center (tracking never
      locked), since `eye_af_frame` does; drop it if it makes the paragraph
      long.
    - Record the verified sample file names and the α6700 off-face cases in
      `learnings.md`.

## Trade-offs and risks

- Wording of `✓`: the column now means "a local sample of this body recorded
  face tracking with a frame on a face", not "every face-tracked shot has a
  face under the frame". The α6700 caveat covers this; the alternative
  (marking the α6700 as `–` or with a footnote) was not taken because the
  body does record the tag and the frame lands on a face in the other
  sample.
- Whether to mention the exact-center rejection of `eye_af_frame` in the
  paragraph is a judgment call on length; either is acceptable.

## Progress

- (2026-10-01) Step 1 complete
