# Learnings: focus-candidate

## Step 1

- Manual check: `riffle-cli candidates <dir>` (release build, 24 threads) on
  the five labeled folders under `/mnt/d/Photos/2026/`, 100 labeled files
  each, no errors. "Precision" is candidates in focus / candidates, "recall"
  is candidates in focus / in-focus frames (`xmpDM:good="True"`).

  | Folder | Candidates | In focus | Precision | In-focus frames | Recall | Not candidate | Unknown | Wall |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | `2026-09-19-focus-sample` | 66 | 64 | 97% | 75 | 85% | 14 | 20 | 2.35 s |
  | `2026-09-19-focus-sample-2` | 65 | 58 | 89% | 70 | 83% | 18 | 17 | 1.19 s |
  | `2026-07-31-focus-sample` | 80 | 80 | 100% | 93 | 86% | 1 | 19 | 0.87 s |
  | `2026-09-13-a-focus-sample` | 60 | 54 | 90% | 78 | 69% | 24 | 16 | 3.28 s |
  | `2026-06-05-focus-sample` | 64 | 54 | 84% | 73 | 74% | 14 | 22 | 0.85 s |
  | Pooled (500) | 335 | 310 | 93% | 389 | 80% | 71 | 94 | |

  The pooled counts are identical to the analysis behind the plan (335
  candidates, 310 in focus, 389 in-focus frames), so the luma, the window and
  the nearest-face rule reproduce it exactly. The first folder's wall time
  includes building the YuNet model and a cold file cache.
- `FocusLocation` lives in the Sony MakerNote, so the synthetic TIFF fixtures
  in `scan.rs` cannot carry an AF point without building an Exif IFD and a
  MakerNote. The decode-failure / panic path with an AF point is tested on
  the private `scan::cue` helper (the `catch_unwind` around `focus_cue`)
  instead, and `extract_faces` itself is tested on the no-AF and the
  unreadable-file cases.

## Step 2

- Version check: main had not moved past the plan's assumptions when this
  step started (`SCHEMA_VERSION = 14`, `EXTRACTOR_VERSION = 4`), so the
  schema goes to v15 and the extractor stays at 4 exactly as planned. If a
  concurrent change lands a v15 first, the `(10..15)` / `version == 14`
  guards, the `prepare` whitelist and every fixture's `user_version` move up
  by one on rebase.
- `EXTRACTOR_VERSION` stays 4: with a trusted AF point, `extract` used to run
  the crop detection only for the face-catch state and passed an empty face
  list to `score_preview` either way (both the eye-AF and the crop branch),
  and `detect_around` returns `point: Some` whenever `focus` is `Some`. The
  no-AF branch still runs the whole-image detection and passes `d.faces`, so
  the thumbnail, the metadata and the score are unchanged.
- `faces_todo`, `write_faces` and `FACES_VERSION` have no caller outside the
  tests until Step 3, so they carry `#[cfg_attr(not(test), expect(dead_code))]`
  (the `Index::clear` precedent). `clippy --all-targets -D warnings` reports
  the constant too, since dead-code analysis does not count uses from dead
  functions; Step 3 must drop all three attributes, or `expect` fails.
- The manual-focus fixture in the index tests sets `shot.focus_mode =
  Some(0)` (`sharpness::MANUAL_FOCUS`), the same as the existing AF-frame
  round-trip test.
- Also fixed the Step 1 `Cue::detection` doc comment: it is `None` without an
  AF point and also in the `Cue::unknown()` default that `scan::cue` falls
  back to after a decode / detection error or a panic.
- `docs/usage.md` still describes the face-catch colors; the plan leaves the
  user docs to Step 4, and until Step 3 every mark is white.
