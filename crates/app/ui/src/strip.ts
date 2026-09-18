// The thumbnail filmstrip down the left edge: a hand-rolled virtual list over
// the cached thumbnails the `thumbnail` command serves.

// Header layout of a `thumbnail` payload, see `crates/app/src/commands.rs`.
const THUMBNAIL_HEADER_LEN = 8;
const THUMBNAIL_KIND_JPEG_V1 = 2;

const strip = document.getElementById("strip") as HTMLDivElement;
const inner = document.getElementById("strip-inner") as HTMLDivElement;

// Cell geometry. The strip column is 160px wide; a cell is a 144x96 image box
// plus the file name, and 96x144 upright after a quarter turn. Read from the
// `--cell-height` custom property in `style.css` (the single source of truth
// for cell placement) rather than duplicated here. A fixed height keeps
// `scrollTop -> index` arithmetic, which is what makes virtualisation cheap.
const CELL_HEIGHT = Number.parseFloat(
  getComputedStyle(inner).getPropertyValue("--cell-height"),
);
// Cells kept beyond the visible range, so a short scroll shows an image that
// is already decoded.
const RANGE_MARGIN = 4;
// Concurrent `thumbnail` invokes. The IPC hop, not the decode, is the cost.
const MAX_IN_FLIGHT = 4;
// Shortest gap between two `refresh` runs. `scan-progress` arrives 10/s and
// each run re-requests every visible cell the scan has not reached yet, which
// kept `MAX_IN_FLIGHT` invokes outstanding for the whole scan and starved the
// rest of the IPC channel. The event carries only `done`/`total`, not which
// files became available, so the re-request cannot be narrowed; throttle it.
const REFRESH_INTERVAL = 1000;

interface Cell {
  el: HTMLDivElement;
  img: HTMLImageElement;
  badge: HTMLSpanElement;
  flag: HTMLSpanElement;
  url: string | null;
}

let files: string[] = [];
let current = 0;
// Bumped on every `setFiles`; a response tagged with an older generation
// belongs to a folder that is no longer open and is dropped.
let generation = 0;
const cells = new Map<number, Cell>();
// Indices with a request in flight or already answered, so a cell scrolling
// back into view does not ask again.
const requested = new Set<number>();
// Indices with an `invoke` currently in flight. Unlike `requested`, `render`
// never clears this, so a cell that scrolls out and back in while its
// request is still pending does not issue a second `invoke`.
const inFlightIndices = new Set<number>();
// Indices the index has no thumbnail for yet (the scan has not reached them,
// or the file errored). Cleared on `refresh` so a `scan-progress` event
// re-requests them.
const missing = new Set<number>();
// When the last `refresh` ran, and the trailing timer for a `refresh` that
// arrived inside `REFRESH_INTERVAL` of it. The trailing run is what keeps the
// authoritative `scan-done` refresh from being dropped.
let lastRefresh = 0;
let refreshTimer: number | null = null;
// Indices whose request failed for a reason other than "not yet scanned".
// Kept separate from `missing` so a real failure is shown once instead of
// being retried forever like a not-yet-scanned file.
const failed = new Set<number>();
// The judgement per index, mirroring the `ratings` map in `main.ts`: `-1` is
// a reject, `1`-`5` stars, and a missing entry is unrated.
const ratings = new Map<number, number>();
// The picked indices, mirroring the `picks` set in `main.ts`.
const picks = new Set<number>();
let inFlight = 0;
let select: (index: number) => void = () => {};

// A rated cell carries its stars in the top-right corner. A picked or
// rejected cell carries a dot in the top-left corner, green or red (the two
// never coexist), and a rejected cell is also dimmed.
function paintRating(index: number, cell: Cell): void {
  const rating = ratings.get(index);
  const rejected = rating === -1;
  const picked = picks.has(index);
  cell.badge.textContent =
    rating === undefined || rejected ? "" : "\u2605".repeat(rating);
  cell.el.classList.toggle("rejected", rejected);
  cell.flag.textContent = picked || rejected ? "\u25CF" : "";
  cell.flag.classList.toggle("pick", picked);
  cell.flag.classList.toggle("reject", rejected);
}

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
  const badge = document.createElement("span");
  badge.className = "rating";
  el.append(badge);
  const flag = document.createElement("span");
  flag.className = "flag";
  el.append(flag);
  el.addEventListener("click", () => {
    select(index);
  });
  inner.append(el);
  const cell: Cell = { el, img, badge, flag, url: null };
  paintRating(index, cell);
  return cell;
}

