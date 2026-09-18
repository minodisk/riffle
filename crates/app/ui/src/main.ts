import * as strip from "./strip.js";

// Header layout of a `preview` payload, see `crates/app/src/commands.rs`.
const PREVIEW_HEADER_LEN = 8;
const PREVIEW_KIND_JPEG_V1 = 1;
// Header layout of a `focus_crop` payload, see `crates/app/src/commands.rs`.
const CROP_HEADER_LEN = 24;
const CROP_KIND_RGBA_V1 = 3;
// Turned on by the Debug menu's `Timing logs` item. That menu only exists in a
// development build, so elsewhere the event never fires and this stays off.
let debugLogging = false;

function debugLog(...args: unknown[]): void {
  if (debugLogging) {
    console.debug(...args);
  }
}

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
  rating: number | null;
  has_sidecar: boolean;
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
const positionEl = document.getElementById("position") as HTMLDivElement;

const worker = new Worker(new URL("./worker.js", import.meta.url), {
  type: "module",
});

let files: string[] = [];
let index = 0;
// Incremented on every page turn; a response tagged with an older sequence
// belongs to a file that is no longer current and is dropped.
let seq = 0;
const orientations = new Map<number, number>();
let shown: { bitmap: ImageBitmap; orientation: number; seq: number } | null = null;
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
// Fills in as the scan progresses; the focus mark needs nothing else from it.
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
// The judgement of every file that has one: `-1` is a reject, `1`-`5` stars,
// and a missing entry is unrated. Filled from `folder_entries` and then owned
// by the keyboard until the next folder open.
const ratings = new Map<string, number>();
// The paths judged through the keyboard in this session, so a refresh from
// `folder_entries` (which may predate the pending sidecar write) does not
// undo what the user just pressed.
const touched = new Set<string>();
// The paths known to have a sidecar: from `folder_entries` for untouched
// files, and set at once by a rating key (the writer creates one). Never
// cleared by the app except on a folder open.
const sidecars = new Set<string>();
// The index of each path in `files`, for handing a rating to the strip.
const fileIndex = new Map<string, number>();
let showFocus = false;
// True while the 1:1 focus check is showing instead of the fitted preview.
let zoomed = false;
// The crop of the file that `cropSeq` identifies, at one JPEG pixel per
// device pixel, with its point of interest in crop pixels. Kept while the
// view is toggled off so toggling back on redraws without a round trip.
let crop: {
  bitmap: ImageBitmap;
  pointX: number;
  pointY: number;
  cropSeq: number;
  orientation: number;
  // The device-pixel viewport size (`canvas.client*` × dpr) at request time,
  // so a resize that lands while a request is already in flight (silently
  // dropped, since `requestCrop` no-ops on `cropInFlight`) is still noticed
  // once this crop settles and can be re-requested.
  requestedWidth: number;
  requestedHeight: number;
} | null = null;
// The same one-in-flight, re-request-if-stale pattern as `inFlight`.
let cropInFlight = false;
// `performance.now()` at the keypress that asked for the crop, for the
// `debugLog` marks.
let zoomStartedAt = 0;

// The crosshair's arm length and the gap left open around the point itself,
// in CSS pixels. The mark only points, so it keeps one size like a cursor
// instead of growing over the subject with the window. The gap keeps the
// lines off the focus point, which is the one pixel the mark exists to show.
const FOCUS_MARK_ARM = 8;
const FOCUS_MARK_GAP = 4;

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

// Mirrors `riffle_core::xmp::sidecar_path`: the name the app would write. A
// case variant already on disk (`FOO.XMP`) is not known here.
function sidecarName(path: string): string {
  return baseName(path).replace(/\.[^.]*$/, "") + ".xmp";
}

// The reject mark and stars, the same text as the strip cell's badge in
// `strip.ts` (`paintRating`).
function ratingText(rating: number): string {
  return rating === -1 ? "\u2715" : "\u2605".repeat(rating);
}

function row(
  list: HTMLDListElement,
  label: string,
  value: string | null,
  className?: string,
): void {
  if (value === null) {
    return;
  }
  const dt = document.createElement("dt");
  dt.textContent = label;
  const dd = document.createElement("dd");
  dd.textContent = value;
  if (className !== undefined) {
    dd.className = className;
  }
  list.append(dt, dd);
}

function line(className: string, text: string): HTMLDivElement {
  const el = document.createElement("div");
  el.className = className;
  el.textContent = text;
  return el;
}

