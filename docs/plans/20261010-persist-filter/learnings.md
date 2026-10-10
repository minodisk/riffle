# Learnings

## Step 1

- `filter_setting` checks `stars` with `Value::as_u64`, which is `None` for a
  float such as `2.5` and for a string such as `"3"`, so both drop without
  extra cases; a float like `5.0` drops too, which is fine because the UI
  sends integers.
- `cargo` is not on the Git Bash `PATH` in this worktree; run it through
  `mise exec -- cargo ...`.
