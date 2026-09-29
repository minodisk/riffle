# Learnings: olympus-orf

## Step 1

- `Tiff::new` now delegates to `new_with_magic(buf, base, end, &[42])`, which
  reads the byte order and then builds the walker through `with_order`. Routing
  both new constructors through the existing one keeps them in use before
  Step 2's `orf.rs` calls them, so no `allow(dead_code)` was needed.
- `with_order` checks only `base <= end <= buf.len()`; it asks for no minimum
  length, since a MakerNote segment is bounded by the prefix and every read
  is range-checked against `end` anyway.
- `cargo` is not on the Git Bash `PATH` here; run it through
  `mise exec -- cargo ...`.
