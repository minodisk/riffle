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
