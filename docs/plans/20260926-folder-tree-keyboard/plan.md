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

# Folder tree keyboard navigation

## Purpose

The folder tree in the left pane (`#folders`, drawn by
`crates/app/ui/src/folders.ts` from the pure state in
`crates/app/ui/src/tree.ts`) is mouse-only: a row click opens the folder
(#449), the expander toggles it, and the tree deliberately takes no keyboard
focus so the arrow keys stay with culling (`previous` / `next` /
`burstPrevious` / `burstNext` in `crates/app/src/shortcuts.rs`). The user
wants to move the folder selection with `Up` / `Down` and do the other tree
operations by keys.

Once done, clicking into the tree gives it the keyboard: `Up` / `Down` /
`Home` / `End` move a cursor row, `Right` / `Left` expand and collapse (or
step into the first child / up to the parent), `Enter` opens the cursor row's
folder exactly as a click does, typing jumps to the next folder whose name
starts with what was typed, and `Escape` hands the keys back to culling.
While the tree has the keyboard, no photo-culling shortcut fires: only the
app-level keymap actions listed under "Allowlist" below still run. The tree
keys are fixed (not part of the rebindable keymap), following the WAI-ARIA
tree pattern.

## Current state (investigated)

- `tree.ts` is pure and tested (`tree.test.ts`): `Tree` = roots +
  `Map<path, TreeNode>` (`children: FolderNode[] | undefined` until listed,
  `expanded`, `rawCount`); `rows(tree): Row[]` (`{ node, depth }`) is the
  visible depth-first list `render()` draws.
- `folders.ts` keeps `tree`, `current` (the open folder, class `.current`,
  `aria-selected`), `render()` (rebuilds the rows with `replaceChildren`;
  each row has `role="treeitem"`, `aria-level`, `aria-expanded` when it can
  expand, `dataset.path`; a row click calls `open(path)`, the expander click
  calls `toggle(path)` with `stopPropagation`), `toggle(path)` (expand +
  re-list through `list_subfolders`, or collapse), and `reveal(path,
  stillCurrent)` (expands the ancestor chain, sets `current`, renders and
  `scrollIntoView({ block: "nearest" })`s the current row).
- `index.html:11`: `<div id="folders" role="tree" aria-label="Folders">`, no
  `tabindex`. `style.css:284-339` styles `#folders`, `.folder`,
  `.folder.current` (`background: #2d4059`), `.expander`, `.name`, `.count`.
  `#folders` is `overflow: auto`, so once it is focusable the browser would
  scroll it on arrow / Home / End / Space unless the handler calls
  `preventDefault()`.
- `main.ts:2577` is the single `window` `keydown` listener: format dialog
  trap, then `if (settings.isOpen) { settings.keydown(event); return; }`,
  then `Escape` for the filter / sort / context menus and Compare, then
  `keymap.get(keyName(event))` -> `runAction` (with `grayscale` special-cased
  before it). It has no notion of where focus is. `keys.ts` `keyName` names
  plain keys as `event.key` lower-cased (`arrowup`, `home`, `enter`,
  `escape`, `space`) and modified keys as `ctrl+alt+shift+meta+<code>`.
- The keymap (`DEFAULTS` in `shortcuts.rs`, 39 actions) binds the four
  arrows and their `shift+` / `alt+` variants; `home`, `end`, `enter` are
  unbound; `tab` is `toggleSides`, so nothing moves focus between controls by
  keyboard. The filter / sort buttons `blur()` themselves after a click, so
  focus normally rests on `body`. `settings.ts`'s `shortcutLabels` names the
  same actions.
- Menu accelerators (`CmdOrCtrl+,` Settings, `CmdOrCtrl+R` Reload Folder,
  and on macOS the app menu's `meta+q` etc.) are native muda accelerators;
  they do not go through the keydown listener. `Open Folder…`, `Undo`,
  `Redo` accelerators mirror the keymap and the keydown runs them.
- Tests run under vitest with `environment: "node"` (root `vite.config.ts`);
  the tested modules are DOM-free (`tree.ts`, `modal.ts`, `selection.ts`),
  and the DOM modules (`folders.ts`, `strip.ts`, `settings.ts`) are thin
  wiring. Keep that split: all navigation, type-ahead and gating logic goes
  in pure modules.
- Docs listing keys: `docs/usage.md` "Folders" bullet (line 12), the
  rebindable `Keys` table (line 222) and the "These keys are fixed" table
  (line 286); `README.md:60` and `README.ja.md:46` "Folder tree" bullet.

## Allowlist: keymap actions that still run while the tree is focused

Derived from `DEFAULTS` in `crates/app/src/shortcuts.rs`. An action is
allowed only when it is about the app's window or the folder, not about a
photo.

| Action | Default key | While the tree is focused |
|---|---|---|
| `open` | `Cmd+O` / `Ctrl+O` | **runs** (opens the folder picker) |
| `toggleLeft` | `F7` | **runs** (hides the tree; focus is then blurred by the browser since the pane is `hidden`, verify) |
| `toggleRight` | `F8` | **runs** |
| `toggleStrip` | `F6` | **runs** |
| `toggleSides` | `Tab` | **runs** (same note as `toggleLeft`) |
| `previous`, `next`, `burstPrevious`, `burstNext`, `burstFramePrevious`, `burstFrameNext`, `extendPrevious`, `extendNext` | arrows | swallowed (the tree owns the arrows) |
| `undo`, `redo` | `Cmd/Ctrl+Z`, `+Shift+Z` | swallowed (they rewrite a photo's judgment; see Trade-offs) |
| `focus`, `zoom`, `grayscale`, `compare` | `f`, `z`, `g`, `v` | swallowed (photo view) |
| `rate1`–`rate5`, `clear`, `reject`, `rejectRest`, `pick`, `unflag`, `clearall` | `1`–`5`, `0`, `x`, `Shift+x`, `p`, `u`, `c` | swallowed (judgments) |
| `red`, `orange`, `yellow`, `green`, `blue`, `pink`, `purple`, `clearlabel` | `Ctrl+Alt+1`–`7`, `+0` | swallowed (labels) |

"Swallowed" = `preventDefault()`, no action, the key is dropped. The
allowlist is the constant `TREE_PASSTHROUGH` (a `Set<string>` of action
names) in the pure gate module, tested, and it is keyed by action name, so it
holds however the user rebinds the keys. Any action added to `DEFAULTS`
later is swallowed until someone adds it to the set, which is the safe
default.

## Steps

- [x] Step 1: Make the tree focusable, move a cursor with `Up` / `Down` / `Home` / `End`, `Escape` leaves, and gate the keymap while the tree is focused
  - Done when:
    - `tree.ts` exports a pure step function over the visible rows, e.g.
      `step(rows: Row[], cursor: string | null, key: "up" | "down" | "home" | "end"): string | null`
      (the path of the row to focus, or `null` when there is no row): `up`
      / `down` go to the previous / next visible row and clamp at the ends
      (no wrap, as in Finder / Explorer); `home` / `end` are the first / last
      row; with a `null` cursor (or a cursor no longer in `rows`, e.g. after
      its parent collapsed) `up` / `down` land on the first row. Unit tests in
      `tree.test.ts` cover: down / up between siblings and across depth
      (into and out of an expanded child), clamping at both ends, `home` /
      `end`, a `null` or stale cursor, and an empty tree.
    - A pure gate, e.g. `treeGate(action: string | undefined): "run" | "swallow"`
      with the `TREE_PASSTHROUGH` set above, in a small module
      (`crates/app/ui/src/treekeys.ts` or inside `tree.ts`, judged in
      implementation) with a test that pins the five allowed names and
      asserts a sample of swallowed ones (`pick`, `next`, `undo`, `rate1`,
      `red`) and that an unknown action is swallowed.
    - `index.html` gives `#folders` `tabindex="0"` (a single focusable
      container; the rows stay non-focusable and the cursor is announced with
      `aria-activedescendant`, so `render()` gives each row a stable id such
      as `folder-row-<index>` and sets the attribute on the container).
    - `folders.ts` keeps a module-level `cursor: string | null` (the path of
      the cursor row), draws it as `.folder.cursor`, and sets it to the open
      folder whenever `reveal` settles (`cursor = current` before the final
      `render()`), so entering the tree starts at the open folder. `reveal`
      keeps highlighting `current` as today.
    - `folders.ts` exports `hasFocus(): boolean`
      (`container.contains(document.activeElement)`) and
      `keydown(event: KeyboardEvent): boolean` (true when the tree consumed
      the key). In this step `keydown` handles `keyName(event)` =
      `arrowup` / `arrowdown` / `home` / `end` (calls `step`, sets
      `cursor`, re-renders or toggles the `.cursor` class, `scrollIntoView({
      block: "nearest" })`s the cursor row, `preventDefault()`), and
      `escape` (`container.blur()`, `preventDefault()`). It returns false
      for anything else.
    - `main.ts`'s window listener, right after the `settings.isOpen` branch
      and before the menu `Escape` branches, gains:
      `if (folders.hasFocus()) { if (folders.keydown(event)) return; const action = keymap.get(key); if (treeGate(action) === "swallow") { if (action !== undefined) event.preventDefault(); return; } }`
      and then falls through to the existing keymap dispatch for an allowed
      action. (`key` = `keyName(event)`; a `null` key returns as today.) So
      with the tree focused, `ArrowUp` moves the cursor and never reaches
      `burstPrevious`; `p` / `x` / `1` / `Ctrl+Alt+1` / `Cmd+Z` do nothing;
      `F7` / `Ctrl+O` work. Keys bound to nothing (e.g. `PageDown`) are left
      to the browser (native scroll of the tree), which is fine.
    - Clicking a row focuses the container (a click inside a `tabindex="0"`
      element does that natively; verify it, and call `container.focus()` in
      the row's click handler if a platform does not). A click keeps opening
      the folder as in #449.
    - `style.css`: a visible cursor ring on `#folders:focus .folder.cursor`
      (e.g. `outline: 1px solid #4a9eff; outline-offset: -1px`, the color
      the format dialog's `:focus-visible` uses at `style.css:632`) and
      `#folders:focus { outline: none }` (or a subtle container ring, judged
      in implementation), so the cursor is shown only while the tree has the
      keyboard and `.current` stays the open-folder highlight when it does
      not.
    - `folders.ts`'s header comment no longer says the tree takes no
      keyboard focus; it says how focus enters and leaves and that the
      culling keys are gated meanwhile.
    - `mise run ci` passes. Manual check for the user: click a folder, press
      `Down` / `Up` — the cursor moves and stays in view, the strip does not
      page and no burst jump happens; `Home` / `End`; `p` / `x` / `1` do
      not judge the photo; `F7` hides the pane; `Escape` then `ArrowDown`
      jumps bursts again and `p` picks again.
  - Implementation approach:
    - Keep `step` DOM-free and take `Row[]` (from `rows(tree)`) rather than a
      `Tree`, so the tests build rows with `addRoots` / `expand` /
      `setChildren` exactly as `tree.test.ts` does today (`drawn()` helper).
    - Routing through `main.ts` (the `settings.keydown` pattern) rather than
      a listener on the container is required here: only `main.ts` holds the
      keymap, and the gate must see the resolved action name. `folders.ts`
      must not import `main.ts`; it only exports `hasFocus` / `keydown`.
    - Use `keyName` from `keys.ts` for the key names so the tree and the
      keymap agree on spelling (`arrowup`, not `ArrowUp`); a lone modifier
      (`null`) is ignored.
    - `render()` rebuilds the rows on every call; the container itself
      survives, so focus is not lost by a re-render. Only the cursor row's
      `scrollIntoView` is needed after a key; do not scroll on every render
      (`reveal` already scrolls the current row once).
    - Moving the cursor does **not** open the folder (decided by the user).
    - Verify what happens to focus when `toggleLeft` / `toggleSides` hides
      `#side` while the tree is focused: browsers blur a focused element
      that becomes `hidden` (focus goes to `body`), so culling keys return
      on their own; if a platform keeps `activeElement` on the hidden
      container, call `container.blur()` from `changePanels` when
      `!panels.left`.

- [ ] Step 2: `Right` / `Left` expand, collapse and step across levels; `Enter` opens the cursor row
  - Done when:
    - `tree.ts` grows the pure decision, e.g.
      `treeKey(tree, cursor, key)` returning a command
      (`{ kind: "focus", path } | { kind: "expand", path } | { kind: "collapse", path } | { kind: "open", path } | null`),
      or extends `step` with `"left"` / `"right"` and a separate lookup —
      pick the shape that keeps Step 1's function intact. Rules (WAI-ARIA
      tree): `right` on a collapsed row that can expand (`children ===
      undefined` or `children.length > 0`, the same test `render()` uses for
      the expander) -> `expand`; `right` on an expanded row with listed
      children -> `focus` its first child; `right` on a leaf or an expanded
      row still listing -> `null`. `left` on an expanded row -> `collapse`;
      `left` on a collapsed row (or a leaf) with a parent -> `focus` the
      parent (the nearest earlier row with a smaller depth); `left` on a
      root -> `null`. `enter` -> `open` the cursor row; with no cursor ->
      `null`. Unit tests in `tree.test.ts` cover every rule, including the
      collapsed-root, leaf and unlisted-but-expanded cases.
    - `folders.keydown` maps `expand` / `collapse` to the existing
      `toggle(path)` (so `Right` re-lists through `list_subfolders` just like
      the expander click, and a listing error collapses and reports as
      today), `focus` to the cursor move + scroll of Step 1, and `open` to
      the `open(path)` callback the row click uses. `arrowleft` /
      `arrowright` / `enter` are consumed (return true, `preventDefault`)
      even when the command is `null`; the Step 1 gate would swallow the
      arrows anyway, this just keeps it explicit.
    - After `Enter` the folder opens through `openDirectory` and `reveal`
      sets `current` and the cursor to it, as any open does; focus stays in
      the tree so the user can keep browsing (verify `reveal`'s
      `scrollIntoView` and the cursor agree).
    - `mise run ci` passes. Manual check for the user: `Right` on a
      collapsed folder lists it and `Right` again enters it; `Left` twice
      goes to the parent and collapses it; `Enter` opens the folder and the
      strip follows; `ArrowLeft` / `ArrowRight` do not page while the tree is
      focused, and do again after `Escape`.
  - Implementation approach:
    - Assumes Step 1 is merged (the focus model, cursor, routing and gate).
    - The parent of a row is derived from `rows(tree)` by depth (walk back
      to the first row with `depth - 1`), not from path string manipulation;
      `tree.ts`'s `ancestorsWithin` / `join` are for revealing a raw path and
      are not needed here.
    - `toggle` and `open` are already the click paths; do not add a second
      listing or open path.

- [ ] Step 3: Type-ahead: typing jumps the cursor to the next folder whose name starts with the typed prefix
  - Done when:
    - `tree.ts` exports two pure pieces, both tested in `tree.test.ts`:
      - `typeAhead(rows: Row[], cursor: string | null, prefix: string): string | null`
        — case-insensitive, searches from the row **after** the cursor,
        wraps around to the top, and returns the path of the first row whose
        `node.name` starts with `prefix`, or `null` (cursor unchanged) when
        none does. When `prefix` is the same character repeated (`"aaa"`),
        it behaves as the single character, so tapping a letter cycles
        through the folders starting with it (WAI-ARIA tree behavior). With a
        `null` cursor the search starts at the first row.
      - a buffer rule, e.g.
        `appendTyped(buffer: { text: string; at: number }, char: string, now: number, timeout = 500): { text: string; at: number }`
        — appends when `now - at <= timeout`, else starts a new buffer with
        `char`. Tests: append within the timeout, reset after it, the repeat
        rule composed with `typeAhead`, no match leaves the cursor, wrap
        around, case-insensitivity, a space inside a name (`"My Photos"`).
    - `folders.keydown` treats a key as type-ahead when `event.key.length
      === 1` and none of `ctrlKey` / `altKey` / `metaKey` is held (`Shift`
      is fine, it only changes the case); it keeps the buffer with
      `Date.now()`, calls `typeAhead`, moves and scrolls the cursor on a hit,
      and consumes the key either way (`preventDefault`, so `Space` does not
      scroll the container and no letter reaches the keymap). The buffer is
      cleared on `blur` of the container.
    - The tree keys of Steps 1-2 are checked before type-ahead (`Enter`,
      `Escape`, the arrows, `Home`, `End` have `event.key.length > 1`, so
      they cannot collide anyway).
    - `mise run ci` passes. Manual check for the user: with the tree
      focused, typing `pi` lands on `Pictures`; tapping `d` twice cycles
      between two folders starting with `d`; after a pause the buffer
      restarts; `p` no longer picks the photo (already true since Step 1).
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged.
    - Keep the timer out of the pure code: `appendTyped` takes `now`, and
      `folders.ts` passes `Date.now()`; no `setTimeout` needed (a stale
      buffer is discarded on the next keystroke by the timestamp).
    - 500 ms is the timeout most tree implementations use; make it a named
      constant, not configurable.

- [ ] Step 4: Document the tree keys and the gate
  - Done when:
    - `docs/usage.md`: the "Folders" bullet says that clicking a folder
      gives the tree the keyboard, lists the keys in a sentence or two, and
      states that while the tree has the keyboard the culling keys are off
      and only `Open Folder`, the pane toggles (`F6` / `F7` / `F8` / `Tab`)
      and the menu accelerators still work, until `Escape` or a click
      elsewhere. The "These keys are fixed and cannot be changed" table
      gains rows "in the folder tree" for `ArrowUp` / `ArrowDown` / `Home`
      / `End`, `ArrowRight` / `ArrowLeft`, `Enter`, typing (type-ahead) and
      `Escape` (the settings tab-strip row at line 292 is the pattern), and
      the existing `Escape` row mentions leaving the tree. The rebindable
      `Keys` table is unchanged (the tree keys are fixed), but a sentence
      under it notes which actions keep working with the tree focused (the
      Allowlist above).
    - `README.md`'s "Folder tree" bullet gains one sentence on the keys, and
      `README.ja.md`'s bullet the same sentence in Japanese, in the same PR.
    - `mise run ci` passes (the formatter ignores `*.md`; nothing else to
      run).
  - Implementation approach:
    - Assumes Steps 1-3 are merged, so the wording matches what shipped.
    - If the caller prefers, fold this into Step 3's PR; it is kept separate
      only so the code PRs stay small.

## Trade-offs and risks

- **`undo` / `redo` while the tree is focused.** Planned: swallowed — they
  rewrite a photo's judgment, which the user asked to keep off. Alternative:
  allow them, since `Cmd+Z` with a folder selected is unlikely to be meant
  for the tree (the tree has nothing to undo). One-line change to
  `TREE_PASSTHROUGH` either way; the caller can move them.
- **A pane toggle rebound to a plain letter vs type-ahead.** Planned: the
  tree's own keys (navigation and type-ahead) are checked before the
  allowlist, so a user who rebinds `toggleLeft` to `l` cannot type-ahead
  `l` while the tree is focused — but `F7` etc. are the defaults and none of
  them is a printable character, so this only bites a deliberate rebind.
  Alternative: allowlist first, which would instead make that letter dead
  for type-ahead. Documented in `usage.md`'s fixed-keys note.
- **`Escape` in the tree while Compare is on.** Planned: `Escape` blurs the
  tree first (the tree branch runs before the menu / Compare `Escape`
  branches in `main.ts`); a second `Escape` leaves Compare. Compare can be
  entered with the tree focused only through the strip / a click (`v` is
  swallowed), so this is rare. Alternative: let `escape` fall through when
  `comparing`, which needs `main.ts` to skip the tree for that key; judged
  in Step 1 if it feels wrong in the manual check. Menus are not a concern:
  their buttons `blur()` on click and the context menu opens on a strip
  right-click, so focus is never in the tree while one is open.
- **Hidden pane keeps focus?** If hiding `#side` with `F7` / `Tab` leaves
  `document.activeElement` on the hidden container on some platform, the
  culling keys would stay gated with no visible tree. Step 1 verifies and,
  if needed, blurs the container from `changePanels`.
- **Fixed keys vs the rebindable keymap.** Planned: fixed. The keymap
  (`shortcuts.rs`, `from_overrides`) is global and one-key-one-action; the
  arrows already belong to `previous` / `next` / `burstPrevious` /
  `burstNext`, so tree actions could not be added with the same defaults
  without a per-context keymap, which does not exist. The tree keys join the
  "fixed" table in `docs/usage.md` next to the settings tab-strip arrows.
- **Gate by action name, not by key.** Planned: the allowlist holds action
  names, so a user who rebinds `open` to `o` still gets `o` = open folder
  in the tree (and loses `o` for type-ahead, see above). Keying by physical
  key would silently break under rebinds.
- **Native scrolling of the focused container.** `#folders` is `overflow:
  auto`; once focusable, unhandled keys (`PageUp` / `PageDown`) scroll it
  natively. All tree keys and type-ahead characters (including `Space`)
  call `preventDefault`; `PageUp` / `PageDown` are left native (page scroll
  of the tree), which is reasonable behavior.
- **Cursor start.** Planned: the open folder (set by `reveal`), else the
  first row. Alternative: remember the last cursor across opens; not needed
  now.

## Progress

- (none yet)
