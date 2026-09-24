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

# Lightroom-style layout: bottom filmstrip, folder tree, panel toggles

## Purpose

Riffle's main window keeps the filmstrip in the left column and has no way to
move between folders other than `File > Open Folder…` or a drop. The user
switches folders often and is used to Lightroom Classic / Lightroom / DxO
PhotoLab, where the left pane is a folder tree, the filmstrip runs along the
bottom, the meta pane is on the right, and each panel can be hidden from the
keyboard (`F6` filmstrip, `F7` left, `F8` right, `Tab` both side panels).
After this work:

- the left pane is a lazily expanded folder tree; clicking a folder opens it
  the way `File > Open Folder…` does, and the open folder is revealed and
  highlighted;
- the filmstrip is horizontal along the bottom, under the viewer, with the
  filter / sort tools and the `N / M` counter in a header bar above it;
- the arrow keys follow the strip's new axis;
- the left pane, the filmstrip and the right pane can each be hidden to give
  the viewer the height a landscape 3:2 frame needs on a 16:9 / 16:10 screen,
  and the hidden state survives a restart.

The user wants this released as a minor version (0.4.0). The project is
pre-1.0 with `bump-patch-for-minor-pre-major`, so a `feat:` alone would only
bump the patch; Step 4 carries a `Release-As: 0.4.0` footer (see Step 4).

