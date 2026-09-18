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


## Step 5

- After Step 4 merged, release-please ran and opened no PR, as expected:
  `bootstrap-sha` was f1ba56d and the only later commit was `ci(release)`
  (#55). The user decided the first release is 0.1.0.
- Dry-run method (same as Step 4): a GitHub clone of `main` (58256f3) in the
  scratchpad, a local bare repo made from it set as the clone's `origin` (the
  `--local` mode runs `git fetch origin` + `git reset --hard origin/main`, so
  with the GitHub `origin` the local commit is wiped and the remote manifest is
  read), then a commit shaped like this repo's squash merge
  (`squash_merge_commit_message: COMMIT_MESSAGES`: a title with `(#56)`, a body
  of `* type: ...` entries, `---------`, `Co-authored-by:`) pushed into the
  bare repo, and
  `npx release-please@17 release-pr --dry-run --token "$(gh auth token)" --repo-url minodisk/riffle --local --local-path <clone>`.
  `gh auth token` has to run outside the clone: the clone's untrusted
  `mise.toml` makes it print a mise error instead of a token (401).
- Results:
  - `Release-As: 0.1.0` footer inside the squash body, `bootstrap-sha` kept:
    release-please splits the body into 4 commits and still reports "No user
    facing commits found" — 0 PRs.
  - `"release-as": "0.1.0"` in the config, `bootstrap-sha` kept: "Setting
    version for . from release-as configuration", then still "No user facing
    commits" — the release-as override does not bypass the empty-changelog
    check, and nothing after f1ba56d is `feat`/`fix`. So `bootstrap-sha` has to
    go (or point earlier).
  - Footer, `bootstrap-sha` removed: a PR opens, but as
    `chore(main): release 1.0.0` — the footer is ignored in the squash shape.
  - `"release-as": "0.1.0"` in the config, `bootstrap-sha` removed, manifest
    `0.0.0` (chosen):

    ```
    Would open 1 pull requests
    title: chore(main): release 0.1.0
    branch: release-please--branches--main
    ## 0.1.0 (2026-09-18)
    ### Features   (25 entries, #4 .. #51)
    ### Bug Fixes  (#11, #54)
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

    The six versioned files already say 0.1.0, so their content does not
    change; `CHANGELOG.md` is created and the manifest goes to 0.1.0. The
    pre-Conventional-Commits first commit (e3fe091) is logged as "could not be
    parsed" and skipped, which is harmless.
- Pending after this PR merges (the user's): merge the release PR; confirm
  installers for all three OSes plus `latest.json` with `darwin-aarch64`,
  `darwin-x86_64`, `windows-x86_64`, `linux-x86_64` and signatures; the
  update-path check needs a second release. Not verified yet.
- The README's installer file names follow Tauri's default bundle naming
  (`Riffle_<version>_aarch64.dmg` etc.); they are not verified against an
  actual release yet.

## Deferred issues (todo candidates)

- Remove `"release-as": "0.1.0"` from `release-please-config.json` once 0.1.0
  is released; otherwise every later release PR is pinned to 0.1.0. Basis:
  Step 5 chose the config pin because a `Release-As` footer is ignored in this
  repo's squash-commit shape (dry run above). File:
  `release-please-config.json`.
