# Learnings

## Step 1

- The grayscale action is handled before `runAction` in the keydown handler,
  since it is the only action tied to a physical key (`event.code`) and a
  keyup. `isModifierCode` was exported from `keys.ts` to test the modifier
  release path.
- Manual GUI verification (hold/release, auto-repeat, `ctrl+g` rebind release
  order, focus loss via `blur`) is pending for the user; if `blur` does not
  fire on WebView2/WKWebView, fall back to `getCurrentWindow().listen("tauri://blur", ...)`.

## Deferred issues (todo candidates)

- (none)

## CI notes

- `a_modifier_only_key_is_dropped_from_a_mixed_override` rebound `zoom` to
  `g`, which now collides with the new default; the test was switched to `m`.
