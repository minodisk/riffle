// The thumbnail filmstrip down the left edge: a hand-rolled virtual list over
// the cached thumbnails the `thumbnail` command serves.

// Header layout of a `thumbnail` payload, see `crates/app/src/commands.rs`.
const THUMBNAIL_HEADER_LEN = 8;
const THUMBNAIL_KIND_JPEG_V1 = 2;

// Cell geometry. The strip column is 160px wide; a cell is a 144x144 image box
// plus the file name. Both orientations fit that box: the thumbnail is
// 404x270, so a quarter turn is 144x96 upright. A fixed height keeps
// `scrollTop -> index` arithmetic, which is what makes virtualisation cheap.
const CELL_HEIGHT = 176;
// Cells kept beyond the visible range, so a short scroll shows an image that
// is already decoded.
const RANGE_MARGIN = 4;
// Concurrent `thumbnail` invokes. The IPC hop, not the decode, is the cost.
const MAX_IN_FLIGHT = 4;

interface Cell {
  el: HTMLDivElement;
  img: HTMLImageElement;
  url: string | null;
}

const strip = document.getElementById("strip") as HTMLDivElement;
const inner = document.getElementById("strip-inner") as HTMLDivElement;

let files: string[] = [];
let current = 0;
// Bumped on every `setFiles`; a response tagged with an older generation
// belongs to a folder that is no longer open and is dropped.
let generation = 0;
const cells = new Map<number, Cell>();
// Indices with a request in flight or already answered, so a cell scrolling
// back into view does not ask again.
const requested = new Set<number>();
// Indices the index has no thumbnail for yet (the scan has not reached them,
// or the file errored). Cleared on `refresh` so a `scan-progress` event
// re-requests them.
const missing = new Set<number>();
let inFlight = 0;
let select: (index: number) => void = () => {};

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

function releaseCell(cell: Cell): void {
  if (cell.url !== null) {
    URL.revokeObjectURL(cell.url);
    cell.url = null;
  }
  cell.el.remove();
}

function createCell(index: number): Cell {
  const el = document.createElement("div");
  el.className = "cell";
  el.style.top = `${index * CELL_HEIGHT}px`;
  const img = document.createElement("img");
  el.append(img);
  const name = document.createElement("span");
  name.textContent = baseName(files[index]);
  el.append(name);
  el.addEventListener("click", () => {
    select(index);
  });
  inner.append(el);
  return { el, img, url: null };
}

function highlight(): void {
  for (const [index, cell] of cells) {
    cell.el.classList.toggle("current", index === current);
  }
}

// The cell nearest the middle of the viewport is wanted first, so scrolling
// fast fills what the user is looking at rather than what it flew past.
function pickNext(): number | null {
  const centre = (strip.scrollTop + strip.clientHeight / 2) / CELL_HEIGHT;
  let best: number | null = null;
  let bestDistance = Infinity;
  for (const index of cells.keys()) {
    if (requested.has(index) || missing.has(index)) {
      continue;
    }
    const distance = Math.abs(index - centre);
    if (distance < bestDistance) {
      best = index;
      bestDistance = distance;
    }
  }
  return best;
}

function request(index: number): void {
  requested.add(index);
  inFlight += 1;
  const currentGeneration = generation;
  window.__TAURI__.core
    .invoke<ArrayBuffer>("thumbnail", { path: files[index] })
    .then((payload) => {
      inFlight -= 1;
      const cell = cells.get(index);
      if (currentGeneration !== generation || cell === undefined) {
        pump();
        return;
      }
      const header = new DataView(payload, 0, THUMBNAIL_HEADER_LEN);
      const kind = header.getUint16(0, true);
      if (kind !== THUMBNAIL_KIND_JPEG_V1) {
        throw new Error(`unknown thumbnail payload kind ${kind}`);
      }
      const orientation = header.getUint16(2, true);
      const blob = new Blob([payload.slice(THUMBNAIL_HEADER_LEN)], {
        type: "image/jpeg",
      });
      cell.url = URL.createObjectURL(blob);
      cell.img.src = cell.url;
      cell.img.className =
        orientation === 6 ? "cw" : orientation === 8 ? "ccw" : "";
      pump();
    })
    .catch(() => {
      // No row yet, or the file errored during the scan: leave the
      // placeholder and let the next `refresh` ask again.
      inFlight -= 1;
      requested.delete(index);
      missing.add(index);
      pump();
    });
}

function pump(): void {
  while (inFlight < MAX_IN_FLIGHT) {
    const next = pickNext();
    if (next === null) {
      return;
    }
    request(next);
  }
}

function render(): void {
  const first = Math.max(
    0,
    Math.floor(strip.scrollTop / CELL_HEIGHT) - RANGE_MARGIN,
  );
  const last = Math.min(
    files.length - 1,
    Math.ceil((strip.scrollTop + strip.clientHeight) / CELL_HEIGHT) +
      RANGE_MARGIN,
  );
  for (const [index, cell] of cells) {
    if (index < first || index > last) {
      releaseCell(cell);
      cells.delete(index);
      // The bytes are gone with the object URL, so a cell coming back has to
      // ask again.
      requested.delete(index);
    }
  }
  for (let index = first; index <= last; index += 1) {
    if (!cells.has(index)) {
      cells.set(index, createCell(index));
    }
  }
  highlight();
  pump();
}

// Show one cell per file, in `list_arw` order, all of them placeholders.
export function setFiles(paths: string[]): void {
  generation += 1;
  for (const cell of cells.values()) {
    releaseCell(cell);
  }
  cells.clear();
  requested.clear();
  missing.clear();
  files = paths;
  current = 0;
  inner.style.height = `${files.length * CELL_HEIGHT}px`;
  strip.scrollTop = 0;
  render();
}

// Highlight `index` and scroll it into view, the `block: "nearest"` way.
export function setCurrent(index: number): void {
  current = index;
  const top = index * CELL_HEIGHT;
  const bottom = top + CELL_HEIGHT;
  if (top < strip.scrollTop) {
    strip.scrollTop = top;
  } else if (bottom > strip.scrollTop + strip.clientHeight) {
    strip.scrollTop = bottom - strip.clientHeight;
  }
  render();
}

// Ask again for the visible cells the index had nothing for, after the scan
// has made progress.
export function refresh(): void {
  missing.clear();
  pump();
}

export function init(onSelect: (index: number) => void): void {
  select = onSelect;
  strip.addEventListener("scroll", render);
  window.addEventListener("resize", render);
}
