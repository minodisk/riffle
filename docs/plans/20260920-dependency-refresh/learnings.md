# Learnings

## Step 1: lockfile refresh

### What actually moved

`cargo update` resolved exactly the set the plan predicted (`Cargo.lock`):

- `cc 1.4.6 -> 1.4.7`
- `find-msvc-tools 0.1.12 -> 0.1.13`
- `tauri 2.11.5 -> 2.11.6`
- `tauri-plugin-log 2.9.1 -> 2.9.2`
- `tauri-plugin-single-instance 2.4.4 -> 2.4.5`
- `tauri-plugin-updater 2.11.0 -> 2.12.0`

`pnpm-lock.yaml`: `@tauri-apps/cli 2.11.4 -> 2.11.5` and its platform binaries
only. No other package moved.

`cargo update --dry-run --verbose` after the refresh reports only upstream-held
transitive pins as "unchanged, newer available": `generic-array 0.14.7`,
`toml 0.8.2`, `toml_edit 0.20.2` and — not named in the plan, same family —
`toml_datetime 0.6.3`.

### Trip-up: `pnpm update` rewrites `package.json`

`pnpm update @tauri-apps/cli` narrowed the `package.json` range from `^2` to
`^2.11.5`, which the step forbids. Fix: `git checkout package.json` followed by
`pnpm install --no-frozen-lockfile`, which keeps the already-resolved 2.11.5 in
the lockfile and restores the `specifier: ^2` entry. Net result is a
lockfile-only change.

### Manual sanity check

**Not performed.** `mise run tauri:dev` could not be run in this environment:
port 1420 was already held by the Vite dev server of another concurrent
worktree session (`lsof -iTCP:1420` shows a foreign `node` PID), so
`beforeDevCommand` (`pnpm vp dev`) aborts with "Port 1420 is already in use".
Killing another session's server was not acceptable, and the port is fixed in
the Tauri config. A `pgrep` for a running `riffle` binary is also unreliable
here for the same reason (another worktree's debug binary matches).

So the launch / menu-icons / `Check for Updates…` sanity run is **still
outstanding** and must be done by hand on macOS before merging. What is
verified: `mise run ci` passes on the refreshed lockfiles, which covers
compiling and clippy-ing the whole app against `tauri 2.11.6` and
`tauri-plugin-updater 2.12.0`, plus both test suites.

## Deferred issues (todo candidates)

- The step-1 manual sanity check (`mise run tauri:dev` on macOS: app launches,
  the five menu icons render as before, `Check for Updates…` completes) was not
  executed — see above. Basis: the Step 1 acceptance criteria in
  `docs/plans/20260920-dependency-refresh/plan.md`; blocked by a port-1420
  conflict with a concurrent worktree, not by the change itself. Related:
  `crates/app/src/main.rs` (menu build), `crates/app/tauri.conf.json`
  (`devUrl` port 1420), `mise.toml` (`tauri:dev` task).
