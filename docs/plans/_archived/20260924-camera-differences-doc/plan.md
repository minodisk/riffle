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

# Move the camera differences to their own page

## Purpose

The READMEs' "Key features" bullets name specific cameras (Sony, SIGMA BF,
M11-P) to explain the focus mark and the sharpness cue, and the Compatibility
section carries a per-camera table of what each body records. Both age badly
as cameras are added, and the table belongs with the detailed docs rather than
the happy-path README. This work:

- rewords the three Japanese sentences in `README.ja.md` that use "めくる",
  which the user finds unclear (Japanese-only wording; `README.md`'s "page
  through" / "Paging" stays as is);
- describes the focus mark and the sharpness cue by what the camera records
  instead of by camera name, in both READMEs;
- moves the per-camera table and its explanatory bullets into a new English
  page, `docs/cameras.md`, verified against the code, and links to it from
  both READMEs and from `docs/usage.md`.

Afterwards the READMEs stay camera-neutral, and a new camera only needs a row
in `docs/cameras.md` plus a checkbox in the Compatibility list.

## Steps

- [x] Step 1: Reword "めくる", make the Key features camera-neutral, and move the per-camera table to `docs/cameras.md`
  - Done when:
    - `README.ja.md` contains no "めく" (`grep -c めく README.ja.md` is 0), with
      exactly these three rewordings and no other Japanese-only content change:
      1. intro bullet: "数千枚のフォルダーでもめくるたびに待たされることがありません" →
         "数千枚のフォルダーでも、写真を切り替えるたびに待たされることがありません"
      2. 最初の一歩 3.: "`↑` `↓` で写真をめくり" → "`↑` `↓` で前後の写真に切り替え"
      3. 等倍ピントチェック bullet: "めくってもズームは保たれます" →
         "別の写真に切り替えてもズームは保たれます"
    - No camera name (Sony, SIGMA, Leica, α7, BF, fp L, M11) appears in the
      "Key features" / "主な機能" bullets of either README. The Focus mark,
      Sharpness cue and Bursts bullets each link to `docs/cameras.md`.
    - `docs/cameras.md` exists, in English, holding the table (AF point / AF
      frame size / face tracking / sub-second capture time per camera) and the
      bullets saying which features each item affects; the "#### What the
      camera records" / "#### カメラが記録する情報と使える機能" subsections are
      gone from both READMEs, and the table appears nowhere else.
    - "Compatibility > RAW formats and cameras" / "対応状況 > RAW 形式とカメラ"
      keeps its checklist of tested cameras and links to `docs/cameras.md`
      (`README.ja.md` marks the link "（英語）").
    - `docs/usage.md`'s Focus mark, Sharpness cue and Bursts bullets link to
      `docs/cameras.md` (links only; their wording, camera names included,
      stays).
    - `README.md` and `README.ja.md` say the same things section by section.
    - `mise run ci` passes (in particular the lychee link and fragment check).
  - Implementation approach:
    - Wording (A) is Japanese-only. Do not touch `README.md`'s "page through"
      / "Paging keeps the zoom." sentences.
    - Key features (B), `README.md` — rewrite the two bullets camera-neutrally,
      keeping the key names and the order of the fallbacks (which matches
      `crates/core/src/sharpness.rs` at HEAD: eye-AF frame → AF point → eyes of
      a detected face when there is no AF point → sharpest region):
      - Focus mark: `f` draws the AF frame the camera used around a crosshair
        on its focus point when the camera records the frame, the crosshair
        alone when it records only a point, and nothing without an AF point or
        on manual-focus shots. End with a link whose text is
        "What the camera records" and whose target is `./docs/cameras.md`.
      - Sharpness cue: scored on the camera's eye-AF frame when the camera
        recorded face tracking, else around the AF point, else on the
        subject's eyes when the camera recorded no AF point and a face is
        found, else the sharpest region. Same link.
      - Bursts: keep the sentence about cameras that record no sub-second
        capture time, but point its link at `./docs/cameras.md` instead of the
        removed `#what-the-camera-records` anchor.
      - Re-check the other bullets with
        `grep -n -i "sony\|sigma\|leica\|m11\|α7\|fp l" README.md README.ja.md`;
        expect hits only in the Compatibility checklist.
    - Key features (B), `README.ja.md` — the same three bullets in Japanese,
      with the same content as the English ones (link text
      "カメラが記録する情報", target `./docs/cameras.md`, followed by "（英語）").
    - New page (C), `docs/cameras.md` — start from the "#### What the camera
      records" subsection of `README.md` (intro sentence, table, three
      bullets). Shape: `# What the camera records`, a one-line lead linking
      back to the README's Compatibility section and to `usage.md`, the table,
      then the bullets. Facts, all verified against the code at HEAD
      (`crates/core/src/arw.rs`, `sharpness.rs`, `scan.rs`,
      `crates/app/ui/src/burst.ts`, `focus.ts`):
      - Rows: Sony α7 V ✓✓✓✓; SIGMA BF ✓ – – –; SIGMA fp L – – – –;
        Leica M11-P – – – –. Keep the README's row order.
      - AF point: focus mark drawn there, 1:1 focus check opens centered on
        it, sharpness scored around it; faces are ignored when there is a
        trusted AF point, even when the point is off every face. Without one:
        no focus mark, 1:1 opens at the frame center, sharpness between the
        eyes of a detected face, else on the sharpest region. A manual-focus
        shot on a body that records the focus mode (Sony `FocusMode` 0) is
        treated as having no AF point (`sharpness::trusted_focus`); DMF and
        bodies that record no mode are not. (The user approved including
        these two precision points.)
      - AF frame size and face tracking: with both (Sony `FocusFrameSize` and
        `AFTracking` = face tracking), sharpness is scored on the camera's
        eye-AF frame, which does not rely on face detection (the scan skips the
        detector on such frames, `scan.rs`).
      - Sub-second capture time: frames within 1 s of the previous one form a
        burst (`BURST_GAP_MS = 1000`); without it (`SubSecTimeOriginal`
        absent) frames are grouped by whole seconds, so a shot taken up to
        about 2 s after the previous frame can still join its burst.
      - No "how to report a camera" paragraph; that stays in the READMEs.
    - READMEs (C): delete the two subsections and, in "RAW formats and
      cameras" / "RAW 形式とカメラ", add one sentence after the checklist
      linking to `./docs/cameras.md` with the path as link text (ja: followed
      by "（英語）"). Keep the existing "If a camera not on the list works, ..."
      paragraph.
    - `docs/usage.md` (links only): add a "What the camera records" pointer to
      `./cameras.md` in the Focus mark, Sharpness cue and Bursts bullets.
    - Links: every link to the new page is a plain path with no fragment.
      Remove every remaining reference to the `#what-the-camera-records` /
      `#カメラが記録する情報と使える機能` anchors
      (`grep -rn "what-the-camera-records\|カメラが記録する情報と使える機能" *.md docs --exclude-dir=plans`
      should find nothing), or lychee `--include-fragments` fails. lychee also
      checks this plan and `learnings.md`: write example paths in backticks,
      never as Markdown links.
    - Keep "Lightroom" before "PhotoLab" wherever both are mentioned; this PR
      does not touch those sentences.
    - Commit: `docs: move the camera differences to their own page`.

## Trade-offs and risks

- **One step vs. three.** Wording, bullets and the new page could be three
  PRs, but the bullets are both rewritten and repointed, and every PR touching
  `README.md` must update `README.ja.md` in the same PR anyway. One PR keeps
  the READMEs in sync in a single review unit.
- **`docs/usage.md` still names cameras.** It is the detailed doc; the user
  approved links only there.
- **Japanese wording of the bullets.** Keep the content identical to the
  English bullets, not the phrasing.

## Progress

- (2026-09-24) Step 1 complete
