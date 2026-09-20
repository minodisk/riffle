# Learnings

## Step 1

- `SidecarError.path` now carries the RAW path; the sidecar's file name moved
  to the front of `message`, so a read error reads `a.ARW: a.xmp: ...` in the
  pane (the write side keeps the sidecar's full path in its message, as the
  plan's trade-off note says).
- Verified no frontend change is needed: `crates/app/ui/src/main.ts` uses the
  field only as the `errors.add` key and for `baseName(path)` (lines ~1057 and
  ~1188), and the `sidecar-error` handler additionally filters on
  `allFiles.includes(payload.path)` — which the RAW path satisfies, whereas
  the sidecar path never did on the read side, so the shared key is strictly
  an improvement. `crates/app/ui/src/errors.test.ts` treats keys as opaque
  strings.

## Step 2

- quick-xml 0.42's `ResolveResult::Bound(Namespace)`: `Namespace::as_ref()`
  yields `&str`, not `&[u8]` (compare against `XMP_NS` directly).
- `resolve_attribute` needs a `QName` borrowed from a local `String`, so the
  helper returning `ResolveResult<'a>` must tie its lifetime to the reader,
  not to the name buffer. That works because the only borrowing variant
  (`Bound`) points into the reader's namespace buffer.
- Both new tests pass on the first run; the existing xmp tests needed no
  change.
