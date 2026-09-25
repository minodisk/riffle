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
