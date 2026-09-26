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

## Step 3

- `typeAhead` searches past the cursor only for a single (or repeated)
  character; a longer prefix searches from the cursor row itself. Searching
  past the cursor for every prefix, as the plan's first rule read, breaks
  the incremental case: `p` lands on `Pictures`, then `pi` would skip it and
  go to a later `Pix`. This is the WAI-ARIA / listbox behavior, and it is
  what makes "typing `pi` lands on `Pictures`" hold with siblings sharing
  the prefix.
- The buffer starts as `{ text: "", at: -Infinity }`, so the first key always
  starts a new buffer without a special case; `blur` of the container resets
  it to that.
- Type-ahead is checked with `event.key` (length 1, no Ctrl / Alt / Meta),
  after the Step 1-2 keys, so `Space` (`event.key === " "`) is consumed too
  and does not scroll the tree. IME composition was not considered (folder
  names typed through an IME would jump per composed keystroke at best);
  not verified on a real WebView.

## Step 4

- The existing `Escape` row of the fixed-keys table gained the tree case
  instead of a second `Escape` row, so the table keeps one row per key.
- The rebind edge case deferred from the Step 3 review is written under the
  rebindable `Keys` table in `docs/usage.md`, next to the list of actions
  that keep running with the tree focused: an allowlisted action rebound to
  a printable character does not fire in the tree, since type-ahead takes
  every printable character first. The README bullets only give the one
  sentence on the keys.
- `docs/usage.md` also says hiding the left pane hands the keys back, which
  is what Step 1's `folders.blur()` from `changePanels` does.

## Deferred issues (todo candidates)

- Round 1 review (item 1) of the type-ahead step flagged that a passthrough
  action (`toggleSides`, `toggleLeft`, `toggleRight`, `toggleStrip`, `open`)
  rebound to a printable key silently stops working while the tree is
  focused, since `folders.ts`'s `typeKey` consumes every printable key
  before `treeGate` is consulted. This was dismissed for now because
  `plan.md`'s "Trade-offs and risks" section already made this call
  deliberately (type-ahead wins over the allowlist) and assigned writing it
  up to Step 4. Step 4 ("Document the tree keys and the gate") must state
  this rebind edge case explicitly in `docs/usage.md`'s fixed-keys note
  (`docs/usage.md`, `README.md`, `README.ja.md`), not just the general
  allowlist wording it already plans to add.
