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
