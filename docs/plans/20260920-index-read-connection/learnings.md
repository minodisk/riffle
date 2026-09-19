# Learnings

## Step 1

- `rusqlite::Connection` is `Send` but not `Sync`, so the concurrency test has
  to wrap the reader in a `Mutex` (as production does) to share it with a
  scoped thread.
- `SQLITE_OPEN_READ_ONLY` worked locally (macOS); no fallback to
  `READ_WRITE` was needed so far.
- `write_batch` cost, 5000 synthetic rows (`entry()` fixture), release build,
  Apple M3 Pro / 36 GB, three runs (a debug build overstates these):

  | chunk | total ms          | ms per transaction     |
  | ----- | ----------------- | ---------------------- |
  | 10    | 49.6 / 53.5 / 56.0 | 0.099 / 0.107 / 0.112 |
  | 50    | 38.5 / 34.0 / 37.3 | 0.385 / 0.340 / 0.373 |
  | 100   | 35.8 / 31.4 / 36.3 | 0.717 / 0.629 / 0.725 |

- BATCH decision: 10 vs 50 costs ~12-19 ms per 5000 files, i.e. ~0.2-0.35%
  of the README's 5.55 s 5000-file first-scan baseline, well under the 2%
  threshold, so `BATCH` stays 10 and the cancel test's `BATCH * 40` fixture
  is unchanged. Not a non-obvious enough fact to add to `docs/agents/tauri-app.md`.
