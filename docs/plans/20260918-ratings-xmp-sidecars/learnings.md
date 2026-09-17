# Learnings

## Step 1: Record the user's Phase 3 confirmations in the README

- The user's Phase 3 confirmations did not cover the whole "Awaiting the
  user's confirmation" list: the filmstrip highlight following paging keys and
  auto-repeat, click-to-page, scrolling a 5000-file strip, and a drag that
  leaves without dropping were not mentioned. The paragraph was reduced to
  those rather than removed, keeping the three-way split
  `docs/agents/tauri-app.md` asks for.
- "A fast second open" is the user's impression, so the README says so
  explicitly and the todo item about missing real-folder numbers stays.

## Step 2: Core: XMP sidecar read, write and patch

- quick-xml 0.42 is `&str`-based (`NsReader::from_str`, `Event<'i>` borrowed
  from the input), which makes byte splicing easy: positions are byte offsets
  into the very slice being patched.
- `buffer_position()` gives exact **per-event** offsets and nothing finer.
  Events are contiguous, so tracking the previous position yields the exact
  byte range of a start tag or a text node — the element form
  (`<xmp:Rating>3</xmp:Rating>`) is spliced straight from the `Text` event's
  range. There is **no attribute-level offset**: `Attribute` in 0.42 carries
  only `key` and a decoded `value`. So the planned fallback is what is used
  for the attribute form: a scan of the start tag's own (exact) range for the
  attribute's raw qname, requiring a preceding whitespace and a following
  `=`, then the quote pair. `attributes_raw()` would give the same text but
  no offset into the document.
- Namespaces are resolved through `reader.resolver().resolve_element()` /
  `resolve_attribute()` after `read_event()`; the bindings of the start tag
  being read are already in scope. The prefix to write with is chosen by
  asking the resolver whether `xmp:` or `xap:` is bound to the XMP namespace
  at that `rdf:Description`, and only otherwise is `xmlns:xmp` declared.
- `read_rating` returns `Some(0)` for `xmp:Rating="0"`: `0` is a legal value
  (unrated), not an absence. Absence is the property not being there at all.

## Deferred issues (todo candidates)

- Core: if a sidecar binds the `xmp` prefix to some *other* namespace and
  binds none to `http://ns.adobe.com/xap/1.0/`, `xmp_prefix` in
  `crates/core/src/xmp.rs` declares `xmlns:xmp` on the `rdf:Description`
  anyway, which rebinds the prefix for the other attributes on that tag. No
  real-world producer does this and handling it would need a generated
  prefix; noted while implementing Step 2.
