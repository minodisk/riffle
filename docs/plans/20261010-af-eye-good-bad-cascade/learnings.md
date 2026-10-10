# Learnings

## Step 1: Merge `OK` into `Bad`

- The single helper is `afEyeState(focus)` in `crates/app/ui/src/focus.ts`
  (`unknown` without a focus, the focus's `candidate` when it is not
  `candidate`, else `photoTier(focus) ?? "not_candidate"`). `focusMark` reads
  it, `stripState` delegates to it, and `main.ts`'s `passes` hands its result
  to `filter.ts`'s `passes()`.
- `filter.ts`'s `passes()` now takes one `afEye?: AfEye` argument (where the
  `candidate` argument was) and lost the trailing `tier` argument, so the
  filter no longer re-derives the state from the candidate and the tier.
  `AfEye` is `MarkState`, whose values equal the `data-candidate` keys of the
  menu items (`good` / `not_candidate` / `unknown`), so the icon loop in
  `main.ts` iterates the states directly. `filter.test.ts` builds its "a
  focus candidate that is not good passes `Bad`" and partition cases through
  `afEyeState`, so the mapping is pinned end to end.
- `SCAN_EYE_SVG` had no other user and was removed, with `scan-eye` from the
  Lucide notice at the top of `icons.ts`.
- The backend still serializes `candidate` / `not_candidate` / `unknown`;
  `photoTier` still requires `candidate === "candidate"`.
- `CLAUDE.md` was not edited (plan rule); the `src/focus.ts` sentence stays a
  follow-up (the plan's "CLAUDE.md follow-ups" and the `todo.md` item's second
  checkbox).

## Step 2: Judge a frame with no trusted AF point on the largest face

