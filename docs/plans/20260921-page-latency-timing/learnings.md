# Learnings

## Step 1

- `decode` is measured from just before `worker.postMessage` to the start of
  the `message` handler's success branch (before `draw()`), so it includes the
  worker queue wait and the bitmap transfer back, not only
  `createImageBitmap`.
- `move()` calls `show()` synchronously, so `pageKeypressAt` is set right
  before `show()` and consumed (or nulled on the `inFlight` no-op) within the
  same tick; the follow-up request issued after a stale response is untimed
  from the keypress, same as crops.

## Step 2

- Used a `log_timing` command rather than `tauri-plugin-log`'s JS API: the
  global `window.__TAURI__.log` would also need the `log:default` capability,
  which `crates/app/capabilities/default.json` does not grant. `debugLog`
  joins its args with `String` so the `zoom keypress` line (a label plus a
  number) is forwarded too.
