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

# Remove Open in DxO PhotoLab

## Purpose

`File > Open in DxO PhotoLab` hands the open folder to PhotoLab. It only ever
worked on macOS (`open -a /Applications/DXOPhotoLab<N>.app <dir>` through
`tauri-plugin-opener`); on Windows PhotoLab ignores a folder argument and a
file argument only builds an "external application selection" project, so a
folder hand-off is not achievable there (verified on the user's Windows
machine with PhotoLab 10). Rather than keep a macOS-only item that fails
elsewhere (`todo.md`, "Open in DxO PhotoLab always fails on Windows"), the
feature is retired on every platform. The `.dop` sidecar support is unrelated
and stays. After this work no menu item, keymap action, settings row, Tauri
command or frontend code for the feature remains, and a settings store that
still holds a `photolab` shortcut override loads unchanged.

## Steps

- [x] Step 1: Remove the feature from the backend, the frontend and the docs
  - Done when:
    - `git grep -i photolab -- crates/app/src/main.rs crates/app/src/shortcuts.rs crates/app/ui/src` returns nothing, and `crates/app/src/commands.rs` matches only the `.dop` sidecar tests (`photolab_ratings_and_rejects_are_read_on_the_first_open_and_xmps_ignored`, `photolab_labels_are_read_on_the_first_open`, `a_pick_is_read_from_a_photolab_dop_and_from_a_lightroom_xmp`, `PHOTOLAB_THREE`), which stay.
    - `Keymap::from_overrides(Some(&json!({"photolab": ["ctrl+shift+o"]})))` loads without error and leaves every other action at its default; covered by a test (the existing `an_unknown_action_does_not_block_the_others` already proves the mechanism; add a `photolab`-named case or extend it so the regression is explicit).
    - `README.md`, `docs/usage.md`, `docs/agents/tauri-app.md` and `todo.md` no longer describe the menu item or its key.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/src/main.rs` (`app_menu`): drop `PHOTOLAB_ID`, the two `photolab` items (`IconMenuItem::with_id_and_native_icon` on macOS, `MenuItem::with_id` elsewhere) and `photolab_key`, the `&photolab` entry in `file.prepend_items`, the `(PHOTOLAB_ID, "photolab")` pair in `refresh`, the `open-in-photolab` emit in `on_event`, `commands::open_in_photolab` in `generate_handler!`, and the doc comment of `build` that lists `Open in DxO PhotoLab`. `NativeIcon` was imported only for this item: remove it from `use tauri::menu::{IconMenuItem, NativeIcon}` (keep `IconMenuItem`; the other macOS icons use it). Keep `use tauri_plugin_opener::OpenerExt` and `.plugin(tauri_plugin_opener::init())`: `open_log_folder` still calls `app.opener().open_path`.
    - `crates/app/src/commands.rs`: delete `open_in_photolab`, `newest_photolab`, the test `newest_photolab_picks_the_highest_major_version`, and the now-unused `use tauri_plugin_opener::OpenerExt;` (it was the only opener use in that file). Shrink `accelerators` from `[Option<String>; 4]` over `["open", "photolab", "undo", "redo"]` to `[Option<String>; 3]`.
    - `crates/app/src/shortcuts.rs`: delete `PHOTOLAB_DEFAULT`, the `("photolab", ...)` row in `DEFAULTS`, and the `photolab` lines in the tests `the_defaults_are_the_full_table`, `the_menu_defaults_follow_the_platform`, `accelerator_for_takes_the_first_convertible_key` and `the_menu_defaults_are_not_forbidden`. Update the `MACOS_MENU` doc comment that names `Open in DxO PhotoLab`. Leave `accelerator_converts_the_key_names` alone: its `shift+meta+o` / `ctrl+shift+o` rows test the converter, not the action. `from_overrides` already warns and skips unknown actions, and `overrides()` only re-emits known bindings, so a stale `photolab` entry is ignored on load and dropped on the next save; no migration code.
    - `crates/app/ui/src/main.ts`: delete `openInPhotoLab`, the `listen("open-in-photolab", ...)` line, and the `case "photolab":` in the keydown action dispatch. `crates/app/ui/src/settings.ts`: delete the `photolab: "Open in DxO PhotoLab"` label. `Shift+Cmd+O` / `Ctrl+Shift+O` becomes a free combination; nothing else needs changing since `forbidden` never listed it.
    - Docs: `README.md` (`File > Open in DxO PhotoLab` under the PhotoLab sidecar bullet), `docs/usage.md` (the "Open in DxO PhotoLab" bullet; reword "Both File menu items are rebindable" to cover only `Open Folder…`, and the key table row), `docs/agents/tauri-app.md` ("nine app items" becomes eight, and the sentence about `with_id_and_native_icon` / `NativeIcon::FollowLinkFreestanding` being the one native icon must be rewritten so it does not describe a removed item; the `NativeIcon` has-no-undo/gear reasoning can stay as the reason every icon is a bundled PNG), `todo.md` (drop the `Shift+Cmd+O` / `photolab` clauses from the macOS verification item, and the whole "App: Open in DxO PhotoLab always fails on Windows" section). Leave `.github/ISSUE_TEMPLATE/software.yml`, `filter.ts`'s "after PhotoLab's" comment, `sidecar.rs`, `crates/core` and `CHANGELOG.md` untouched: they concern `.dop` support or history. Line numbers drift; locate by content.
    - Commit as `feat(app)!: remove Open in DxO PhotoLab` with a `BREAKING CHANGE:` footer naming the removed menu item, `photolab` shortcut action and `open_in_photolab` command.

## Trade-offs and risks

- Commit type: `feat(app)!:` (chosen by the user). release-please (`bump-minor-pre-major` + `bump-patch-for-minor-pre-major`, version 0.2.x) turns it into a minor bump with a "Breaking changes" changelog heading. The repository has no prior `!:` commit, so this sets the precedent.
- The unmerged branch `fix/photolab-windows` (registry lookup for PhotoLab on Windows) is abandoned, not merged. Its plan folder only exists on that branch, so nothing on `main` needs archiving.
- `tauri-plugin-opener` is kept because `Help > Open Log Folder` uses it. Removing the dependency is out of scope.
- A user who rebound `photolab` keeps an orphan key in the `shortcuts` store until the next save; harmless (logged as "ignoring the shortcut for an unknown action"). No migration is added.

## Progress

- (none yet)
