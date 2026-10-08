# Data set and labels

The folders under `D:\photos\2026` the burst keep-candidate score is measured
on, the label rules, and the inventory of the Step 1 dump (`riffle-cli
features`, 2026-10-08). The dumps are one `<folder>.tsv` per folder, with
`<folder>.output.txt` (the stems of `Output/`, extension removed) next to
each, under `D:\Photos\tests\2026-10-08-burst-keep-score\dump\`.
[`dump.sh`](dump.sh) wrote them; [`inventory.py`](inventory.py) reads
them into the table below.

## Label rules

Decided by the user on 2026-10-08, with two refinements found while taking
the inventory (the virtual copies and the DeepPRIME DNGs below).

- **A frame is one shot, named by its base stem.** In the DNG folders
  without sidecars, DxO PhotoLab's DeepPRIME output DNGs
  (`<stem>-DxO_DeepPRIME 3.dng`) sit next to the camera files. They are
  copies of a shot, not shots, so the base stem drops the `-DxO_...` suffix.
  When a folder has both files, the camera file's row stands for the frame;
  when it has only the DeepPRIME DNG (`2026-01-08`, `2026-02-14`,
  `2026-05-26`), that DNG's row is used. The DeepPRIME DNG carries a
  full-size JPEG preview, the same capture time and no AF point.
- **Sidecar folders: the `.dop` flag is authoritative, the XMP flag counts
  only for a frame with no `.dop`.** The dump's `dop` column is
  `dop::read_flag`, which reads the `.dop`'s first item only.
- **A picked virtual copy picks its frame.** In `2026-06-05` (199 frames)
  and `2026-09-13-b` (228 frames) the user picked a PhotoLab virtual copy
  (the `.dop`'s second item, `ShouldProcess = 0`) and left the master item
  unflagged (`ShouldProcess = 2`), so the `dop` column reads `None` for
  them. `Output/` holds the virtual copy's export as `<stem>_<n>.jpg`. A
  frame whose stem appears in `Output/` as `<stem>_<n>` is therefore a pick.
  This is the same set as "any item of the `.dop` has `ShouldProcess = 0`":
  in every sidecar folder of the data set, the frames with a pick on any
  `.dop` item equal the frames of `Output/` exactly (checked with a
  throwaway script on 2026-10-08).
- **DNG folders without sidecars: a frame whose base stem is in `Output/`
  is a pick.** These folders are reported in their own rows (`dng-output`),
  apart from the sidecar-labeled ones.
- **Everything else is a non-pick, which is unlabeled**: it failed a
  technical check, or it passed and was not chosen (see the plan's Purpose).
  A reject is the flag that decided the frame (the `.dop`'s, else the
  XMP's) being `Reject`. Rejects are counted here but carry no more label
  weight than any other non-pick.

## Data set

36 folders, 33,464 frames (33,519 files).

- **ARW, sidecar-labeled (27 folders, Sony):** `2026-06-05`, `2026-06-13`,
  `2026-06-14`, `2026-06-20`, `2026-07-02`, `2026-07-05`, `2026-07-11`,
  `2026-07-18`, `2026-07-21`, `2026-07-22`, `2026-07-23`, `2026-07-24`,
  `2026-07-26`, `2026-07-30`, `2026-07-31`, `2026-08-01`, `2026-08-02`,
  `2026-08-08`, `2026-08-15`, `2026-08-16`, `2026-08-22`, `2026-08-29`,
  `2026-09-13-b`, `2026-09-19`, `2026-09-27-a`, `2026-09-27-b`,
  `2026-10-03`.
- **DNG, sidecar-labeled (2 folders, Leica M11-P):** `2026-08-29-l`,
  `2026-09-05`.
- **DNG, `Output/`-labeled (7 folders):** `2026-01-08`, `2026-02-14`,
  `2026-03-29`, `2026-04-12`, `2026-04-18` (Leica M11-P), `2026-05-22`,
  `2026-05-26` (Sigma BF). `2026-05-22` is the only folder of these whose
  frames have an AF point (`af`); it still has no cue face.

## Excluded folders

| Reason | Folders |
| --- | --- |
| Unfinished (culling not done) | `2026-09-13-a`, `2026-09-27-c` |
| Every frame picked (sidecars) | `2026-06-02` (DNG), `2026-06-06`, `2026-06-09`, `2026-06-21`, `2026-06-27` |
| Every frame exported (`Output/`) | `2026-01-01`, `2026-01-03`, `2026-01-09`, `2026-01-25`, `2026-01-31`, `2026-02-01`, `2026-02-08`, `2026-02-15`, `2026-02-22`, `2026-03-07`, `2026-03-08`, `2026-03-15`, `2026-03-22`, `2026-03-28`, `2026-03-31`, `2026-04-10`, `2026-04-19`, `2026-04-25`, `2026-05-01`, `2026-05-03`, `2026-05-05`, `2026-05-09`, `2026-05-11`, `2026-05-14`, `2026-05-17`, `2026-05-18`, `2026-05-23` |
| All but one to six frames exported (`Output/` not clearly smaller than the frame count) | `2026-02-21` (258 of 259), `2026-03-21` (454 of 460), `2026-04-26` (111 of 112), `2026-05-02` (17 of 18) |
| No label (no sidecars, no `Output/`) | `2026-01-10`, `2026-02-24`, `2026-02-26`, `2026-02-27`, `2026-03-19`, `2026-03-23` |
| No RAW file | `2026-04-08` (JPEGs and `Output/` only), `100LEICA` (an undated card copy, no labels) |

`2026-04-10` has two files, but they are one frame (the camera DNG and its
DeepPRIME DNG), exported, so it has no negative.

## Inventory

Counted per frame. AF: a trusted AF point (`af`). Cue face: the scan's cue
found a face near the AF point (`cue_side` present). Judged face: the face
`eyes_of` judges is at least `EYES_MIN_FACE` = 60 px. Pose: the head pose
solved. `.dop` picks: the `dop` column (first item) Pick. XMP picks: the
`xmp` column Pick, whatever the `.dop` says. `Output/`: frames in `Output/`.
Virtual-copy picks: frames in `Output/` only as `<stem>_<n>`. Picks (rule):
the label rules above. XMP / `.dop` disagree: frames with both sidecars
whose flags differ (in `2026-09-13-b` all 228 are the virtual-copy picks:
XMP Pick, master item unflagged). Errors: rows with `err` in any column.

| Folder | Kind | Frames | AF | Cue face | Judged face | Pose | `.dop` picks | XMP picks | `Output/` | Virtual-copy picks | Picks (rule) | Rejects | XMP / `.dop` disagree | Errors |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | arw | 1337 | 1337 | 908 | 834 | 829 | 0 | 0 | 199 | 199 | 199 | 0 | 0 | 0 |
| `2026-06-13` | arw | 933 | 933 | 772 | 617 | 612 | 118 | 0 | 118 | 0 | 118 | 0 | 0 | 0 |
| `2026-06-14` | arw | 1520 | 1520 | 1360 | 859 | 845 | 161 | 0 | 161 | 0 | 161 | 0 | 0 | 0 |
| `2026-06-20` | arw | 1050 | 1050 | 948 | 744 | 736 | 133 | 0 | 133 | 0 | 133 | 0 | 0 | 0 |
| `2026-07-02` | arw | 106 | 106 | 106 | 106 | 106 | 15 | 0 | 15 | 0 | 15 | 0 | 0 | 0 |
| `2026-07-05` | arw | 1415 | 1415 | 1231 | 866 | 846 | 255 | 0 | 255 | 0 | 255 | 0 | 0 | 0 |
| `2026-07-11` | arw | 468 | 468 | 441 | 439 | 438 | 57 | 0 | 57 | 0 | 57 | 0 | 0 | 0 |
| `2026-07-18` | arw | 1545 | 1545 | 1196 | 1082 | 1078 | 292 | 0 | 292 | 0 | 292 | 0 | 0 | 0 |
| `2026-07-21` | arw | 32 | 32 | 30 | 24 | 24 | 7 | 0 | 7 | 0 | 7 | 0 | 0 | 0 |
| `2026-07-22` | arw | 189 | 189 | 176 | 139 | 136 | 32 | 0 | 32 | 0 | 32 | 0 | 0 | 0 |
| `2026-07-23` | arw | 18 | 18 | 17 | 16 | 16 | 4 | 0 | 4 | 0 | 4 | 0 | 0 | 0 |
| `2026-07-24` | arw | 218 | 218 | 173 | 139 | 138 | 34 | 0 | 34 | 0 | 34 | 0 | 0 | 0 |
| `2026-07-26` | arw | 64 | 64 | 52 | 52 | 52 | 7 | 0 | 7 | 0 | 7 | 0 | 0 | 0 |
| `2026-07-30` | arw | 936 | 936 | 860 | 783 | 781 | 105 | 0 | 105 | 0 | 105 | 0 | 0 | 0 |
| `2026-07-31` | arw | 349 | 349 | 289 | 162 | 158 | 52 | 0 | 52 | 0 | 52 | 0 | 0 | 0 |
| `2026-08-01` | arw | 3401 | 3401 | 2868 | 1822 | 1786 | 463 | 0 | 463 | 0 | 463 | 0 | 0 | 0 |
| `2026-08-02` | arw | 150 | 150 | 129 | 125 | 125 | 28 | 0 | 28 | 0 | 28 | 0 | 0 | 0 |
| `2026-08-08` | arw | 415 | 415 | 367 | 358 | 354 | 73 | 0 | 73 | 0 | 73 | 0 | 0 | 0 |
| `2026-08-15` | arw | 324 | 324 | 288 | 288 | 288 | 37 | 0 | 37 | 0 | 37 | 0 | 0 | 0 |
| `2026-08-16` | arw | 59 | 59 | 59 | 59 | 59 | 6 | 0 | 6 | 0 | 6 | 0 | 0 | 0 |
| `2026-08-22` | arw | 2677 | 2677 | 2481 | 1863 | 1838 | 266 | 1 | 266 | 0 | 267 | 0 | 0 | 0 |
| `2026-08-29` | arw | 3451 | 3451 | 3157 | 2043 | 2015 | 475 | 0 | 475 | 0 | 475 | 0 | 0 | 0 |
| `2026-09-13-b` | arw | 1795 | 1795 | 1513 | 1097 | 1079 | 4 | 232 | 232 | 228 | 232 | 7 | 228 | 0 |
| `2026-09-19` | arw | 2134 | 2134 | 1952 | 1691 | 1669 | 282 | 287 | 282 | 0 | 282 | 50 | 5 | 0 |
| `2026-09-27-a` | arw | 4830 | 4830 | 4367 | 3289 | 3231 | 729 | 731 | 729 | 0 | 729 | 1 | 2 | 0 |
| `2026-09-27-b` | arw | 1240 | 1240 | 1082 | 832 | 818 | 171 | 171 | 171 | 0 | 171 | 0 | 0 | 0 |
| `2026-10-03` | arw | 1803 | 1803 | 1615 | 1397 | 1383 | 186 | 191 | 186 | 0 | 186 | 0 | 5 | 0 |
| `2026-08-29-l` | dng-sidecar | 211 | 0 | 0 | 200 | 199 | 33 | 0 | 33 | 0 | 33 | 0 | 0 | 0 |
| `2026-09-05` | dng-sidecar | 411 | 0 | 0 | 204 | 204 | 66 | 1 | 66 | 0 | 67 | 1 | 1 | 0 |
| `2026-01-08` | dng-output | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 2 | 0 | 0 | 0 |
| `2026-02-14` | dng-output | 12 | 0 | 0 | 11 | 11 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | 0 |
| `2026-03-29` | dng-output | 155 | 0 | 0 | 105 | 105 | 0 | 0 | 25 | 0 | 25 | 0 | 0 | 0 |
| `2026-04-12` | dng-output | 10 | 0 | 0 | 4 | 4 | 0 | 0 | 7 | 0 | 7 | 0 | 0 | 0 |
| `2026-04-18` | dng-output | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | 0 |
| `2026-05-22` | dng-output | 37 | 37 | 0 | 0 | 0 | 0 | 0 | 19 | 0 | 19 | 0 | 0 | 0 |
| `2026-05-26` | dng-output | 163 | 0 | 0 | 130 | 130 | 0 | 0 | 15 | 0 | 15 | 0 | 0 | 0 |
| **Total** | arw | 32459 | 32459 | 28437 | 21726 | 21440 | 3992 | 1613 | 4419 | 427 | 4420 | 58 | 240 | 0 |
| **Total** | dng-sidecar | 622 | 0 | 0 | 404 | 403 | 99 | 1 | 99 | 0 | 100 | 1 | 1 | 0 |
| **Total** | dng-output | 383 | 37 | 0 | 250 | 250 | 0 | 0 | 70 | 0 | 70 | 0 | 0 | 0 |

Reading the table:

- Every ARW frame has a trusted AF point, so the ARW eyes columns all come
  from the AF path (the face nearest the AF point); every sidecar DNG frame
  takes the whole-image path (the largest confident face).
- The rule's picks exceed `Output/` by one in `2026-08-22` and
  `2026-09-05`: a frame with an XMP pick and no `.dop`, never exported.
- The `dng-output` folders hold 70 picks among 383 frames; four of them
  (`2026-01-08`, `2026-02-14`, `2026-04-18`, and `2026-05-22`, which has no
  face) give a handful of frames or none with a judged face, so that block
  rests mostly on `2026-03-29` and `2026-05-26`.
