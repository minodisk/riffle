# Learnings

## Step 1

- Granularity: the log's `files=1 ... canceled=true in 8030ms` means a single
  file's pipeline took 8 s, so one stage (a cold `read_preview` on a contended
  disk, or the whole-image YuNet inference / mozjpeg decode under 22 competing
  threads) can still be seconds long. The stage-level checks bound the wait to
  one stage per worker, not zero; going inside mozjpeg or the ONNX inference
  is out of scope.
- The first pass checks after `read_preview` (which is also "before the
  thumbnail encode"), after the thumbnail encode (which is also "before the
  whole-image `detect_around`", nothing costly sits between them), and before
  `score_preview`. In `focus_cue_unless` the check after `decode_rgb` and the
  one before `detect_around_rgb` are likewise one check: only the cheap luma
  conversion sits between them.
- `extract` / `extract_faces` / `focus_cue` wrap their cancel-aware variants
  with a never-set flag and `expect`, so the public signatures (and
  `crates/cli`) are unchanged.
- Testing a mid-file cancel through `for_each_path` deterministically: a
  `per_file` closure that sets the flag itself before calling the variant gets
  past the pre-dispatch check, so every file is abandoned and `on_item` is
  never called.
- `cargo` is not on the Bash tool's PATH here; run it through `mise exec --`.
- `README.md` / `README.ja.md` do not describe cancel behavior; no change.
- The manual GUI check (open a large folder, then another, and compare the
  `scan extract ... canceled=true in Nms` time in Riffle.log) is left to the
  user.
