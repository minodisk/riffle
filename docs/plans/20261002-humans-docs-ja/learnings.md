# Learnings

## Step 1

- `cameras.ja.md` still links to `./usage.md` (English): `usage.ja.md` does
  not exist yet. Step 3 must repoint it to `./usage.ja.md`.
- The camera table rows of `cameras.ja.md` were copied byte for byte from
  `cameras.md` (only the header row is translated) and checked with `diff`;
  both drafts were written in the scratchpad, copied, and checked with
  `diff -q`.
- The Japanese files keep one line per paragraph, as `README.ja.md` does,
  rather than the English files' 80-column wrapping.
- Dropping `（英語）` in `README.ja.md` left a link whose text ends in Latin
  (`docs/humans/cameras.ja.md`) directly against Japanese; a space was added
  after it, as `README.ja.md` does after `[YuNet](...)`. Steps 2-3 should do
  the same when they drop the tag.
- In prose, "MakerNote" is translated as メーカーノート; the meta pane group
  name `Maker note` stays as a code-ish label. Mermaid node labels and table
  headers are translated too. See `headings.md` for headings, slugs and terms.

## Step 2

- `performance.ja.md` was drafted in the scratchpad, copied and checked with
  `diff -q`; heading levels, the 123 table rows and every number of
  `performance.md` were compared mechanically (the only extra digits are
  Japanese counters such as `1 回目`).
- Prose inside table cells (`every body`, `none: ...`, the step and folder
  descriptions) is translated; numbers, units, `files/s`, `->` arrows and
  code spans stay. Headings keep product names, `n=` and the platform in
  full-width parentheses. Headings and new terms are in `headings.md`.
- In-text references to other sections ("see "Real folders on Windows"
  below") are plain quoted text in the English file, not links, so they
  became 「後述の「...」を参照」 with the translated heading, still not links.
- `README.ja.md`'s performance link sat at the end of its line, so dropping
  `（英語）` needed no extra space.

## Step 3

- `usage.ja.md` was drafted in the scratchpad, copied and checked with
  `diff -q`. Headings (6), top-level bullets (35), sub-bullets (13), table
  rows (66) and links (11) match `usage.md`; the sorted code spans of both
  files differ only where an English code span wraps across a line
  (`Face tracking`, `Lock On AF`, ...), which the one-line-per-paragraph
  Japanese file joins.
- Each feature bullet is one line, however long (the `Folders` bullet is
  one paragraph in the English file too), following Step 1's rule.
- Feature bullet names that are UI labels (`Move Rejected to Trash…`,
  `Sequence JPEG Timestamps…`, `Undo`, ...) stay English so the in-text
  "see **...**" cross-references still match the menus; the rest are
  translated. Quoted dialog and status strings ("12 more folders with no
  rejects") stay English. The `max(...)` code span of the new-time rule is
  kept and followed by a Japanese gloss in parentheses.
- `README.ja.md`'s three `usage.md` lines got the same space-after-link fix
  as Step 1 when `（英語）` was dropped. The remaining `（英語）` in
  `README.ja.md` is after `CONTRIBUTING.md`, which has no Japanese version;
  Step 4's "no `（英語）` in `README.ja.md`" check has to allow that one.
- `cameras.ja.md`'s link text was `usage.md`; it now reads `usage.ja.md`,
  matching the file-name style of the original.
- A stray `python -` heredoc in a shell command hung on Windows (the REPL
  waits on a console handle); avoid bare `python -` in the Bash tool.
