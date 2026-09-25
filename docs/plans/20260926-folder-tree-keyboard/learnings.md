# Learnings

## Step 1

- The gate lives in its own DOM-free module, `crates/app/ui/src/treekeys.ts`
  (`TREE_PASSTHROUGH`, `treeGate`), with `treekeys.test.ts`; `step` went into
  `tree.ts` next to `rows`, since it only reads `Row[]`.
- Hidden pane and focus: could not be verified on a real WebView here.
  Chromium (WebView2) runs focus fixup when a focused element becomes
  hidden, but WebKit (macOS) has not always done so, so `changePanels` now
  calls `folders.blur()` whenever the left pane ends up hidden rather than
  relying on the browser. Cheap and makes both platforms agree.
- Click focus: left to the browser (a click inside a `tabindex="0"`
  container focuses it on mousedown in both WebView2 and WebKit); no explicit
  `container.focus()` in the row click handler. If the manual check shows the
  tree not taking the keyboard after a click on some platform, add it there.
- The cursor is redrawn by a full `render()` per key (the rows are rebuilt
  anyway); only the cursor row is scrolled into view after a key, and
  `reveal` keeps scrolling the current row once.
- `Escape` in the tree blurs it before the menu / Compare `Escape` branches,
  as planned; a second `Escape` leaves Compare.

## Step 2

- `treeKey(rows, cursor, key)` takes `Row[]` like `step` rather than a
  `Tree`: the rows already carry each node's `children` / `expanded`, and the
  parent is the nearest earlier row with a smaller depth, so nothing else is
  needed. A stale cursor (not among the rows) yields `null` for every key.
- A folder can be expanded yet have no subfolders (reveal expands the open
  folder itself, and it may be a leaf). `render()` draws no expander for it,
  so `Left` treats it as a leaf and goes to the parent instead of issuing an
  invisible collapse; `Right` on it does nothing.
- `Left` on an expanded row still listing (`children === undefined`)
  collapses it, like the expander click would.
- `folders.keydown` routes `expand` / `collapse` to the existing `toggle`
  and `open` to the row-click callback; the cursor stays on the toggled row.
  After `Enter`, `reveal` sets the cursor to the opened folder, and the
  container survives `render()`, so focus stays in the tree (not verified
  on a real WebView here).
