# Learnings

## Step 1

- The symlink fallback lives in `folders.rs` as a private `kind(entry)`
  (is_dir, is_file) used by `folders::list`, plus a `pub(crate) is_file(entry)`
  wrapper for `commands.rs::list_dir`. A broken symlink is neither, so
  `list_dir` skips it as a RAW and still falls through to the sidecar check,
  as `path.is_file()` did before.
- `start_scan`'s body moved verbatim into a `spawn_blocking` closure; the early
  `return Ok(())`s return from the closure, and the outer `.await` maps a join
  error to `String`. No `.await` sits inside the critical section, so the
  `Scans` lock is still held continuously from the `latest_id` check through
  the `scan-state` emit, and the two lock comments still read true.
- `remember_folder` stays "logged, not returned": the join error and the store
  error are folded into one `Result` before the `eprintln!`.
- `cargo test list_` matches no test names in this crate (the listing tests
  are named `one_listing_...`, `unreadable_directory_...` etc.); the new
  `symlinked_raw_file_is_listed` is `#[cfg(unix)]`, so it compiles and runs
  only on macOS/Linux CI, not on the Windows machine this step was built on.
- The manual GUI check in the plan (open list timings under a running scan,
  clicking the blank row area, the arrow and the count badge) could not be
  done by the implementer and is left to the user.
