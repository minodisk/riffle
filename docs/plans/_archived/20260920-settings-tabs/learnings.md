# Learnings

## Step 1

- Tab labels name each section, so the per-section `<h2>` headings were dropped.
- Generalizing `#debug[hidden]` to `[hidden] { display: none }` is needed because
  the tab buttons' `display` styling would otherwise override the `hidden`
  attribute.
- Hand verification in `cargo tauri dev` (Debug tab visibility, arrow keys vs.
  key capture) was not possible from the implementation agent; it remains for
  the user.
