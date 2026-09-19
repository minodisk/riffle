# Learnings

## Step 1

- `keyName` takes a structural `Pick<KeyboardEvent, ...>` so Vitest (node
  environment, no `KeyboardEvent`) can pass plain objects.
- Rust tests for `add` / `from_overrides` pick the forbidden key by
  `cfg!(target_os = "macos")` (the `MACOS` const), while `forbidden` itself is
  tested with both platform flags so both lists run on every CI OS.
- Manual GUI confirmation (Ctrl+Alt+1 under `.dop`, Shift+J no longer paging,
  binding `meta+k`, `Cmd+W` during capture) is left to the user; not verified
  here.

## Deferred issues (todo candidates)

- None.
