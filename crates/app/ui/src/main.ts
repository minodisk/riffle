import { type Binding, isUnboundModifier, keyName } from "./keys.js";
import * as strip from "./strip.js";

// Header layout of a `preview` payload, see `crates/app/src/commands.rs`.
const PREVIEW_HEADER_LEN = 8;
const PREVIEW_KIND_JPEG_V1 = 1;
// Header layout of a `focus_crop` payload, see `crates/app/src/commands.rs`.
const CROP_HEADER_LEN = 32;
const CROP_KIND_RGBA_V2 = 4;
// Turned on by the settings window's `Timing logs` item through the `debug`
// event. That item only shows in a development build, so elsewhere this stays
// off.
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
  pick: boolean;
  label: string | null;
  has_sidecar: boolean;
  exif: Exif | null;
}

// Mirrors `Exif` in `crates/app/src/exif.rs`: `value` sorts, `label` shows.
interface Labelled {
  value: number;
  label: string;
}

interface Exif {
  camera: string | null;
  lens: string | null;
  aperture: Labelled | null;
  shutter: Labelled | null;
  iso: Labelled | null;
  focal_length: Labelled | null;
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
  focus_distance: string | null;
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

// Every RAW file in the open folder, in `list_arw` order.
let allFiles: string[] = [];
// The files that pass the filter, in the same order. `index`, the strip and
// paging all work on this view.
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
// The metadata last read. While the current file's read is outstanding it
// still holds the previous file's (`metaStale`), so the rows keep their place
// instead of collapsing and reappearing on every step while paging quickly.
let meta: Metadata | null = null;
let metaStale = false;
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
// The picked paths, owned the same way as `ratings`. A pick is PhotoLab's
// flag and coexists with stars; it exists only while `.dop` is selected.
const picks = new Set<string>();
// The colour label of every file that has one, owned the same way as
// `ratings`.
const labels = new Map<string, string>();
// The selected sidecar format (`"xmp"` or `"dop"`), from `sidecar_format` at
// launch and the `sidecar-format` event after a switch.
let sidecarFormat = "xmp";
// The paths judged through the keyboard in this session, so a refresh from
// `folder_entries` (which may predate the pending sidecar write) does not
// undo what the user just pressed.
const touched = new Set<string>();
// The index of each path in `files`, for handing a rating to the strip.
const fileIndex = new Map<string, number>();
// The filter menu in the strip pane, after PhotoLab's: the checked items of
// one group are OR-ed, the groups AND-ed, and a group with nothing checked
// lets everything through. `0` stars is unrated, which a reject also counts
// as, since it carries no stars here.
type Flag = "picked" | "untagged" | "rejected";
const shownFlags = new Set<Flag>();
const shownStars = new Set<number>();
// The EXIF groups, keyed by label (two estimated apertures with one label can
// differ in value). Focal length is keyed by the range's label instead.
type ExifGroup = "camera" | "lens" | "aperture" | "shutter" | "iso" | "focal";
const exifGroups: { group: ExifGroup; heading: string }[] = [
  { group: "camera", heading: "Camera" },
  { group: "lens", heading: "Lens" },
  { group: "aperture", heading: "Aperture" },
  { group: "shutter", heading: "Shutter speed" },
  { group: "iso", heading: "ISO" },
  { group: "focal", heading: "Focal length" },
];
const shownExif = new Map<ExifGroup, Set<string>>(
  exifGroups.map(({ group }) => [group, new Set<string>()]),
);
// Half-open `[lower, upper)`, so a frame falls in exactly one.
const focalRanges: { upper: number; label: string }[] = [
  { upper: 24, label: "<24 mm" },
  { upper: 35, label: "24–35 mm" },
  { upper: 50, label: "35–50 mm" },
  { upper: 85, label: "50–85 mm" },
  { upper: 135, label: "85–135 mm" },
  { upper: 200, label: "135–200 mm" },
  { upper: Infinity, label: ">200 mm" },
];

function focalRange(mm: number): number {
  return focalRanges.findIndex(({ upper }) => mm < upper);
}

// The label and sort key of `group` for one file, or null if it has none.
function exifKey(exif: Exif | null | undefined, group: ExifGroup): { label: string; order: number | string } | null {
  if (!exif) {
    return null;
  }
  switch (group) {
    case "camera":
    case "lens": {
      const text = exif[group];
      return text === null ? null : { label: text, order: text };
    }
    case "focal": {
      if (exif.focal_length === null) {
        return null;
      }
      const at = focalRange(exif.focal_length.value);
      return { label: focalRanges[at].label, order: at };
    }
    default: {
      const labelled = exif[group];
      return labelled === null ? null : { label: labelled.label, order: labelled.value };
    }
  }
}
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
// `performance.now()` at the `Space` that asked for a crop, cleared by the
// crop that keypress produced. Only that one crop can be timed from the
// keypress; a resize or a page turn starts its own request.
let zoomKeypressAt: number | null = null;
// A resize fires continuously while the window edge is dragged, and each
// crop costs a partial decode plus a multi-megabyte IPC payload, so the
// re-request waits for the drag to settle. 150ms is long enough to swallow a
// drag's stream of events and short enough to feel immediate once released;
// until it fires, the held crop keeps being drawn into the new viewport.
const CROP_RESIZE_DEBOUNCE_MS = 150;
let cropResizeTimer: number | null = null;

// Re-requests the crop once the viewport has stopped changing. The `seq` of
// the file the resize was seen for is captured here rather than re-read in
// the callback, so a timer that outlives a page turn or a folder change is
// dropped instead of applied.
function scheduleCropForResize(): void {
  const current = seq;
  if (cropResizeTimer !== null) {
    window.clearTimeout(cropResizeTimer);
  }
  cropResizeTimer = window.setTimeout(() => {
    cropResizeTimer = null;
    if (!zoomed || current !== seq) {
      return;
    }
    if (crop !== null && !cropViewportStale()) {
      return;
    }
    requestCrop();
  }, CROP_RESIZE_DEBOUNCE_MS);
}

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

// Redraw the right pane: the current file's name, its shooting settings,
// and any note (an error, the scan's progress, the opening hint). Also
// refreshes the strip pane's `N / M` counter.
function renderMeta(): void {
  positionEl.textContent =
    files.length > 0 ? `${index + 1} / ${files.length}` : "";
  metaEl.replaceChildren();
  if (files.length > 0) {
    metaEl.append(
      line("name", meta === null || metaStale ? baseName(files[index]) : meta.name),
    );
    if (meta !== null) {
      const list = document.createElement("dl");
      row(list, "Aperture", meta.aperture);
      row(list, "Shutter", meta.shutter);
      row(list, "ISO", meta.iso);
      row(list, "Focal length", meta.focal_length);
      row(list, "Exposure", meta.exposure_bias);
      row(list, "Focus distance", meta.focus_distance);
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
}

// Hand the open folder to DxO PhotoLab for developing.
function openInPhotoLab(): void {
  if (openDir === null) {
    return;
  }
  window.__TAURI__.core
    .invoke("open_in_photolab", { dir: openDir })
    .catch((e: unknown) => setStatus(`Could not open PhotoLab: ${String(e)}`));
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

// Record a judgement locally: the `ratings` map, the `picks` set and the
// strip cell. `null` is unrated.
function applyRating(path: string, rating: number | null, pick: boolean, label: string | null): void {
  if (rating === null) {
    ratings.delete(path);
  } else {
    ratings.set(path, rating);
  }
  if (pick) {
    picks.add(path);
  } else {
    picks.delete(path);
  }
  if (label === null) {
    labels.delete(path);
  } else {
    labels.set(path, label);
  }
  const at = fileIndex.get(path);
  if (at !== undefined) {
    strip.setRating(at, rating, pick, label);
  }
}

function passes(path: string): boolean {
  const rating = ratings.get(path);
  const flag: Flag = picks.has(path)
    ? "picked"
    : rating === -1
      ? "rejected"
      : "untagged";
  const stars = rating === undefined || rating === -1 ? 0 : rating;
  return (
    (shownFlags.size === 0 || shownFlags.has(flag)) &&
    (shownStars.size === 0 || shownStars.has(stars)) &&
    exifGroups.every(({ group }) => {
      const set = shownExif.get(group)!;
      if (set.size === 0) {
        return true;
      }
      const key = exifKey(entries.get(path)?.exif, group);
      return key !== null && set.has(key.label);
    })
  );
}

// Rebuild `files` from `allFiles` after a filter change or a judgement. The
// current file stays current if it still passes; otherwise the next passing
// file after it (in `allFiles` order) takes over, or the last one before it.
function refilter(anchor: string | undefined = files[index]): void {
  const next = allFiles.filter(passes);
  if (next.length === files.length && next.every((path, at) => path === files[at])) {
    return;
  }
  files = next;
  fileIndex.clear();
  files.forEach((path, at) => {
    fileIndex.set(path, at);
  });
  strip.setFiles(files);
  files.forEach((path, at) => {
    strip.setRating(at, ratings.get(path) ?? null, picks.has(path), labels.get(path) ?? null);
  });
  if (files.length === 0) {
    index = 0;
    seq += 1;
    shown?.bitmap.close();
    shown = null;
    meta = null;
    zoomed = false;
    draw();
    renderMeta();
    return;
  }
  let at = anchor === undefined ? undefined : fileIndex.get(anchor);
  if (at === undefined && anchor !== undefined) {
    const from = allFiles.indexOf(anchor);
    const after = allFiles.slice(from + 1).find(passes);
    const before = allFiles.slice(0, from).reverse().find(passes);
    const target = after ?? before;
    at = target === undefined ? undefined : fileIndex.get(target);
  }
  index = at ?? 0;
  if (files[index] === anchor) {
    strip.setCurrent(index);
    renderMeta();
  } else {
    show();
  }
}

// A judgement key, given the current file's judgement and returning the new
// one: update the map and redraw first, then tell the backend. The invoke is
// never awaited for anything visible; only its failure is, which reverts the
// entry (if the folder is still the one it belongs to) and says so in the
// status line.
function judge(
  next: (
    rating: number | null,
    pick: boolean,
    label: string | null,
  ) => [number | null, boolean, string | null],
): void {
  if (files.length === 0) {
    return;
  }
  const path = files[index];
  const previous = ratings.get(path) ?? null;
  const previousPick = picks.has(path);
  const previousLabel = labels.get(path) ?? null;
  const [rating, pick, label] = next(previous, previousPick, previousLabel);
  // Idempotent: pressing the current value again does nothing at all, which
  // is what makes key auto-repeat harmless.
  if (previous === rating && previousPick === pick && previousLabel === label) {
    return;
  }
  touched.add(path);
  applyRating(path, rating, pick, label);
  renderMeta();
  refilter(path);
  const token = folderToken;
  // `label` only carries a meaningful value once `folder_entries` has told us
  // this path's label; before that, `labelKnown: false` tells the backend to
  // keep whatever it already has instead of clearing it, unless this key
  // set the label itself.
  void window.__TAURI__.core
    .invoke("set_rating", {
      path,
      rating: rating ?? 0,
      pick,
      label,
      labelKnown: entries.has(path) || label !== previousLabel,
    })
    .catch((err: unknown) => {
      if (token !== folderToken) {
        return;
      }
      touched.delete(path);
      applyRating(path, previous, previousPick, previousLabel);
      refilter();
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
          applyRating(row.path, row.rating, row.pick, row.label);
        }
      }
      rebuildExifMenu();
      renderMeta();
      draw();
      refilter();
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
    zoomKeypressAt = null;
    return;
  }
  cropInFlight = true;
  const current = seq;
  // Every mark below is measured from this request, not from the last
  // keypress: a crop asked for by a resize or a page turn has nothing to do
  // with a `Space` pressed minutes ago.
  const requestStartedAt = performance.now();
  const keypressAt = zoomKeypressAt;
  zoomKeypressAt = null;
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
      const invokeAt = performance.now() - requestStartedAt;
      if (current !== seq) {
        if (zoomed) {
          requestCrop();
        }
        return;
      }
      const header = new DataView(payload, 0, CROP_HEADER_LEN);
      const kind = header.getUint16(0, true);
      if (kind !== CROP_KIND_RGBA_V2) {
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
      const bitmapAt = performance.now() - requestStartedAt;
      const readMs = header.getUint32(20, true) / 1000;
      const decodeMs = header.getUint32(24, true) / 1000;
      const sinceKeypress =
        keypressAt === null
          ? ""
          : ` keypressToPixels=${(performance.now() - keypressAt).toFixed(1)}ms`;
      debugLog(
        `zoom crop=${bitmapAt.toFixed(1)}ms` +
          ` read=${readMs.toFixed(1)}ms` +
          ` decode=${decodeMs.toFixed(1)}ms` +
          ` ipc=${(invokeAt - readMs - decodeMs).toFixed(1)}ms` +
          ` bitmap=${(bitmapAt - invokeAt).toFixed(1)}ms` +
          sinceKeypress,
      );
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
      // window, so check here and re-fire if the settled crop is stale. The
      // re-fire goes through the debounce too, or a drag would simply chain
      // one crop into the next.
      if (zoomed && cropViewportStale()) {
        scheduleCropForResize();
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
    if (crop === null || crop.cropSeq !== seq || cropViewportStale()) {
      zoomKeypressAt = performance.now();
      debugLog("zoom keypress", zoomKeypressAt);
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
      metaStale = false;
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
      meta = null;
      metaStale = false;
      renderMeta();
    });
}

function show(): void {
  seq += 1;
  metaStale = true;
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
      for (const set of shownExif.values()) {
        set.clear();
      }
      allFiles = found;
      files = found.filter(passes);
      index = 0;
      openDir = folder;
      void window.__TAURI__.core.invoke("remember_folder", { dir: folder });
      entries.clear();
      rebuildExifMenu();
      ratings.clear();
      picks.clear();
      labels.clear();
      touched.clear();
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
        setStatus(
          allFiles.length === 0 ? "No RAW (ARW/DNG) files in that folder." : undefined,
        );
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

void window.__TAURI__.event.listen("open-in-photolab", openInPhotoLab);

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
      setStatus("Drop a single folder or RAW file.");
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
          setStatus("Drop a single folder or RAW file.");
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

// A sidecar the writer could not write: the judgement is still in the index
// and is retried on the next open of the folder, so this is a note, not a
// revert.
void window.__TAURI__.event.listen<{ path: string; message: string }>(
  "sidecar-error",
  ({ payload }) => {
    if (!allFiles.includes(payload.path)) {
      return;
    }
    setStatus(`${baseName(payload.path)}: ${payload.message}`);
  },
);

// The settings window switched the format and the backend has reset the index:
// reopen the folder so the strip and the meta pane show the newly selected
// format's judgements. A fresh token drops any open still in flight.
void window.__TAURI__.event.listen<string>("sidecar-format", ({ payload }) => {
  sidecarFormat = payload;
  // The label keys' defaults follow the format.
  void window.__TAURI__.core.invoke<Binding[]>("shortcuts").then((bindings) => {
    applyKeymap(bindings);
  });
  if (openDir === null) {
    return;
  }
  openDirectory(openDir, newFolderToken()).catch((err: unknown) => {
    setStatus(String(err));
  });
});

void window.__TAURI__.core
  .invoke<string>("sidecar_format")
  .then((format) => {
    sidecarFormat = format;
  });

// The settings window's "Timing logs" item toggles this through the `debug`
// event; read the initial state too, so a reloaded main window stays in sync
// with the backend's `TimingLogs` state.
void window.__TAURI__.event.listen<boolean>("debug", ({ payload }) => {
  debugLogging = payload;
});
void window.__TAURI__.core.invoke<boolean>("timing_logs").then((enabled) => {
  debugLogging = enabled;
});

const filterToggle = document.getElementById("filter-toggle") as HTMLButtonElement;
const filterMenu = document.getElementById("filter-menu") as HTMLDivElement;
const filterItems = filterMenu.querySelectorAll<HTMLButtonElement>("[data-flag], [data-stars]");
const filterExif = document.getElementById("filter-exif") as HTMLDivElement;

function exifSelected(): boolean {
  return [...shownExif.values()].some((set) => set.size > 0);
}

// Rebuild the EXIF sections from `entries`: one item per label present in the
// folder. A checked label no longer present is dropped, so a stale selection
// cannot hide everything.
function rebuildExifMenu(): void {
  filterExif.replaceChildren();
  for (const { group, heading } of exifGroups) {
    const found = new Map<string, number | string>();
    for (const row of entries.values()) {
      const key = exifKey(row.exif, group);
      if (key !== null) {
        found.set(key.label, key.order);
      }
    }
    const set = shownExif.get(group)!;
    for (const label of set) {
      if (!found.has(label)) {
        set.delete(label);
      }
    }
    if (found.size === 0) {
      continue;
    }
    const sorted = [...found].sort(([a, x], [b, y]) =>
      typeof x === "number" && typeof y === "number"
        ? x - y || a.localeCompare(b)
        : String(x).localeCompare(String(y)),
    );
    const title = document.createElement("div");
    title.className = "heading";
    title.textContent = heading;
    filterExif.append(document.createElement("hr"), title);
    for (const [label] of sorted) {
      const item = document.createElement("button");
      item.type = "button";
      item.setAttribute("role", "menuitemcheckbox");
      item.setAttribute("aria-checked", String(set.has(label)));
      item.dataset.group = group;
      item.dataset.value = label;
      item.textContent = label;
      filterExif.append(item);
    }
  }
  filterToggle.classList.toggle(
    "active",
    shownFlags.size + shownStars.size > 0 || exifSelected(),
  );
}

function setFilterMenuOpen(open: boolean): void {
  filterMenu.hidden = !open;
  filterToggle.setAttribute("aria-expanded", String(open));
}

// Mirror the two sets onto the menu's check marks and the button's lit state,
// then rebuild the view.
function filterChanged(): void {
  for (const item of filterItems) {
    const { flag, stars } = item.dataset;
    const checked =
      flag !== undefined ? shownFlags.has(flag as Flag) : shownStars.has(Number(stars));
    item.setAttribute("aria-checked", String(checked));
  }
  for (const item of filterExif.querySelectorAll<HTMLButtonElement>("[data-group]")) {
    const { group, value } = item.dataset;
    item.setAttribute("aria-checked", String(shownExif.get(group as ExifGroup)!.has(value!)));
  }
  filterToggle.classList.toggle(
    "active",
    shownFlags.size + shownStars.size > 0 || exifSelected(),
  );
  refilter();
}

filterToggle.addEventListener("click", () => {
  // Drop the focus, or `Space` (the 1:1 toggle) would press it again.
  filterToggle.blur();
  setFilterMenuOpen(!!filterMenu.hidden);
});

// The menu stays open while items are toggled, so several can be checked in
// one go; a click anywhere else closes it.
document.addEventListener("mousedown", (event) => {
  if (!filterMenu.hidden && !(event.target as Element).closest("#filter")) {
    setFilterMenuOpen(false);
  }
});

for (const item of filterItems) {
  item.addEventListener("click", () => {
    item.blur();
    const { flag, stars } = item.dataset;
    const set: Set<string | number> =
      flag !== undefined ? shownFlags : shownStars;
    const value = flag ?? Number(stars);
    if (set.has(value)) {
      set.delete(value);
    } else {
      set.add(value);
    }
    filterChanged();
  });
}

filterExif.addEventListener("click", (event) => {
  const item = (event.target as Element).closest<HTMLButtonElement>("[data-group]");
  if (item === null) {
    return;
  }
  item.blur();
  const set = shownExif.get(item.dataset.group as ExifGroup)!;
  const value = item.dataset.value!;
  if (set.has(value)) {
    set.delete(value);
  } else {
    set.add(value);
  }
  filterChanged();
});

(document.getElementById("filter-reset") as HTMLButtonElement).addEventListener(
  "click",
  (event) => {
    (event.currentTarget as HTMLButtonElement).blur();
    shownFlags.clear();
    shownStars.clear();
    for (const set of shownExif.values()) {
      set.clear();
    }
    filterChanged();
  },
);

openEl.addEventListener("click", openFolder);
reopenLastFolder();

// Key -> action, from the `shortcuts` command. Empty until it resolves.
let keymap = new Map<string, string>();

function applyKeymap(bindings: Binding[]): void {
  keymap = new Map(bindings.flatMap(({ action, keys }) => keys.map((key) => [key, action] as const)));
}

void window.__TAURI__.core.invoke<Binding[]>("shortcuts").then(applyKeymap);

// The settings window rebinds keys; this window culls with the result.
void window.__TAURI__.event.listen<Binding[]>("shortcuts-changed", ({ payload }) => {
  applyKeymap(payload);
});

window.addEventListener("keydown", (event) => {
  if (isUnboundModifier(event)) {
    return;
  }
  const key = keyName(event);
  if (key === "escape" && !filterMenu.hidden) {
    setFilterMenuOpen(false);
    event.preventDefault();
    return;
  }
  const action = keymap.get(key);
  switch (action) {
    case "previous":
      move(-1);
      break;
    case "next":
      move(1);
      break;
    case "focus":
      showFocus = !showFocus;
      draw();
      break;
    case "zoom":
      toggleZoom();
      break;
    case "open":
      openFolder();
      break;
    case "rate1":
    case "rate2":
    case "rate3":
    case "rate4":
    case "rate5": {
      const stars = Number(action.slice(-1));
      judge((_, pick, label) => [stars, pick, label]);
      break;
    }
    case "reject":
      // Sticky, not a toggle: reject twice is still a reject, and it replaces
      // a pick. Unflag undoes it.
      judge((_rating, _pick, label) => [-1, false, label]);
      break;
    case "pick":
      // Sticky like reject, replacing a reject; XMP has no pick, so a no-op there.
      if (sidecarFormat !== "dop") {
        return;
      }
      judge((rating, _pick, label) => [rating === -1 ? null : rating, true, label]);
      break;
    case "unflag":
      // Clears a reject or a pick; does nothing to a file with neither.
      judge((rating, _pick, label) => [rating === -1 ? null : rating, false, label]);
      break;
    case "clear":
      // Clears the stars or the reject and leaves a pick alone.
      judge((_, pick, label) => [null, pick, label]);
      break;
    case "red":
    case "orange":
    case "yellow":
    case "green":
    case "blue":
    case "pink":
    case "purple": {
      // Toggles: the same colour again clears it; another colour replaces it.
      const name = action.charAt(0).toUpperCase() + action.slice(1);
      judge((rating, pick, label) => [rating, pick, label === name ? null : name]);
      break;
    }
    case "clearlabel":
      judge((rating, pick) => [rating, pick, null]);
      break;
    default:
      return;
  }
  event.preventDefault();
});

window.addEventListener("resize", () => {
  draw();
  if (zoomed) {
    scheduleCropForResize();
  }
});

function checkForUpdate(): void {
  window.__TAURI__.updater
    .check()
    .then((update) => {
      if (!update) {
        return;
      }
      const bar = document.getElementById("update") as HTMLDivElement;
      const text = document.getElementById("update-text") as HTMLSpanElement;
      const install = document.getElementById(
        "update-install",
      ) as HTMLButtonElement;
      text.textContent = `Riffle v${update.version} is available`;
      install.addEventListener("click", () => {
        install.disabled = true;
        let total = 0;
        let downloaded = 0;
        text.textContent = "Downloading…";
        update
          .downloadAndInstall((progress) => {
            if (progress.event === "Started") {
              total = progress.data.contentLength ?? 0;
            } else if (progress.event === "Progress") {
              downloaded += progress.data.chunkLength;
              if (total > 0) {
                const percent = Math.min(
                  100,
                  Math.floor((downloaded / total) * 100),
                );
                text.textContent = `Downloading ${percent}%`;
              }
            } else {
              text.textContent = "Installing…";
            }
          })
          .then(() => {
            text.textContent = "Restarting…";
            return window.__TAURI__.process.relaunch();
          })
          .catch((e: unknown) => {
            console.debug("update install failed", e);
            text.textContent = `Update failed: ${e instanceof Error ? e.message : String(e)}`;
            install.textContent = "Retry";
            install.disabled = false;
          });
      });
      bar.hidden = false;
    })
    .catch((e: unknown) => console.debug("update check failed", e));
}

renderMeta();
draw();
checkForUpdate();

export {};
