# Learnings

## Step 1

- The default lives only in `auto_advance_setting` (`crates/app/src/commands.rs`);
  flipping `unwrap_or(false)` to `unwrap_or(true)` covers both the absent-key and
  the store-open-failure paths of `load_settings`. The frontend placeholders
  (`let autoAdvance = false;` in `main.ts`, the unchecked checkbox in
  `index.html`) were left as planned (Option A in the plan's Trade-offs).
