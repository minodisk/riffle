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

# Reveal a folder in the OS file manager from the folder tree

## Purpose

The folder tree browses home and the mounted volumes, but there is no way to
jump from a folder in it to the same folder in Finder / Explorer / the Linux
file manager (to rename it, look at the non-RAW files next to the RAWs, and
so on). This adds a right-click menu on a folder tree row with one item that
reveals the folder in the platform's file manager, worded the way each
platform words it: `Reveal in Finder` on macOS, `Reveal in File Explorer` on
Windows, `Open Containing Folder` on Linux (VS Code's wording; the Linux
label is generic because the file manager varies).

The user chose the entry point (the right-click menu, no keyboard shortcut)
and the behavior (reveal: select the folder in its parent, not open its
contents).

## Steps

- [x] Step 1: Right-click menu on a folder tree row that reveals the folder in the OS file manager
  - Done when:
    - Right-clicking a row in the folder tree opens a one-item menu at the
      pointer; clicking the item reveals that folder in the OS file manager
      (the folder selected in its parent, as `Reveal in Finder` does in
      Finder and VS Code). A right-click does not open the folder. Whether it
      moves the tree's keyboard cursor to the right-clicked row is left to the
      implementation (either is acceptable; record the choice in
      `learnings.md`), but the culling key gate must not change.
    - The item's label follows the platform: `Reveal in Finder` (macOS),
      `Reveal in File Explorer` (Windows), `Open Containing Folder` (Linux
      and anything else).
    - A failure (folder gone, opener error) is reported through the existing
      status line (`setStatus`), the way `list_subfolders` errors are, not
      swallowed.
    - The menu closes on an outside `mousedown` and on `Escape`, like the
      strip's menu (reuse the same `#context-menu` element and CSS; do not
      add a second menu element).
    - Tests: a Rust unit test asserts the label constant per platform (the
      same shape as `the_size_base_follows_the_platforms_file_manager` in
      `crates/app/src/commands.rs`), and a vitest test covers whatever pure
      logic is added to `context.ts` (see approach). `mise run ci` passes.
    - Docs: `docs/usage.md`'s Folders bullet, `README.md`'s Folder tree
      bullet and `README.ja.md`'s matching bullet mention the right-click
      item (README.md and README.ja.md in the same PR, per CLAUDE.md).
      `CLAUDE.md`'s Layout paragraph gets a short mention next to
      `src/folders.rs` if a new command lands there.
  - Implementation approach (as far as it is known):
    - Rust, `crates/app/src/folders.rs` (keep the folder tree's commands
      together): add `#[tauri::command] pub async fn reveal_folder(app:
      AppHandle, path: String) -> Result<(), String>` that calls
      `app.opener().reveal_item_in_dir(&path)` (`use
      tauri_plugin_opener::OpenerExt;`), inside
      `tauri::async_runtime::spawn_blocking` since the opener shells out /
      calls COM (see `docs/agents/tauri-app.md`: sync commands run on the
      main thread; `folder_roots` / `list_subfolders` in the same file are
      the pattern). Map the error with `.map_err(|e| e.to_string())`.
      Register it in `invoke_handler` in `crates/app/src/main.rs` next to
      `folders::folder_roots` / `folders::list_subfolders` (line ~576).
      No capability entry is needed for a Rust-side opener call
      (`Help > Open Log Folder` has none; `crates/app/capabilities/default.json`
      stays untouched).
    - Label: a `pub const REVEAL_LABEL: &str = if cfg!(target_os = "macos")
      { "Reveal in Finder" } else if cfg!(target_os = "windows") { "Reveal in
      File Explorer" } else { "Open Containing Folder" };` in `folders.rs`,
      exposed by a sync command (e.g. `fn reveal_label() -> &'static str`,
      like `debug_build` in `main.rs:446`; a sync command is fine here since
      it does no IO), with a `#[test]` asserting the value on the current
      platform. The frontend invokes it once at init and caches it.
    - Frontend, `crates/app/ui/src/folders.ts`: in `render()` add a
      `contextmenu` listener on each row that `preventDefault()`s and calls
      a new `onContextMenu(path, x, y)` callback passed through `init`
      (extend `init(onOpen, onError)` to `init(onOpen, onError,
      onContextMenu)`, mirroring `strip.init(onSelect, onContextMenu)`).
      Also `preventDefault()` on the container's own `contextmenu` so the
      empty area below the rows shows no WebView menu (strip.ts:511 does the
      same).
    - Frontend, `crates/app/ui/src/main.ts`: the `#context-menu` element and
      `openContextMenu` are owned here. Factor the item-DOM building out of
      `openContextMenu(x, y)` so a second entry point can show a different
      item list at a position (e.g. `showMenu(groups: MenuItem[][], x, y)`
      with `openContextMenu` calling it with `contextMenuGroups(...)`), then
      add the folder entry point: one group with one item
      `{ action: "revealFolder", label: <cached label>, shortcut: "",
      checked: undefined }` whose click invokes `reveal_folder` with the
      row's path and reports a rejection via `setStatus`. Do not route it
      through `runAction` (that is the culling keymap; no shortcut). Confirm
      `Escape` closes it: check how the existing menu is closed on `Escape`
      in the key handler and make sure the folder menu goes through the same
      `closeContextMenu` (the folder tree may hold the keyboard at that
      moment, and `folders.keydown` consumes `Escape` to blur the tree, so
      verify the order of the two handlers).
    - Frontend, `crates/app/ui/src/context.ts` (+ `context.test.ts`): keep
      pure logic here so it is testable, e.g. a `folderMenuGroups(label:
      string): MenuItem[][]` returning the single-item group, with a vitest
      test. Keep it small; if it would be a one-liner that only wraps an
      object literal, it is acceptable to inline it in `main.ts` and rely on
      the Rust label test instead (say which in `learnings.md`).
    - The reveal targets the folder itself (selected in its parent). For a
      root row (`C:\`, `/Volumes/X`, home) the opener still resolves a
      parent; verify on the dev platform that revealing a drive root does
      not error, and if it does, fall back to `open_path` for roots or
      simply report the error through the status line (decide during
      implementation, note in `learnings.md`).
    - Docs wording lives in `docs/usage.md:12-36`, `README.md:60-63`,
      `README.ja.md:46`.

## Trade-offs and risks

- **Reveal (select in parent) vs open (show the folder's contents).** Reveal,
  chosen by the user. `reveal_item_in_dir` selects the folder inside its
  parent window, matching Finder's / VS Code's `Reveal` semantics.
- **Where the label is decided.** Rust constant + one invoke (chosen: the
  platform is known at compile time, it matches the `SIZE_BASE` precedent,
  and it is unit-tested in Rust) vs. a pure TypeScript `revealLabel(userAgent)`
  (user-agent sniffing of the WebView is a weaker signal).
- **Linux label.** VS Code uses `Open Containing Folder` on Linux; other apps
  use `Show in File Manager` / `Reveal in File Manager`. The plan takes VS
  Code's. Cheap to change.
- **Strip right-click item ("reveal this file").** Not included. The strip's
  menu is built from the rebindable culling actions (`contextMenuGroups` maps
  each item to a keymap action and `runAction`), so a non-keymap item there
  needs a new item kind. A natural follow-up PR reusing `reveal_folder`
  (renamed `reveal_path`) and the platform label.
- **Right-click and the tree's keyboard cursor.** Either way the culling key
  gate must not change (a right-click must not give the tree the keyboard
  silently or take it away).

## Progress

- (2026-09-26) Step 1 complete
