# Learnings

## Step 1

- `labels` in `main.ts` stores the capitalised label name (`"Red"`), while the
  action names are lower-case (`red`); the checked test compares against the
  capitalised name.
- The menu buttons use `role="menuitemradio"` since `aria-checked` is not a
  valid attribute on a plain `menuitem`.
- A rating of `0` (as well as `null`) counts as "No stars".
- Not verified manually on a small window (no GUI in this environment);
  `menuPosition` clamping and `overflow-y: auto` are unchanged.
