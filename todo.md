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

### App: `decode::decode_rgb`'s CLI callers are unguarded against mozjpeg panics

`mozjpeg` panics rather than returning `Err` on malformed JPEG input. Phase 3 wrapped only the scan path's call in `catch_unwind` (`crates/core/src/scan.rs`); `focusbox` / `bench` / `crop` in `crates/cli/src/main.rs` call `decode_rgb` directly, so a corrupt file aborts the CLI process instead of reporting an error.

#### TODO

- [ ] Wrap the CLI's `decode_rgb` call sites in `catch_unwind`, or move the guard into `decode_rgb` itself.

### App: `riffle-cli info`/`focusbox`/`bench` read whole files instead of the bounded prefix

`info` / `focusbox` / `bench` in `crates/cli/src/main.rs` still call `std::fs::read(path)` for the whole file rather than `reader::read_head`'s bounded prefix, unlike the scan path and unlike `crop` (which moved to `reader::read_full`, a ranged read, in Phase 5 Step 1). This may be inherent: these subcommands need the full-size `JpgFromRaw`, which sits past the 1 MiB prefix.

#### TODO

- [ ] Decide whether these subcommands can move to `reader::read_head`/`reader::read_full` with a whole-file fallback, or whether they inherently need the whole file and this is not worth changing.

### App: `riffle-cli bench`'s crop row skips sensor→JPEG scaling

`bench()` in `crates/cli/src/main.rs` still feeds `FocusLocation` coordinates straight into `decode_crop` without the sensor→JPEG scaling that Phase 5 Step 1 added to `crop`. It happens to be correct on the current test file (sensor and JPEG are both 7008 wide) but wrong on any body where they differ.

#### TODO

- [ ] Apply the same sensor→JPEG scaling in `bench()`'s crop row that `crop()` uses.

### App: `focus_crop`'s header has no full-JPEG size, so the zoom placeholder's scale is approximate

The `focus_crop` payload header (`crates/app/src/commands.rs`, `crop_payload`) has a reserved 4-byte word but does not carry the full JPEG's width/height. The frontend's placeholder scale in `drawZoom()` (`crates/app/ui/src/main.ts`) falls back to the index row's `FocusLocation` sensor width, which is exact only when the JpgFromRaw size equals the sensor size.

#### TODO

- [ ] Carry the full JPEG width/height in the `focus_crop` header's reserved word and use it in `drawZoom()` instead of the sensor-width approximation.

### App: the held crop bitmap is not released when paging with the 1:1 view off

`crates/app/ui/src/main.ts` only `close()`s the previous crop bitmap when a new crop replaces it. Paging to a new file with the zoom view off leaves one stale crop bitmap alive until the next `Space` press.

#### TODO

- [ ] Release the held crop bitmap in `show()` when paging with zoom off, not only when a new crop arrives.

### App: no rescan when new files appear in an already-open folder

`scan_folder` in `crates/app/src/commands.rs` reconciles the index only when a folder is opened, so a file added to an already-open folder is not picked up until the folder is reopened.

#### TODO

- [ ] Add a folder watcher, or a rescan on window refocus, to catch new files without a reopen.

### App: the SQLite index is never pruned or `VACUUM`ed

The database in `crates/app/src/index.rs` never evicts rows for folders that are not reopened, and deleting rows does not shrink the file without `VACUUM`. Measured at ~20.8KB per row (104,177,664 bytes for 5000 rows), so it grows without bound.

#### TODO

- [ ] Add a size cap or LRU eviction for the index, with a `VACUUM` step.

### App: the filmstrip re-requests every visible placeholder on each `scan-progress` event

`refresh` in `crates/app/ui/src/strip.ts`, driven from the `scan-progress` handler at ~10/s, re-requests every visible placeholder rather than only the indices the scan has newly passed. It is bounded by the 4-in-flight cap and the visible range, but it is avoidable IPC. Relatedly, the strip discovers whether a file has a thumbnail by invoking `thumbnail` and treating an `Err` as "not yet", rather than reading `has_thumb` from the `folder_entries` map, because keeping that map fresh during a scan would mean the same 10/s full re-read.

#### TODO

- [ ] Use the `done` counter or per-file `has_thumb` state to request only newly available thumbnails, if the strip turns out to be IPC-bound.

### App: filmstrip cell geometry assumes 3:2 thumbnails

`.cell img` in `crates/app/ui/style.css` is a fixed 144x96 box, matching the current 404x270 pipeline. A body with a differently shaped IFD0 preview would letterbox harmlessly (`object-fit: contain`) but waste cell space.

#### TODO

- [ ] Revisit if a camera body with a non-3:2 preview turns up.

### App: a multi-file drop is rejected wholesale, and drag-hover gives no early feedback

The `tauri://drag-drop` handler in `crates/app/ui/src/main.ts` rejects a multi-item drop outright, even when every item shares one parent folder. Separately, the `body.dragging` overlay in `crates/app/ui/style.css` looks the same whether or not the payload will be accepted, although Tauri's `drag-enter` event already carries the paths.

#### TODO

- [ ] Take the common parent folder of a multi-file drop instead of rejecting it.
- [ ] Indicate during drag-hover whether the drop will be accepted.

### App: paging keys follow `event.key`, not the physical layout

The paging key handler in `crates/app/ui/src/main.ts` matches on `event.key`, so on a non-QWERTY layout (Dvorak, AZERTY) WASD and HJKL land on scattered physical keys.

#### TODO

- [ ] Revisit only if a user asks; a fix would be an `event.code` fallback or a key-config layer.

### App: real-folder scan and second-open numbers are still missing

Every Phase 3 performance figure in the README (5.55s first scan, 34.4ms second open, the per-file timings) was measured on 5000 symlinks to one inode, or on freshly `cp`-copied files — never on a real folder of 5000 distinct ARWs on real hardware. Only the user can close this.

#### TODO

- [ ] Measure first-scan and second-open times on a real folder of ~5000 distinct ARW files, and update the README's numbers.
