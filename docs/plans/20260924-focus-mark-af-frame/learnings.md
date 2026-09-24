# Learnings

## Step 1

- The frame travels to the frontend as `focus.frame: { width, height } | null`
  (a `FocusSize` struct in `index.rs`) rather than two nullable numbers, so
  the TypeScript side checks one field. `manual_focus` sits next to it on the
  same `Focus`, so a row without a focus point carries neither.
- The rectangle is added to the crosshair's path with `context.rect`, so the
  existing two stroke passes draw it with no extra code; `lineCap = "square"`
  only affects the open crosshair arms, the closed rectangle uses the default
  miter join.
- `focus.test.ts` uses draw sizes that divide the sensor size exactly (700 x
  468 for 7008 x 4672) so `toEqual` needs no float tolerance.
- The v12 migration fixture also sets `extractor = 1` before faking v12, so
  the test shows the rows survive the `ALTER` and are then re-extracted
  because of the `EXTRACTOR_VERSION` bump alone.
- The first `mise run fmt` in this fresh worktree failed with `Command "vp"
  not found` although `node_modules/.bin/vp` existed; `pnpm install
  --frozen-lockfile` ("Already up to date") made it work, and `mise run ci`
  then passed.
