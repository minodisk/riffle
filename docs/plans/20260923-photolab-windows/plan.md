<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Open in DxO PhotoLab on Windows

## Purpose

`File > Open in DxO PhotoLab` (`open_in_photolab` in
`crates/app/src/commands.rs`) only knows the macOS layout: it reads
`/Applications`, picks the highest `DXOPhotoLab<N>.app` with
`newest_photolab`, and hands the folder to it through
`tauri-plugin-opener`'s `open_path`. On Windows `read_dir("/Applications")`
fails and the UI shows `Could not open PhotoLab: 指定されたパスが見つかりません。
(os error 3)`.

On Windows PhotoLab registers itself under
`HKLM\SOFTWARE\DxO\DxO PhotoLab <N>` with an `InstallPath` `REG_SZ`
(verified on the user's machine: `DxO PhotoLab 10` ->
`C:\Program Files\DxO\DxO PhotoLab 10\`, `DxO PhotoLab 9` likewise), next to
sibling keys for other DxO products (`DxO FilmPack 8`, `DxO PureRAW 5`,
`DxO PureRAW 6`) that must not be picked. The executable is
`<InstallPath>DxO.PhotoLab.exe`. Reading the registry rather than assuming
`Program Files` also honours a custom install directory. After this work the
menu item launches the newest PhotoLab with the open folder on Windows, and a
missing install reads "DxO PhotoLab is not installed" on every platform
instead of a raw OS error.

## Steps

- [ ] Step 1: Locate PhotoLab through the registry and launch it on Windows, and turn a missing install into "not installed" everywhere (incomplete: the manual launch check on the user's Windows machine is still pending; see learnings.md)
  - Done when:
    - On Windows with `DxO PhotoLab 9` and `DxO PhotoLab 10` registered under `HKLM\SOFTWARE\DxO` (plus FilmPack and PureRAW siblings), `Open in DxO PhotoLab` launches `<InstallPath of DxO PhotoLab 10>DxO.PhotoLab.exe` with the open folder as its argument, including a folder whose path contains spaces.
    - When `HKLM\SOFTWARE\DxO` does not exist, holds no `DxO PhotoLab <N>` subkey, or the chosen subkey has no string `InstallPath`, the command returns `Err("DxO PhotoLab is not installed")`. On macOS the same message is returned when `/Applications` cannot be read or holds no `DXOPhotoLab<N>.app` (today a `read_dir` failure surfaces the OS error). Linux returns the same "not installed" error without touching the file system or registry.
    - The macOS launch call (`app.opener().open_path(dir, Some("/Applications/<bundle>"))`) is unchanged.
    - A pure name-selection function for the Windows key names has unit tests in the style of `newest_photolab_picks_the_highest_major_version`, runnable on any OS, covering: `DxO PhotoLab 9` vs `DxO PhotoLab 10` picks 10; `DxO FilmPack 8`, `DxO PureRAW 5`, `DxO PureRAW 6`, `DxO PhotoLab X`, `DxO PhotoLab` (no number) are never chosen; an empty iterator gives `None`.
    - `mise run ci` passes; the Rust tests pass on Linux; the Windows branch compiles (`cargo check -p riffle-app --target x86_64-pc-windows-msvc` if the target is installed, otherwise a build on the user's Windows machine) and the manual check on the user's machine launches PhotoLab 10 on the folder.
  - Implementation approach:
    - Files: `crates/app/src/commands.rs` (command body, a new selector, tests in `mod tests`) and `crates/app/Cargo.toml` (one target-gated dependency). No frontend change: `openInPhotoLab` in `crates/app/ui/src/main.ts` only displays the error string. `Cargo.lock` should not change beyond adding `winreg` to `riffle-app`'s dependency list, since `winreg 0.55.0` is already resolved (via `embed-resource`).
    - Dependency: add `[target.'cfg(windows)'.dependencies] winreg = "0.55"` to `crates/app/Cargo.toml`. Chosen over `windows-registry` (also in the lock, 0.6.1, via `hyper-util`) because its API is long-stable and its locked version is not tied to reqwest's; chosen over raw `windows-sys` to avoid hand-written unsafe FFI. If `winreg 0.55` turns out to need a different `windows-sys` than the lock provides, fall back to `windows-registry = "0.6"`; either is acceptable, note the choice in `learnings.md`.
    - Keep `newest_photolab` (macOS bundle names) and its test as they are. Add a sibling pure function, e.g. `newest_photolab_key(names: impl Iterator<Item = String>) -> Option<String>`, that strips the exact prefix `"DxO PhotoLab "`, parses the remainder as `u32`, and returns the key name with the highest version. Exact-prefix matching means `DxO PureRAW 6` / `DxO FilmPack 8` cannot match. Both selectors are plain functions compiled on every OS (no `cfg`) so their tests run on Linux CI, like the existing one.
    - Split `open_in_photolab` into cfg-gated arms, following the `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]` convention used throughout `crates/app/src/main.rs` (see `docs/agents/tauri-app.md`, "Adding a macOS menu item needs the same cfg split as its siblings"): `target_os = "macos"` (existing body, but map the `read_dir` error to "DxO PhotoLab is not installed" instead of `e.to_string()`), `windows`, and `not(any(target_os = "macos", windows))` returning `Err("DxO PhotoLab is not installed".into())`. Keep unused-variable / unused-import lints clean on Linux (`app` is only used on macOS; `OpenerExt` is imported at the top of the file, check whether anything else in `commands.rs` uses it before gating the import).
    - Windows arm: open `HKEY_LOCAL_MACHINE\SOFTWARE\DxO` read-only (`RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\DxO")`), collect `enum_keys()` names (skipping erroring entries), feed them to the selector, open the chosen subkey, read `InstallPath` as `String` (`get_value::<String, _>("InstallPath")`), and build `PathBuf::from(install_path).join("DxO.PhotoLab.exe")` (`join` copes with the trailing backslash the value carries). Every registry failure on that path maps to "DxO PhotoLab is not installed". No `KEY_WOW64_*` flag: the app is a 64-bit process and the keys were verified in the 64-bit view with `reg.exe`.
    - Launching on Windows: `std::process::Command::new(exe).arg(&dir).spawn()`, mapping the error with `e.to_string()`, not `app.opener().open_path(dir, Some(exe))`. Verified in the vendored sources: `tauri-plugin-opener` 2.5.5 enables `open`'s `shellexecute-on-windows` feature, so `open_path` with a `with` program ends in `open/src/windows.rs::with_detached`, which calls `ShellExecuteExW` with `lpFile = exe` and `lpParameters = dir` unquoted; a folder path containing a space would reach PhotoLab as several arguments. `Command::arg` quotes correctly. Drop the `Child` handle (do not `wait`); PhotoLab is a GUI process, no `CREATE_NO_WINDOW` needed.
    - Update the doc comment on `open_in_photolab` to describe both lookups (`DXOPhotoLab<N>.app` in `/Applications` on macOS; `HKLM\SOFTWARE\DxO\DxO PhotoLab <N>\InstallPath` on Windows) and why the version is looked up rather than hard-coded.
    - Verify on the user's Windows machine that `DxO.PhotoLab.exe <folder>` opens that folder in PhotoLab's browser (the assumption the macOS `open -a` path relies on). Record the result in `learnings.md`. If it only accepts files, see "Trade-offs and risks".
    - Commit as `fix(app): open the folder in DxO PhotoLab on Windows`.