Constraints (from `CLAUDE.md` and the task): everything committed in English,
`README.ja.md` updated in the same PR as `README.md`; frontend under
`crates/app/ui` (Vite+, `mise run ci`), backend Tauri 2 in `crates/app`; new
TypeScript logic gets vitest tests and new Rust logic gets unit tests; no
speculative configurability (no setting for the strip's position). Every step
leaves the app working. GUI checks are manual confirmations by the user (see
`docs/agents/tauri-app.md`, "GUI automation does not work on this Mac"); list
them in the PR.

## Steps

- [x] Step 1: Move the filmstrip to a horizontal strip along the bottom, with the filter / sort tools in its header bar, and rotate the arrow keys
  - Done when:
    - `crates/app/ui/index.html`: `body` is a column of `#main` (the row
      `#side` | `#viewer` | `#info`) and a new `#film` block under it. `#film`
      holds `#strip-bar` (the existing `#tools` with `#filter` / `#sort`, and
      `#position`) above `#strip` / `#strip-inner`. `#side` keeps only the
      `#open` button (at its top; Step 3 puts the tree under it). No element
      id used by `main.ts` / `strip.ts` is renamed.
    - `crates/app/ui/style.css`: `#film` is `flex: none`; `#strip` scrolls on
      the x axis only (`overflow-x: auto; overflow-y: hidden`, same thin dark
      scrollbar and `scrollbar-gutter: stable`); `#strip-inner` is the cell
      height tall and gets `--cell-width` (146px cell + 8px gap = 154px) as
      the single source of truth instead of `--cell-height`; `.cell` keeps its
      146px width and 168px height and is placed by `left`, with `top: 7px`.
      The burst band `::before` bridges the gap to the left (`left: calc(-1px
      - var(--cell-gap)); right: -1px; top/bottom: -5px`), `burst-first`
      rounds the left corners and `burst-last` the right ones. `#filter-menu`
      opens upward from the header bar (`bottom: calc(100% + 4px)`, anchored
      to `#filter` / `#strip-bar`, `max-height` bounded by the viewport), and
      `#sort-menu` likewise; the CSS comments on `#side` ("The strip is on the
      left rather than the bottom…", "The containing block for the filter
      fly-out…") are rewritten for the new geometry, not left stale.
    - `crates/app/ui/src/strip.ts`: the axis swap — `CELL_WIDTH` read from
      `--cell-width`, `scrollLeft` / `clientWidth` in `pickNext`, `render`,
      `setFiles` (`keepScroll` clamps against the width) and `setCurrent`,
      `inner.style.width`, `el.style.left`. A `wheel` listener on `#strip`
      turns a vertical wheel delta into horizontal scrolling (`scrollLeft +=
      deltaY` when `deltaX` is 0, with `preventDefault`), as Lightroom's
      filmstrip does; trackpad horizontal swipes already scroll natively. The
      header comment no longer says "down the left edge".
    - `crates/app/src/shortcuts.rs` `DEFAULTS` are rotated a quarter turn to
      match the strip: `previous` = `arrowleft`, `next` = `arrowright`,
      `extendPrevious` / `extendNext` = `shift+arrowleft` / `shift+arrowright`,
      `burstPrevious` / `burstNext` = `arrowup` / `arrowdown`,
      `burstFramePrevious` / `burstFrameNext` = `alt+arrowleft` /
      `alt+arrowright`. The existing shortcut tests are updated where they
      assert those defaults. `crates/app/ui/src/settings.ts` `shortcutLabels`
      says "Extend selection left / right".
    - `main.ts` needs no dispatch change (actions are named); only comments
      that mention the vertical strip or `Shift+ArrowUp` are updated.
    - Docs: `docs/usage.md` "Filmstrip" (runs along the bottom, counter in its
      header), "Bursts" and the `Keys` table use the new arrows; `README.md`
      "Filmstrip" and "Bursts" bullets and `README.ja.md` lines 46 / 51 are
      updated in the same PR. `docs/agents/tauri-app.md`'s two notes that
      describe the vertical geometry ("`flex: none; width: min-content` to
      size a column by its fixed-width child" and "Strip cell geometry") get a
      one-line amendment saying the strip is now horizontal and which
      property is the source of truth; nothing else in the guide is touched.
    - `mise run ci` passes. Manual check listed in the PR for the user: bursts
      band correctly, click / Cmd-click / Shift-click / right-click on cells,
      wheel scrolls the strip, the filter menu opens upward and is not
      clipped, a landscape and a portrait frame both fit the viewer.
  - Implementation approach:
    - Keep `strip.ts`'s public API (`setFiles`, `setCurrent`, `setSelected`,
      `setRating`, …) unchanged so `main.ts` is untouched apart from comments.
    - Tools location, and why: the filter and sort act on the strip, and
      Lightroom Classic keeps its filter controls in the filmstrip's header
      bar, so `#tools` moves into `#strip-bar` together with `#position`.
      This also keeps them visible when the left pane is hidden (Step 4). The
      alternative (top of the left pane) is in Trade-offs.
    - Follow `docs/agents/tauri-app.md`: "Style the strip placeholder on
      `.cell img:not([src])`", "Paint a full-bleed band … `isolation:
      isolate`", "`margin-left: auto` on a flex item only pushes items after
      it" (the `#filter { order: 1; margin-left: auto }` trick keeps working
      inside the bar), and "The strip context menu is HTML" (nothing to
      change; it is `position: fixed`).
    - The guide's "Give the current folder one token" and "A derived-state
      refresh has to run even when `refilter` short-circuits" are unaffected:
      `setFiles` still bumps `generation`.

- [x] Step 2: Folder listing commands for the tree (Rust)
  - Done when:
    - New `crates/app/src/folders.rs` (registered in `main.rs`'s
      `generate_handler!`; `commands.rs` is already 3000 lines) with two
      `async` commands wrapping `spawn_blocking`:
      - `folder_roots(app) -> Vec<FolderNode>`: the tree's top level, in
        order: the home directory (`app.path().home_dir()`), then mounted
        volumes — macOS: the children of `/Volumes`; Linux: the children of
        `/mnt`, `/media/*` and `/run/media/*` that are directories (this
        covers WSL's `/mnt/c`, `/mnt/d`); Windows: every existing drive root
        `A:\`..`Z:\` (a `Path::exists` probe each). Roots that do not exist
        are dropped; the list has no duplicates (the home volume is not
        listed twice via `/Volumes/Macintosh HD`, e.g. by canonical path).
      - `list_subfolders(dir: String) -> Result<Folder, String>` where
        `Folder { raw_count: usize, children: Vec<FolderNode> }` and
        `FolderNode { name: String, path: String }`: one `read_dir` of `dir`,
        counting the entries `riffle_core::scan::is_raw_file` accepts and
        collecting the child directories, skipping names starting with `.`
        (and, on Windows, entries with the hidden attribute if that is a
        one-liner; otherwise document that only dot-names are skipped),
        sorted case-insensitively by name. A `read_dir` failure is the
        `Err` string (`"{dir}: {e}"`, like `list_dir`), so an unreadable
        folder shows as such in the tree rather than as empty.
    - Unit tests on a temp dir: children sorted and dot-dirs skipped,
      `raw_count` counts `.ARW` / `.DNG` files and ignores others, a missing
      dir is `Err`; `folder_roots` is exercised only for "the home dir is
      first and every entry exists".
    - `CLAUDE.md`'s layout paragraph mentions `src/folders.rs` (the folder
      tree's listing commands).
    - `mise run ci` passes. No frontend change; the app behaves as before.
  - Implementation approach:
    - Do not probe each child for RAW files (that is one `read_dir` per
      child, slow on network shares); the RAW count is only of the listed
      folder itself. See Trade-offs if the caller wants marks on children.
    - Follow the guide's "Synchronous commands run on the main thread": both
      commands are `async` + `spawn_blocking` (directory IO on a share can
      block for seconds).
    - Paths are returned as `to_string_lossy` strings like `list_arw`;
      `openDirectory` in `main.ts` takes exactly that.

- [ ] Step 3: The folder tree in the left pane
  - Done when:
    - `crates/app/ui/src/tree.ts` holds the pure tree state, tested in
      `tree.test.ts`: a node map keyed by path with `children:
      FolderNode[] | undefined` (undefined = not listed yet), `expanded`,
      `rawCount`; functions `expand` / `collapse` / `setChildren` returning
      new state, and `ancestorsWithin(roots, path)` returning the chain of
      paths from the root that contains `path` down to `path` itself (or
      `null` when no root contains it), handling `/`, `\` and drive-root
      separators and a case-insensitive drive letter on Windows-style paths.
    - `index.html` / `style.css`: `#folders` under `#open` in `#side`, a
      scrollable tree (`role="tree"`, rows with an expander glyph, the name,
      and the RAW count of an expanded folder as a small gray badge);
      `#side` gets a fixed width (e.g. 220px like `#info`) now that the strip
      no longer sizes it, and its `min-content` sizing goes.
    - `main.ts` (or a small DOM module next to `tree.ts`): on startup, after
      `shortcuts` / `sort_order` resolve, `folder_roots` fills the top level.
      Clicking the expander lists children through `list_subfolders` (once;
      collapsing keeps them, re-expanding re-lists so a new card shows up).
      Clicking a name calls `openDirectory(path, newFolderToken())`, the same
      path `dropped_folder` uses, guarded by `formatGate.isOpen` like
      `openFolder`. Whenever `openDir` changes (`openDirectory`, including
      the reopen of `lastFolder` at launch and a drop), the tree expands the
      ancestor chain from `ancestorsWithin`, marks the open folder's row
      `.current`, and `scrollIntoView({ block: "nearest" })`s it; when the
      folder is under no root (a UNC path, say), its own root is added as an
      extra top-level node for the session so it can still be revealed.
      Errors from `list_subfolders` go to `setStatus` like other command
      errors. The tree takes no keyboard focus and no arrow keys; those stay
      with culling.
    - Docs: `docs/usage.md` gets a "Folders" feature bullet (what the tree
      lists, that clicking opens a folder, that `Open folder` and a drop still
      work for anything else, that the open folder is highlighted); the
      Filmstrip bullet's opening sentence says where the tree sits. `README.md`
      "Key features" gets one line and `README.ja.md` the same line in
      Japanese.
    - `mise run ci` passes. Manual check for the user: launch reveals the last
      folder in the tree; clicking a sibling folder opens it and the strip,
      viewer and meta pane follow; an external drive appears under the roots.
  - Implementation approach:
    - Assumes Step 1 (left pane free of the strip) and Step 2 (commands) are
      merged.
    - Keep the tree's state pure and tested as `selection.ts` / `burst.ts`
      are; the DOM part follows `strip.ts`'s style (module-level elements,
      an `init(onOpen)` callback).
    - Reveal must check the folder token before touching the DOM (guide:
      "Give the current folder one token, not one counter per feature"): the
      sequence of `list_subfolders` awaits along the ancestor chain can
      outlive a second open.
    - `folder-changed` (the watcher) does not touch the tree; a re-expand
      re-lists, which is enough.

- [ ] Step 4: Hide and show the left pane, the filmstrip and the right pane from the keymap, and remember the state
  - Done when:
    - `crates/app/src/shortcuts.rs` `DEFAULTS` gains, after `open` /
      `undo` / `redo`, `toggleLeft` (`f7`), `toggleRight` (`f8`),
      `toggleStrip` (`f6`) and `toggleSides` (`tab`); `settings.ts`
      `shortcutLabels` names them ("Show / hide the left pane", …). The
      existing forbidden lists need no change (`tab` alone is not reserved;
      `meta+tab` / `alt+tab` are). A test asserts the four defaults and that
      `accelerator("f6")` and `accelerator("tab")` are `None` (no menu item
      is added).
    - `crates/app/src/commands.rs`: `panels(app) -> Value` and
      `set_panels(app, panels: Value)` store `{ "left": bool, "strip": bool,
      "right": bool }` under the `panels` settings key (sync commands, the
      small-file-write exception in the guide, like `set_auto_advance`);
      unknown or malformed values fall back to all visible. Registered in
      `main.rs`. A unit test covers the parse fallback.
    - `crates/app/ui/src/panels.ts` (tested): `Panels = { left, strip, right
      }`, `toggle(panels, which)`, and `toggleSides(panels)` = hide both side
      panes when either is visible, show both otherwise (Lightroom's `Tab`).
    - `main.ts`: the state is read from `panels` at startup before the first
      `draw()`; `runAction` handles the four actions by updating the state,
      setting `hidden` on `#side`, `#film`, `#info`, calling `draw()` (the
      canvas is sized from `clientWidth` / `clientHeight` in `draw`, and
      `scheduleCropForResize()` when zoomed, as the `resize` handler does),
      re-rendering the strip when it is shown again (its `render` saw
      `clientWidth` 0 while hidden; call `strip.setCurrent(index)` or the
      existing resize path), and invoking `set_panels`. `style.css` scopes any
      `display` override on those ids to `:not([hidden])` (guide: "Scope an
      id's `display` override to `:not([hidden])`"). The format dialog's
      `Tab` trap runs before the keymap and is unchanged.
    - Docs: `docs/usage.md` gets a "Panels" feature bullet and four `Keys`
      rows (`F6` filmstrip, `F7` left pane, `F8` right pane, `Tab` both side
      panes), notes that the filter / sort tools and the counter are hidden
      with the strip and that the state is remembered; the fixed-keys table's
      `Tab` row is reworded (the dialog trap still applies while the dialog is
      open). `README.md` "Filmstrip" bullet mentions the panel keys in one
      sentence; `README.ja.md` in sync.
    - Release: the Step 4 PR's squash commit message ends with the footer
      `Release-As: 0.4.0` (put it at the end of the PR body so the squash
      commit carries it), so release-please proposes 0.4.0 for the whole
      layout change.
    - `mise run ci` passes. Manual check for the user: each key toggles its
      panel, `Tab` twice restores both side panes, a hidden strip comes back
      with the current cell in view, the state is the same after a restart.
  - Implementation approach:
    - Assumes Steps 1 and 3 are merged (`#film` exists; `#side` is the tree).
    - Binding plain `Tab` means the main window no longer moves focus between
      its buttons with Tab (Lightroom behaves the same); see Trade-offs.
    - No `View` menu items: `accelerator()` cannot express a modifier-less
      key and a menu item would need the `cfg` split described in the guide
      ("Adding a macOS menu item needs the same `cfg` split as its siblings").

## Trade-offs and risks

- **Where the filter / sort tools live.** Planned: the filmstrip's header
  bar (`#strip-bar`), next to the `N / M` counter — Lightroom Classic's
  filmstrip header carries its filter controls, the tools act on the strip,
  and they stay available when the left pane is hidden. Consequence: `F6`
  hides them with the strip. Alternatives: (a) the top of the left pane,
  above the tree — hidden by `F7` / `Tab` instead, and the filter menu keeps
  opening sideways; (b) keep the 28px `#strip-bar` visible when `F6`
  collapses only the thumbnails, so the tools and the counter are never
  hidden, at the cost of 28px of viewer height. The plan takes the header
  bar and hides the whole `#film`; switching to (b) only changes Step 4.
- **Arrow keys after the rotation.** Planned: `ArrowLeft` / `ArrowRight` =
  previous / next file, `Shift+Left/Right` extend the selection, `ArrowUp` /
  `ArrowDown` = previous / next burst, `Alt+Left/Right` = frame within the
  burst — the current mapping turned a quarter turn, so nothing is unlearned.
  Alternative: keep `Up` / `Down` for files as well as adding `Left` /
  `Right`, and move bursts to `Home` / `End`-style keys; more keys to
  document for little gain. Risk: `from_overrides` skips a stored override
  whose key another action's *default* now holds, so a user who had bound
  `arrowleft` / `arrowright` to some other action (possible only after
  removing them from the burst actions) silently loses that override and
  gets a warning in the log; the shortcuts panel still shows the truth.
- **"Contains RAW" marks in the tree.** Planned: none on children; only the
  expanded folder's own RAW count (free, from the same `read_dir`). Marking
  every child needs a `read_dir` per child — fine on an SSD (a few ms for
  200 children) but seconds on SMB, and it would have to be a second,
  cancellable pass so the tree renders first. If the caller wants marks,
  Step 2 adds a `has_raw(paths: Vec<String>) -> Vec<bool>` command with a
  bounded read per dir and Step 3 fills the marks after the children show.
- **Tree roots.** Planned: home + mounted volumes, plus the open folder's own
  root when it is under neither (UNC shares, an unusual mount point). Home
  covers `Pictures`, and the volume lists cover cards, external drives and
  WSL's `/mnt/<drive>`. Alternative: the filesystem root(s) only — every
  folder reachable, but the user's photos are five clicks down. Another:
  remember folders previously opened (the index's `folders` table) as a
  "Recent" root; not planned, easy to add later.
- **`Tab` as a shortcut.** Binding plain `Tab` and calling `preventDefault`
  removes keyboard focus traversal among the main window's buttons (filter,
  sort, `Open folder`). Lightroom does the same; the settings window is a
  separate context and keeps native Tab. If that is unwanted, bind only `F6`
  / `F7` / `F8` and leave `toggleSides` with a default such as `shift+tab`
  or none.
- **Persisting the panel state.** Planned as a `panels` settings key through
  a sync command (one small JSON write per toggle, the pattern
  `set_auto_advance` uses). Alternative: session-only state, which drops the
  command and the test; the task allows either "if cheap" — it is cheap.
- **No `View` menu.** The toggles are keymap-only; a menu would need the
  macOS `IconMenuItem` `cfg` split and PNG icons, and could not display a
  modifier-less accelerator anyway. Discoverability rests on the shortcuts
  panel and `docs/usage.md`.
- **Wheel scrolling of a horizontal strip.** Planned: `deltaY` scrolls the
  strip horizontally (Lightroom's behavior). Risk: a trackpad user who
  swipes vertically over the strip expecting nothing; trackpads deliver
  `deltaX` for horizontal swipes, which is left native.
- **Viewer height.** The bottom strip (~200px with the bar) takes height a
  16:9 screen lacks for a landscape 3:2 frame; that is exactly what `F6`
  and `Tab` are for, and the persisted state keeps the user's choice.
- **Left pane width.** With the strip gone, `#side` needs a fixed width
  (Step 3 proposes 220px, matching `#info`); a resizable splitter is not
  planned (no speculative configurability).

## Progress

- (2026-09-25) Step 1 complete
