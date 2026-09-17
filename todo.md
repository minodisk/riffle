# todo

## Tooling / CI

### Tooling: `.gitignore` is missing `tmp/`

`tmp/` (scratch space used by the PR tooling) is untracked and shows up in every `git status`.

#### TODO

- [ ] Add `tmp/` to `.gitignore`.

### Tooling: no frontend formatter/linter

`mise run fmt` only formats Rust; `crates/app/ui/**` (TypeScript) is unchecked by any formatter or linter.

#### TODO

- [ ] Add a frontend formatter/linter (Prettier or Biome) and wire it into `mise.toml` so `mise run fmt` covers `crates/app/ui/`.

### Tooling: no Markdown link checker in CI

`README.md`'s anchor link to `#running-the-app` is not checked by anything; `mise run ci` has no Markdown link checker.

#### TODO

- [ ] Add a Markdown link checker to `mise run ci` (or otherwise verify README anchors) covering `mise.toml` and `README.md`.

### Tooling: `create-pr.sh` hard-codes a label that doesn't exist in this repo

`.claude/skills/pr/scripts/create-pr.sh` hard-codes `--label ai-coauthored`. That label does not exist in this repository (it's a convention carried over from the repo these skills were ported from), so the first PR creation in a fresh clone fails with `could not add label: 'ai-coauthored' not found` until something creates the label and retries.

#### TODO

- [ ] Either create the `ai-coauthored` label deliberately as part of repo setup, or remove the hard-coded `--label` flag from `.claude/skills/pr/scripts/create-pr.sh`.

### Tooling: `delete_merged_branches.sh` fails on a fresh clone with no `origin/HEAD`

`tools/git/delete_merged_branches.sh` derives its target branch from `git symbolic-ref refs/remotes/origin/HEAD`. A repository cloned while still empty has no such symbolic ref, so the script fails with `fatal: ref refs/remotes/origin/HEAD is not a symbolic ref`, surfacing as a `merger` failure after the first merge. Fixable per-clone with `git remote set-head origin -a`, but the script gives no hint about this.

#### TODO

- [ ] Have `tools/git/delete_merged_branches.sh` detect the missing symbolic ref and either run `git remote set-head origin -a` itself or print a clear message pointing at the fix.

### Tooling: no `.claude/settings.json`, though the skills assume one

This repository has no `.claude/settings.json` and no `.claude/settings.local.json`. The ported skills repeatedly assume a permission allowlist exists — the local-review step claims `Bash(date *)` is "already on the `.claude/settings.json` allowlist", and several scripts insist on being called by relative path so they match allow rules that are not there. Those claims are currently false. Separately, nothing records which permissions this workflow actually needs, so a fresh clone re-approves everything by hand. The wrap-up's `settings-promoter` step also returns `FAILED` rather than a no-op, because its `NO_CHANGES` state covers "no diff", not "no file".

#### TODO

- [ ] Decide whether to check in a `.claude/settings.json` carrying the allowlist the skills assume.
- [ ] If not, correct the skill text in `.claude/skills/develop/SKILL.md` and `.claude/skills/pr/SKILL.md` that claims an allowlist exists.

### Tooling: `mise run git:main` leaves the local `main` branch stale

The `git:main` task runs `git fetch origin main` then `git checkout --detach origin/main`. It never moves the local `main` branch ref. A whole `develop` run happens on a detached HEAD, so nothing surfaces the drift — until someone runs `git checkout main` and silently gets the tree from before the run. After the Phase 2 run, local `main` was 9 commits behind `origin/main` and checking it out reverted the working tree.

`merger` compounds this by reporting "local main now at `<sha>`" after calling the task, which is the detached HEAD's position, not the branch's.

#### TODO

- [ ] Have `mise run git:main` fast-forward the local `main` branch as well as pointing HEAD at `origin/main` (or stop implying it updates `main`).
- [ ] Correct `merger`'s post-merge report so it does not claim the local `main` branch moved when only HEAD did. Path: `.claude/agents/merger.md`.

## Cross-cutting / other

### App: unmeasured end-to-end per-page latency

End-to-end per-page latency (IPC + `createImageBitmap`) is unmeasured, since the GUI could not be driven from this development machine. Only the Rust-side file-read cost was measured; see the Phase 4 baseline table in the README.

#### TODO

- [ ] Add a timing readout in the status line, or a Rust-side benchmark that includes the IPC hop, before Phase 4 tuning work begins.

### App: orphaned `ping` command

The `ping` command in `crates/app/src/main.rs` is left over from Step 2 and now has no caller (the placeholder frontend that used it was replaced in Step 4).

#### TODO

- [ ] Remove the unused `ping` command from `crates/app/src/main.rs` (and its registration).
