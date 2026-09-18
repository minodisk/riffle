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

## Step 3

- `RunEvent::ExitRequested` needs `Builder::build(...)` + `App::run(|app, event| ...)`
  instead of `Builder::run(...)`: the one-argument `run` takes the context and
  gives no place to hook the event. The closure's first argument is an
  `&AppHandle`, so `app.state::<AppWriter>()` works there and the drain is a
  plain synchronous `flush(DRAIN_TIMEOUT)` on the main thread. Blocking there is
  fine (and necessary) precisely because the run loop is on its way out; this is
  the one place the "never block the main thread" rule in
  `docs/agents/tauri-app.md` does not apply.
- The writer's `Drop` is deliberately not relied on for the drain. During
  process teardown a managed state's `Drop` is not guaranteed to run, and even
  if it did, the channel disconnect would race the process exit rather than
  being waited on.
- The writer thread is a plain `std::thread` blocking on `recv_timeout` against
  the earliest pending deadline. `recv_timeout(Duration::ZERO)` is a valid
  "flush now" and `saturating_duration_since` keeps an already-elapsed deadline
  at zero rather than panicking, so the debounce loop needs no special case.
- `mark_written` compares the rating with SQL `IS`, not `=`: the unrated state
  is NULL and `rating = NULL` is never true, which would have left every
  cleared rating dirty forever.
- `0` and "unrated" are collapsed in `set_rating` (the row stores NULL). The
  distinction only exists in the sidecar, where `write_rating(existing, None)`
  writes `0` into an existing sidecar and nothing at all when there is none.
- `dirty_rows` is `#[allow(dead_code)]` for now: only the tests call it until
  Step 4 wires the folder-open flush.

## Step 4: App: reconcile sidecars on folder open, and flush dirty rows

- The note carried in from Step 3 checks out: `set_rating` derives `dir` from
  `Path::new(&path).parent()`, and every path the frontend can hand it comes
  from `list_arw` / `folder_entries`, both of which list a **canonicalized**
  directory, so the parent of such a path is exactly the `dir` string
  `files.dir` holds. `dirty_rows(dir)` therefore finds the rows this step
  needs. Nothing was changed for it.
- `dirty_rows` is read **after** the parsed sidecars have been stored, not
  during `reconcile_sidecars`. That ordering is what makes the sidecar win
  over a dirty row: `store_sidecar_ratings` clears `dirty`, so the row is gone
  from the list by the time the writer is told. It also keeps the six rules in
  one place instead of splitting the dirty set across two queries.
- The lock is released between `reconcile_sidecars` (which decides what to
  parse) and `store_sidecar_ratings` (which stores the parse and clears
  `dirty`), so a `set_rating` landing in that window would otherwise be
  silently discarded by the "sidecar wins" write. `reconcile_sidecars` now
  hands back the `dirty` flag it observed for each path alongside the parse
  job, and `store_sidecar_ratings` only applies its update when the row's
  `dirty` still matches that snapshot, so a rating set during the window is
  left in place instead of being overwritten by a sidecar read taken before
  it.
- A sidecar that cannot be read or parsed leaves its row untouched and is not
  an error. It then stays dirty if it was, and the writer's own
  `xmp::write_rating` refuses to patch unparseable XML, so such a file is
  never overwritten from either side. An oversize sidecar is different: it is
  well-formed XML the writer would happily patch, so its path is also kept
  out of what `reconcile_sidecars_of` hands `dirty_rows` back to the caller,
  even if the row is still dirty, or that patch would overwrite an external
  edit that was never read.
- `Writer::set_now` was added rather than a second debounce constant: the
  pending map now stores a deadline per path instead of "the instant of the
  last update", which makes a zero debounce one call site rather than a flag
  threaded through `flush`.
- Tripped on it while writing a test: the fixture sidecar's
  `22-rdf-syntax-ns#` contains the digit the test was replacing to simulate an
  external edit, so a `replace('2', "3")` quietly broke the RDF namespace and
  the parse failed rather than winning. Rewriting the whole fixture is the
  only safe way to fake an external edit.

### Measurement

Second open of a 5000-file folder, Rust side only (list, `stat` every file,
`reconcile`, reconcile the sidecars, `entries`):

| | Median |
|---|---|
| No sidecars | 31.3ms |
| 5000 sidecars, unchanged stat | 67.4ms |

Conditions, stated because they matter: **5000 one-KB regular files named
`*.ARW`, not symlinks and not real ARWs**, in a temp folder on the local APFS
disk, warm page cache, Apple Silicon Mac, `cargo test --release`, median of 7
runs after 3 warm-up runs, from a temporary `#[ignore]`d test in
`crates/app/src/commands.rs` that was deleted before committing.

**These numbers are not comparable to Phase 3's 34.4ms** and must not be read
as replacing it: that one was 5000 symlinks to one real ARW. The two folders
differ, so only the *delta* here is meaningful — the sidecar pass costs about
36ms for 5000 sidecars on this machine, and that is a directory listing plus
one `stat` per sidecar, not a parse, since an unchanged stat parses nothing. A
first open of a folder full of foreign sidecars pays 5000 parses on top and
was **not measured**. Nothing here was measured on a real folder of distinct
ARWs, and nothing here says anything about how the app feels.

## Deferred issues (todo candidates)

