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
