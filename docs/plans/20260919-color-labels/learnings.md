# Learnings

## Step 1: XMP `xmp:Label`

- `locate` now takes the property's local name. The element form no longer
  returns on the `Text` event; it waits for the end tag so it can report the
  whole element range for removal (the start position is remembered at the
  start tag). `quick_xml`'s `LocalName::as_ref()` compares against `&str`
  directly, not `&[u8]`.
- Clearing an element-form label removes its whole line only when nothing but
  whitespace shares the line; otherwise just the element.
- Values are spliced raw with no escaping, per the plan; a label containing
  `"` or `<` would produce broken XMP. Not a concern for the known
  vocabularies.
