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

# Rebindable File menu accelerators: `Open Folder…` and `Open in DxO PhotoLab`

## Purpose

`open` (open a folder) is the one culling-adjacent action that belongs in the
File menu like any desktop app, but it is bound to the plain key `o`
(`crates/app/src/shortcuts.rs`, `DEFAULTS`) and has no menu item.
`File > Open in DxO PhotoLab` exists but is menu-only: it has no key, is not
in the shortcuts panel and cannot be rebound. This work adds
`File > Open Folder…`, gives both items an accelerator that mirrors the
keymap (default `meta+o` / `ctrl+o` for `open`, `shift+meta+o` / `ctrl+shift+o`
for `photolab`), and lists both in the shortcuts panel so the user can rebind
them. `Edit > Undo` (`CmdOrCtrl+Z`) and `Settings...` (`CmdOrCtrl+,`) keep
their fixed accelerators and stay out of the keymap; they are out of scope.
It closes the `todo.md` entry "App: `open` stays on a plain key instead of
the File menu", deferred from `default-shortcuts-cleanup` (#191).

Investigation results (Tauri 2.11.5, muda 0.19.3; see
`~/.cargo/registry/src/*/tauri-2.11.5/src/menu/normal.rs`,
`~/.cargo/registry/src/*/tauri-2.11.5/src/app.rs`,
`~/.cargo/registry/src/*/muda-0.19.3/src/accelerator.rs` and
`muda-0.19.3/src/platform_impl/{macos,windows,gtk}/mod.rs`):

- `MenuItem::set_accelerator<S: AsRef<str>>(&self, Option<S>) -> tauri::Result<()>`
  exists (also on `IconMenuItem`, which `Open in DxO PhotoLab` is on macOS).
  It parses the string with `.parse().ok()`, so an unparseable accelerator
  silently becomes `None`: Riffle must only hand it strings it knows muda
  accepts. It runs on the main thread through `run_item_main_thread`, fine
  from the sync `update_keymap` command (itself on the main thread).
- muda's macOS `set_key_accelerator(None)` does **not** clear the
  `NSMenuItem` key equivalent (the `setKeyEquivalent` loop only runs for
  `Some`); Windows (accelerator table) and GTK (`remove_accelerator`) do
  clear it. Rebuilding the menu with `AppHandle::set_menu(menu)` clears it
  on every platform (`set_menu` re-inits the macOS app menu and reassigns
  the app-wide menu to every window elsewhere).
- muda's accelerator grammar: modifiers `Ctrl`, `Alt`, `Shift`, `Cmd`
  (`Cmd`/`Super` both map to the Command / Windows key), `CmdOrCtrl`; keys
  are single letters, digits, the punctuation characters `, . / ; ' [ ] \ \` - =`
  verbatim, and the names `Space`, `Escape`, `Enter`, `Tab`, `Backspace`,
  `Delete`, `Home`, `End`, `PageUp`, `PageDown`, `Insert`, `ArrowUp` /
  `ArrowDown` / `ArrowLeft` / `ArrowRight`, `F1`–`F24` (case-insensitive).
  Riffle's key names (`crates/app/ui/src/keys.ts`: `ctrl+alt+shift+meta+` +
  the key from `event.code`, plain keys from `event.key`) map onto this
  almost one to one.
