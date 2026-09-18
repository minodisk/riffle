# Learnings

## Step 1

- Putting a `plugins.updater` block in `tauri.conf.json` makes
  `tauri::generate_context!()` expand to code that references `serde_json`, so
  `crates/app` needs `serde_json` as a direct dependency or the build fails
  with "could not find `serde_json` in the list of imported crates".
- Local `pnpm tauri build --bundles app` with a throwaway key (generated in a
  temp dir, then deleted) produced `target/release/bundle/macos/Riffle.app.tar.gz`
  and `Riffle.app.tar.gz.sig`. Tauri warns that the key does not match the
  committed pubkey, which is expected for a throwaway key.
- Manual check awaiting the user's confirmation: in `mise run app`,
  `typeof window.__TAURI__.updater.check === "function"` in the devtools
  console.