function highlight(): void {
  for (const [index, cell] of cells) {
    cell.el.classList.toggle("current", index === current);
    cell.el.classList.toggle("failed", failed.has(index));
  }
}

// The cell nearest the middle of the viewport is wanted first, so scrolling
// fast fills what the user is looking at rather than what it flew past.
function pickNext(): number | null {
  const centre = (strip.scrollTop + strip.clientHeight / 2) / CELL_HEIGHT;
  let best: number | null = null;
  let bestDistance = Infinity;
  for (const index of cells.keys()) {
    if (
      requested.has(index) ||
      inFlightIndices.has(index) ||
      missing.has(index) ||
      failed.has(index)
    ) {
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
  inFlightIndices.add(index);
  inFlight += 1;
  const currentGeneration = generation;
  window.__TAURI__.core
    .invoke<ArrayBuffer>("thumbnail", { path: files[index] })
    .then((payload) => {
      const cell = cells.get(index);
      if (currentGeneration !== generation || cell === undefined) {
        return;
      }
      requested.add(index);
      const header = new DataView(payload, 0, THUMBNAIL_HEADER_LEN);
      const kind = header.getUint16(0, true);
      if (kind !== THUMBNAIL_KIND_JPEG_V1) {
        throw new Error(`unknown thumbnail payload kind ${kind}`);
      }
      const orientation = header.getUint16(2, true);
      const blob = new Blob([payload.slice(THUMBNAIL_HEADER_LEN)], {
        type: "image/jpeg",
      });
      if (cell.url !== null) {
        URL.revokeObjectURL(cell.url);
      }
      cell.url = URL.createObjectURL(blob);
      cell.img.src = cell.url;
      cell.img.className =
        orientation === 6
          ? "cw"
          : orientation === 8
            ? "ccw"
            : orientation === 3
              ? "half"
              : "";
    })
    .catch((err: unknown) => {
      // A response for a folder that is no longer open: bookkeeping below
      // belongs to the new folder's `setFiles` reset (or to a re-request
      // that has already replaced this entry), so leave it alone.
      if (currentGeneration !== generation) {
        return;
      }
      requested.delete(index);
      // The index row exists but its `thumb` column is NULL only once the
      // file has actually been processed and failed (see
      // `Index::thumbnail` in `crates/app/src/index.rs`), so this message
      // means a permanent failure rather than "not yet scanned". Record it
      // separately so it is shown once instead of retried on every
      // `refresh`.
      if (String(err).includes("no cached thumbnail")) {
        failed.add(index);
      } else {
        missing.add(index);
      }
    })
    .finally(() => {
      inFlight -= 1;
      if (currentGeneration === generation) {
        inFlightIndices.delete(index);
        highlight();
      }
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

// Record the judgement of one file, repainting its cell when it is on screen.
// `null` is unrated.
export function setRating(index: number, rating: number | null, pick: boolean): void {
  if (rating === null) {
    ratings.delete(index);
  } else {
    ratings.set(index, rating);
  }
  if (pick) {
    picks.add(index);
  } else {
    picks.delete(index);
  }
  const cell = cells.get(index);
  if (cell !== undefined) {
    paintRating(index, cell);
  }
}

// Show one cell per file, in `list_arw` order, all of them placeholders.
export function setFiles(paths: string[]): void {
  generation += 1;
  for (const cell of cells.values()) {
    releaseCell(cell);
  }
  cells.clear();
  requested.clear();
  inFlightIndices.clear();
  missing.clear();
  failed.clear();
  if (refreshTimer !== null) {
    clearTimeout(refreshTimer);
    refreshTimer = null;
  }
  lastRefresh = 0;
  ratings.clear();
  picks.clear();
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
// has made progress. Throttled to one run per `REFRESH_INTERVAL`, with a
// trailing run so the last caller still takes effect.
export function refresh(): void {
  const wait = lastRefresh + REFRESH_INTERVAL - Date.now();
  if (wait <= 0) {
    lastRefresh = Date.now();
    missing.clear();
    pump();
    return;
  }
  if (refreshTimer === null) {
    refreshTimer = setTimeout(() => {
      refreshTimer = null;
      refresh();
    }, wait);
  }
}

export function init(onSelect: (index: number) => void): void {
  select = onSelect;
  strip.addEventListener("scroll", render);
  window.addEventListener("resize", render);
}