## Trade-offs and risks

- **Registry crate.** `winreg 0.55` chosen (already in the lock via `embed-resource`, stable API, tiny dependency footprint). `windows-registry 0.6.1` is the alternative already in the lock; its API has shifted between minor versions and its locked version follows `hyper-util`, so a reqwest bump could pull a second copy. Raw `windows-sys` rejected (unsafe FFI for no gain).
- **Command vs `open_path` on Windows.** `std::process::Command` chosen. `open_path(dir, Some(exe))` works for space-free folders but passes the folder unquoted to `ShellExecuteExW`; accepting that leaves a real bug for folders such as `2026 09 trip`.
- **Does `DxO.PhotoLab.exe` accept a folder argument?** Not verified at planning time. If PhotoLab launches but ignores the folder, pass the first image file of the folder instead (PhotoLab opens the containing folder for a file). Decide after the manual check; do not implement both up front.
- **Registry vs. `Program Files` scan.** Registry chosen so a custom install directory is honoured. If PhotoLab is present on disk but its registry key is missing (e.g. a copied installation), it will read as "not installed"; a directory-scan fallback is deliberately not added.
- **macOS error message change.** Mapping a `/Applications` `read_dir` failure to "DxO PhotoLab is not installed" is a tiny behaviour change on macOS (the launch call itself is untouched). If strict macOS parity is wanted, keep `e.to_string()` there.
- **Linux.** Returns "not installed" without scanning anything, instead of today's `read_dir("/Applications")` OS error.

## Progress

- (none yet)
