import { type Binding, isModifierCode, keyName } from "./keys.js";
import * as strip from "./strip.js";
import * as folders from "./folders.js";
import { type Exif, type ExifGroup, exifKey } from "./exif.js";
import { advancesAfter } from "./advance.js";
import { History } from "./undo.js";
import { ErrorList } from "./errors.js";
import {
  type Flag,
  type Orientation,
  anchorAfterFilter,
  passes as filterPasses,
} from "./filter.js";
import { type SortKey, orderFiles } from "./sort.js";
import { relativeSharpness } from "./sharpness.js";
import { burstFrameStep, burstMarks, burstStep, groupBursts, type BurstMember } from "./burst.js";
import { placeholderRect } from "./zoom.js";
import {
  FOCUS_MARK_COLORS,
  type FaceReady,
  type Faces,
  applyFaceReady,
  faceMarks,
  focusMark,
} from "./focus.js";
import { FaceCache, NO_FACES } from "./faces.js";
import { type TrashSummary, rejectedPaths, trashedStatus } from "./trash.js";
import {
  RUNNING_NOTE,
  type SequenceDone,
  SequenceFlow,
  type SequencePreview,
  changedLine,
  doneStatus,
  failureText,
  progressStatus,
  rebuildNotice,
  rowText,
} from "./sequence.js";
import { FILTERED_TEXT, NO_FILES_TEXT, emptyState, openHint } from "./empty.js";
import { type MenuItem, contextMenuGroups, folderMenuGroups, menuPosition } from "./context.js";
import { type FocusCandidate, type Metadata, metaGroups } from "./meta.js";
import { FormatGate } from "./firstrun.js";
import { type McpRequest, type ViewApi, respond } from "./companion.js";
import { initSettings } from "./settings.js";
import { SettingsModal, cycleFocus } from "./modal.js";
import { type Panels, toggle, toggleSides } from "./panels.js";
import { treeGate } from "./treekeys.js";
import {
  type ScanDone,
  type ScanStarted,
  refreshOnFacesDone,
  refreshOnProgress,
  refreshOnScanDone,
  refreshTimingLine,
} from "./refresh.js";
import {
  COMPARE_NEEDS_FRAMES,
  comparePaneAt,
  comparisonCandidates,
  loadComparisonFrames,
  reconcileActive,
} from "./compare.js";
import {
  type Command,
  type PickFlag,
  type Selection,
  all,
  click,
  extend,
  judgments,
  prune,
  selectionOf,
  single,
  targets,
} from "./selection.js";

// Header layout of a `preview` payload, see `crates/app/src/commands.rs`.
const PREVIEW_HEADER_LEN = 8;
const PREVIEW_KIND_JPEG_V1 = 1;
// Header layout of a `focus_crop` payload, see `crates/app/src/commands.rs`.
const CROP_HEADER_LEN = 32;
const CROP_KIND_RGBA_V3 = 5;
// Turned on by the settings modal's `Timing logs` item. That item only
// shows in a development build, so elsewhere this stays off.
let debugLogging = false;

function debugLog(...args: unknown[]): void {
  if (debugLogging) {
    console.debug(...args);
    void window.__TAURI__.core.invoke("log_timing", { line: args.map(String).join(" ") });
  }
}

interface Focus {
  sensor_w: number;
  sensor_h: number;
  x: number;
  y: number;
  frame: { width: number; height: number } | null;
  manual_focus: boolean;
  candidate: "candidate" | "not_candidate" | "unknown";
  eye_focus: number | null;
}

interface IndexedFile {
  path: string;
  orientation: number;
  capture_time: string | null;
  subsec: string | null;
  focus: Focus | null;
  has_thumb: boolean;
  rating: number | null;
  flag: PickFlag;
  label: string | null;
  has_sidecar: boolean;
  sharpness: number | null;
  exif: Exif | null;
}

interface DecodeResponse {
  seq: number;
  bitmap?: ImageBitmap;
  error?: string;
}

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;
const metaEl = document.getElementById("meta") as HTMLDivElement;
const metaStatusEl = document.getElementById("meta-status") as HTMLDivElement;
const positionEl = document.getElementById("position") as HTMLDivElement;
const emptyEl = document.getElementById("empty") as HTMLDivElement;
const formatDialog = document.getElementById("format-dialog") as HTMLDivElement;
const formatError = document.getElementById("format-error") as HTMLParagraphElement;
const settings = initSettings({
  applyKeymap,
  setAutoAdvance: (enabled) => {
    autoAdvance = enabled;
  },
  setDebugLogging: (enabled) => {
    debugLogging = enabled;
  },
});
const formatButtons = [...formatDialog.querySelectorAll<HTMLButtonElement>("button[data-format]")];

// Closed until a sidecar format is saved; no folder opens before that.
const formatGate = new FormatGate();

const worker = new Worker(new URL("./worker.js", import.meta.url), {
  type: "module",
});

// Linux's WebKitGTK only; resolved once, awaited before every post so the
// first preview cannot race it.
const previewPixelLimit = window.__TAURI__.core
  .invoke<number | null>("preview_pixel_limit")
  .catch(() => null);

// Every RAW file in the open folder, in `list_arw` order.
let allFiles: string[] = [];
// The strip order, kept across folder opens within the session.
let sortKey: SortKey = "name";
// The files that pass the filter, in `sortKey` order. `index`, the strip and
// paging all work on this view.
let files: string[] = [];
let index = 0;
// Incremented on every page turn; a response tagged with an older sequence
// belongs to a file that is no longer current and is dropped.
let seq = 0;
const orientations = new Map<number, number>();
// `performance.now()` marks of the preview request for each `seq` awaiting
// its decoded bitmap, for the `page …` timing line.
const pageTimings = new Map<
  number,
  { startedAt: number; invokeMs: number; postedAt: number; keypressAt: number | null }
>();
// `performance.now()` at the key that turned the page, cleared by the preview
// request that page turn produced. A strip click or a folder open starts its
// own request, which is not timed from a keypress.
let pageKeypressAt: number | null = null;
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
// A transient line in the status block at the bottom of the right pane: an
// error.
let note: string | undefined;
// How far the current scan got, or null when nothing is scanning. Events
// carry the id of the scan that emitted them; only events whose id matches
// `scanId` are applied, so the stragglers of a canceled scan (including one
// canceled by reopening the very same folder) are ignored even though they
// carry the same `dir`.
let scanId: number | null = null;
let scanning: string | null = null;
// How far the sequencing run got, or null when none runs.
let sequencing: string | null = null;
// True between `start_scan` and its `faces-done`, i.e. through both scan
// passes. A rescan asked for while it is true is deferred (`resyncPending`)
// rather than canceling the scan.
let scanRunning = false;
// The files the first pass failed on, kept for the final status after the
// second pass.
let scanErrors = 0;
// The first pass's `scan-done` total, so its `faces-done` can skip a re-read
// when neither pass wrote anything.
let scanDone: ScanDone | null = null;
// The rows `scan_folder`'s reconcile changed, so its `scan-done` can skip a
// re-read when neither it nor the scan pass wrote anything.
let scanStarted: ScanStarted | null = null;

function setScanRunning(running: boolean): void {
  scanRunning = running;
  settings.setScanRunning(running);
}
// Mints a per-call id for `startScan` so its `.then`/`.catch` can tell
// whether a later call (a re-open of the same folder included) has already
// superseded it, since `folder !== openDir` can't detect that case.
let scanSeq = 0;
let currentScan = 0;
// True while a rescan's `list_arw` is outstanding, and true when a trigger
// arrived while one was, the way `refreshEntries` keeps one read in flight.
let resyncInFlight = false;
let resyncPending = false;
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
// The current file the last `scan-progress`-triggered refresh was for, so a
// tick re-reads only when that file's row lands or the current file changes.
let progressRefreshedFor: string | null = null;
// The stars of every file that has some: `1`-`5`, and a missing entry is
// unrated. Filled from `folder_entries` and then owned by the keyboard until
// the next folder open.
const ratings = new Map<string, number>();
// The pick / reject of every flagged file, owned the same way as `ratings`.
// It coexists with the stars under both sidecar formats.
const flags = new Map<string, "pick" | "reject">();
// The color label of every file that has one, owned the same way as
// `ratings`.
const labels = new Map<string, string>();
// The sharpness score of every file the index has one for, from
// `folder_entries`; cleared with `labels`.
const sharpness = new Map<string, number>();
// The faces the focus mark draws, detected per file when first shown with
// the mark on; cleared with `sharpness`.
const faceCache = new FaceCache();
// Every file's burst, from `groupBursts` over `allFiles` in capture order
// whatever the sort; recomputed whenever `entries` is refreshed.
let bursts = new Map<string, BurstMember>();
// The paths judged through the keyboard in this session, so a refresh from
// `folder_entries` (which may predate the pending sidecar write) does not
// undo what the user just pressed.
const touched = new Set<string>();
// The strip's multi-selection; `files[index]` is its focused member.
let selection: Selection = single(undefined);
// The index of each path in `files`, for handing a rating to the strip.
const fileIndex = new Map<string, number>();
// A judgment's file and its state before it, for `Edit > Undo`. An entry is
// a batch, undone as one: a single key judges one file, reject-rest several.
// Per folder: `openDirectory` clears it.
type Judgment = { path: string; rating: number | null; flag: PickFlag; label: string | null };
const history = new History<Judgment[]>(100);
// The pre-undo state of each undone batch, for `Edit > Redo`. A new
// judgment forgets it, as every editor does.
const redoable = new History<Judgment[]>(100);
// Sidecar problems, kept until dismissed rather than in the transient `note`.
const errors = new ErrorList();
const shownFlags = new Set<Flag>();
const shownStars = new Set<number>();
const shownLabels = new Set<string>();
const shownOrientations = new Set<Orientation>();
const shownCandidates = new Set<FocusCandidate>();
// The EXIF groups, keyed by label (two estimated apertures with one label can
// differ in value). Focal length is keyed by the range's label instead.
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
let showFocus = false;
// The event.code of the key holding the grayscale preview, or null when off.
let grayscaleHeld: string | null = null;
// True while the 1:1 focus check is showing instead of the fitted preview.
let zoomed = false;
// Side-by-side culling view. With a multi-selection it compares up to four
// selected files; otherwise it compares the current file with the sharpest
// frame in its burst. The bitmaps are independent of `shown`, which remains
// ready for an immediate return to the single-image view.
let comparing = false;
let compareSeq = 0;
let compareFrames: { path: string; bitmap: ImageBitmap; orientation: number }[] = [];
let compareActivePath: string | null = null;
// The crop of the file that `cropSeq` identifies, at one JPEG pixel per
// device pixel, with its point of interest in crop pixels. Kept while the
// view is toggled off so toggling back on redraws without a round trip.
let crop: {
  bitmap: ImageBitmap;
  pointX: number;
  pointY: number;
  cropSeq: number;
  orientation: number;
  // The full JPEG size the crop was cut from, for placing the placeholder.
  fullWidth: number;
  fullHeight: number;
  // The device-pixel viewport size (`canvas.client*` × dpr) at request time,
  // so a resize that lands while a request is already in flight (silently
  // dropped, since `requestCrop` no-ops on `cropInFlight`) is still noticed
  // once this crop settles and can be re-requested.
  requestedWidth: number;
  requestedHeight: number;
} | null = null;
// The same one-in-flight, re-request-if-stale pattern as `inFlight`.
let cropInFlight = false;
// `performance.now()` at the zoom key that asked for a crop, cleared by the
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
// The detected faces, apart from every mark color above.
const FACE_MARK_COLOR = "#3ff";
const FACE_MARK_EYE_RADIUS = 2.5;

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

