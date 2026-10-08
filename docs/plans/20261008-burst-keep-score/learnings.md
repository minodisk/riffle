# Learnings: keep-candidate score within a burst

## Step 1

- `riffle-cli features <dir>... [threads]` prints the header and the records
  on stdout and its totals (`N files ..., E with an error: Ts total`) on
  stderr, so `> folder.tsv` is the table alone. One rayon pool runs both the
  analysis and the eyes path per file; the per-file ms are taken inside the
  worker, so under 24 threads they include contention.
- The dump (release build copied to
  `D:\Photos\tests\2026-10-08-burst-keep-score\riffle-cli.exe`, 24 threads,
  one folder per run, [dump.sh](dump.sh)): 36 folders, 33,519 files, no
  error, **503 s wall** in total (the CLI's own totals, which leave out the
  model build, sum to 484 s; `2026-09-27-a`'s 4830 ARWs took 67 s, about 14 ms of wall per ARW).
  The DNG folders cost more: `2026-03-29` (197 DNGs) 9.5 s, `2026-05-26`
  (163 DeepPRIME DNGs) 10.2 s.
- Per-file ms on a worker (24 threads, so inflated by contention):
  ARW (32,459) analysis median 141.3 / p95 159.8 ms, eyes path median
  203.5 / p95 229.9 ms; DNG (1060) analysis median 309.1 / p95 750.4 ms,
  eyes path median 412.9 / p95 1150.0 ms. All files: analysis mean 149.1,
  max 1396.6 ms; eyes path mean 191.4, max 2632.8 ms. Not comparable with
  the single-threaded 80-92 ms of `eyes_of` in `docs/humans/performance.md`.
- **Virtual copies hide picks from `dop::read_flag`.** The first inventory
  gave `2026-06-05` zero picks and `2026-09-13-b` four, against 199 and 232
  `ShouldProcess = 0` lines and `Output/` files. In those folders the user
  picked a PhotoLab virtual copy: the `.dop`'s second item has
  `ShouldProcess = 0`, the master item `2`, and `Output/` holds
  `<stem>_1.jpg`. `read_flag` reads the first item only (as the app does),
  so the `dop` column says `None`. The label rule now also takes a frame
  whose `<stem>_<n>` is in `Output/`; a throwaway script checked that, in
  every sidecar folder of the data set, the frames with a pick on any item
  equal `Output/`'s frames exactly. The CLI was left as the plan says (no
  new parsing); the rule lives in [inventory.py](inventory.py) and
  [data.md](data.md).
- **DeepPRIME DNGs in the sidecar-less folders.** Most of the 2026-01 to
  2026-05 DNG folders hold `<stem>-DxO_DeepPRIME 3.dng` files, which
  `scan::is_raw_file` lists like any DNG: `2026-03-29` has 155 camera DNGs
  and 42 DeepPRIME ones, and `2026-01-08`, `2026-02-14`, `2026-05-26` have
  only DeepPRIME DNGs. `Output/` names them by the DeepPRIME stem. Counting
  files made several all-exported folders look partly exported (and
  `2026-04-10`'s two files are one exported frame). The inventory groups by
  the base stem (the `-DxO_...` suffix removed) and prefers the camera
  file's row. A DeepPRIME DNG has a full-size JPEG preview, the capture
  time, and no AF point.
- Two of the "Leica" `Output/` folders are Sigma BF (`BF_*.DNG`):
  `2026-05-22` (camera DNGs, every frame `af` but no cue face and no judged
  face, a night folder) and `2026-05-26` (DeepPRIME DNGs only). The
  stripping of a virtual copy's `_<n>` only applies when the stem is not a
  frame itself, since `BF_00634` already ends in `_<digits>`.
- Every ARW frame of the data set has a trusted AF point; no sidecar DNG
  frame has one, so the ARW eyes columns come from the AF path and the
  Leica ones from the whole-image path.
- In Git Bash, a `python - <<'EOF'` here-document mangled `'\\'` in one
  attempt; `chr(92)` or a script file avoided it.

## Step 2

- No `numpy` in the local Python 3.13, so [metrics.py](metrics.py) is
  standard library only (the AUC by rank sums, percentiles by index). It
  runs in about 1.5 s over all 36 dumps.
- **The relative sharpness hand cuts are unusable.** `rel < 0.7` fails
  60% of the ARW picks: the median pick is at 0.585 of its burst's
  maximum. The AF-region sharpness changes with what is under the AF point
  (a face, lettering, a ball) from frame to frame, so the hand check found
  sharp frames at `rel` 0.11-0.67. Only a 99% pick-keeping point
  (`rel >= 0.027`) protects the picks. See [results.md](results.md).
- **The first frame of a burst is a better ranking than any feature**
  (pairwise AUC 0.653 against sharpness 0.577): 39.5% of first frames are
  picks against 19.2% from the sixth on. Worth keeping in mind for Step 3
  (the capture position is not a technical check, so it is not a feature
  of the plan, but it is a baseline to beat).
- The pose separates nothing (pick false-fail equals the non-pick flag
  rate at every cut, AUC around 0.5), and the hand check found every
  pose-only flag a fine frame (profiles, faces looking up).
- `python -I` does not stop bytecode writes: a scratch script that put
  this folder on `sys.path` to import `metrics` left a `__pycache__/`
  here, removed before the commit. Python's text-mode `open(..., "w")` on
  Windows also wrote CRLF into `plan.md` and `metrics.py`; `sed -i
  's/\r$//'` put them back to LF. A Git Bash heredoc containing a quoted
  `'EOF'` and backticks failed to parse once; writing the text with the
  Write tool avoided it.
- The hand check (30 flagged non-picks of rule (e)) was the agent's own
  look at the crops, as the caller asked, not the user's: failed 9, OK 16,
  unsure 5. The crops are in
  `D:\Photos\tests\2026-10-08-burst-keep-score\handcheck\` for the user to
  re-check.

## Deferred issues (todo candidates)

- **The app's burst "best" mark compares different textures.** Found in
  Step 2 ([results.md](results.md), Reading): the sharpness cue that
  `crates/app/ui/src/sharpness.ts` `relativeSharpness` marks as the
  burst's best (Compare's green bar) ranks the user's picks only slightly
  above random (pairwise AUC 0.577 against 0.495, the first frame 0.653),
  and the hand check found sharp frames at 0.11-0.67 of the burst maximum,
  because the AF-region score follows the content under the AF point. If
  Step 3's Decision does not replace it, the mark's wording or the
  measurement (e.g. on the face only) may deserve its own todo item.
- **The head pose misreads a face behind a ball.** Hand check 16 of
  [results.md](results.md) (`D:\photos\2026\2026-07-05\_DSC2445.ARW`): a
  profile looking up behind the ball reads pitch -68 deg. Basis: Step 2
  hand check; related `crates/core/src/pose.rs`. Fits the pending review
  of the head-pose labels in `todo.md`.
