# Learnings

## Step 1

- Joining the reader thread before `ROLLBACK` makes the test structural: a
  blocked reader would return `SQLITE_BUSY` after the 300 ms `busy_timeout`
  set in `Index::open_reader`, so the read's `unwrap()` (and hence
  `join().unwrap()`) fails. No timer left in the test.
- `Duration` and `Instant` stay imported in `crates/app/src/index.rs`: both are
  still used by production code and by other tests, so nothing became unused.
