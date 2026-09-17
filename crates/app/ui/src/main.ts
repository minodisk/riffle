import * as strip from "./strip.js";

// Header layout of a `preview` payload, see `crates/app/src/commands.rs`.
const PREVIEW_HEADER_LEN = 8;
const PREVIEW_KIND_JPEG_V1 = 1;

interface Focus {
  sensor_w: number;
  sensor_h: number;
  x: number;
  y: number;
}

interface IndexedFile {
  path: string;
  orientation: number;
  focus: Focus | null;
  has_thumb: boolean;
}

// Mirrors `Metadata` in `crates/app/src/commands.rs`: already formatted for
// display, so a field is either a string to show or null to leave out.
interface Metadata {
  name: string;
  camera: string | null;
  lens: string | null;
  aperture: string | null;
  shutter: string | null;
  iso: string | null;
  focal_length: string | null;
  exposure_bias: string | null;
  captured_at: string | null;
}

interface DecodeResponse {
  seq: number;
  bitmap?: ImageBitmap;
  error?: string;
}

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;
const metaEl = document.getElementById("meta") as HTMLDivElement;
const openEl = document.getElementById("open") as HTMLButtonElement;

const worker = new Worker(new URL("./worker.js", import.meta.url), {
  type: "module",
});

let files: string[] = [];
let index = 0;
// Incremented on every page turn; a response tagged with an older sequence
// belongs to a file that is no longer current and is dropped.
let seq = 0;
const orientations = new Map<number, number>();
let shown: { bitmap: ImageBitmap; orientation: number } | null = null;
// True while a `preview` invoke is outstanding. Keeps at most one request in
// flight; when it settles, if `index` moved on in the meantime, exactly one
// follow-up request is issued for the latest index.
let inFlight = false;
// The metadata of the current file, or null while it is still being read.
let meta: Metadata | null = null;
// The same one-in-flight, re-request-if-stale pattern as `inFlight`, so
// holding a paging key down does not queue up a read per file passed.
let metaInFlight = false;
// A transient line under the metadata: an error, or the opening hint.
let note: string | undefined = "Press \u201co\u201d or click \u201cOpen folder\u201d.";
// How far the current scan got, or null when nothing is scanning. Events
// carry the id of the scan that emitted them; only events whose id matches
// `scanId` are applied, so the stragglers of a cancelled scan (including one
// cancelled by reopening the very same folder) are ignored even though they
// carry the same `dir`.
let scanId: number | null = null;
let scanning: string | null = null;
// Reserved by the picker before its dialog opens, and minted by a drop only
// once its dropped path has resolved (see `newFolderToken` and `dropCounter`
// below). When two folder opens race, the `list_arw` result of the one whose
// token is no longer current (e.g. A resolves after B was opened) is dropped
// instead of overwriting `files` with a stale folder's contents.
let folderToken = 0;
// The indexed rows of the open folder, keyed by the path `list_arw` returned.
// Fills in as the scan progresses; the focus box needs nothing else from it.
const entries = new Map<string, IndexedFile>();
// The folder the entries belong to, so `scan-progress` can ask for them again.
let openDir: string | null = null;
// True while a `folder_entries` invoke is outstanding. Keeps at most one
// request in flight, so a 10/s `scan-progress` stream while the user is
// paged ahead of the scan does not queue up a full re-read on every tick,
// and two overlapping reads cannot land out of order and clobber `entries`
// with a stale snapshot.
let entriesInFlight = false;
// True when a refresh was requested while one was already in flight. Re-run
// once the in-flight one settles, so the authoritative `scan-done` refresh
// is never silently dropped just because a `scan-progress` refresh happened
// to be outstanding at that moment.
let entriesPending = false;
let showFocus = true;

// Side of the focus box as a fraction of the image's short side, so it reads
// the same on portrait and landscape.
const FOCUS_BOX_FRACTION = 0.05;

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

function row(list: HTMLDListElement, label: string, value: string | null): void {
  if (value === null) {
    return;
  }
  const dt = document.createElement("dt");
  dt.textContent = label;
  const dd = document.createElement("dd");
  dd.textContent = value;
  list.append(dt, dd);
}

