# Learnings

## Step 1

- The language-switch row sits inside a centered `<p align="center">` block
  like the icon. Markdown link syntax is not parsed inside a one-line HTML
  block on GitHub, so the row uses `<a href>` tags; it renders as
  `English | 日本語` with only the other language linked.
- Both drafts were copied verbatim from the scratchpad and verified with
  `diff -q` before the row was added; no other edits were made to them.
- The first `mise run fmt` failed with pnpm's "Command \"vp\" not found"
  although `node_modules/.bin/vp` existed; running `pnpm exec vp fmt` directly
  and then `mise run fmt` again both succeeded, so it was transient. `mise run
  ci` passed (lychee: 0 errors, including `README.ja.md` fragments).