// Redraw the right pane: the current file's name, its shooting settings,
// any note (an error, the scan's progress, the opening hint), and last its
// sidecar section. Also refreshes the strip pane's `N / M` counter.
function renderMeta(): void {
  positionEl.textContent =
    files.length > 0 ? `${index + 1} / ${files.length}` : "";
  metaEl.replaceChildren();
  if (files.length > 0) {
    metaEl.append(line("name", meta?.name ?? baseName(files[index])));
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
  // Driven by `zoomed` rather than `note`, so paging or an error does not
  // erase the mode indicator while the 1:1 view is still showing.
  if (zoomed) {
    metaEl.append(line("note", "1:1"));
  }
  if (files.length > 0) {
    metaEl.append(sidecarSection(files[index]));
  }
}

// What the XMP sidecar holds, apart from the EXIF rows above (which the app
// never writes). A file whose entry has not arrived yet shows the name with
// no note, since the app does not know yet whether a sidecar exists.
function sidecarSection(path: string): HTMLElement {
  const section = document.createElement("section");
  section.className = "sidecar";
  const known = entries.has(path) || touched.has(path);
  const header = line("header", sidecarName(path));
  if (known && !sidecars.has(path)) {
    header.append(line("note", "(not created)"));
  }
  const list = document.createElement("dl");
  const rating = ratings.get(path);
  row(
    list,
    "Rating",
    rating === undefined ? "\u2013" : ratingText(rating),
    rating === undefined ? undefined : rating === -1 ? "rejected" : "stars",
  );
  section.append(header, list);
  return section;
}

// Set the transient note, or clear it when called with no argument.
function setStatus(extra?: string): void {
  note = extra;
  renderMeta();
}

function draw(): void {
  if (zoomed) {
    drawZoom();
    return;
  }
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
  drawFocusMark(drawWidth, drawHeight);
  context.restore();
}

// Record a judgement locally: the `ratings` map and the strip cell. `null` is
// unrated.
function applyRating(path: string, rating: number | null): void {
  if (rating === null) {
    ratings.delete(path);
  } else {
    ratings.set(path, rating);
  }
  const at = fileIndex.get(path);
  if (at !== undefined) {
    strip.setRating(at, rating);
  }
}

// A rating key: update the map and redraw first, then tell the backend. The
// invoke is never awaited for anything visible; only its failure is, which
// reverts the entry (if the folder is still the one it belongs to) and says
// so in the status line.
function rate(rating: number | null): void {
  if (files.length === 0) {
    return;
  }
  const path = files[index];
  const previous = ratings.get(path) ?? null;
  // Idempotent: pressing the current value again does nothing at all, which
  // is what makes key auto-repeat harmless.
  if (previous === rating) {
    return;
  }
  touched.add(path);
  applyRating(path, rating);
  if (rating !== null) {
    sidecars.add(path);
  }
  renderMeta();
  const token = folderToken;
  void window.__TAURI__.core
    .invoke("set_rating", { path, rating: rating ?? 0 })
    .catch((err: unknown) => {
      if (token !== folderToken) {
        return;
      }
      touched.delete(path);
      applyRating(path, previous);
      setStatus(String(err));
    });
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
  const token = folderToken;
  entriesInFlight = true;
  void window.__TAURI__.core
    .invoke<IndexedFile[]>("folder_entries", { dir })
    .then((rows) => {
      entriesInFlight = false;
      if (entriesPending) {
        entriesPending = false;
        refreshEntries();
      }
      if (dir !== openDir || token !== folderToken) {
        return;
      }
      entries.clear();
      for (const row of rows) {
        entries.set(row.path, row);
        if (!touched.has(row.path)) {
          applyRating(row.path, row.rating);
          if (row.has_sidecar) {
            sidecars.add(row.path);
          } else {
            sidecars.delete(row.path);
          }
        }
      }
      renderMeta();
      draw();
    })
    .catch(() => {
      entriesInFlight = false;
      if (entriesPending) {
        entriesPending = false;
        refreshEntries();
      }
      // A folder with no index cache simply has no focus marks.
    });
}

// FocusLocation is in unrotated sensor coordinates, so the point is scaled
// onto the unrotated preview and drawn inside the same transform the image
// got; drawing it after the rotation would put it on the wrong edge. Mirrors
// the arithmetic in `riffle-cli focusbox`.
//
// The tag records a point, not an area — there is no AF rectangle in it — so
// the mark is a crosshair pointing at that point rather than a box implying a
// size the file never stated.
function drawFocusMark(drawWidth: number, drawHeight: number): void {
  if (!showFocus || files.length === 0) {
    return;
  }
  const focus = entries.get(files[index])?.focus;
  if (focus === undefined || focus === null) {
    return;
  }
  const x = -drawWidth / 2 + (focus.x * drawWidth) / focus.sensor_w;
  const y = -drawHeight / 2 + (focus.y * drawHeight) / focus.sensor_h;
  const arm = FOCUS_MARK_ARM;
  const gap = FOCUS_MARK_GAP;
  // A fixed colour over a dark outline: the colour carries the mark on most
  // photos, and the outline still draws its edge where the subject shares the
  // colour. The same path is stroked twice, the outline first and wider, and
  // square caps give the arm ends the same 1px outline as their sides.
  context.save();
  context.lineCap = "square";
  context.beginPath();
  context.moveTo(x - gap - arm, y);
  context.lineTo(x - gap, y);
  context.moveTo(x + gap, y);
  context.lineTo(x + gap + arm, y);
  context.moveTo(x, y - gap - arm);
  context.lineTo(x, y - gap);
  context.moveTo(x, y + gap);
  context.lineTo(x, y + gap + arm);
  context.strokeStyle = "rgba(0, 0, 0, 0.8)";
  context.lineWidth = 4;
  context.stroke();
  context.strokeStyle = "#3f3";
  context.lineWidth = 2;
  context.stroke();
  context.restore();
}

// The 1:1 view: the same rotation `draw()` applies, with the focus point at
// the canvas centre. Everything here is in device pixels, so the `scale(dpr,
// dpr)` of `draw()` is deliberately not applied. The crop is cut in unrotated
// JPEG coordinates, exactly like the focus mark, so the rotation carries it.
function drawZoom(): void {
  const dpr = window.devicePixelRatio;
  canvas.width = Math.round(canvas.clientWidth * dpr);
  canvas.height = Math.round(canvas.clientHeight * dpr);
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.save();
  context.translate(canvas.width / 2, canvas.height / 2);
  // The placeholder (the scaled preview) is only drawn while `shown` belongs
  // to the current file; otherwise it would position the previous file's
  // bitmap with the new file's focus point.
  const placeholderShown = shown !== null && shown.seq === seq ? shown : null;
  const orientation =
    crop !== null && crop.cropSeq === seq ? crop.orientation : (placeholderShown?.orientation ?? 1);
  if (orientation === 6) {
    context.rotate(Math.PI / 2);
  } else if (orientation === 8) {
    context.rotate(-Math.PI / 2);
  } else if (orientation === 3) {
    context.rotate(Math.PI);
  }
  const focus = files.length > 0 ? entries.get(files[index])?.focus : undefined;
  if (placeholderShown !== null && focus !== undefined && focus !== null) {
    // The full JPEG is taken to be the sensor size, which is what the crop
    // was scaled onto; one preview pixel is then `scale` JPEG pixels.
    const scale = focus.sensor_w / placeholderShown.bitmap.width;
    const width = placeholderShown.bitmap.width * scale;
    const height = placeholderShown.bitmap.height * scale;
    const x = (focus.x * width) / focus.sensor_w;
    const y = (focus.y * height) / focus.sensor_h;
    context.drawImage(placeholderShown.bitmap, -x, -y, width, height);
  }
  if (crop !== null && crop.cropSeq === seq) {
    context.drawImage(crop.bitmap, -crop.pointX, -crop.pointY);
  }
  context.restore();
}

// True when the held crop was cut for a viewport size that no longer
// matches the canvas (a resize while zoomed).
function cropViewportStale(): boolean {
  if (crop === null) {
    return false;
  }
  const dpr = window.devicePixelRatio;
  return (
    crop.requestedWidth !== Math.round(canvas.clientWidth * dpr) ||
    crop.requestedHeight !== Math.round(canvas.clientHeight * dpr)
  );
}

function requestCrop(): void {
  if (cropInFlight || files.length === 0) {
    return;
  }
  cropInFlight = true;
  const current = seq;
  const dpr = window.devicePixelRatio;
  const requestedWidth = Math.round(canvas.clientWidth * dpr);
  const requestedHeight = Math.round(canvas.clientHeight * dpr);
  window.__TAURI__.core
    .invoke<ArrayBuffer>("focus_crop", {
      path: files[index],
      width: requestedWidth,
      height: requestedHeight,
    })
    .then(async (payload) => {
      cropInFlight = false;
      debugLog("zoom invoke", performance.now() - zoomStartedAt);
      if (current !== seq) {
        if (zoomed) {
          requestCrop();
        }
        return;
      }
      const header = new DataView(payload, 0, CROP_HEADER_LEN);
      const kind = header.getUint16(0, true);
      if (kind !== CROP_KIND_RGBA_V1) {
        throw new Error(`unknown crop payload kind ${kind}`);
      }
      const orientation = header.getUint16(2, true);
      const width = header.getUint32(4, true);
      const height = header.getUint32(8, true);
      // `putImageData` ignores the rotation transform, so the pixels go
      // through a bitmap; no JPEG is involved, so no worker is needed.
      const pixels = new ImageData(
        new Uint8ClampedArray(payload, CROP_HEADER_LEN),
        width,
        height,
      );
      const bitmap = await createImageBitmap(pixels);
      debugLog("zoom bitmap", performance.now() - zoomStartedAt);
      if (current !== seq) {
        bitmap.close();
        if (zoomed) {
          requestCrop();
        }
        return;
      }
      crop?.bitmap.close();
      crop = {
        bitmap,
        pointX: header.getUint32(12, true),
        pointY: header.getUint32(16, true),
        cropSeq: current,
        orientation,
        requestedWidth,
        requestedHeight,
      };
      draw();
      // The viewport may have been resized while this request was in
      // flight; `requestCrop` silently no-ops on `cropInFlight` during that
      // window, so check here and re-fire if the settled crop is stale.
      if (zoomed && cropViewportStale()) {
        requestCrop();
      }
    })
    .catch((err: unknown) => {
      cropInFlight = false;
      if (current !== seq) {
        if (zoomed) {
          requestCrop();
        }
        return;
      }
      setStatus(String(err));
    });
}

// `Space` toggles the 1:1 view. The crop already held for this file is
// reused; otherwise one is requested and the scaled preview stands in.
function toggleZoom(): void {
  if (files.length === 0) {
    return;
  }
  zoomed = !zoomed;
  if (zoomed) {
    zoomStartedAt = performance.now();
    debugLog("zoom keypress", zoomStartedAt);
    if (crop === null || crop.cropSeq !== seq || cropViewportStale()) {
      requestCrop();
    }
  }
  renderMeta();
  draw();
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
  if (zoomed) {
    requestCrop();
  }
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
  shown = { bitmap, orientation, seq: responseSeq };
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

// Reopen the folder the last session had open, if it still exists.
function reopenLastFolder(): void {
  const token = newFolderToken();
  window.__TAURI__.core
    .invoke<string | null>("last_folder")
    .then((folder) => {
      if (folder === null || token !== folderToken) {
        return;
      }
      return openDirectory(folder, token);
    })
    .catch(() => {
      // The folder went away after the check; keep the opening hint.
    });
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
      void window.__TAURI__.core.invoke("remember_folder", { dir: folder });
      entries.clear();
      ratings.clear();
      touched.clear();
      sidecars.clear();
      fileIndex.clear();
      files.forEach((path, at) => {
        fileIndex.set(path, at);
      });
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
  // Only when the row the focus mark needs is still missing; `entriesInFlight`
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

// The Debug menu's `Timing logs` item; absent from a distributable build, so
// the listener simply never fires there.
void window.__TAURI__.event.listen<boolean>("debug", ({ payload }) => {
  debugLogging = payload;
});

// A sidecar the writer could not write: the judgement is still in the index
// and is retried on the next open of the folder, so this is a note, not a
// revert.
void window.__TAURI__.event.listen<{ path: string; message: string }>(
  "sidecar-error",
  ({ payload }) => {
    if (!fileIndex.has(payload.path)) {
      return;
    }
    setStatus(`${baseName(payload.path)}: ${payload.message}`);
  },
);

openEl.addEventListener("click", openFolder);
reopenLastFolder();

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
  } else if (key === " ") {
    toggleZoom();
  } else if (key === "o") {
    openFolder();
  } else if (key >= "1" && key <= "5") {
    rate(Number(key));
  } else if (key === "x") {
    // Sticky, not a toggle: `x` twice is still a reject. `u` undoes it.
    rate(-1);
  } else if (key === "u") {
    if (files.length === 0 || (ratings.get(files[index]) ?? null) !== -1) {
      return;
    }
    rate(null);
  } else if (key === "0") {
    rate(null);
  } else {
    return;
  }
  event.preventDefault();
});

window.addEventListener("resize", () => {
  draw();
  if (zoomed) {
    requestCrop();
  }
});

renderMeta();
draw();

export {};