function line(className: string, text: string): HTMLDivElement {
  const el = document.createElement("div");
  el.className = className;
  el.textContent = text;
  return el;
}

// Redraw the right pane: the current file's name, its position in the
// folder, its shooting settings, and any note (an error, the scan's
// progress, the opening hint).
function renderMeta(): void {
  metaEl.replaceChildren();
  if (files.length > 0) {
    metaEl.append(line("name", meta?.name ?? baseName(files[index])));
    metaEl.append(line("position", `${index + 1} / ${files.length}`));
    if (meta !== null) {
      const list = document.createElement("dl");
      row(list, "Aperture", meta.aperture);
      row(list, "Shutter", meta.shutter);
      row(list, "ISO", meta.iso);
      row(list, "Focal length", meta.focal_length);
      row(list, "Exposure", meta.exposure_bias);
      row(list, "Camera", meta.camera);
      row(list, "Lens", meta.lens);
      row(list, "Captured", meta.captured_at);
      metaEl.append(list);
    }
  }
  if (note !== undefined) {
    metaEl.append(line("note", note));
  }
  if (scanning !== null) {
    metaEl.append(line("note", scanning));
  }
}

// Set the transient note, or clear it when called with no argument.
function setStatus(extra?: string): void {
  note = extra;
  renderMeta();
}

function draw(): void {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  const dpr = window.devicePixelRatio;
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  context.clearRect(0, 0, canvas.width, canvas.height);
  if (shown === null) {
    return;
  }
  const { bitmap, orientation } = shown;
  const quarterTurn = orientation === 6 || orientation === 8;
  const uprightWidth = quarterTurn ? bitmap.height : bitmap.width;
  const uprightHeight = quarterTurn ? bitmap.width : bitmap.height;
  const scale = Math.min(width / uprightWidth, height / uprightHeight);
  const drawWidth = bitmap.width * scale;
  const drawHeight = bitmap.height * scale;
  context.save();
  context.translate(canvas.width / 2, canvas.height / 2);
  context.scale(dpr, dpr);
  if (orientation === 6) {
    context.rotate(Math.PI / 2);
  } else if (orientation === 8) {
    context.rotate(-Math.PI / 2);
  } else if (orientation === 3) {
    context.rotate(Math.PI);
  }
  context.drawImage(
    bitmap,
    -drawWidth / 2,
    -drawHeight / 2,
    drawWidth,
    drawHeight,
  );
  drawFocusBox(drawWidth, drawHeight);
  context.restore();
}

function refreshEntries(): void {
  if (openDir === null) {
    return;
  }
  if (entriesInFlight) {
    entriesPending = true;
    return;
  }
  const dir = openDir;
  entriesInFlight = true;
  void window.__TAURI__.core
    .invoke<IndexedFile[]>("folder_entries", { dir })
    .then((rows) => {
      entriesInFlight = false;
      if (entriesPending) {
        entriesPending = false;
        refreshEntries();
      }
      if (dir !== openDir) {
        return;
      }
      entries.clear();
      for (const row of rows) {
        entries.set(row.path, row);
      }
      draw();
    })
    .catch(() => {
      entriesInFlight = false;
      if (entriesPending) {
        entriesPending = false;
        refreshEntries();
      }
      // A folder with no index cache simply has no focus boxes.
    });
}

// FocusLocation is in unrotated sensor coordinates, so the point is scaled
// onto the unrotated preview and drawn inside the same transform the image
// got; drawing it after the rotation would put it on the wrong edge. Mirrors
// the arithmetic in `riffle-cli focusbox`.
function drawFocusBox(drawWidth: number, drawHeight: number): void {
  if (!showFocus || files.length === 0) {
    return;
  }
  const focus = entries.get(files[index])?.focus;
  if (focus === undefined || focus === null) {
    return;
  }
  const x = -drawWidth / 2 + (focus.x * drawWidth) / focus.sensor_w;
  const y = -drawHeight / 2 + (focus.y * drawHeight) / focus.sensor_h;
  const side = Math.min(drawWidth, drawHeight) * FOCUS_BOX_FRACTION;
  context.strokeStyle = "rgba(0, 0, 0, 0.8)";
  context.lineWidth = 6;
  context.strokeRect(x - side / 2, y - side / 2, side, side);
  context.strokeStyle = "rgba(255, 255, 255, 0.95)";
  context.lineWidth = 2;
  context.strokeRect(x - side / 2, y - side / 2, side, side);
}