- `forbidden()` refuses only the keys in `MACOS_MENU` / `OTHER_MENU` (the
  fixed accelerators the app sets, `CmdOrCtrl+,` and `CmdOrCtrl+Z`, plus the
  default menu's own) and the system lists. Neither `meta+o` / `ctrl+o` nor
  `shift+meta+o` / `ctrl+shift+o` is in any list, so the "exemption" is that
  keymap-derived accelerators are never added to those lists.
- `main.ts` calls `event.preventDefault()` for every handled keydown, and
  `settings.ts` does so while a row captures. On macOS WebKit reports a
  `preventDefault`ed key equivalent as handled, so the menu is normally not
  consulted for it; whether that holds here is a real-device check below.
- `crates/app/ui/src/settings.ts` labels the `open` row
  "Open in DxO PhotoLab" today, which is wrong (`open` opens a folder); the
  step corrects it while adding the `photolab` row.
- An empty default key list works end to end in the current code:
  `Keymap::defaults()` copies `&[]`, `overrides()` skips an action whose
  keys equal its (empty) default, `parse_keys` refuses `[]` only for stored
  overrides, `remove` refuses the last key regardless, and the panel renders
  zero key chips plus the `+` button. Not needed here, as both actions get a
  real default.

## Steps

- [x] Step 1: Add `File > Open Folder…`, put `open` and `photolab` in the keymap with modifier defaults, and make both menu accelerators follow the keymap
  - Done when:
    - `File > Open Folder…` appears at the top of the File menu (above
      `Open in DxO PhotoLab`) on every platform and runs the same
      `openFolder()` path as the shortcut; `Open in DxO PhotoLab` keeps its
      position, label and macOS icon.
    - With no `shortcuts` override, `open` is `meta+o` on macOS and `ctrl+o`
      elsewhere, and `photolab` is `shift+meta+o` on macOS and
      `ctrl+shift+o` elsewhere; both menu items show the matching accelerator
      (`⌘O` / `⇧⌘O`, `Ctrl+O` / `Ctrl+Shift+O`) and both keys work from the
      main window (`photolab` with a folder open).
    - `open` and `photolab` appear in the settings window's shortcuts panel
      as "Open folder" and "Open in DxO PhotoLab", next to each other in the
      position `open` has today; adding / removing / resetting their keys
      works, and an existing override `{"open": ["o"]}` loads and works
      exactly as before.
    - After the user changes either action's keys, that menu item's
      accelerator changes to the first key of the action that converts to an
      accelerator, or to none when no key converts (e.g. only `o`), on macOS
      included (the old key equivalent must not stay live); the other item is
      unaffected.
    - Once an action is moved off its default combination, that combination
      can be bound to another action (nothing in `forbidden()` refuses it).
    - Unit tests in `shortcuts.rs` cover: both platform defaults; the key
      name to accelerator conversion (letters, digits, punctuation, named
      keys, modifier order, and the unconvertible cases); the "first
      convertible key" rule through `accelerator_for` for both actions;
      `forbidden()` not refusing either default; the defaults still binding
      no key twice.
    - Real-device check on macOS (recorded in `learnings.md`): (a) one
      `Cmd+O` press in the main window opens the picker exactly once and one
      `Shift+Cmd+O` hands the folder to PhotoLab exactly once (no double fire
      from keydown + menu); (b) with either action rebound, the new key
      works and the old one no longer does; (c) in the settings window,
      pressing `Cmd+O` or `Shift+Cmd+O` while a row captures does not
      trigger the menu action (or, if it does, the result is written down in
      the trade-offs of this plan and the chosen mitigation applied);
      (d) both items clicked with the mouse work. Windows / Linux behavior
      is reasoned from the muda sources and noted as unverified in
      `learnings.md` if no machine is at hand.
    - README's key table, the menu bullets and the "already used by the
      app's menu" sentence are updated for both items; the `todo.md` section
      is removed.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/src/shortcuts.rs`
      - Move `const MACOS` above `DEFAULTS` and add
        `const OPEN_DEFAULT: &str = if MACOS { "meta+o" } else { "ctrl+o" };`
        and
        `const PHOTOLAB_DEFAULT: &str = if MACOS { "shift+meta+o" } else { "ctrl+shift+o" };`,
        used as `("open", &[OPEN_DEFAULT])` and `("photolab", &[PHOTOLAB_DEFAULT])`,
        with `photolab` right after `open` (both are non-culling actions;
        the panel shows `DEFAULTS` order). `DEFAULTS` stays one flat table;
        only these two entries differ per platform. Why this `photolab`
        default: it is the conventional "variant of Open" chord, it needs a
        `ctrl`/`alt`/`meta` modifier so it can be an accelerator, it collides
        with no existing default (`ctrl+alt+0`–`7` are the labels), and it
        passes `forbidden()` on both platforms (`meta+` is only refused off
        macOS, where the default is `ctrl+shift+o`). Update
        `the_defaults_are_the_full_table` and any test that relied on `o`
        or on `j`-style free keys.
      - Add `pub fn accelerator(key: &str) -> Option<String>` converting a
        Riffle key name to a muda accelerator string: modifiers
        `ctrl`→`Ctrl`, `alt`→`Alt`, `shift`→`Shift`, `meta`→`Cmd`, joined
        with `+` in that order; the key: a single ASCII letter upper-cased,
        a digit as is, the punctuation characters muda takes verbatim
        (`- = , . / ; ' [ ] \ \``), `space`→`Space`, `escape`→`Escape`,
        `enter`→`Enter`, `tab`→`Tab`, `backspace`→`Backspace`,
        `delete`→`Delete`, `home`/`end`/`pageup`/`pagedown`/`insert`,
        `arrowup`/`arrowdown`/`arrowleft`/`arrowright`, `f1`–`f24`.
        Anything else returns `None` (muda would reject it, and Tauri would
        silently drop it). Return `None` too for a key with no `ctrl`, `alt`
        or `meta` modifier (a plain or shift-only key): a modifier-less menu
        key equivalent would fire on every `o` typed anywhere, including
        text fields, and double with the webview's own handling. Keep the
        table explicit; do not add `muda` as a dependency just to validate.
      - Add `pub fn accelerator_for(&self, action: &str) -> Option<String>`
        on `Keymap`: the first key of `action`, in stored order, for which
        `accelerator()` is `Some`; `None` for an unknown action. This is the
        single-accelerator rule ("first convertible key"), shared by both
        items; document it on the function. Two call sites (`open`,
        `photolab`); no registry of menu-backed actions beyond the two
        constants the menu module holds.
      - Add a comment on `MACOS_MENU` / `OTHER_MENU` saying that the
        `Open Folder…` and `Open in DxO PhotoLab` accelerators are
        deliberately not listed: each is derived from the keymap, so it is
        always one of its own action's keys and can never collide with
        another action. No change to `forbidden()` itself. Consequence, to
        state in a test: once an action is rebound, its old combination is
        free for other actions.
    - `crates/app/src/main.rs` (`app_menu`)
      - Add `OPEN_FOLDER_ID` and an `Open Folder…` `MenuItem::with_id(handle,
        OPEN_FOLDER_ID, "Open Folder…", true, accelerator)` (ellipsis
        character, as `Check for Updates…`), prepended to `File` together
        with the existing `photolab` item and the separator
        (`file.prepend_items(&[&open, &photolab, &separator])`). Keep the
        plain `MenuItem` on every platform for the new item (see trade-offs
        on a native icon). The existing `photolab` item passes its
        accelerator in the last argument of `with_id_and_native_icon` /
        `with_id` instead of `None::<&str>`.
      - `on_event`: `OPEN_FOLDER_ID` emits `"open-folder"` to the frontend,
        as `open-in-photolab` and `undo` do (the frontend owns the picker
        flow and the folder token). `PHOTOLAB_ID` is unchanged.
      - Make the accelerators an input of the build:
        `pub fn build(handle: &AppHandle, open: Option<&str>, photolab: Option<&str>)`
        (or a tiny two-field struct if that reads better). Recommended
        wiring (one code path, and it sidesteps the macOS
        `set_accelerator(None)` bug): drop `.menu(app_menu::build)` from the
        builder, and in `setup`, right after `load_settings`, call
        `app.set_menu(app_menu::build(app.handle(), keymap.accelerator_for("open").as_deref(), keymap.accelerator_for("photolab").as_deref())?)`.
        Keep `.on_menu_event(app_menu::on_event)` on the builder. Then
        expose `pub fn refresh(app: &AppHandle, keymap: &Keymap) -> tauri::Result<()>`
        (or similar) that rebuilds and `set_menu`s. Confirm during
        implementation that a menu set from `setup` shows on the main window
        on macOS and that `set_menu` while the settings window is focused
        does not disturb it; if either fails, fall back to the alternative
        below.
      - Alternative if the rebuild misbehaves: keep `.menu(...)`, build the
        items with the platform defaults, store both handles in managed
        state (`MenuItem` / `IconMenuItem` are `Clone + Send + Sync`), and
        call `set_accelerator(...)` from `setup` and on every keymap change.
        This needs a workaround for the macOS `None` case (rebuild only
        then), so it is second choice.
    - `crates/app/src/commands.rs`
      - In `update_keymap`, compute the pair
        `(keymap.accelerator_for("open"), keymap.accelerator_for("photolab"))`
        before and after `change`; when the pair differs, refresh the menu
        (call the `app_menu` helper, logging a failure like the store save).
        `reset_shortcuts` and `reset_shortcut` go through `update_keymap`
        already.
    - `crates/app/ui/src/main.ts`
      - `window.__TAURI__.event.listen("open-folder", openFolder)` next to
        the `open-in-photolab` / `undo` listeners, and a
        `case "photolab": openInPhotoLab(); break;` in the keydown switch.
        The rest of the dispatch is unchanged: `keymap` already maps the
        keys to actions and the handler `preventDefault`s. If the
        real-device check shows a double fire, the fix is to skip the
        accelerator key in keydown (either the frontend applies the same
        "first convertible key" rule, or the backend adds the accelerator to
        the `Binding` payload); record the choice in `learnings.md`.
    - `crates/app/ui/src/settings.ts`: `shortcutLabels.open` becomes
      "Open folder" and `photolab: "Open in DxO PhotoLab"` is added after
      it (order comes from the backend's `bindings()`; the map only labels).
    - `README.md`: key table rows `Cmd+O` / `Ctrl+O` "open a folder
      (`File > Open Folder…`)" and `Shift+Cmd+O` / `Ctrl+Shift+O` "open the
      folder in DxO PhotoLab (`File > Open in DxO PhotoLab`)"; a bullet for
      the new menu item in the features list and a note on the PhotoLab
      bullet that both accelerators follow the first key of the action that
      can be one; adjust the sentence on combinations the app's menu already
      uses so the reader knows these two are rebindable unlike `Cmd+Z` /
      `Cmd+,`.
    - `todo.md`: remove the "App: `open` stays on a plain key instead of the
      File menu" section.
    - Docs: if the real-device check teaches something about key equivalents
      versus webview keydown, add it to `docs/agents/tauri-app.md` (tagged
      Hit / Inferred as that guide does).

## Trade-offs and risks

- **Rebuild the menu vs `set_accelerator`.** `set_accelerator` is the API the
  todo names, and it works for `Some(..)` on every platform, but on macOS
  muda 0.19.3 ignores `None`, leaving a stale key equivalent live once the
  user binds an action to plain keys only. Rebuilding through `set_menu` is
  one code path and clears correctly, at the cost of recreating the whole
  menu on each change of either action's keys (rare, cheap) and an
  unverified risk of the macOS menu bar flickering or the settings window
  losing focus. The plan recommends the rebuild and keeps `set_accelerator`
  as the fallback; the implementer decides after trying the rebuild on the
  device.
- **`photolab` default.** `shift+meta+o` / `ctrl+shift+o` was chosen by the
  user over `ctrl+alt+o` (one constant for both platforms, but reads as a
  label key and hits AltGr on Windows) and over no default at all.
- **Plain keys never become the accelerator.** A user who keeps the old
  `{"open": ["o"]}` override sees `Open Folder…` with no accelerator. The
  alternative (a modifier-less key equivalent) would fire the menu on every
  typed `o` and double with the webview, so it is not taken.
- **Which key becomes the accelerator.** "First key in stored order that
  converts" is predictable and matches the panel's display order. The
  alternative "the last key added" would follow the user's latest edit but
  is not derivable from the stored list. First-convertible is chosen.
- **Defaults freed after rebinding.** Because each accelerator always
  mirrors one of its own action's keys, freeing the default combination for
  another action can never clash with the menu. This is the chosen
  behavior; the alternative (permanently reserving them) contradicts
  "rebindable".
- **Double fire and capture interference are real-device questions.** On
  macOS, WebKit's `performKeyEquivalent` sends the key to the page first and
  returns handled when JS `preventDefault`s, so the menu should not fire
  for a key the page handled; if that proves wrong, the mitigations are
  listed in the step. For the settings capture, adding a key the action
  already holds is refused anyway ("is bound to …" / "already bound"), so
  the only possible side effect is the menu action firing; if observed,
  `on_event` can skip the action when `app.get_focused_window()` is the
  settings window, or the settings window can ignore the event.
- **Windows / Linux verification.** Only macOS is likely at hand. muda's
  Windows accelerator table and GTK `remove_accelerator` paths are read as
  correct for both `Some` and `None`; the plan asks to note the platforms
  actually tested in `learnings.md` rather than claim them verified.
- **Native icon on macOS for `Open Folder…`.** `NativeIcon::Folder` exists,
  but the macos-menu-icons work found several `NativeIcon`s to be legacy
  color bitmaps rather than template images and only used verified ones. A
  plain `MenuItem` is the minimum; adding an icon is a separate, small
  follow-up if wanted.
- **Menu label.** `Open Folder…` with the ellipsis character matches
  `Check for Updates…`; `Settings...` uses three dots, so the existing menu
  is not consistent either. Not fixed here (surgical change).

## Progress

- (2026-09-20) Step 1 complete: added `File > Open Folder…`, put `open` and
  `photolab` in the keymap with platform-dependent modifier defaults, and made
  both menu accelerators follow the keymap. Landed in `5a7d52f`
  (feat(app): add File > Open Folder… with keymap-driven accelerators). The
  real-device checks on macOS remain unverified and are filed in `todo.md`.
