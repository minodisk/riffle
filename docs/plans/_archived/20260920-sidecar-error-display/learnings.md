# Learnings

## Step 1

- The todo item's second TODO ("revert the optimistic has-sidecar flag") was
  already closed by #54 (`f1ba56d`), which removed the `sidecars` set and the
  sidecar header from the meta pane; `has_sidecar` only survives as an unused
  field of the `IndexedFile` interface. Wrap-up can drop both `todo.md`
  headings ("a `sidecar-error` event can be missed under key-mashing" and "an
  unparseable or oversize sidecar fails silently on folder open").
- A truncated XMP cut at an element boundary (after a self-closing
  `rdf:Description`) still parses as far as `read_rating` is concerned; the
  test cuts inside the attribute value to get a reliable parse error.
- `ErrorList` keeps a superseded entry in its original position (a `Map`
  `set` on an existing key does that), pinned by `errors.test.ts`.
- The test helper `reconcile_listed` keeps returning only the dirty rows; a
  sibling `reconcile_listed_with_errors` returns the problems too, so the
  existing call sites did not change.