function row(list: HTMLDListElement, label: string, value: string): void {
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

// The keymap last applied, so the empty state can name the `open` key.
// Empty until the `shortcuts` invoke resolves.
let keyBindings: Binding[] = [];

// The centered message over the viewer: the clickable opening hint when no
// folder is open, or why an open folder shows nothing.
function renderEmpty(): void {
  const state = emptyState(openDir, allFiles.length, files.length);
  emptyEl.hidden = state === "none";
  if (state === "none") {
    emptyEl.removeAttribute("data-state");
    emptyEl.textContent = "";
    return;
  }
  emptyEl.dataset.state = state;
  emptyEl.textContent =
    state === "no-folder"
      ? openHint(keyBindings)
      : state === "no-files"
        ? NO_FILES_TEXT
        : FILTERED_TEXT;
}

emptyEl.addEventListener("click", () => {
  if (emptyEl.dataset.state === "no-folder") {
    openFolder();
  }
});

// Redraw the right pane: the current file's name and its shooting settings
// in the scrolling metadata block, and any note (an error, the scan's
// progress, the 1:1 indicator, the sticky errors) in the status block pinned
// to the bottom. Also refreshes the strip pane's `N / M` counter.
function renderMeta(): void {
  renderTitle();
  renderEmpty();
  positionEl.textContent = files.length === 0 ? "" : `${index + 1} / ${files.length}`;
  if (selection.selected.size > 1) {
    positionEl.textContent += ` \u00B7 ${selection.selected.size} selected`;
  }
  metaEl.replaceChildren();
  metaStatusEl.replaceChildren();
  if (files.length > 0) {
    metaEl.append(line("name", meta === null || metaStale ? baseName(files[index]) : meta.name));
    for (const group of metaGroups(
      meta,
      sharpness.get(files[index]) ?? null,
      entries.get(files[index])?.focus,
    )) {
      metaEl.append(line("group", group.heading));
      const list = document.createElement("dl");
      for (const { label, value } of group.rows) {
        row(list, label, value);
      }
      metaEl.append(list);
    }
  }
  if (note !== undefined) {
    metaStatusEl.append(line("note", note));
  }
  if (scanning !== null) {
    metaStatusEl.append(line("note", scanning));
  }
  if (sequencing !== null) {
    metaStatusEl.append(line("note", sequencing));
  }
  // Driven by `zoomed` rather than `note`, so paging or an error does not
  // erase the mode indicator while the 1:1 view is still showing.
  if (zoomed) {
    metaStatusEl.append(line("note", "1:1"));
  }
  if (comparing) {
    metaStatusEl.append(line("note", `Compare · ${compareCandidates().length} frames`));
  }
  for (const { key, message } of errors.list()) {
    const el = line("error", message);
    const dismiss = document.createElement("button");
    dismiss.textContent = "\u00d7";
    dismiss.title = "Dismiss";
    dismiss.addEventListener("click", () => {
      errors.dismiss(key);
      renderMeta();
    });
    el.append(dismiss);
    metaStatusEl.append(el);
  }
}

// The window title last sent, so paging only crosses IPC when it changes.
let shownTitle = "Riffle";

// Show the open folder and the current file in the title bar.
function renderTitle(): void {
  const parts = ["Riffle"];
  if (openDir !== null) {
    parts.push(baseName(openDir));
  }
  if (files.length > 0) {
    parts.push(baseName(files[index]));
  }
  const title = parts.join(" \u2014 ");
  if (title === shownTitle) {
    return;
  }
  shownTitle = title;
  void window.__TAURI__.window.getCurrentWindow().setTitle(title);
}

// `File > Move Rejected to Trash…`: hand the rejects of the open folder to the
// backend, which confirms before moving anything.
function trashRejected(): void {
  if (openDir === null) {
    setStatus("No folder is open");
    return;
  }
  if (scanRunning) {
    setStatus("a scan is running; wait for it to finish");
    return;
  }
  const paths = rejectedPaths(allFiles, flags);
  if (paths.length === 0) {
    setStatus("No rejected files in this folder");
    return;
  }
  const dir = openDir;
  const token = folderToken;
  window.__TAURI__.core
    .invoke<TrashSummary | null>("trash_rejected", { dir, paths })
    .then((summary) => {
      if (dir !== openDir || token !== folderToken || summary === null) {
        return;
      }
      const failed = new Set(summary.failed.map(({ path }) => path));
      for (const path of paths) {
        if (failed.has(path)) {
          continue;
        }
        ratings.delete(path);
        flags.delete(path);
        labels.delete(path);
        sharpness.delete(path);
        touched.delete(path);
      }
      // An undo of a trashed file would `set_rating` a path that is gone and
      // mint an orphan sidecar.
      const gone = (entry: Judgment) => !failed.has(entry.path) && paths.includes(entry.path);
      history.removeWhere((batch) => batch.every(gone));
      redoable.removeWhere((batch) => batch.every(gone));
      for (const { path, message } of summary.failed) {
        errors.add(path, `${baseName(path)}: could not move to the Trash: ${message}`);
      }
      setStatus(trashedStatus(summary));
      resync();
    })
    .catch((err: unknown) => {
      if (dir !== openDir || token !== folderToken) {
        return;
      }
      setStatus(String(err));
    });
}

// `File > Sequence JPEG Timestamps…`: pick an export folder, preview the
// times a run would write, then run it with progress and cancel. Its errors
// join the sticky `errors` list, keyed by path.
const sequenceFlow = new SequenceFlow();
// Only its key decisions are used: Escape and Tab, as in the settings modal.
const sequenceKeys = new SettingsModal();
const sequenceDialog = document.getElementById("sequence-dialog") as HTMLDivElement;
const sequenceSource = document.getElementById("sequence-source") as HTMLSpanElement;
const sequenceOutput = document.getElementById("sequence-output") as HTMLSpanElement;
const sequenceRebuild = document.getElementById("sequence-rebuild") as HTMLParagraphElement;
const sequenceRows = document.getElementById("sequence-rows") as HTMLOListElement;
const sequenceCount = document.getElementById("sequence-count") as HTMLParagraphElement;
const sequenceFailed = document.getElementById("sequence-failed") as HTMLUListElement;
const sequenceRunning = document.getElementById("sequence-running") as HTMLParagraphElement;
const sequenceRunButton = document.getElementById("sequence-run") as HTMLButtonElement;
const sequenceCancelButton = document.getElementById("sequence-cancel") as HTMLButtonElement;
sequenceRunning.textContent = RUNNING_NOTE;

function sequenceTimestamps(): void {
  if (!formatDialog.hidden || settings.isOpen || !sequenceFlow.start()) {
    return;
  }
  window.__TAURI__.core
    .invoke<string | null>("pick_folder")
    .then((dir) => {
      if (!sequenceFlow.picked(dir) || dir === null) {
        return;
      }
      return window.__TAURI__.core
        .invoke<SequencePreview>("sequence_preview", { dir })
        .then((preview) => {
          if (sequenceFlow.previewed()) {
            showSequencePreview(dir, preview);
          }
        });
    })
    .catch((err: unknown) => {
      sequenceFlow.fail();
      setStatus(String(err));
    });
}

function showSequencePreview(dir: string, preview: SequencePreview): void {
  setFilterMenuOpen(false);
  setSortMenuOpen(false);
  closeContextMenu();
  sequenceSource.textContent = dir;
  sequenceOutput.textContent = preview.output_dir;
  const notice = rebuildNotice(preview);
  sequenceRebuild.textContent = notice ?? "";
  sequenceRebuild.hidden = notice === null;
  sequenceRows.replaceChildren(
    ...preview.rows.map((row) => {
      const item = document.createElement("li");
      item.textContent = rowText(row);
      item.classList.toggle("unchanged", !row.changed);
      return item;
    }),
  );
  sequenceCount.textContent = changedLine(preview.rows);
  sequenceFailed.replaceChildren(
    ...preview.failed.map((failure) => {
      const item = document.createElement("li");
      item.textContent = failureText(failure);
      return item;
    }),
  );
  sequenceRunning.hidden = true;
  sequenceRunButton.disabled = preview.rows.length === 0;
  sequenceDialog.hidden = false;
  (sequenceRunButton.disabled ? sequenceCancelButton : sequenceRunButton).focus();
}

function closeSequenceDialog(): void {
  sequenceDialog.hidden = true;
}

function runSequence(): void {
  const dir = sequenceFlow.run();
  if (dir === null) {
    return;
  }
  sequenceRunButton.disabled = true;
  sequenceRunning.hidden = false;
  sequenceCancelButton.focus();
  sequencing = progressStatus(0, 0);
  renderMeta();
  window.__TAURI__.core
    .invoke<number>("sequence_run", { dir })
    .then((runId) => {
      const { done, cancel } = sequenceFlow.started(runId);
      if (done !== null) {
        finishSequence(done);
      } else if (cancel) {
        void window.__TAURI__.core.invoke("sequence_cancel", { runId });
      }
    })
    .catch((err: unknown) => {
      sequenceFlow.fail();
      closeSequenceDialog();
      sequencing = null;
      setStatus(String(err));
    });
}

function finishSequence(payload: SequenceDone): void {
  if (!sequenceFlow.done(payload)) {
    return;
  }
  closeSequenceDialog();
  for (const failure of payload.failed) {
    errors.add(failure.path, failureText(failure));
  }
  sequencing = null;
  setStatus(doneStatus(payload));
}

function dismissSequence(): void {
  const decision = sequenceFlow.dismiss();
  if (decision.kind === "close") {
    closeSequenceDialog();
  } else if (decision.kind === "cancel" && decision.runId !== null) {
    void window.__TAURI__.core.invoke("sequence_cancel", { runId: decision.runId });
  }
}

// Every key stops here while the dialog is open, so none reaches the strip.
function sequenceKeydown(event: KeyboardEvent): void {
  const decision = sequenceKeys.key(keyName(event));
  if (decision.kind === "close") {
    event.preventDefault();
    dismissSequence();
  } else if (decision.kind === "focus") {
    event.preventDefault();
    const buttons = [sequenceRunButton, sequenceCancelButton].filter((b) => !b.disabled);
    const from = buttons.indexOf(document.activeElement as HTMLButtonElement);
    buttons[cycleFocus(buttons.length, from, decision.step)].focus();
  }
}

sequenceRunButton.addEventListener("click", runSequence);
sequenceCancelButton.addEventListener("click", dismissSequence);

void window.__TAURI__.event.listen<{ run_id: number; done: number; total: number }>(
  "sequence-progress",
  ({ payload }) => {
    if (!sequenceFlow.accepts(payload.run_id)) {
      return;
    }
    sequencing = progressStatus(payload.done, payload.total);
    renderMeta();
  },
);

void window.__TAURI__.event.listen<SequenceDone>("sequence-done", ({ payload }) => {
  finishSequence(payload);
});

// Set the transient note, or clear it when called with no argument.
function setStatus(extra?: string): void {
  note = extra;
  renderMeta();
}

function draw(): void {
  if (comparing) {
    drawCompare();
    return;
  }
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
  context.drawImage(bitmap, -drawWidth / 2, -drawHeight / 2, drawWidth, drawHeight);
  drawFocusMark(drawWidth, drawHeight);
  drawFaceMarks(drawWidth, drawHeight);
  context.restore();
}

function compareCandidates(): string[] {
  return comparisonCandidates(files, index, selection.selected, bursts, sharpness);
}

function closeCompareFrames(): void {
  for (const frame of compareFrames) frame.bitmap.close();
  compareFrames = [];
}

function stopComparing(): void {
  comparing = false;
  compareSeq += 1;
  closeCompareFrames();
  compareActivePath = null;
}

function drawCompare(): void {
  const dpr = window.devicePixelRatio;
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.save();
  context.scale(dpr, dpr);
  const count = compareFrames.length;
  const columns = count <= 2 ? Math.max(count, 1) : 2;
  const rows = Math.max(Math.ceil(count / columns), 1);
  const gap = 4;
  const cellWidth = (width - gap * (columns - 1)) / columns;
  const cellHeight = (height - gap * (rows - 1)) / rows;
  const best = Math.max(...compareFrames.map(({ path }) => sharpness.get(path) ?? -Infinity));
  compareFrames.forEach(({ path, bitmap, orientation }, at) => {
    const col = at % columns;
    const row = Math.floor(at / columns);
    const x = col * (cellWidth + gap);
    const y = row * (cellHeight + gap);
    const labelHeight = 28;
    const quarterTurn = orientation === 6 || orientation === 8;
    const uprightWidth = quarterTurn ? bitmap.height : bitmap.width;
    const uprightHeight = quarterTurn ? bitmap.width : bitmap.height;
    const scale = Math.min(cellWidth / uprightWidth, (cellHeight - labelHeight) / uprightHeight);
    const drawWidth = bitmap.width * scale;
    const drawHeight = bitmap.height * scale;
    context.save();
    context.beginPath();
    context.rect(x, y, cellWidth, cellHeight - labelHeight);
    context.clip();
    context.translate(x + cellWidth / 2, y + (cellHeight - labelHeight) / 2);
    if (orientation === 6) context.rotate(Math.PI / 2);
    else if (orientation === 8) context.rotate(-Math.PI / 2);
    else if (orientation === 3) context.rotate(Math.PI);
    context.drawImage(bitmap, -drawWidth / 2, -drawHeight / 2, drawWidth, drawHeight);
    context.restore();
    const score = sharpness.get(path);
    const isBest = score !== undefined && score === best && compareFrames.length > 1;
    const isActive = path === compareActivePath;
    context.fillStyle = isBest ? "#244c31" : "#252525";
    context.fillRect(x, y + cellHeight - labelHeight, cellWidth, labelHeight);
    context.fillStyle = isBest ? "#6bdc8a" : "#ddd";
    context.font = "12px system-ui, sans-serif";
    context.textBaseline = "middle";
    const suffix =
      (score === undefined ? "" : `  ·  ${score.toFixed(1)}`) +
      `${isBest ? "  BEST" : ""}${isActive ? "  ACTIVE" : ""}`;
    context.fillText(`${baseName(path)}${suffix}`, x + 8, y + cellHeight - labelHeight / 2);
    context.strokeStyle = isBest ? "#6bdc8a" : "#444";
    context.lineWidth = isBest ? 2 : 1;
    context.strokeRect(x + 0.5, y + 0.5, cellWidth - 1, cellHeight - 1);
    if (isActive) {
      context.strokeStyle = "#fff";
      context.lineWidth = 2;
      context.strokeRect(x + 3.5, y + 3.5, cellWidth - 7, cellHeight - 7);
    }
  });
  context.restore();
}

async function loadCompare(): Promise<void> {
  const paths = compareCandidates();
  compareActivePath = reconcileActive(paths, compareActivePath, files[index]);
  if (paths.length < 2) {
    stopComparing();
    renderMeta();
    draw();
    return;
  }
  const request = ++compareSeq;
  closeCompareFrames();
  drawCompare();
  try {
    const frames = await loadComparisonFrames(
      paths,
      async (path) => {
        const payload = await window.__TAURI__.core.invoke<ArrayBuffer>("preview", { path });
        const header = new DataView(payload, 0, PREVIEW_HEADER_LEN);
        if (header.getUint16(0, true) !== PREVIEW_KIND_JPEG_V1) {
          throw new Error("unknown preview payload");
        }
        const orientation = header.getUint16(2, true);
        const bitmap = await createImageBitmap(
          new Blob([payload.slice(PREVIEW_HEADER_LEN)], { type: "image/jpeg" }),
        );
        return { path, bitmap, orientation };
      },
      (frame) => frame.bitmap.close(),
    );
    if (!comparing || request !== compareSeq) {
      for (const frame of frames) frame.bitmap.close();
      return;
    }
    compareFrames = frames;
    drawCompare();
  } catch (error) {
    if (comparing && request === compareSeq) setStatus(String(error));
  }
}

function toggleCompare(): void {
  if (comparing) {
    stopComparing();
    renderMeta();
    draw();
    return;
  }
  const candidates = compareCandidates();
  if (candidates.length < 2) {
    setStatus(COMPARE_NEEDS_FRAMES);
    return;
  }
  comparing = true;
  compareActivePath = files[index] ?? candidates[0];
  if (zoomed) toggleZoom();
  renderMeta();
  void loadCompare();
}

// Make the comparison pane under the pointer the active one. Returns whether
// a pane was hit.
function activateComparePane(clientX: number, clientY: number): boolean {
  if (!comparing || compareFrames.length === 0) return false;
  const rect = canvas.getBoundingClientRect();
  const at = comparePaneAt(
    clientX - rect.left,
    clientY - rect.top,
    rect.width,
    rect.height,
    compareFrames.length,
  );
  const frame = at === null ? undefined : compareFrames[at];
  if (frame === undefined) return false;
  compareActivePath = frame.path;
  drawCompare();
  renderMeta();
  return true;
}

canvas.addEventListener("click", (event) => {
  activateComparePane(event.clientX, event.clientY);
});

canvas.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  if (files.length === 0) return;
  if (comparing && !activateComparePane(event.clientX, event.clientY)) return;
  openContextMenu(event.clientX, event.clientY);
});

