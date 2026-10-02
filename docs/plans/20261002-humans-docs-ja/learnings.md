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
