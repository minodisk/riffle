# Learnings

## Step 1

- The scratchpad render was still present with the recorded sha256, so it was
  copied with `cp`; `cmp` against `crates/app/icons/source.png` exits 0.
- `pnpm exec tauri icon` (tauri-cli 2.11.5) regenerated all 17 bundled files
  in place and again wrote `ios/` and `android/`, removed afterwards as the
  established procedure says. `menu/`, `tauri.conf.json` and the READMEs have
  no diff.
- At 32x32 the green sticker survives as a few green pixels at the top-left of
  the photo (8 pixels by a simple green threshold), so it reads as a dot; the
  edge print is texture, as the plan accepts. Corners are fully transparent at
  both 32px and 256px.
- `cargo build -p riffle-app` succeeds with the new `icons/128x128.png`
  embedded by `main.rs`.

## Step 2

- Before cleaning, the scratchpad `gen.py` run in a fresh folder wrote an
  `icon.svg` byte-identical to the scratchpad `icon.svg`, so the approved
  render's SVG came from that script as-is.
- After the cleanup (unused `stickerfill` / `stickershadow` defs removed,
  comments fixed, English header, paths from `Path(__file__)`, Chrome run by
  the script), `python3 tools/macos/app-icon/gen.py` run from `/tmp` wrote a
  `crates/app/icons/source.png` that is byte-identical to Step 1's (`cmp`
  exits 0, 379142 bytes), so no Pillow pixel comparison was needed. The only
  difference in the new `icon.svg` from the scratchpad one is the removed defs.
- Headless Chrome prints a series of `CVDisplayLinkCreateWithCGDisplay failed`
  and `task_policy_set` errors to stderr on every run; they are harmless and
  the screenshot is still written.
- `tools/macos/app-icon/photo.png` is a byte copy of the scratchpad photo
  (sha256 `abb28d5d...4638`, as recorded in the plan).