- `candidate::face_cue_unless(rgb, width, height, orientation, &face,
  detection, cancel) -> Option<Cue>` is the cue of a given face
  (`scored_face` + `cue_of`); it computes the `luma` itself so it stays at
  seven arguments (clippy's `too_many_arguments`), and `focus_cue_unless`
  now calls it after the detection (the luma moved from before the
  detection to after it; same values).
- `scan::extract_analysis_unless`'s no-AF branch: `whole_image_detection`
  (the old `whole_image_faces`, keeping the `Detection`), a cancel check,
  then `whole_image_cue`: `eyes::judged_face(&faces, None)`; no face is
  `Cue::unknown()` with no decode; else one full-size `decode_rgb` and
  `face_cue_unless`, wrapped in the same `catch_unwind` as the AF branch
  (shared as `guarded`). A decode failure or panic is `Cue::unknown()`. A
  mesh failure is not Unknown: as on the AF branch, the eye window scores
  the face (the plan's wording said "mesh failure keeps Unknown"; the
  shared `face_cue_unless` keeps the AF branch's behavior instead).
- The scan's unit tests use a fabricated `Detection` (no face fixture
  exists in the repo, and YuNet finds no face on a synthetic JPEG), so
  `whole_image_cue` takes the detection as an argument rather than
  detecting inside.
- `Focus` is now built for every RAW row (`is_raw_file` on the path) or any
  row with a recorded point; a JPEG with no point keeps `focus: null`. The
  point moved into `point: Option<FocusPoint>`; the MCP `get_photo` JSON
  follows the same shape (its description was left as is).
- Frontend: `focusMark` returns `null` for `point === null`; `stripState` /
  `afEyeState` / `photoTier` / `passes()` needed no logic change. The 1:1
  placeholder in `main.ts` now reads `focus?.point` (it used to skip a
  `null` focus, which a no-point RAW no longer is). `meta.ts` needed no
  logic change; its "shows the stored eye state, openness and head pose
  without a judgment" test already covers a stored no-AF value.
- Docs: the plan listed "under about 60 px" among the white (unknown)
  cases, but a face under 60 px is scored on the eye window and gets a
  Good/Bad state (Bad, as it has no EAR), so the docs keep it in the
  "never good" sentence, not in "white".
- Stored vs on-demand: a temporary example (not committed) ran
  `scan::extract_analysis` and the `read_eyes` path side by side. On
  `D:\photos\2026\2026-02-01` (146 Leica DNG, 72 with a judged face) and
  12 manual-focus ARWs with a face from `D:\photos\samples\ARW\` (84
  manual-focus files there, e.g. `ILCE-6400_DSC00405.ARW`,
  `ILCE-7M3_DSC03036.ARW`), the stored EAR, yaw, pitch and roll equal the
  on-demand ones to 4 decimals on every file, and `cue_side` equals
  `judged_side` on all 276 faced files of the two Leica folders: the mesh
  overlay sits on the judged face. No manual-focus ARW exists in the
  burst-keep-score data set (all 1023 `noaf` rows there are DNG).
- Results of the two Leica folders after the change: `2026-02-01` 66
  Candidate / 6 NotCandidate / 74 Unknown (was 146 Unknown); `2026-09-05`
  145 / 59 / 207 (was 411 Unknown). The sharpness column is unchanged on
  every file.
- Cost (`riffle-cli features <dirs> 1`, `analysis_ms`, release, Windows
  11): the clean runs (before at 17:05, after at 17:12, before another
  session loaded the machine; one after-run file, `L1005115.DNG`, stalled
  890 s, timed 123 ms alone, and is excluded):

  | folder | files | faced no-AF before | after | no face before | after |
  | --- | --- | --- | --- | --- | --- |
  | `2026-02-01` | 146 | 93.5 ms (72) | 121.0 ms | 98.5 ms (74) | 82.9 ms |
  | `2026-09-05` | 411 | 95.3 ms (203) | 123.3 ms | 98.3 ms (207) | 82.6 ms |

  So about +28 ms per faced no-AF file (one full-size RGB decode, the
  luma and the mesh); a file with no face pays nothing (its drift down is
  noise). Two interleaved before/after runs under load gave +33 to +48 ms
  on faced files and none on face-free ones. The `todo.md` decode item now
  says the path decodes up to three times and that scoring from the RGB
  decode is the follow-up.
- `todo.md`: the no-AF good-photo item is checked; a new item checks the
  cuts on starred no-AF frames; the `Eyes` / `Head pose` real-device items
  on `2026-02-01` now expect the stored values after `FACES_VERSION` 9.
- CLAUDE.md follow-up (not edited, plan rule): the Layout sentences on
  `run_faces_scan`, `eyes_of` and `src/candidate.rs` need "else the largest
  confident face on the whole preview when there is no trusted AF point".

## Step 3: The sharpness score on face-free frames

- Rejects the data actually held: 58 in the ARW folders (50 in
  `2026-09-19`, 7 in `2026-09-13-b` as 4 `.dop` + 3 XMP-only, 1 in
  `2026-09-27-a`; the plan's 57 omitted the last one) and 1
  in `2026-09-05`, plus 20 in the folder the user named,
  `D:\Photos\samples\ARW\good-mark-2026-10-09` (`.dop` `ShouldProcess = 1`
  and XMP `xmpDM:good="False"`, all 1-star; its 160 other frames are
  unflagged 2- to 5-star). `2026-09-13-a` and `2026-09-27-c` hold none. On
  the face-free frames only **9** remain (7 in `2026-09-19`, 2 in
  `2026-09-13-b`): every good-mark frame has a face at its AF point, so the
  pick-vs-reject leg of the criterion (30 rejects in a folder) could not be
  run, and the 9 are reported descriptively.
- Decision C rests on the pick vs non-pick measures: the folder-stratified
  AUC (0.579 absolute, 0.536 / 0.598 relative, needed 0.70) and the cut held
  out by folder (best lift 1.25, needed 2.0 at 20% marked).
- The score has no fitted parameter, so "held-out AUC" was defined (before
  the numbers, in `sharpness-fallback.md`) as the folder-stratified AUC; the
  held-out loop only chooses the cut.
- The burst-best-mark dumps were taken at `22a309f9`, before Step 2, so all
  36 folders plus good-mark were re-dumped with a release CLI from
  `3df2fcb6` into `D:\Photos\tests\2026-10-10-facefree\dump\` (about 13
  minutes at 24 threads). Against the old dumps only 708 no-AF rows changed
  state (all `Unknown` -> a state) and no `sharpness` changed, which
  confirms Step 2 leaves the AF branch and the score alone.
- Only 244 bursts of two or more frames are entirely face-free; most
  face-free frames (3267 of 3970 in bursts) share a burst with a faced
  frame, so the relative form is also reported on those, against the whole
  burst, as `relativeSharpness` would see them.
- Tripped up: Python's text-mode `open(..., "w")` on Windows wrote CRLF into
  `plan.md`; write with `newline=""` (or strip the `\r` afterwards). And
  `riffle-cli.exe` needs `D:/...` paths, not Git Bash's `/d/...`, when
  called from Python.

## Step 4: Record that face-free frames stay Unknown (no ship)

- Decision C was already written with its numbers by Step 3, so Step 4
  only added the one-sentence reason to the Focus mark text of `README.md` /
  `README.ja.md` and `docs/humans/usage.md` / `usage.ja.md`, and the
  `todo.md` item "App: re-measure a Good / Bad for frames with no face".
- The plan's "57 rejects" (Context and Trade-offs) was corrected to 58,
  matching the count above.
- No app code changed; the `analyzed` flag of the ship branch was not
  added.

## Deferred issues (todo candidates)

- Pending manual check (Step 2, checkbox ticked on the automated criteria):
  on Windows or macOS, open `D:\photos\2026\2026-02-01` in a build with
  this change and let the second pass re-run (`FACES_VERSION` 9). Expected:
  Leica DNGs with a face get the strip's `scan-face` (good) or `scan` (bad)
  icon although `f` draws no crosshair; the filter's `AF eye` `Good` /
  `Bad` items list them; the meta pane shows `AF eye in focus`, `Eyes` and
  `Head pose` for them from the stored values, matching the on-demand mesh
  overlay; a DNG with no face stays without an icon and under `Unknown`.
  Repeat on a manual-focus ARW with a face (e.g.
  `D:\photos\samples\ARW\ILCE-6400_DSC00405.ARW` copied into a test folder
  under `D:\Photos\tests\`): same, and still no crosshair. Files:
  `crates/app/ui/src/focus.ts`, `crates/app/ui/src/main.ts`,
  `crates/app/src/index.rs`.
- Step 3 (Decision C, no ship): the face-free pick vs reject leg of the
  pre-registered criterion could not be tested (9 face-free rejects, none
  of the 20 good-mark rejects is face-free). Re-measure once a folder holds
  30 or more face-free rejects, or a cue other than the sharpness score
  exists for frames with no face. Basis: `sharpness-fallback.md`,
  `facefree.py` (this plan folder); Step 4 added it to `todo.md` as "App:
  re-measure a Good / Bad for frames with no face", so the wrap-up adds no
  second item. Files:
  `crates/app/ui/src/focus.ts` (where a rule would go).
