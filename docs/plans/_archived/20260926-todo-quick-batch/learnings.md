# Learnings

## Step 1

- `EXTRACTOR_VERSION` bumped from 5 to 6. The `DSC-` gate changes two stored
  outputs of `extract` for older `DSC-` bodies with `FocusMode` 0: the
  `manual_focus` column and the `sharpness` score (the AF window instead of
  the face's eyes or the tile maximum). The re-extract also resets
  `faces_extractor` to 0, so the second pass re-runs on those rows and their
  `eye_focus` follows the new `trusted_focus`; no `FACES_VERSION` bump needed.
- The DSC gate lives next to `Shot` in `crates/core/src/arw.rs` as
  `pub fn excluded_dsc`; `DSC_EXCEPTIONS` stays private since only the
  function crosses the crate boundary.
- `wsl_drive_mounts` in `crates/app/src/folders.rs` is compiled under
  `cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))`, so
  its test runs on every platform while macOS/Windows builds get no dead-code
  warning.
- `grep -n "Debug > Timing" docs/**/*.md` is not literally empty: the phrase
  survives in this plan's own text and in archived plans under
  `docs/plans/_archived/` (historical records, left untouched). The live doc
  `docs/performance.md` no longer contains it.
- A fresh worktree has no `node_modules`, so `mise run fmt` fails with
  `Command "vp" not found`; `mise exec -- pnpm install --frozen-lockfile`
  fixes it (plain `pnpm` is not on the Git Bash `PATH`).