// Record a judgment locally: the `ratings` and `flags` maps and the strip
// cell. `null` (or `0`) is unrated.
function applyRating(
  path: string,
  rating: number | null,
  flag: PickFlag,
  label: string | null,
): void {
  if (rating === null || rating === 0) {
    ratings.delete(path);
  } else {
    ratings.set(path, rating);
  }
  if (flag === "none") {
    flags.delete(path);
  } else {
    flags.set(path, flag);
  }
  if (label === null) {
    labels.delete(path);
  } else {
    labels.set(path, label);
  }
  const at = fileIndex.get(path);
  if (at !== undefined) {
    strip.setRating(at, ratings.get(path) ?? null, flag, label);
  }
}

function flagOf(path: string): PickFlag {
  return flags.get(path) ?? "none";
}

function passes(path: string): boolean {
  return filterPasses(
    {
      flags: shownFlags,
      stars: shownStars,
      labels: shownLabels,
      orientations: shownOrientations,
      candidates: shownCandidates,
      exif: shownExif,
    },
    {
      rating: ratings.get(path) ?? null,
      flag: flagOf(path),
      label: labels.get(path) ?? null,
    },
    entries.get(path)?.exif,
    entries.get(path)?.orientation,
    entries.get(path)?.focus?.candidate,
  );
}

