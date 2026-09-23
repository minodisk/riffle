# Learnings

## Step 1: Render the meta pane as EXIF (standard + maker note) and Riffle groups

- The `if (meta !== null)` gate in `renderMeta()` had to go entirely rather
  than be edited inside: the Riffle group must render when `meta === null`, so
  `metaGroups()` takes `meta` as nullable and the loop runs unconditionally.
- The `0.75rem` gap below the file name moved from `#meta dl` to the group
  heading (`#meta .group`), and `#meta dl` now has no margin; the maker note
  divider (`#meta .section`) reuses the existing `#3a3a3a` border color.
- A `meta` with every field null (possible for a file with no readable EXIF)
  yields no EXIF group at all, since empty sections and groups are dropped.
- Not verified on a real device in this step (GUI automation does not work
  here): the rhythm with a Sony and a Leica file and in the `metaStale` state
  still needs a manual look.
- The first `mise run fmt` in this fresh worktree failed with
  `Command "vp" not found` although `node_modules/.bin/vp` was present; a
  `pnpm install --frozen-lockfile` (a no-op) and a re-run passed. Treat it as
  environmental.
