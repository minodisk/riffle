import * as strip from "./strip.js";

// Header layout of a `preview` payload, see `crates/app/src/commands.rs`.
const PREVIEW_HEADER_LEN = 8;
const PREVIEW_KIND_JPEG_V1 = 1;

interface DecodeResponse {
  seq: number;
  bitmap?: ImageBitmap;
  error?: string;
}

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;
const statusEl = document.getElementById("status") as HTMLSpanElement;
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
// How far the current scan got, or null when nothing is scanning. Events
// carry the id of the scan that emitted them; only events whose id matches
// `scanId` are applied, so the stragglers of a cancelled scan (including one
// cancelled by reopening the very same folder) are ignored even though they
// carry the same `dir`.
let scanId: number | null = null;
let scanning: string | null = null;
// Bumped at the start of every `openFolder`, so that when two overlapping
// `openFolder` calls race, the `list_arw` result of the one that is no
// longer current (e.g. A resolves after B was opened) is dropped instead of
// overwriting `files` with a stale folder's contents.
let folderToken = 0;

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

function setStatus(extra?: string): void {
  const parts: string[] = [];
  if (files.length === 0) {
    parts.push(extra ?? "No ARW files in that folder.");
  } else {
    parts.push(`${index + 1} / ${files.length}  ${baseName(files[index])}`);
    if (extra !== undefined) {
      parts.push(extra);
    }
  }
  if (scanning !== null) {
    parts.push(scanning);
  }
  statusEl.textContent = parts.join("  —  ");
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
  }
  context.drawImage(
    bitmap,
    -drawWidth / 2,
    -drawHeight / 2,
    drawWidth,
    drawHeight,
  );
  context.restore();
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

function show(): void {
  seq += 1;
  strip.setCurrent(index);
  setStatus();
  requestPreview();
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

function openFolder(): void {
  folderToken += 1;
  const token = folderToken;
  window.__TAURI__.core
    .invoke<string | null>("pick_folder")
    .then((folder) => {
      if (folder === null) {
        return;
      }
      return window.__TAURI__.core
        .invoke<string[]>("list_arw", { dir: folder })
        .then((found) => {
          if (token !== folderToken) {
            return;
          }
          files = found;
          index = 0;
          strip.setFiles(files);
          seq += 1;
          shown?.bitmap.close();
          shown = null;
          draw();
          scanning = null;
          scanId = null;
          void window.__TAURI__.core
            .invoke<{ total: number; scan_id: number }>("scan_folder", { dir: folder })
            .then(({ scan_id }) => {
              scanId = scan_id;
              return window.__TAURI__.core.invoke("start_scan", { scanId: scan_id });
            })
            .catch((err: unknown) => {
              setStatus(String(err));
            });
          if (files.length === 0) {
            setStatus();
            return;
          }
          show();
        });
    })
    .catch((err: unknown) => {
      setStatus(String(err));
    });
}

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
  setStatus();
  strip.refresh();
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
  setStatus();
  strip.refresh();
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
  } else if (key === "o") {
    openFolder();
  } else {
    return;
  }
  event.preventDefault();
});

window.addEventListener("resize", draw);

draw();

export {};