function requestPreview(): void {
  if (inFlight || files.length === 0) {
    return;
  }
  inFlight = true;
  const current = seq;
  window.__TAURI__.core
    .invoke<ArrayBuffer>("preview", { path: files[index] })
    .then((payload) => {
      inFlight = false;
      if (current !== seq) {
        requestPreview();
        return;
      }
      const header = new DataView(payload, 0, PREVIEW_HEADER_LEN);
      const kind = header.getUint16(0, true);
      if (kind !== PREVIEW_KIND_JPEG_V1) {
        throw new Error(`unknown preview payload kind ${kind}`);
      }
      orientations.set(current, header.getUint16(2, true));
      const jpeg = payload.slice(PREVIEW_HEADER_LEN);
      worker.postMessage({ seq: current, jpeg }, [jpeg]);
    })
    .catch((err: unknown) => {
      inFlight = false;
      if (current !== seq) {
        requestPreview();
        return;
      }
      setStatus(String(err));
    });
}

function requestMetadata(): void {
  if (metaInFlight || files.length === 0) {
    return;
  }
  metaInFlight = true;
  const current = seq;
  window.__TAURI__.core
    .invoke<Metadata>("metadata", { path: files[index] })
    .then((found) => {
      metaInFlight = false;
      if (current !== seq) {
        requestMetadata();
        return;
      }
      meta = found;
      renderMeta();
    })
    .catch(() => {
      metaInFlight = false;
      if (current !== seq) {
        requestMetadata();
        return;
      }
      // A file whose metadata cannot be read keeps its name and position;
      // the preview request reports the error itself.
      renderMeta();
    });
}

function show(): void {
  seq += 1;
  meta = null;
  strip.setCurrent(index);
  setStatus();
  requestPreview();
  requestMetadata();
}

worker.addEventListener("message", (event: MessageEvent<DecodeResponse>) => {
  const { seq: responseSeq, bitmap, error } = event.data;
  const orientation = orientations.get(responseSeq) ?? 1;
  orientations.delete(responseSeq);
  if (responseSeq !== seq) {
    bitmap?.close();
    return;
  }
  if (bitmap === undefined) {
    setStatus(error ?? "decode failed");
    return;
  }
  shown?.bitmap.close();
  shown = { bitmap, orientation };
  setStatus();
  draw();
});

function move(delta: number): void {
  if (files.length === 0) {
    return;
  }
  const next = Math.min(Math.max(index + delta, 0), files.length - 1);
  if (next === index) {
    return;
  }
  index = next;
  show();
}

strip.init((selected) => {
  if (selected === index) {
    return;
  }
  index = selected;
  show();
});

// Reserve the right to be the folder the UI shows. The picker reserves its
// token before its dialog opens; a drop mints its token only after
// `dropped_folder` resolves, so an ignored drop cannot cancel an open picker
// dialog. That asymmetry leaves drops unordered among themselves, which
// `dropCounter` (below) orders separately. Either way, the async work of the
// side that lost the race is dropped instead of writing into the other's UI.
function newFolderToken(): number {
  folderToken += 1;
  return folderToken;
}

function openDirectory(folder: string, token: number): Promise<void> {
  return window.__TAURI__.core
    .invoke<string[]>("list_arw", { dir: folder })
    .then((found) => {
      if (token !== folderToken) {
        return;
      }
      files = found;
      index = 0;
      openDir = folder;
      entries.clear();
      refreshEntries();
      strip.setFiles(files);
      seq += 1;
      shown?.bitmap.close();
      shown = null;
      draw();
      scanning = null;
      scanId = null;
      void window.__TAURI__.core
        .invoke<{ total: number; scan_id: number }>("scan_folder", {
          dir: folder,
        })
        .then(({ scan_id }) => {
          if (token !== folderToken) {
            return;
          }
          scanId = scan_id;
          return window.__TAURI__.core.invoke("start_scan", {
            scanId: scan_id,
          });
        })
        .catch((err: unknown) => {
          setStatus(String(err));
        });
      if (files.length === 0) {
        meta = null;
        setStatus("No ARW files in that folder.");
        return;
      }
      show();
    });
}

