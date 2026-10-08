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

## Deferred issues (todo candidates)

- **The app's `.dop` reader ignores a picked virtual copy.** Found in
  Step 1 (the inventory): `dop::read_flag` (`crates/core/src/dop.rs`
  `locate`, first `Items` entry only) reads `None` for 199 frames of
  `2026-06-05` and 228 of `2026-09-13-b` whose virtual copy (a later
  item) is picked in PhotoLab. Riffle therefore shows those frames
  unflagged, and `trash.rs` `collect_folder` would not treat them as
  picked. Whether Riffle should read any item's flag (and which item it
  writes) is a product decision; this plan only works around it in the
  label rule ([data.md](data.md)).
- **Nearly-all-exported DNG folders left out.** `2026-02-21` (258 of 259
  frames exported), `2026-03-21` (454 of 460), `2026-04-26` (111 of 112) and
  `2026-05-02` (17 of 18) carry one to six negatives each but were excluded
  as "`Output/` not clearly smaller than the RAW count" (the data-set
  decision). If Step 2 wants more `Output/`-labeled negatives, they can be
  dumped with `dump.sh`'s loop and added to the `dng-output` block. Basis:
  the Step 1 inventory ([data.md](data.md), Excluded folders).
