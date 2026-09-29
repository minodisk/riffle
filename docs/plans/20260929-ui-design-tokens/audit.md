# UI part audit: shadcn counterpart and migration step

Audited on 2026-09-29 on branch `ui-design-tokens-step-2` (Steps 1 and 2
merged into the plan's baseline): `crates/app/ui/index.html`,
`crates/app/ui/style.css` and every DOM-building module (`main.ts`,
`strip.ts`, `folders.ts`, `settings.ts`; `empty.ts` supplies text only).
"Counterpart" is the shadcn/ui component whose look the part takes, always
as a plain CSS class on the tokens. "Exception" marks colors that stay
app-semantic.

| Part | Where | Today | shadcn counterpart / class | Step |
| --- | --- | --- | --- | --- |
| Window, body text | `html, body` | `--background` / `--foreground` | tokens | 1 (done) |
| Side panes (tree, filmstrip, info) | `#side`, `#film`, `#info` | `--sidebar` | Sidebar surface | 1 (done) |
| Settings modal surface, title, close, actions | `#settings-*` | `.dialog*`, `.button ghost` | Dialog | 1 (done) |
| Settings inputs, select, checkboxes, radios, labels | `#label-name-*`, `#label-names-language`, `#auto-advance`, `#mcp-enabled`, `#debug-timing`, `input[name=sidecar-format]` | `.input`, `.select`, `.checkbox`, `.field` | Input, Select (native), Checkbox / RadioGroup (native + `accent-color`), Label | 1 (done) |
| Settings buttons (Reset, Reset all, Clear Cache, Copy, chip ×, +) | `settings.ts` | `.button outline` / `ghost` | Button | 1 (done) |
| Shortcut key chips | `#settings-dialog .chip` | `--muted`, `--radius-sm` | Badge secondary (id rule) | 1 (done) |
| MCP endpoint / examples code blocks | `#settings-dialog .copyable pre` | `--muted` | Code block (id rule) | 1 (done) |
| Settings status / notes | `#settings-status`, `#clear-index-note`, `#mcp-status` | `--destructive`, `--muted-foreground` | text tokens | 1 (done) |
| Settings horizontal tabs | `#settings-tabs` | `--accent` selected | replaced by `.sidenav` / `.sidenav-item` (Sidebar menu) | 6 |
| First-run format dialog | `#format-dialog`, `#format-box`, `#format-choices` | `.dialog*`, `.button outline` | Dialog, Button outline | 2 (done) |
| Trash confirmation | `#trash-dialog` | `.dialog*`, `.button destructive` / `outline`, rows on `--muted` | Dialog, Button, list on muted | 2 (done) |
| Sequence dialog | `#sequence-dialog` | `.dialog*`, `.button primary` / `outline`, rows on `--muted`, `.changed` in `--pick-color` (exception) | Dialog, Button | 2 (done) |
| Filter / sort toggles | `#filter-toggle`, `#sort-toggle` | `#999`, `#2a2a2a`, `#3a3a3a`, `3px`, `.active` `#4a9eff` | `.button outline`, `.active` on `--primary` | 3 |
| Position counter | `#position` | `#999` | `--muted-foreground` | 3 |
| Filter / sort / context menus | `#filter-menu`, `#sort-menu`, `#context-menu` | `#262626`, `#444`, `6px`, shadow | `.menu` (DropdownMenu content) | 3 |
| Menu items, hover, checked | `[role^=menuitem]` buttons (static + `main.ts` builders) | `#ccc`, hover `#3a3a3a`, checked `#2f4a6a` fill | `.menu-item` (DropdownMenuItem), Lucide `check` indicator (CheckboxItem / RadioItem) | 3 |
| Menu separators, headings | `hr`, `.heading` | `#3a3a3a`, `#888` | `.menu-separator` (Separator), `.menu-label` (DropdownMenuLabel) | 3 |
| Context menu shortcut hint | `#context-menu .shortcut` | `#888` | `--muted-foreground` (DropdownMenuShortcut); `.kbd` optional | 3 |
| Filter dots, face icon, stars | `#filter-menu .dot`, `.face`, `.on` / `.off` | `#666`, `#555`; `--label-*`, `--pick-color`, `--reject-color`, `--stars-color` (exceptions) | neutral parts on `--muted-foreground` / `--border` | 3 |
| Scrollbars | `#strip`, `#folders`, `#meta`, menus | `#444` thin | `.scrollarea` (ScrollArea look, `scrollbar-color: var(--border) transparent`) | 3 (menus, strip), 4 (tree), 5 (meta) |
| Strip cell frame, current, selected | `.cell`, `.cell.current`, `.cell.selected` | `3px`, `#f0f0f0` / `#333`, `#777` / `#2a2a2a` | `--radius-sm`; `--primary` + `--accent`; `--ring` + `--muted` | 3 |
| Strip placeholder | `.cell img:not([src])` | `#2a2a2a` | Skeleton (`--muted`) | 3 |
| Strip badges: rating, flag, label strip, count, candidate | `.cell span.*` | `--stars-color`, `--pick-color`, `--reject-color`, `--label` (exceptions); count `#ddd`; `#000` halos (exception) | Badge look kept; count on `--foreground` | 3 |
| Sharpness bar | `.cell span.sharpness` | `#888`, `.best` `--pick-color` (exception) | `--muted-foreground` | 3 |
| Failed cell, rejected dim, burst band | `.cell.failed`, `.cell.rejected`, `.cell.burst::before` | `#402020`, opacity, white 8% tint | exceptions (app semantics) | — |
| Strip inline rename | `.cell input.name` | `#fff` / `#111`, outline `#4a9eff` | `--foreground` / `--background`, `--ring` | 3 |
| Folder rows, hover, current, selected, cursor | `.folder*`, `#folders:focus .folder.cursor` | `#ccc`, `#222`, `#2d4059`, `#2a2f36`, `#4a9eff` | `.sidebar-item` (SidebarMenuButton, active = accent + weight), `--sidebar-ring` | 4 |
| Folder expander, count, failed name | `.folder .expander`, `.count`, `.failed .name` | `#888`, `#999` / `#2a2a2a`, `#d66` | `--muted-foreground`, `.badge` (Badge secondary), `--destructive` | 4 |
| Tree inline rename | `.folder input.name` | `#fff` / `#111`, outline `#4a9eff` | `--foreground` / `--background`, `--sidebar-ring` | 4 |
| Drop feedback | `body.dragging #main` | `#4a9eff` outline | `--ring` | 4 |
| Empty states (no folder / no files / filtered) | `#empty[data-state]` | `#999` | `.empty` (Empty) | 5 |
| Meta pane groups, keys, values, file name | `#meta .group`, `dt`, `dd`, `.name` | `#999` | section label / description on `--muted-foreground` | 5 |
| Status lines: scan / faces progress, 1:1, compare, view-only, held op | `#meta-status .note` | `#999` | `--muted-foreground` (text; no Progress bar exists) | 5 |
| Error rows and dismiss | `#meta-status .error`, its button | `#e0a040`, unstyled button | `--destructive` (Alert destructive look), `.button ghost` | 5 |
| Viewer canvas, grayscale, compare labels, active ring | `#canvas`, `main.ts` `drawCompare` | `#252525`, `#ddd`, `#444`, `#fff`; `#244c31` / `#6bdc8a` (exception) | tokens read via `getComputedStyle` (`--card`, `--foreground`, `--border`, `--primary`) | 5 |
| Focus mark | `main.ts`, `focus.ts` `FOCUS_MARK_COLORS` | candidate colors, black halo | exception (app semantics) | — |
| Native `title` tooltips | `row.title`, `cell.name.title`, `dismiss.title` | OS-rendered | no Tooltip component (native) | — |
| Panel splitters | none (fixed 220px panes) | — | no Resizable handle | — |
| Toasts | none (errors are meta status rows) | — | — | — |
| Settings side navigation | `#settings-tabs` (vertical) | new | `.sidenav`, `.sidenav-item`, `.icon` (Sidebar menu with Lucide icons) | 6 |