function ordered(): string[] {
  return orderFiles(sortKey, allFiles, (path) => {
    const entry = entries.get(path);
    return {
      captureTime: entry?.capture_time ?? undefined,
      subsec: entry?.subsec ?? undefined,
      // A reject sorts after the unrated files whatever its stars.
      rating: flagOf(path) === "reject" ? -1 : ratings.get(path),
    };
  });
}

// Rebuild `files` from `allFiles` after a filter, sort or judgment change.
// The current file stays current if it still passes; otherwise the next
// passing file after it (in sort order) takes over, or the last one before it,
// or the empty view. A judgment that drops the current file out of the
// filter therefore hides it at once and moves on to the next passing file.
// Returns whether it rebuilt the strip with `strip.setFiles`.
function refilter(anchor: string | undefined = files[index], keepScroll = false): boolean {
  const order = ordered();
  const next = order.filter(passes);
  if (next.length === files.length && next.every((path, at) => path === files[at])) {
    if (comparing) void loadCompare();
    return false;
  }
  files = next;
  fileIndex.clear();
  files.forEach((path, at) => {
    fileIndex.set(path, at);
  });
  strip.setFiles(files, keepScroll);
  files.forEach((path, at) => {
    strip.setRating(at, ratings.get(path) ?? null, flagOf(path), labels.get(path) ?? null);
  });
  applySharpness();
  applyBursts();
  applyCandidates();
  if (files.length === 0) {
    closeContextMenu();
    index = 0;
    selection = prune(selection, files, index);
    if (comparing) stopComparing();
    seq += 1;
    shown?.bitmap.close();
    shown = null;
    meta = null;
    zoomed = false;
    draw();
    renderMeta();
    return true;
  }
  const target = anchorAfterFilter(order, passes, anchor);
  index = (target === undefined ? undefined : fileIndex.get(target)) ?? 0;
  selection = prune(selection, files, index);
  paintSelection();
  if (files[index] === anchor) {
    strip.setCurrent(index);
    renderMeta();
    if (comparing) void loadCompare();
  } else {
    show();
  }
  return true;
}

// A judgment key over the selection's targets: the command's new value is
// decided once from the focused file and set on every target, each keeping
// the fields the command does not touch. Update the maps and redraw first,
// then tell the backend per file. The invokes are never awaited for anything
// visible; only a failure is, which reverts that file (if the folder is still
// the one it belongs to) and says so in the status line. Returns the number
// of targets when anything changed, else 0.
function judge(command: Command, forceLabel = false): number {
  if (files.length === 0) {
    return 0;
  }
  const current = (comparing ? compareActivePath : null) ?? files[index];
  const paths = comparing ? [current] : targets(selection, files, index);
  return record(paths, current, command, forceLabel) ? paths.length : 0;
}

// The changes `command` makes to `paths`, its value decided from `focused`,
// pushed as one undo entry and committed with `anchor` kept current. Returns
// whether anything changed.
function record(
  paths: readonly string[],
  focused: string,
  command: Command,
  forceLabel = false,
  anchor = focused,
): boolean {
  // Idempotent: pressing the current value again does nothing at all, which
  // is what makes key auto-repeat harmless. A forced label still goes out
  // while the file's real label is unknown.
  const changes: Change[] = judgments(
    paths,
    focused,
    (path) => ({
      rating: ratings.get(path) ?? null,
      flag: flagOf(path),
      label: labels.get(path) ?? null,
    }),
    command,
    (path) => forceLabel && !entries.has(path),
  ).map(({ before, after }) => ({ before, ...after, forceLabel }));
  if (changes.length === 0) {
    return false;
  }
  const batch = changes.map(({ before }) => before);
  history.push(batch);
  redoable.clear();
  commit(changes, forgetOnFail(history, batch), () => anchor);
  return true;
}

// `rejectRest`: reject every other member of the current file's burst, over
// `allFiles` so a member the filter hides is rejected too, as one undo entry.
// Members already rejected are skipped, so the batch holds real changes only.
function rejectRest(): void {
  if (files.length === 0) {
    return;
  }
  const current = files[index];
  const burst = bursts.get(current);
  if (burst === undefined || burst.size < 2) {
    return;
  }
  const changes: Change[] = [];
  for (const path of allFiles) {
    if (path === current || bursts.get(path)?.burst !== burst.burst) {
      continue;
    }
    const before = {
      path,
      rating: ratings.get(path) ?? null,
      flag: flagOf(path),
      label: labels.get(path) ?? null,
    };
    if (before.flag === "reject") {
      continue;
    }
    changes.push({ before, rating: before.rating, flag: "reject", label: before.label });
  }
  if (changes.length === 0) {
    return;
  }
  const batch = changes.map(({ before }) => before);
  history.push(batch);
  redoable.clear();
  // The current file stays current unless the filter now hides it.
  commit(changes, forgetOnFail(history, batch), () => current);
}

// A failed write drops its file from `batch`, and the batch from `from` once
// every file of it has failed: the rest still changed and stay undoable.
function forgetOnFail(from: History<Judgment[]>, batch: Judgment[]): (failed: Judgment) => void {
  return (failed) => {
    const at = batch.indexOf(failed);
    if (at !== -1) {
      batch.splice(at, 1);
    }
    if (batch.length === 0) {
      from.remove(batch);
    }
  };
}

// `forceLabel` sends `labelKnown: true` even before `folder_entries` has told
// us the file's label, so the sidecar's label is cleared whatever it is.
type Change = {
  before: Judgment;
  rating: number | null;
  flag: PickFlag;
  label: string | null;
  forceLabel?: boolean;
};

// Apply every change locally and refilter once, then tell the backend per
// file; each `before` is its file's state to revert to when its invoke fails.
// `anchor` picks the file to keep current once the new state is applied (the
// first change's file by default).
function commit(
  changes: Change[],
  onFail?: (failed: Judgment) => void,
  anchor?: () => string | undefined,
): void {
  for (const { before, rating, flag, label } of changes) {
    touched.add(before.path);
    applyRating(before.path, rating, flag, label);
  }
  renderMeta();
  refilter(anchor === undefined ? changes[0].before.path : anchor());
  for (const change of changes) {
    send(change, onFail);
  }
}

function send(
  { before, rating, flag, label, forceLabel }: Change,
  onFail?: (failed: Judgment) => void,
): void {
  const { path } = before;
  const token = folderToken;
  // `label` only carries a meaningful value once `folder_entries` has told us
  // this path's label; before that, `labelKnown: false` tells the backend to
  // keep whatever it already has instead of clearing it, unless this key
  // set the label itself.
  void window.__TAURI__.core
    .invoke("set_rating", {
      path,
      rating: rating ?? 0,
      flag,
      label,
      labelKnown: forceLabel === true || entries.has(path) || label !== before.label,
    })
    .catch((err: unknown) => {
      if (token !== folderToken) {
        return;
      }
      onFail?.(before);
      touched.delete(path);
      applyRating(path, before.rating, before.flag, before.label);
      refilter();
      setStatus(String(err));
    });
}

// `Edit > Undo` and `Edit > Redo`: pop the most recent batch off `from`, push
// its files' current states onto `to` and restore the popped states. A single
// file becomes current unless the filter now hides it; a batch leaves the
// current file where it is.
function step(from: History<Judgment[]>, to: History<Judgment[]>, verb: string): void {
  if (openDir === null) {
    return;
  }
  const popped = from.pop();
  const batch = popped?.filter((entry) => allFiles.includes(entry.path)) ?? [];
  if (batch.length === 0) {
    return;
  }
  const changes = batch.map((entry) => ({
    before: {
      path: entry.path,
      rating: ratings.get(entry.path) ?? null,
      flag: flagOf(entry.path),
      label: labels.get(entry.path) ?? null,
    },
    rating: entry.rating,
    flag: entry.flag,
    label: entry.label,
  }));
  to.push(changes.map(({ before }) => before));
  // A file the filter now hides leaves the current file where it is.
  const shownPath = files[index];
  if (batch.length > 1) {
    commit(changes, undefined, () => shownPath);
    setStatus(`${verb} ${batch.length} files`);
    return;
  }
  const { path } = batch[0];
  commit(changes, undefined, () => (passes(path) ? path : shownPath));
  const at = fileIndex.get(path);
  const name = path.split(/[\\/]/).pop();
  if (at === undefined) {
    setStatus(`${verb} ${name} (hidden by the filter)`);
    return;
  }
  if (at !== index) {
    index = at;
    show();
  }
  setStatus(`${verb} ${name}`);
}

function undo(): void {
  step(history, redoable, "Undid");
}

function redo(): void {
  step(redoable, history, "Redid");
}

// Hand the strip each visible file's score relative to its burst, or to the
// singles around it. The comparison runs over `allFiles` in capture order, so
// neither the filter nor the sort changes it.
function applySharpness(): void {
  const result = relativeSharpness(allFiles, (path) => {
    const entry = entries.get(path);
    const member = bursts.get(path);
    return {
      captureTime: entry?.capture_time ?? undefined,
      subsec: entry?.subsec ?? undefined,
      score: sharpness.get(path) ?? null,
      burst: member !== undefined && member.size > 1 ? member.burst : null,
    };
  });
  files.forEach((path, at) => {
    strip.setSharpness(at, result.get(path) ?? null);
  });
}

// Hand the strip each displayed file's place in its burst, so the bracket
// opens and closes where a filter or sort separates members.
function applyBursts(): void {
  burstMarks(files, bursts).forEach((value, at) => {
    strip.setBurst(at, value);
  });
}