- Core: if a sidecar binds the `xmp` prefix to some *other* namespace and
  binds none to `http://ns.adobe.com/xap/1.0/`, `xmp_prefix` in
  `crates/core/src/xmp.rs` declares `xmlns:xmp` on the `rdf:Description`
  anyway, which rebinds the prefix for the other attributes on that tag. No
  real-world producer does this and handling it would need a generated
  prefix; noted while implementing Step 2.
- The `sidecar-error` event is emitted but nothing displays it yet; the status
  line handling is Step 5 of this plan (`crates/app/ui/src/main.ts`). Not an
  out-of-plan issue, just noted so it is not mistaken for a gap.
- A failed sidecar write is retried only on the next open of that folder: the
  writer does not schedule a retry of its own. Basis: Step 3's error path in
  `crates/app/src/sidecar.rs` leaves the row dirty and reports. Acceptable per
  decision 3, but worth a todo if a locked SD card turns out to be common.
- App: an unparseable or oversize sidecar is skipped silently on a folder
  open (`reconcile_sidecars_of` in `crates/app/src/commands.rs` drops the
  `Err`), so a file whose sidecar another tool corrupted shows no rating and
  no reason. Reporting it to the status line needs a count or an event the
  frontend can show; out of scope for Step 4, which was told not to touch
  `main.ts`.
- App: the sidecar pass reads the directory a second time (`list_arw_in` then
  `list_sidecars_in`). One listing that returns both would halve that part of
  a folder open; noted while implementing Step 4, not done because
  `list_arw_in` is also `list_arw`'s implementation and shared with paths that
  do not want sidecars.

## Step 5

- Key collision check against the current `crates/app/ui/src/main.ts` (which
  now carries Phase 5's `Space`): the handled keys are the paging set
  (arrows, `w`, `a`, `s`, `d`, `h`, `j`, `k`, `l`), `f`, `Space` and `o`.
  `0`, `1`-`5`, `u` and `x` are all free, so this step adds them without
  touching a reserved key.
- The frontend keeps two maps rather than one: `ratings` (path -> value) is
  what the canvas badge and the status line read, and `touched` records the
  paths judged through the keyboard in this session so a `folder_entries`
  refresh (which can predate the pending sidecar write, and fires on every
  `scan-done`) cannot undo a keypress. Both are cleared in `openDirectory`,
  which is where the plan's "authoritative until the next folder open" lands.
- The strip's API is index-based (`setRating(index, rating)`), but everything
  else in `main.ts` is path-keyed, so `openDirectory` builds a `fileIndex`
  path -> index map once instead of an `indexOf` per update.
- The canvas badge is drawn inside `draw()` (and `drawZoom()`), not written
  into the DOM, so it survives a resize and is repainted during key auto-
  repeat, as the step asked.
- `.cell span` in `style.css` already positioned the file name; the badge is
  a second `<span>`, so its rule has to reset `bottom`/`left`/`width`
  explicitly rather than only setting `top`/`right`.
- The revert path checks `folderToken`, per `docs/agents/tauri-app.md`: no new
  counter was minted. Reverting only matters for the folder the rating
  belongs to; after a folder change the map has already been cleared.
- Not verified here: everything in this step that needs a running window.
  GUI automation is denied on this machine, so the README lists the Phase 6
  behaviours under "Awaiting the user's confirmation".

## Deferred issues (todo candidates)

- App: a `sidecar-error` event overwrites whatever note is in the status line
  and is cleared by the next page turn, so an error raised while the user is
  mashing keys can be missed. Basis: Step 5's "shows its message once in the
  status line" is met literally, but `note` is a single transient slot
  (`crates/app/ui/src/main.ts`, `setStatus`). A dedicated, sticky error area
  would be a UI change beyond this step.

## Step 6: Documentation and status update

- The Step 4 measurement was already in `README.md` ("What the sidecar pass
  adds to a folder open (Phase 6)") with its conditions intact and correctly
  placed next to the other measurement sections; it was left untouched rather
  than restated, so the numbers exist in exactly one place.
- The README's three-way split already had Phase 6 paragraphs from Steps 4 and
  5. Step 6 only sharpened the honesty of the tool-compatibility sentence:
  **no** reader has been confirmed by the user, and DxO PhotoLab could not be
  checked here at all (the web sources returned 403 during planning). The new
  "Ratings and XMP sidecars" section says which claims are *reported* by
  exiftool's tag reference and by Bridge/darktable/Lightroom documentation and
  which are verified here (none of them).
- The six reconciliation rules are now prose in the README as well as the
  plan; they were written from decision 2 rather than re-derived from the
  code, then checked against `reconcile_sidecars` for agreement.
- `docs/agents/tauri-app.md` gained the `RunEvent::ExitRequested` drain on the
  Rust side (`Builder::build` + `App::run`, and why `Drop` is not the drain,
  including that this is the one exemption from "never block the main thread"),
  and two corollaries on the folder-token section: a same-folder reopen is a
  new open with the same `dir`, so the token is what catches it, and a
  once-registered event listener needs a payload guard of its own — which is
  what `sidecar-error`'s `fileIndex.has(payload.path)` check is.
- Documentation only; nothing under `crates/` was changed, and the parallel
  session's Phase 5 README sections were left alone.
