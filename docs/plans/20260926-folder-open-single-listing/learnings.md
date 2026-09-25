# Learnings

## Step 1: Reuse `list_arw`'s listing in `scan_folder`

- `list_dir` split into `read_listing` (one `read_dir`: sorted RAW paths plus
  the format's sidecars as `(lower-cased name, path)`, no stats) and
  `stat_sidecars` (the `SidecarStat` map `reconcile_sidecars_of` takes).
  `list_arw` keeps only the unstatted listing, so the first paint pays no
  per-sidecar `stat` (the cost PR #449 removed); `scan_folder` stats the
  sidecars after taking the listing, in the `spawn_blocking` that used to list.
- The cache is `AppListing(Mutex<Option<CachedListing>>)` in `commands.rs`,
  registered next to `Scans` in `main.rs`, keyed by canonical dir, sidecar
  format and the directory's mtime. `take_listing` is the pure reuse decision:
  it takes the listing only on a full match (so it is reused once), and leaves
  a non-matching one in place for the `scan_folder` it may belong to. A failed
  directory stat (`None` mtime) never matches, and `list_arw` caches nothing
  when it cannot read the mtime.
- Staleness: the directory mtime is read just before `list_arw`'s `read_dir`
  (not after), so a file added or removed during or after the listing bumps
  the mtime past the cached value and `scan_folder` re-lists. That closes the
  window between `list_arw`'s `read_dir` and `scan_folder`'s `watch::set`;
  changes after `watch::set` reach the watcher (`folder-changed` -> resync).
  The remaining weakness is a coarse mtime (FAT's 2 s), where a change in the
  same tick can slip past the check. Documented on `AppListing`.
- After the split, `list_arw_in` and `list_folder_in` are only used by tests,
  so both are `#[cfg(test)]` (otherwise the non-test build warns about dead
  code). `list_folder_in` stays as the composition of the two halves.
- The `scan list:` log line gains `reused=true|false`; its `ms` now covers the
  sidecar stats plus, when not reused, the `read_dir`.
- `cargo` is not on the Git Bash PATH here; `mise exec -- cargo test -p
  riffle-app <filter>` works for a quick run.
