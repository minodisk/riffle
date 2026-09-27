# Learnings: dop-photolab-uuids

## Step 1

- `uuids: Option<Uuids>` went in as the last parameter of `dop::write_rating` /
  `dop::write_label` (after `now`, matching `template`'s order). That makes
  `write_rating` seven parameters, which is exactly clippy's
  `too_many_arguments` limit, so no `#[allow]` was needed.
- The owned `Option<Uuids>` (not `Option<&Uuids>`) keeps the fresh path free of
  clones: the value moves straight into `template`. Step 2 hands the same
  lookup result to both `write_rating` and `write_label` only when both mint,
  which never happens (the label write patches the rated bytes), so moving it
  into the first call is enough.
- The test helper that pulls the Uuids back out of a template splits on the
  `}\n,\n}\n,\n` that closes the item and `Items`; the item Uuid is the first
  `Uuid = "..."` line before it and the source Uuid the first one after it.
- A Python heredoc edit that wrote Rust `"\n"` escapes produced literal
  newlines inside the string; editing Rust string escapes through a second
  language's escaping is error-prone, prefer the Edit tool for those lines.
