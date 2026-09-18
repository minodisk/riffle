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

## Step 2

- `release.yml` is `workflow_dispatch` only and was verified pre-merge with
  actionlint (`mise run ci`). The four-runner dispatch run
  (`gh workflow run release.yml` on `main`) is pending post-merge; its run URL,
  the per-platform bundle + `.sig` artifacts, and any runner-specific fix
  (`nasm` on `macos-15-intel`, `rpm` on `ubuntu-latest`) are still to be
  recorded here.
- The cargo cache is keyed by `matrix.os`, not `runner.os`: `macos-latest`
  (arm64) and `macos-15-intel` (x86_64) share `runner.os == macOS` and would
  otherwise restore each other's `target`.
- tauri-action does not install JS dependencies itself, so the workflow runs
  `pnpm install --frozen-lockfile` before it.

## Step 3

- The release-shaping rules sit before `*.md` and `crates/*` in `classify()`,
  because the `case` is first-match; `crates/app/tauri.conf.json` would otherwise
  fall to the `crates/*` safe rule.
- This step's own PR touches `.claude/**`, so the check flags it as approval.
