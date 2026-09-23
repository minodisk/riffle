# Learnings

## Step 1

- Registry crate: `winreg = "0.55"` worked as planned. It resolved to the
  `winreg 0.55.0` already in the lock (via `embed-resource`, on
  `windows-sys 0.59.0`); `Cargo.lock` only gained `winreg` in `riffle-app`'s
  dependency list. The `windows-registry` fallback was not needed.
- The cfg split is done with three `#[tauri::command]` functions of the same
  name, each behind its own `cfg`, rather than `cfg` blocks inside one body.
  Tauri injects `AppHandle` only when a command asks for it, so the Windows
  arm takes just `dir` and the Linux arm takes nothing, which keeps
  unused-variable lints quiet without `let _ =`. `OpenerExt` was only used by
  the macOS arm, so its import is now `#[cfg(target_os = "macos")]`.
- CI runs `cargo clippy --all-targets -- -D warnings`, so the two pure
  selectors, compiled on every OS for their tests, carry
  `#[cfg_attr(not(<their OS>), allow(dead_code))]`; otherwise Linux (and the
  other OS) fail on `dead_code` in the non-test build.
- Windows compile check: the `x86_64-pc-windows-gnu` target was already
  installed in WSL, so `cargo clippy -p riffle-app --all-targets --target
  x86_64-pc-windows-gnu -- -D warnings` was run and passed (the msvc target is
  not installed; nothing was installed). The macOS arm was not compiled
  locally (no macOS target); it is the previous body plus the `read_dir`
  error mapping and relies on the macOS CI job.
- Manual check pending (not verified): on the user's Windows machine, confirm
  that `File > Open in DxO PhotoLab` launches PhotoLab 10 and that
  `DxO.PhotoLab.exe <folder>` opens that folder in PhotoLab's browser,
  including a folder whose path contains a space. If PhotoLab ignores a folder
  argument, see "Trade-offs and risks" in `plan.md` (pass the first image file
  instead).
- In a fresh worktree `mise run fmt` fails with `Command "vp" not found`
  until `node_modules` exists; `mise run ci` runs `pnpm install` first, after
  which `mise run fmt` passes. Environmental, not a code issue.