function openFolder(): void {
  const token = newFolderToken();
  window.__TAURI__.core
    .invoke<string | null>("pick_folder")
    .then((folder) => {
      if (folder === null || token !== folderToken) {
        return;
      }
      return openDirectory(folder, token);
    })
    .catch((err: unknown) => {
      setStatus(String(err));
    });
}

// Tauri intercepts HTML5 drag-and-drop, so a DOM `drop` event never carries a
// usable path; the paths arrive only through these webview events.
function setDragging(dragging: boolean): void {
  document.body.classList.toggle("dragging", dragging);
}

for (const event of ["tauri://drag-enter", "tauri://drag-over"]) {
  void window.__TAURI__.event.listen(event, () => {
    setDragging(true);
  });
}

void window.__TAURI__.event.listen("tauri://drag-leave", () => {
  setDragging(false);
});

// Orders drops among themselves: a later drop's `dropped_folder` call may
// resolve before an earlier one's, so each handler checks it is still the
// most recent drop before minting a folder token (which would otherwise let
// a stale, slow-resolving drop overwrite a newer one).
let dropCounter = 0;

void window.__TAURI__.event.listen<{ paths: string[] }>(
  "tauri://drag-drop",
  ({ payload }) => {
    setDragging(false);
    const [path] = payload.paths;
    if (path === undefined || payload.paths.length > 1) {
      setStatus("Drop a single folder or ARW file.");
      return;
    }
    const drop = ++dropCounter;
    window.__TAURI__.core
      .invoke<string | null>("dropped_folder", { path })
      .then((folder) => {
        if (drop !== dropCounter) {
          return;
        }
        if (folder === null) {
          setStatus("Drop a single folder or ARW file.");
          return;
        }
        return openDirectory(folder, newFolderToken());
      })
      .catch((err: unknown) => {
        setStatus(String(err));
      });
  },
);

void window.__TAURI__.event.listen<{
  dir: string;
  scan_id: number;
  done: number;
  total: number;
}>("scan-progress", ({ payload }) => {
  if (payload.scan_id !== scanId) {
    return;
  }
  scanning = `scanning ${payload.done} / ${payload.total}`;
  renderMeta();
  strip.refresh();
  // Only when the row the focus box needs is still missing; `entriesInFlight`
  // in `refreshEntries` keeps a 10/s progress stream from queuing up a
  // full re-read on every tick while it stays missing.
  if (files.length > 0 && !entries.has(files[index])) {
    refreshEntries();
  }
});

void window.__TAURI__.event.listen<{
  dir: string;
  scan_id: number;
  total: number;
  errors: number;
}>("scan-done", ({ payload }) => {
  if (payload.scan_id !== scanId) {
    return;
  }
  scanning = payload.errors === 0 ? null : `${payload.errors} failed`;
  renderMeta();
  strip.refresh();
  refreshEntries();
});

openEl.addEventListener("click", openFolder);

// Letter keys are matched lower-cased, so Shift+J pages like j does.
const pagingKeys = new Map<string, number>([
  ["arrowright", 1],
  ["arrowdown", 1],
  ["s", 1],
  ["d", 1],
  ["j", 1],
  ["l", 1],
  ["arrowleft", -1],
  ["arrowup", -1],
  ["w", -1],
  ["a", -1],
  ["h", -1],
  ["k", -1],
]);

window.addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey) {
    return;
  }
  const key = event.key.toLowerCase();
  const delta = pagingKeys.get(key);
  if (delta !== undefined) {
    move(delta);
  } else if (key === "f") {
    showFocus = !showFocus;
    draw();
  } else if (key === "o") {
    openFolder();
  } else {
    return;
  }
  event.preventDefault();
});

window.addEventListener("resize", draw);

renderMeta();
draw();

export {};
