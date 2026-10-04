# Learnings: sony-arw-preview-full

## Step 1

- The fallback must live inside the non-DNG branch of `arw::parse` (the
  `else` of the strip gate). Applied after the whole `let (preview, full)`
  expression, it also hit the DNG strip results, whose synthetic test
  lengths do not grow with the pixel size, and broke
  `picks_dng_strip_jpegs_by_size_not_by_index`,
  `short_entries_ignore_the_padding_in_their_high_half` and
  `picks_dng_strip_jpegs_out_of_a_big_endian_sub_ifd_array`.
- Consumers checked by grep: `reader::read_full` (via `Kind::Full`), the CLI
  `info` / `bench` / `check`; `scan.rs` and `crates/app/src/index.rs` never
  read `full`, so no `EXTRACTOR_VERSION` bump.
- `riffle-cli bench` prints no pixel size; the size below was read from the
  SOF of the JPEG at `full`'s offset / length (printed by `riffle-cli info`).
  `full` equals the preview (same offset / length) on every sample:

  | Body | Sample | full offset / length | full decode |
  |---|---|---|---|
  | α9 II | `ILCE-9M2_sony_a9_ii_10.arw` | 135330 / 466710 | 1616x1080 |
  | α7R IV | `ILCE-7RM4_DSC00395.ARW` | 135330 / 553344 | 1616x1080 |
  | α7R IVA | `ILCE-7RM4A_DSC00551.ARW` | 135330 / 714371 | 1616x1080 |
  | α7C | `ILCE-7C_DSC00018.ARW` (4:3) | 135330 / 312841 | 1440x1080 |
  | α7C | `ILCE-7C_DSC00107[1].ARW` | 135330 / 447618 | 1616x1080 |
  | α6400 | `ILCE-6400_DSC00087.ARW` | 135330 / 288774 | 1616x1080 |
  | α6600 | `ILCE-6600_sony_a6600_10.arw` | 139426 / 912086 | 1616x1080 |
  | ZV-E10 | `ZV-E10_DSC00002.ARW` | 114850 / 430332 | 1920x1080 |

  The `bench` "JpgFromRaw full decode" is now 25-90 ms on these files (the
  thumbnail took 0.2 ms).
- `riffle-cli check D:\photos\samples\ARW`: 555 ok, 5 failed. The five are
  truncated files (`*_minimized_*` Shotkit α7C samples, one `Unknown_`, and
  `ILCE-9_SonyILCE9.arw`) that already fail at the scan and preview stages,
  independent of this change.
- `riffle-cli info` re-confirmed the AF point (`FocusLocation`) on every
  sample of the seven bodies and sub-second only on the α9 II, α7C and
  ZV-E10, matching the rows taken from the archived sony-arw-coverage table.
- Local checks: this worktree had no `node_modules`, so `mise run fmt` failed
  on the missing `vite-plus` binary until `pnpm install --frozen-lockfile`.
  One `mise run ci` run then failed with 22 vitest "Failed to start forks
  worker ... Timeout waiting for worker to respond" errors (all 149 UI tests
  passed; no UI file is touched here), a machine-load flake; the next run
  passed.

## Deferred issues (todo candidates)

- **Pending manual check (user)**: the 1:1 view in the running app on one of
  the seven bodies. Platform: Windows (or any desktop build). Steps: open
  `D:\photos\samples\ARW` (or a folder with e.g. `ILCE-6400_DSC00087.ARW` /
  `ZV-E10_DSC00002.ARW`), select the file, open the 1:1 focus check.
  Expected: a sharp crop of the 1616x1080 preview (1920x1080 on the ZV-E10)
  around the AF point, not a blurry upscaled 160x120 thumbnail. Step 1's
  checkbox was ticked on the automated criteria (unit tests, CLI `info` /
  SOF sizes, docs, `mise run ci`). Basis: plan Step 1 "Done when"; files
  `crates/core/src/arw.rs`, `crates/core/src/reader.rs`.
