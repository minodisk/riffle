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

# Remove the Open folder button

## Purpose

The left pane now holds the folder tree, which is how a folder is normally
opened, so the `Open folder` button above it is a leftover from before the
tree existed. Removing it gives the tree the whole pane. The folder picker
stays reachable through `File > Open Folder…`, the `open` key and the
empty-state hint, because the tree hides hidden folders and only roots at
the home folder and the mounted volumes (an unmapped UNC path, say, is
unreachable from it).

## Steps

- [x] Step 1: Remove the `Open folder` button from the left pane
  - Done when:
    - `#side` in `crates/app/ui/index.html` holds only `#folders`; the tree
      fills the pane.
    - `main.ts` has no `openEl` (both the lookup at line 100 and the click
      listener at line 2404 are gone); `openFolder()`, the `open` keymap
      case, the `open-folder` menu event handling and the empty-state click
      are untouched and still open the picker.
    - `style.css` has no `#open` / `#open:hover` rules, and the `#side`
      comment no longer mentions the button (e.g. "The left pane: the folder
      tree. Its width is fixed, like #info's, ...").
    - `docs/usage.md` no longer describes the button: the **Folders** bullet
      says the left pane is the folder tree, and the sentence about what
      still opens folders the tree does not reach names only
      `File > Open Folder…`, the `open` key / empty-state hint and a drop.
    - `mise run ci` passes (format, lint, type-check, tests, Rust).
  - Implementation approach:
    - Files: `crates/app/ui/index.html`, `crates/app/ui/src/main.ts`,
      `crates/app/ui/style.css`, `docs/usage.md`.
    - `#side` is already a flex column with `#folders { flex: 1; min-height: 0 }`,
      so no layout change beyond deleting the `#open` rules is expected;
      confirm in `pnpm exec vp dev` (or the built app) that the tree starts at
      the top of the pane and scrolls. Keep `#side`'s fixed `220px` width: it
      was introduced because min-content wrapped the button label, but the
      tree has no natural width either.
    - Leave `crates/app/ui/src/settings.ts:16` (`open: "Open folder"`) alone;
      it is the label of the keymap action in the Shortcuts settings, not the
      button.
    - `README.md` / `README.ja.md` and `docs/agents/**` do not mention the
      button, so they are not touched.
    - No existing test selects `#open`, so no test changes are expected; add
      none.
    - Commit: `feat(app): remove the Open folder button from the left pane`

## Trade-offs and risks

- Commit type: the change is user-visible (a control disappears), so it is
  `feat(app)`, which puts it in the release notes.
- The `open` keymap label in the Shortcuts settings still reads
  "Open folder". It is still accurate (the key opens the folder picker), so
  it is left as is.
- Discoverability: with the button gone, a user who has never opened the
  File menu has the empty-state hint (which names the key and says to click
  it) and the tree. That is judged sufficient; if it turns out not to be,
  adding a mention of `File > Open Folder…` to `openHint` in `empty.ts`
  would be a separate follow-up.

## Progress

- (none yet)
