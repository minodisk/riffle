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

## Step 3

- Filtering inside `parse_keys` means the empty check has to run twice: once
  on the raw array (so `[]` stays "not a non-empty list of keys") and once
  after the retain (so `["control"]` also falls back to the default).
- `pending` no longer needs to carry the raw `Value`: the only consumer was
  `inactive.insert`, which now stores `Value::from(keys.clone())`, so the
  tuple shrank to `(index, keys)`.
- No frontend change: `MODIFIER_KEYS` in `crates/app/ui/src/keys.ts` already
  rejects lone modifiers at capture time; `MODIFIER_ONLY` in
  `crates/app/src/shortcuts.rs` mirrors it and carries a sync comment.