// Hand the strip whether each displayed file is a focus candidate.
function applyCandidates(): void {
  files.forEach((path, at) => {
    strip.setCandidate(at, entries.get(path)?.focus?.candidate === "candidate");
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
  const start = performance.now();
  void window.__TAURI__.core
    .invoke<IndexedFile[]>("folder_entries", { dir })
    .then((rows) => {
      const invoked = performance.now();
      entriesInFlight = false;
      if (entriesPending) {
        entriesPending = false;
        refreshEntries();
      }
      if (dir !== openDir || token !== folderToken) {
        return;
      }
      entries.clear();
      sharpness.clear();
      faceCache.clear();
      for (const row of rows) {
        entries.set(row.path, row);
        if (row.sharpness !== null) {
          sharpness.set(row.path, row.sharpness);
        }
        if (!touched.has(row.path)) {
          applyRating(row.path, row.rating, row.flag, row.label);
        }
      }
      const rebuilt = performance.now();
      bursts = groupBursts(allFiles, (path) => {
        const entry = entries.get(path);
        return {
          captureTime: entry?.capture_time ?? undefined,
          subsec: entry?.subsec ?? undefined,
        };
      });
      const grouped = performance.now();
      rebuildExifMenu();
      const exifed = performance.now();
      renderMeta();
      const metaed = performance.now();
      draw();
      const drawn = performance.now();
      applySharpness();
      const sharpened = performance.now();
      applyBursts();
      const bracketed = performance.now();
      applyCandidates();
      const marked = performance.now();
      const setFiles = refilter();
      const end = performance.now();
      debugLog(
        refreshTimingLine({
          rows: rows.length,
          invoke: invoked - start,
          entries: rebuilt - invoked,
          bursts: grouped - rebuilt,
          exif: exifed - grouped,
          meta: metaed - exifed,
          draw: drawn - metaed,
          sharpness: sharpened - drawn,
          applyBursts: bracketed - sharpened,
          candidates: marked - bracketed,
          refilter: end - marked,
          setFiles,
          total: end - start,
        }),
      );
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
// When Sony `FocusFrameSize` is valid, the AF frame the camera used is drawn
// around the point as well, in the same sensor coordinates; a body that
// records only the point gets the crosshair alone, and a manual-focus shot,
// whose recorded point is not trusted, gets no mark. The mark is green for a
// focus candidate (the eyes of the face nearest the AF point are sharp),
// orange when that face's eyes are not sharp, and white when Riffle does not
// know: no face near the point, or the second scan pass has not reached the
// file yet.
function drawFocusMark(drawWidth: number, drawHeight: number): void {
  if (!showFocus || files.length === 0) {
    return;
  }
  const mark = focusMark(entries.get(files[index])?.focus, drawWidth, drawHeight);
  if (mark === null) {
    return;
  }
  const { x, y, rect, candidate } = mark;
  const arm = FOCUS_MARK_ARM;
  const gap = FOCUS_MARK_GAP;
  // A state color over a dark outline: the color carries the mark on most
  // photos, and the outline still draws its edge where the subject shares the
  // color. The same path is stroked twice, the outline first and wider, and
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
  if (rect !== null) {
    context.rect(rect.x, rect.y, rect.width, rect.height);
  }
  context.strokeStyle = "rgba(0, 0, 0, 0.8)";
  context.lineWidth = 4;
  context.stroke();
  context.strokeStyle = FOCUS_MARK_COLORS[candidate];
  context.lineWidth = 2;
  context.stroke();
  context.restore();
}

// The faces Riffle detects on the current file's preview, in the same
// unrotated coordinates as the AF mark: a box per face and a dot between its
// eyes. They are detected when first drawn and appear once `faces_of`
// answers; the 1:1 view and Compare do not draw them.
function drawFaceMarks(drawWidth: number, drawHeight: number): void {
  if (!showFocus || files.length === 0) {
    return;
  }
  // `shown` still holds the previous file's bitmap between `show()` and the
  // new preview's decode. Skip drawing (and requesting) faces until `shown`
  // belongs to the current file, or a fast response would paint the new
  // file's boxes on the old bitmap, same as `drawZoom`'s `shown.seq === seq`
  // guard.
  if (shown === null || shown.seq !== seq) {
    return;
  }
  const path = files[index];
  const found = faceCache.get(path);
  if (found === undefined) {
    requestFaces(path);
    return;
  }
  if (found.faces.length === 0) {
    return;
  }
  const marks = faceMarks(found.faces, found.width, found.height, drawWidth, drawHeight);
  // The same outline-then-color passes as the AF mark.
  context.save();
  context.beginPath();
  for (const { rect } of marks) {
    context.rect(rect.x, rect.y, rect.width, rect.height);
  }
  context.strokeStyle = "rgba(0, 0, 0, 0.8)";
  context.lineWidth = 4;
  context.stroke();
  context.strokeStyle = FACE_MARK_COLOR;
  context.lineWidth = 2;
  context.stroke();
  context.beginPath();
  for (const { eye } of marks) {
    context.moveTo(eye.x + FACE_MARK_EYE_RADIUS, eye.y);
    context.arc(eye.x, eye.y, FACE_MARK_EYE_RADIUS, 0, 2 * Math.PI);
  }
  context.strokeStyle = "rgba(0, 0, 0, 0.8)";
  context.stroke();
  context.fillStyle = FACE_MARK_COLOR;
  context.fill();
  context.restore();
}

// A failed detection is logged and cached as no faces, so a bad file is not
// retried on every draw.
function requestFaces(path: string): void {
  const token = faceCache.request(path);
  if (token === null) {
    return;
  }
  void window.__TAURI__.core
    .invoke<Faces>("faces_of", { path })
    .catch((err: unknown) => {
      console.error(err);
      return NO_FACES;
    })
    .then((found) => {
      if (faceCache.settle(path, token, found) && files[index] === path) {
        draw();
      }
    });
}

// The 1:1 view: the same rotation `draw()` applies, with the focus point at
// the canvas center. Everything here is in device pixels, so the `scale(dpr,
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
    const full =
      crop !== null && crop.cropSeq === seq
        ? { width: crop.fullWidth, height: crop.fullHeight }
        : null;
    const { x, y, width, height } = placeholderRect(
      placeholderShown.bitmap.width,
      placeholderShown.bitmap.height,
      focus,
      full,
    );
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
  // with a zoom key pressed minutes ago.
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
      if (kind !== CROP_KIND_RGBA_V3) {
        throw new Error(`unknown crop payload kind ${kind}`);
      }
      const orientation = header.getUint16(2, true);
      const width = header.getUint32(4, true);
      const height = header.getUint32(8, true);
      // `putImageData` ignores the rotation transform, so the pixels go
      // through a bitmap; no JPEG is involved, so no worker is needed.
      const pixels = new ImageData(new Uint8ClampedArray(payload, CROP_HEADER_LEN), width, height);
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
        fullWidth: header.getUint16(28, true),
        fullHeight: header.getUint16(30, true),
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

// The zoom key (`z`) toggles the 1:1 view. The crop already held for this
// file is reused; otherwise one is requested and the scaled preview stands
// in.
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
    pageKeypressAt = null;
    return;
  }
  inFlight = true;
  const current = seq;
  const requestStartedAt = performance.now();
  const keypressAt = pageKeypressAt;
  pageKeypressAt = null;
  window.__TAURI__.core
    .invoke<ArrayBuffer>("preview", { path: files[index] })
    .then((payload) => {
      const invokeMs = performance.now() - requestStartedAt;
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
      return previewPixelLimit.then((maxPixels) => {
        if (current !== seq) {
          // `inFlight` was already cleared above, and the page turn that
          // invalidated this request already started its own
          // `requestPreview()` from `show()`. Calling it again here would
          // start a duplicate `preview` invoke for the same `seq`.
          return;
        }
        orientations.set(current, header.getUint16(2, true));
        const jpeg = payload.slice(PREVIEW_HEADER_LEN);
        pageTimings.set(current, {
          startedAt: requestStartedAt,
          invokeMs,
          postedAt: performance.now(),
          keypressAt,
        });
        worker.postMessage({ seq: current, jpeg, maxPixels }, [jpeg]);
      });
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
  } else {
    crop?.bitmap.close();
    crop = null;
  }
  if (comparing) void loadCompare();
}

worker.addEventListener("message", (event: MessageEvent<DecodeResponse>) => {
  const { seq: responseSeq, bitmap, error } = event.data;
  const orientation = orientations.get(responseSeq) ?? 1;
  orientations.delete(responseSeq);
  const timing = pageTimings.get(responseSeq);
  pageTimings.delete(responseSeq);
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
  const decodedAt = performance.now();
  draw();
  if (timing !== undefined) {
    const drawnAt = performance.now();
    const sinceKeypress =
      timing.keypressAt === null
        ? ""
        : ` keypressToPixels=${(drawnAt - timing.keypressAt).toFixed(1)}ms`;
    debugLog(
      `page invoke=${timing.invokeMs.toFixed(1)}ms` +
        ` decode=${(decodedAt - timing.postedAt).toFixed(1)}ms` +
        ` total=${(drawnAt - timing.startedAt).toFixed(1)}ms` +
        sinceKeypress,
    );
  }
});

function moveBurst(direction: -1 | 1): void {
  if (files.length === 0) {
    return;
  }
  const ids = files.map((path, at) => bursts.get(path)?.burst ?? -1 - at);
  const next = burstStep(ids, index, direction);
  if (next === index) {
    return;
  }
  index = next;
  selection = single(files[index]);
  paintSelection();
  pageKeypressAt = performance.now();
  show();
}

function moveBurstFrame(direction: -1 | 1): void {
  if (files.length === 0) {
    return;
  }
  const ids = files.map((path, at) => bursts.get(path)?.burst ?? -1 - at);
  const next = burstFrameStep(ids, index, direction);
  if (next === index) {
    return;
  }
  index = next;
  selection = single(files[index]);
  paintSelection();
  pageKeypressAt = performance.now();
  show();
}

function move(delta: number): void {
  if (files.length === 0) {
    return;
  }
  const next = Math.min(Math.max(index + delta, 0), files.length - 1);
  if (next === index) {
    return;
  }
  index = next;
  selection = single(files[index]);
  paintSelection();
  pageKeypressAt = performance.now();
  show();
}

// Moves the focus by one and grows or shrinks the range from the anchor.
function extendSelection(delta: -1 | 1): void {
  if (files.length === 0) {
    return;
  }
  const extended = extend(selection, files, index, delta);
  selection = extended.selection;
  paintSelection();
  if (extended.index === index) {
    renderMeta();
    if (comparing) void loadCompare();
    return;
  }
  index = extended.index;
  pageKeypressAt = performance.now();
  show();
}

// Selects every file the strip shows; the focus stays put and anchors it.
function selectAllFiles(): void {
  if (files.length === 0) {
    return;
  }
  selection = all(files, index);
  paintSelection();
  renderMeta();
  if (comparing) void loadCompare();
}

const contextMenu = document.getElementById("context-menu") as HTMLDivElement;

function closeContextMenu(): void {
  contextMenu.hidden = true;
}

function openContextMenu(x: number, y: number): void {
  const focused = (comparing ? compareActivePath : null) ?? files[index];
  const state = {
    rating: ratings.get(focused) ?? null,
    flag: flagOf(focused),
    label: labels.get(focused) ?? null,
  };
  showMenu(contextMenuGroups(keyBindings, state), x, y, runAction);
}

// Fill `#context-menu` with `groups` at (`x`, `y`); a click on an item
// closes the menu and hands its action to `run`.
function showMenu(groups: MenuItem[][], x: number, y: number, run: (action: string) => void): void {
  contextMenu.replaceChildren(
    ...groups.flatMap((group, i) => {
      const items: HTMLElement[] = group.map(({ action, label, shortcut, checked }) => {
        const item = document.createElement("button");
        item.type = "button";
        if (checked === undefined) {
          item.setAttribute("role", "menuitem");
        } else {
          item.setAttribute("role", "menuitemradio");
          item.setAttribute("aria-checked", String(checked));
        }
        const name = document.createElement("span");
        name.textContent = label;
        const key = document.createElement("span");
        key.className = "shortcut";
        key.textContent = shortcut;
        item.append(name, key);
        item.addEventListener("click", () => {
          closeContextMenu();
          run(action);
        });
        return item;
      });
      return i === 0 ? items : [document.createElement("hr"), ...items];
    }),
  );
  contextMenu.hidden = false;
  const { left, top } = menuPosition(
    x,
    y,
    contextMenu.offsetWidth,
    contextMenu.offsetHeight,
    window.innerWidth,
    window.innerHeight,
  );
  contextMenu.style.left = `${left}px`;
  contextMenu.style.top = `${top}px`;
}

document.addEventListener("mousedown", (event) => {
  if (!contextMenu.hidden && !(event.target as Element).closest("#context-menu")) {
    closeContextMenu();
  }
});

// Push the selection to the strip as indices into `files`.
function paintSelection(): void {
  strip.setSelected(
    [...selection.selected].flatMap((path) => {
      const at = fileIndex.get(path);
      return at === undefined ? [] : [at];
    }),
  );
}

// A Cmd/Ctrl+click only changes the selection; a Shift+click also moves the
// focus to the clicked file, so the range always holds the focused file.
strip.init(
  (selected, modifiers) => {
    selection = click(selection, files, index, selected, modifiers);
    paintSelection();
    focusFile(files[modifiers.toggle ? index : selected]);
  },
  (selected, x, y) => {
    if (!selection.selected.has(files[selected])) {
      selection = single(files[selected]);
      paintSelection();
    }
    if (selected !== index) {
      index = selected;
      show();
    }
    openContextMenu(x, y);
  },
);

const revealLabel = window.__TAURI__.core.invoke<string>("reveal_label");

// A folder clicked in the tree opens the way a drop does; a right-click
// offers to reveal it in the OS file manager.
folders.init(
  (path) => {
    if (!formatGate.isOpen) {
      return;
    }
    openDirectory(path, newFolderToken()).catch((err: unknown) => {
      setStatus(String(err));
    });
  },
  setStatus,
  (path, x, y) => {
    void revealLabel.then((label) => {
      showMenu(folderMenuGroups(label), x, y, () => {
        window.__TAURI__.core.invoke("reveal_folder", { path }).catch((err: unknown) => {
          setStatus(String(err));
        });
      });
    });
  },
);

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
      // The folder went away after the check; the opening hint stays up.
    });
}

// The diff-and-scan chain both folder paths share: `scan_folder` reconciles
// the index against the disk (deleting the rows of files that are gone or
// changed) and returns the ids of the work left, which `start_scan` runs.
// Shared by `openDirectory` (the reset path) and `resync` (the keep-state
// path).
function startScan(folder: string): Promise<void> {
  // Set before `scan_folder` resolves so a `resync()` during the prepare
  // phase (folder listing plus index and sidecar reconcile) is deferred too,
  // not just during `start_scan` — otherwise it starts a second
  // `scan_folder` that stampedes this one's `scanId`.
  setScanRunning(true);
  scanSeq += 1;
  const seq = scanSeq;
  currentScan = seq;
  return window.__TAURI__.core
    .invoke<{
      total: number;
      scan_id: number;
      sidecar_errors: { path: string; message: string }[];
      changed: number;
    }>("scan_folder", {
      dir: folder,
    })
    .then(({ scan_id, sidecar_errors, changed }) => {
      if (seq !== currentScan) {
        // A later `startScan` call (a different folder, or a deliberate
        // re-open of this same one) now owns `scanRunning` / `resyncPending`,
        // so leave them alone.
        return;
      }
      if (sidecar_errors.length > 0) {
        for (const { path, message } of sidecar_errors) {
          errors.add(path, `${baseName(path)}: ${message}`);
        }
        renderMeta();
      }
      scanId = scan_id;
      scanStarted = { scanId: scan_id, changed };
      return window.__TAURI__.core.invoke<void>("start_scan", {
        scanId: scan_id,
      });
    })
    .catch((err: unknown) => {
      if (seq !== currentScan) {
        return;
      }
      setScanRunning(false);
      drainResync();
      setStatus(String(err));
    });
}

// Bring the open folder in line with the disk without losing anything the
// session holds: ratings, flags, labels, sharpness, `touched`, the undo
// history, the errors and the preview all stay, and the current file stays
// current (or, when it was deleted, gives way to the neighbor
// `anchorAfterFilter` picks). `entries` is left alone here; the
// `folder_entries` read on `scan-done` replaces the map wholesale, so rows of
// files that are gone drop out there.
//
// The same open, so no new folder token is minted: a listing that lands after
// another folder was opened is dropped by the guard below.
function resync(): void {
  if (openDir === null) {
    return;
  }
  // A rescan while a scan runs would cancel and restart it (`scan_folder`
  // joins the running scan first), so it waits for `faces-done` instead. A
  // burst of triggers collapses into the one pending rescan.
  if (scanRunning || resyncInFlight) {
    resyncPending = true;
    return;
  }
  const dir = openDir;
  const token = folderToken;
  const anchor = files[index];
  resyncInFlight = true;
  window.__TAURI__.core
    .invoke<string[]>("list_arw", { dir })
    .then((found) => {
      resyncInFlight = false;
      if (dir !== openDir || token !== folderToken) {
        drainResync();
        return;
      }
      allFiles = found;
      refilter(anchor, true);
      return startScan(dir);
    })
    .catch((err: unknown) => {
      resyncInFlight = false;
      drainResync();
      setStatus(String(err));
    });
}

function drainResync(): void {
  if (!resyncPending) {
    return;
  }
  resyncPending = false;
  resync();
}

function openDirectory(folder: string, token: number): Promise<void> {
  if (!formatGate.isOpen) {
    return Promise.resolve();
  }
  return window.__TAURI__.core.invoke<string[]>("list_arw", { dir: folder }).then((found) => {
    if (token !== folderToken) {
      return;
    }
    closeContextMenu();
    stopComparing();
    for (const set of shownExif.values()) {
      set.clear();
    }
    allFiles = found;
    entries.clear();
    ratings.clear();
    files = ordered().filter(passes);
    index = 0;
    selection = single(files[0]);
    openDir = folder;
    void folders.reveal(folder, () => token === folderToken);
    void window.__TAURI__.core.invoke("remember_folder", { dir: folder });
    rebuildExifMenu();
    flags.clear();
    labels.clear();
    sharpness.clear();
    faceCache.clear();
    bursts = new Map();
    touched.clear();
    history.clear();
    redoable.clear();
    errors.clear();
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
    scanDone = null;
    scanStarted = null;
    progressRefreshedFor = null;
    setScanRunning(false);
    resyncPending = false;
    void startScan(folder);
    if (files.length === 0) {
      meta = null;
      setStatus();
      return;
    }
    show();
  });
}

function openFolder(): void {
  if (!formatGate.isOpen) {
    return;
  }
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

// The menu accelerators of keymap actions (Open Folder, Undo, Redo) stay out
// of the way while the settings or the sequence modal is open, as their keys
// do.
function modalOpen(): boolean {
  return settings.isOpen || sequenceFlow.isOpen;
}

void window.__TAURI__.event.listen("open-folder", () => {
  if (!modalOpen()) openFolder();
});
// `File > Reload Folder`, and the main window regaining focus: both rescan
// the open folder in place. The focus that follows launch finds no folder
// open yet, or a scan running, so it costs nothing. The focus listener is
// scoped to this window: a global `event.listen` also receives other
// windows' focus.
void window.__TAURI__.event.listen("reload-folder", resync);
void window.__TAURI__.window.getCurrentWindow().listen("tauri://focus", () => {
  // Skip while the settings modal is open: closing its Clear Cache confirm
  // dialog refocuses this window, and a resync here races clear_index for
  // the Scans lock (`a scan is running`). The folder watcher still catches
  // on-disk changes, and `index-cleared` reopens the folder after a clear.
  if (!settings.isOpen) resync();
});
// The folder watcher's trigger, debounced in Rust. The listener outlives every
// folder, so an event for a folder that is no longer open is dropped.
void window.__TAURI__.event.listen<{ dir: string }>("folder-changed", ({ payload }) => {
  if (payload.dir !== openDir) {
    return;
  }
  resync();
});
void window.__TAURI__.event.listen("trash-rejected", trashRejected);
void window.__TAURI__.event.listen("sequence-timestamps", sequenceTimestamps);
void window.__TAURI__.event.listen("undo", () => {
  if (!modalOpen()) undo();
});
void window.__TAURI__.event.listen("redo", () => {
  if (!modalOpen()) redo();
});
// `Edit > Select All` replaces the predefined item, so it selects a focused
// text input's text itself, and otherwise gates on focus as the keydown path
// does. If the accelerator also reaches the keydown handler, the second run
// is harmless: selecting all is idempotent.
void window.__TAURI__.event.listen("select-all", () => {
  const active = document.activeElement;
  if (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) {
    active.select();
  } else if (!modalOpen() && !folders.hasFocus()) {
    selectAllFiles();
  }
});

// Make `path` the focused file after the selection changed, as a strip click
// does.
function focusFile(path: string): void {
  const at = fileIndex.get(path);
  if (at === undefined) return;
  if (at === index) {
    renderMeta();
    if (comparing) void loadCompare();
    return;
  }
  index = at;
  show();
}

// The MCP companion reads and drives the view through this; every getter is
// live.
const view: ViewApi = {
  get folder() {
    return openDir;
  },
  get files() {
    return files;
  },
  get index() {
    return index;
  },
  get selection() {
    return selection.selected;
  },
  get bursts() {
    return bursts;
  },
  sharpness,
  ratings,
  flags,
  labels,
  get zoomed() {
    return zoomed;
  },
  get comparing() {
    return comparing;
  },
  get compareActive() {
    return compareActivePath;
  },
  get sort() {
    return sortKey;
  },
  get filtered() {
    return filterActive();
  },
  showPhoto(path) {
    selection = single(path);
    paintSelection();
    focusFile(path);
  },
  selectPhotos(paths) {
    selection = selectionOf(paths);
    paintSelection();
    focusFile(paths[0]);
  },
  setMode(mode) {
    if ((mode === "compare") !== comparing) toggleCompare();
    if (mode !== "compare" && (mode === "zoom") !== zoomed) toggleZoom();
  },
  judge(paths, command) {
    const current = (comparing ? compareActivePath : null) ?? files[index];
    record(paths, paths.includes(current) ? current : paths[0], command, false, current);
  },
};
void window.__TAURI__.event.listen<McpRequest>("mcp-request", ({ payload }) => {
  void respond(payload, view).then((reply) =>
    window.__TAURI__.core.invoke("mcp_reply", { ...reply }),
  );
});

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

void window.__TAURI__.event.listen<{ paths: string[] }>("tauri://drag-drop", ({ payload }) => {
  setDragging(false);
  if (!formatGate.isOpen) {
    return;
  }
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
});

void window.__TAURI__.event.listen<{
  dir: string;
  scan_id: number;
  done: number;
  total: number;
  ready: string[];
}>("scan-progress", ({ payload }) => {
  if (payload.scan_id !== scanId) {
    return;
  }
  scanning = `scanning ${payload.done} / ${payload.total}`;
  renderMeta();
  strip.markReady(payload.ready);
  // Only when the row the focus mark needs is still missing, and then only on
  // the tick that commits it or once the current file changed, so a 10/s
  // progress stream does not re-read every row on every tick while it waits.
  const current = files.length > 0 ? files[index] : undefined;
  const hasRow = current !== undefined && entries.has(current);
  if (refreshOnProgress(current, hasRow, payload.ready, progressRefreshedFor)) {
    progressRefreshedFor = current ?? null;
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
  scanErrors = payload.errors;
  scanDone = { scanId: payload.scan_id, total: payload.total };
  scanning = payload.errors === 0 ? null : `${payload.errors} failed`;
  renderMeta();
  strip.refresh();
  if (refreshOnScanDone(scanStarted, payload.scan_id, payload.total)) {
    refreshEntries();
  }
});

// The second pass: the focus candidate state of the files it has written,
// patched into `entries` in place rather than re-read.
void window.__TAURI__.event.listen<{
  dir: string;
  scan_id: number;
  done: number;
  total: number;
  ready: FaceReady[];
}>("faces-progress", ({ payload }) => {
  if (payload.scan_id !== scanId) {
    return;
  }
  scanning = `focus ${payload.done} / ${payload.total}`;
  const current = applyFaceReady(entries, payload.ready, files[index]);
  applyCandidates();
  // The candidate filter fills in as the pass runs; the strip keeps its
  // scroll offset, as on a resync.
  if (shownCandidates.size > 0) {
    refilter(files[index], true);
  }
  renderMeta();
  if (current) {
    draw();
  }
});

void window.__TAURI__.event.listen<{
  dir: string;
  scan_id: number;
  total: number;
  errors: number;
}>("faces-done", ({ payload }) => {
  if (payload.scan_id !== scanId) {
    return;
  }
  setScanRunning(false);
  const failed = scanErrors + payload.errors;
  scanning = failed === 0 ? null : `${failed} failed`;
  renderMeta();
  if (refreshOnFacesDone(scanDone, payload.scan_id, payload.total)) {
    refreshEntries();
  }
  drainResync();
});

// A sidecar the writer could not write: the writer retries it a few times,
// and if it still fails the judgment is still in the index and is retried
// on the next open of the folder, so this is a sticky error, not a revert.
void window.__TAURI__.event.listen<{ path: string; message: string }>(
  "sidecar-error",
  ({ payload }) => {
    if (!allFiles.includes(payload.path)) {
      return;
    }
    errors.add(payload.path, `${baseName(payload.path)}: ${payload.message}`);
    renderMeta();
  },
);

// The settings modal switched the format and the backend has reset the index:
// reopen the folder so the strip and the meta pane show the newly selected
// format's judgments. A fresh token drops any open still in flight.
void window.__TAURI__.event.listen<string>("sidecar-format", () => {
  if (openDir === null) {
    return;
  }
  openDirectory(openDir, newFolderToken()).catch((err: unknown) => {
    setStatus(String(err));
  });
});

// The settings modal cleared the index cache: the open folder's thumbnails
// and cached metadata are gone, so reopen it and let the scan fill them in
// again. A fresh token drops any open still in flight.
void window.__TAURI__.event.listen("index-cleared", () => {
  if (openDir === null) {
    return;
  }
  openDirectory(openDir, newFolderToken()).catch((err: unknown) => {
    setStatus(String(err));
  });
});

// The settings modal's "Timing logs" item toggles this; read the backend's
// `TimingLogs` state too, so a reloaded main window keeps it.
void window.__TAURI__.core.invoke<boolean>("timing_logs").then((enabled) => {
  debugLogging = enabled;
});

// The settings modal's Auto-advance checkbox sets this.
let autoAdvance = false;
void window.__TAURI__.core.invoke<boolean>("auto_advance").then((enabled) => {
  autoAdvance = enabled;
});

const filterToggle = document.getElementById("filter-toggle") as HTMLButtonElement;
const filterMenu = document.getElementById("filter-menu") as HTMLDivElement;
const filterItems = filterMenu.querySelectorAll<HTMLButtonElement>(
  "[data-flag], [data-stars], [data-label], [data-orientation], [data-candidate]",
);
const filterExif = document.getElementById("filter-exif") as HTMLDivElement;

function exifSelected(): boolean {
  return [...shownExif.values()].some((set) => set.size > 0);
}

// Whether any flag, star, label, orientation, candidate or EXIF filter is
// checked.
function filterActive(): boolean {
  return (
    shownFlags.size +
      shownStars.size +
      shownLabels.size +
      shownOrientations.size +
      shownCandidates.size >
      0 || exifSelected()
  );
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
  filterToggle.classList.toggle("active", filterActive());
}

function setFilterMenuOpen(open: boolean): void {
  filterMenu.hidden = !open;
  filterToggle.setAttribute("aria-expanded", String(open));
}

// Mirror the sets onto the menu's check marks and the button's lit state,
// then rebuild the view.
function filterChanged(): void {
  for (const item of filterItems) {
    const { flag, stars, label, orientation, candidate } = item.dataset;
    const checked =
      flag !== undefined
        ? shownFlags.has(flag as Flag)
        : label !== undefined
          ? shownLabels.has(label)
          : orientation !== undefined
            ? shownOrientations.has(orientation as Orientation)
            : candidate !== undefined
              ? shownCandidates.has(candidate as FocusCandidate)
              : shownStars.has(Number(stars));
    item.setAttribute("aria-checked", String(checked));
  }
  for (const item of filterExif.querySelectorAll<HTMLButtonElement>("[data-group]")) {
    const { group, value } = item.dataset;
    item.setAttribute("aria-checked", String(shownExif.get(group as ExifGroup)!.has(value!)));
  }
  filterToggle.classList.toggle("active", filterActive());
  refilter();
}

filterToggle.addEventListener("click", () => {
  // Drop the focus, or the zoom key (the 1:1 toggle) would press it again.
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
    const { flag, stars, label, orientation, candidate } = item.dataset;
    const set: Set<string | number> =
      flag !== undefined
        ? shownFlags
        : label !== undefined
          ? shownLabels
          : orientation !== undefined
            ? shownOrientations
            : candidate !== undefined
              ? shownCandidates
              : shownStars;
    const value = flag ?? label ?? orientation ?? candidate ?? Number(stars);
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
    shownLabels.clear();
    shownOrientations.clear();
    shownCandidates.clear();
    for (const set of shownExif.values()) {
      set.clear();
    }
    filterChanged();
  },
);

const sortToggle = document.getElementById("sort-toggle") as HTMLButtonElement;
const sortMenu = document.getElementById("sort-menu") as HTMLDivElement;
const sortItems = sortMenu.querySelectorAll<HTMLButtonElement>("[data-sort]");

function setSortMenuOpen(open: boolean): void {
  sortMenu.hidden = !open;
  sortToggle.setAttribute("aria-expanded", String(open));
}

sortToggle.addEventListener("click", () => {
  sortToggle.blur();
  setSortMenuOpen(!!sortMenu.hidden);
});

document.addEventListener("mousedown", (event) => {
  if (!sortMenu.hidden && !(event.target as Element).closest("#sort")) {
    setSortMenuOpen(false);
  }
});

for (const item of sortItems) {
  item.addEventListener("click", () => {
    item.blur();
    setSortKey(item.dataset.sort as SortKey);
    void window.__TAURI__.core.invoke("set_sort_order", { order: sortKey });
    setSortMenuOpen(false);
    refilter();
  });
}

function setSortKey(key: SortKey): void {
  sortKey = key;
  for (const item of sortItems) {
    item.setAttribute("aria-checked", String(item.dataset.sort === key));
  }
}

function showFormatDialog(): void {
  formatDialog.hidden = false;
  formatButtons[0].focus();
}

for (const button of formatButtons) {
  button.addEventListener("click", () => {
    for (const b of formatButtons) {
      b.disabled = true;
    }
    formatError.hidden = true;
    window.__TAURI__.core
      .invoke("choose_sidecar_format", { format: button.dataset.format })
      .then(
        () => {
          formatDialog.hidden = true;
          formatGate.open();
        },
        (err: unknown) => {
          formatError.textContent = String(err);
          formatError.hidden = false;
        },
      )
      .finally(() => {
        for (const b of formatButtons) {
          b.disabled = false;
        }
        if (!formatDialog.hidden) {
          button.focus();
        }
      });
  });
}

// Ask for the developing software while no sidecar format is saved. A failed
// check lets folders open, as the backend does for a store it cannot open.
void window.__TAURI__.core.invoke<boolean>("sidecar_format_saved").then(
  (saved) => {
    if (saved) {
      formatGate.open();
    } else {
      showFormatDialog();
    }
  },
  () => {
    formatGate.open();
  },
);
const side = document.getElementById("side") as HTMLElement;
const film = document.getElementById("film") as HTMLElement;
const info = document.getElementById("info") as HTMLElement;
let panels: Panels = { left: true, strip: true, right: true };

// Show and hide the panes, then fit the viewer to the space they leave. The
// strip rendered nothing while hidden (its width was 0), so it renders again
// around the current cell when it comes back.
function applyPanels(next: Panels): void {
  const stripShown = next.strip && !panels.strip;
  panels = next;
  side.hidden = !panels.left;
  film.hidden = !panels.strip;
  info.hidden = !panels.right;
  if (!panels.strip) {
    setFilterMenuOpen(false);
    setSortMenuOpen(false);
  }
  if (stripShown) {
    strip.setCurrent(index);
  }
  draw();
  if (zoomed) {
    scheduleCropForResize();
  }
}

function changePanels(next: Panels): void {
  applyPanels(next);
  // A hidden pane must not keep the keyboard, or the culling keys stay gated.
  if (!panels.left) {
    folders.blur();
  }
  void window.__TAURI__.core.invoke("set_panels", { panels });
}

void window.__TAURI__.core.invoke<Panels>("panels").then(applyPanels, () => {});

// Apply the remembered sort before the last folder opens, so it comes up in
// that order.
const sortLoaded = window.__TAURI__.core
  .invoke<SortKey>("sort_order")
  .then(setSortKey, () => {})
  .finally(() => {
    formatGate.whenOpen(reopenLastFolder);
  });

// `Settings...` in the menu. The first-launch dialog is modal already, so the
// settings wait until it is answered.
void window.__TAURI__.event.listen("open-settings", () => {
  if (!formatDialog.hidden || sequenceFlow.busy) {
    return;
  }
  setFilterMenuOpen(false);
  setSortMenuOpen(false);
  closeContextMenu();
  settings.open();
});

// Key -> action, from the `shortcuts` command. Empty until it resolves.
let keymap = new Map<string, string>();

function applyKeymap(bindings: Binding[]): void {
  keymap = new Map(
    bindings.flatMap(({ action, keys }) => keys.map((key) => [key, action] as const)),
  );
  keyBindings = bindings;
  renderEmpty();
}

const keymapLoaded = window.__TAURI__.core.invoke<Binding[]>("shortcuts").then(applyKeymap);

// The tree's roots come after the keymap and the sort order, which the first
// frame needs more.
void Promise.allSettled([sortLoaded, keymapLoaded]).then(folders.loadRoots);

window.addEventListener("keydown", (event) => {
  // The dialog's buttons take Enter and Space natively, but Tab would move
  // focus past them to controls behind the overlay (there is no `inert` on
  // the `safari13` target), so trap it by cycling within `formatButtons`.
  if (!formatDialog.hidden) {
    if (event.key === "Tab") {
      event.preventDefault();
      const from = formatButtons.indexOf(document.activeElement as HTMLButtonElement);
      const delta = event.shiftKey ? -1 : 1;
      const next = (from + delta + formatButtons.length) % formatButtons.length;
      formatButtons[next].focus();
    }
    return;
  }
  if (settings.isOpen) {
    settings.keydown(event);
    return;
  }
  if (sequenceFlow.isOpen) {
    sequenceKeydown(event);
    return;
  }
  const key = keyName(event);
  if (key === null) {
    return;
  }
  // Ahead of the tree, whose own `Escape` would hand the keys back instead.
  if (!contextMenu.hidden) {
    closeContextMenu();
    if (key === "escape") {
      event.preventDefault();
      return;
    }
  }
  if (folders.hasFocus()) {
    if (folders.keydown(event)) {
      return;
    }
    const action = keymap.get(key);
    if (treeGate(action) === "swallow") {
      if (action !== undefined) {
        event.preventDefault();
      }
      return;
    }
  }
  if (key === "escape" && !filterMenu.hidden) {
    setFilterMenuOpen(false);
    event.preventDefault();
    return;
  }
  if (key === "escape" && !sortMenu.hidden) {
    setSortMenuOpen(false);
    event.preventDefault();
    return;
  }
  if (key === "escape" && comparing) {
    toggleCompare();
    event.preventDefault();
    return;
  }
  const action = keymap.get(key);
  if (action === "grayscale") {
    setGrayscale(event.code);
    event.preventDefault();
    return;
  }
  if (action !== undefined && runAction(action)) {
    event.preventDefault();
  }
});

// Match the physical key: with a modified binding, whichever key goes up first ends the hold.
window.addEventListener("keyup", (event) => {
  if (grayscaleHeld !== null && (event.code === grayscaleHeld || isModifierCode(event.code))) {
    setGrayscale(null);
  }
});

window.addEventListener("blur", () => {
  setGrayscale(null);
});

function setGrayscale(code: string | null): void {
  grayscaleHeld = code;
  canvas.classList.toggle("grayscale", code !== null);
}

function runAction(action: string): boolean {
  const current = files[index];
  let judged = 0;
  switch (action) {
    case "previous":
      move(-1);
      break;
    case "next":
      move(1);
      break;
    case "burstPrevious":
      moveBurst(-1);
      break;
    case "burstNext":
      moveBurst(1);
      break;
    case "burstFramePrevious":
      moveBurstFrame(-1);
      break;
    case "burstFrameNext":
      moveBurstFrame(1);
      break;
    case "extendPrevious":
      extendSelection(-1);
      break;
    case "extendNext":
      extendSelection(1);
      break;
    case "selectAll":
      selectAllFiles();
      break;
    case "focus":
      showFocus = !showFocus;
      draw();
      break;
    case "zoom":
      toggleZoom();
      break;
    case "compare":
      toggleCompare();
      break;
    case "open":
      openFolder();
      break;
    case "undo":
      undo();
      break;
    case "redo":
      redo();
      break;
    case "toggleLeft":
      changePanels(toggle(panels, "left"));
      break;
    case "toggleRight":
      changePanels(toggle(panels, "right"));
      break;
    case "toggleStrip":
      changePanels(toggle(panels, "strip"));
      break;
    case "toggleSides":
      changePanels(toggleSides(panels));
      break;
    case "rate1":
    case "rate2":
    case "rate3":
    case "rate4":
    case "rate5": {
      const stars = Number(action.slice(-1));
      judged = judge(() => (own) => ({ ...own, rating: stars }));
      break;
    }
    case "reject":
      // Sticky, not a toggle: reject twice is still a reject, and it replaces
      // a pick and keeps the stars. Unflag undoes it.
      judged = judge(() => (own) => ({ ...own, flag: "reject" }));
      break;
    case "rejectRest":
      rejectRest();
      break;
    case "pick":
      // Sticky like reject, replacing a reject and keeping the stars.
      judged = judge(() => (own) => ({ ...own, flag: "pick" }));
      break;
    case "unflag":
      // Clears a reject or a pick and keeps the stars; does nothing to a file
      // with neither.
      judge(() => (own) => ({ ...own, flag: "none" }));
      break;
    case "clear":
      // Clears the stars and leaves the flag alone.
      judge(() => (own) => ({ ...own, rating: null }));
      break;
    case "red":
    case "orange":
    case "yellow":
    case "green":
    case "blue":
    case "pink":
    case "purple": {
      // Toggles: the same color again clears it; another color replaces it.
      const name = action.charAt(0).toUpperCase() + action.slice(1);
      judge((focused) => {
        const label = focused.label === name ? null : name;
        return (own) => ({ ...own, label });
      });
      break;
    }
    case "clearlabel":
      judge(() => (own) => ({ ...own, label: null }));
      break;
    case "clearall":
      // Clears the stars, the flag and the label in one undo entry.
      judge(() => () => ({ rating: null, flag: "none", label: null }), true);
      break;
    default:
      return false;
  }
  // A file that dropped out of the filter already moved the cursor on.
  if (
    judged === 1 &&
    autoAdvance &&
    !comparing &&
    advancesAfter(action) &&
    files[index] === current
  ) {
    move(1);
  }
  return true;
}

window.addEventListener("resize", () => {
  draw();
  if (zoomed) {
    scheduleCropForResize();
  }
});

renderMeta();
draw();
