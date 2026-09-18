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
  actionlint (`mise run ci`). The post-merge dispatch run
  (https://github.com/minodisk/riffle/actions/runs/35321308413) succeeded on all
  four runners: `macos-latest` built the `.dmg` and `Riffle.app.tar.gz` + `.sig`;
  `macos-15-intel` succeeded without `nasm`; `windows-latest` built the `.msi`
  and `-setup.exe`, each with `.sig`; `ubuntu-latest` built the `.deb`, `.rpm`
  and `.AppImage`, each with `.sig`. No runner-specific fix was needed.
- Defect: `uploadWorkflowArtifacts` is not a tauri-action@v0 input. The run
  logged "Unexpected input(s) 'uploadWorkflowArtifacts'" and has zero
  artifacts. Fixed in Step 4 (see there). Step 2's artifact criterion is only
  satisfied once Step 4 is merged and a dispatch run shows the artifacts.
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

## Step 4

- The `rust` strategy at the root fails on this workspace. Dry run
  (`release-please release-pr --dry-run --release-type rust`, v17.9.0) aborted
  with `Error: is not a package manifest (might be a cargo workspace)` from
  `CargoToml.updateContent`: the strategy updates the root `Cargo.toml` as a
  package even when it has no `[package]`. Fell back to `simple` with
  `extra-files` (Decision 1 updated). `Cargo.lock` does not go stale after
  all: a `toml` extra-file with `$.package[?(@.source === undefined)].version`
  bumps exactly the three workspace members. Equality filters on `@.name`
  matched nothing and `startsWith`/`match` are rejected by jsonpath-plus's
  safe evaluator, so `source` is the working discriminator. The Trade-offs
  bullet on `Cargo.lock` drift is therefore moot.
- `package.json` had no `version`; `GenericJson` silently skips a missing
  path, so the field was added at `0.1.0`.
- The `release-pr --dry-run` CLI reads the config from the remote branch, so a
  config not yet on `main` fails with "Missing required manifest config". The
  dry run used `--local --local-path <scratch clone>` against a scratch
  `origin` (a GitHub clone of `main` at f1ba56d plus a commit adding the config,
  plus an empty `feat(app):` commit because `bootstrap-sha` is `main` HEAD and
  nothing releasable follows it otherwise). Output:

  ```
  Would open 1 pull requests
  title: chore(main): release 0.1.1
  branch: release-please--branches--main
  updates: 9
  file version.txt did not exist
    CHANGELOG.md:  [class Changelog extends DefaultUpdater]
    version.txt:  [class DefaultUpdater]
    crates/app/Cargo.toml:  [class GenericToml]
    crates/core/Cargo.toml:  [class GenericToml]
    crates/cli/Cargo.toml:  [class GenericToml]
    Cargo.lock:  [class GenericToml]
    package.json:  [class GenericJson]
    crates/app/tauri.conf.json:  [class GenericJson]
    .release-please-manifest.json:  [class ReleasePleaseManifest extends DefaultUpdater]
  ```

  `version.txt` is `createIfMissing: false`, so it is not created. The dry run
  lists updaters, not diffs; applying each configured updater to the working
  tree at 0.1.1 changed exactly `version = "0.1.0"` in the three `Cargo.toml`,
  the three `riffle-*` entries of `Cargo.lock`, and `"version"` in
  `package.json` and `tauri.conf.json`, with no other reformatting.
- Setting `package-name` put the component into the branch name
  (`release-please--branches--main--components--riffle`); it is left unset so
  the branch matches Decision 5.
- The Step 2 artifact defect is fixed here: `uploadWorkflowArtifacts` is
  removed and a dispatch-only `actions/upload-artifact@v4` step uploads
  `target/release/bundle/**` as `bundle-<matrix.os>`.
- `build` needs `always()` in its `if`: on a dispatch `release-please` is
  skipped, and a skipped dependency would otherwise skip `build` too. On a
  dispatch `needs.release-please.outputs.tag_name` is empty, so `tagName` is
  empty as required.
- Not verifiable pre-merge: the release PR opening with `ci` running on it,
  and a dispatch run showing the four artifacts.

