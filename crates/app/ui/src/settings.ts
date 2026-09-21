import { type Binding, displayKey, keyName } from "./keys.js";
import { nextTab } from "./tabs.js";

const shortcutLabels: Record<string, string> = {
  previous: "Previous",
  next: "Next",
  burstPrevious: "Previous burst",
  burstNext: "Next burst",
  burstFramePrevious: "Previous frame in burst",
  burstFrameNext: "Next frame in burst",
  extendPrevious: "Extend selection up",
  extendNext: "Extend selection down",
  open: "Open folder",
  photolab: "Open in DxO PhotoLab",
  focus: "Focus mark",
  zoom: "1:1 zoom",
  rate1: "1 star",
  rate2: "2 stars",
  rate3: "3 stars",
  rate4: "4 stars",
  rate5: "5 stars",
  reject: "Reject",
  rejectRest: "Reject the rest of the burst",
  pick: "Pick",
  unflag: "Un-reject / un-pick",
  clear: "0 star",
  red: "Red label",
  orange: "Orange label",
  yellow: "Yellow label",
  green: "Green label",
  blue: "Blue label",
  pink: "Pink label",
  purple: "Purple label",
  clearlabel: "Clear label",
  clearall: "Clear",
};

const shortcutsRows = document.getElementById("shortcuts-rows") as HTMLTableElement;
const status = document.getElementById("status") as HTMLDivElement;
const sidecarRadios = document.querySelectorAll<HTMLInputElement>('input[name="sidecar-format"]');
const autoAdvance = document.getElementById("auto-advance") as HTMLInputElement;
const debugTiming = document.getElementById("debug-timing") as HTMLInputElement;
const indexSize = document.getElementById("index-size") as HTMLParagraphElement;
const clearIndex = document.getElementById("clear-index") as HTMLButtonElement;
const clearIndexNote = document.getElementById("clear-index-note") as HTMLParagraphElement;
const tablist = document.getElementById("tabs") as HTMLDivElement;
const tabs = [...tablist.querySelectorAll<HTMLButtonElement>('[role="tab"]')];
let shortcutBindings: Binding[] = [];
let scanRunning = false;
let clearInFlight = false;
// Set by the first `scan-state`, so the initial `scan_running` answer is not
// applied over a newer state that arrived while it was in flight.
let scanStateSeen = false;
// The action whose row waits for a key.
let capturing: string | null = null;

function renderShortcuts(): void {
  shortcutsRows.replaceChildren(
    ...shortcutBindings.map(({ action, keys }) => {
      const row = document.createElement("tr");
      const label = document.createElement("td");
      label.textContent = shortcutLabels[action] ?? action;
      const keysCell = document.createElement("td");
      keysCell.className = "keys";
      const resetCell = document.createElement("td");
      row.append(label, keysCell, resetCell);
      for (const key of keys) {
        const chip = document.createElement("span");
        chip.className = "chip";
        chip.textContent = displayKey(key);
        const remove = document.createElement("button");
        remove.type = "button";
        remove.textContent = "×";
        remove.setAttribute("aria-label", `Remove ${displayKey(key)}`);
        remove.addEventListener("click", () => {
          void updateShortcuts("remove_shortcut_key", { action, key });
        });
        chip.append(remove);
        keysCell.append(chip);
      }
      if (capturing === action) {
        const prompt = document.createElement("span");
        prompt.className = "capturing";
        prompt.textContent = "Press a key...";
        keysCell.append(prompt);
      } else {
        const add = document.createElement("button");
        add.type = "button";
        add.className = "add";
        add.textContent = "+";
        add.setAttribute("aria-label", "Add a key");
        add.addEventListener("click", () => {
          capturing = action;
          status.textContent = "";
          renderShortcuts();
        });
        keysCell.append(add);
      }
      const reset = document.createElement("button");
      reset.type = "button";
      reset.textContent = "Reset";
      reset.addEventListener("click", () => {
        void updateShortcuts("reset_shortcut", { action });
      });
      resetCell.append(reset);
      return row;
    }),
  );
}

// The backend also emits `shortcuts-changed`, which the main window culls by.
async function updateShortcuts(command: string, args?: Record<string, unknown>): Promise<void> {
  capturing = null;
  try {
    shortcutBindings = await window.__TAURI__.core.invoke<Binding[]>(command, args);
    status.textContent = "";
  } catch (error) {
    status.textContent = String(error);
  }
  renderShortcuts();
}

function showSidecarFormat(format: string): void {
  for (const radio of sidecarRadios) {
    radio.checked = radio.value === format;
  }
}

void window.__TAURI__.core.invoke<Binding[]>("shortcuts").then((bindings) => {
  shortcutBindings = bindings;
  renderShortcuts();
});
void window.__TAURI__.core.invoke<string>("sidecar_format").then(showSidecarFormat);
void window.__TAURI__.event.listen<string>("sidecar-format", ({ payload }) => {
  showSidecarFormat(payload);
});
void window.__TAURI__.core.invoke<boolean>("auto_advance").then((enabled) => {
  autoAdvance.checked = enabled;
});
void window.__TAURI__.event.listen<boolean>("auto-advance", ({ payload }) => {
  autoAdvance.checked = payload;
});
void window.__TAURI__.core.invoke<boolean>("debug_build").then((debug) => {
  (document.getElementById("tab-debug") as HTMLButtonElement).hidden = !debug;
});
void window.__TAURI__.core.invoke<boolean>("timing_logs").then((enabled) => {
  debugTiming.checked = enabled;
});

// The backend formats the size in the units of the platform's file manager.
function showIndexSize(size: string): void {
  indexSize.textContent = `Index cache: ${size}`;
}

function showIndexClearing(): void {
  indexSize.textContent = "Clearing the index cache…";
}

void window.__TAURI__.core.invoke<string>("index_size").then(showIndexSize);

function updateClearButton(): void {
  clearIndex.disabled = scanRunning || clearInFlight;
  clearIndexNote.hidden = !scanRunning;
}

void window.__TAURI__.core.invoke<boolean>("scan_running").then((running) => {
  if (scanStateSeen) {
    return;
  }
  scanRunning = running;
  updateClearButton();
});
void window.__TAURI__.event.listen<boolean>("scan-state", ({ payload }) => {
  scanStateSeen = true;
  const scanEnded = scanRunning && !payload;
  scanRunning = payload;
  if (scanEnded && !clearInFlight) {
    void window.__TAURI__.core.invoke<string>("index_size").then((size) => {
      if (!clearInFlight) showIndexSize(size);
    });
  }
  updateClearButton();
});
void window.__TAURI__.event.listen("index-clearing", () => {
  showIndexClearing();
});

for (const radio of sidecarRadios) {
  radio.addEventListener("change", () => {
    status.textContent = "";
    window.__TAURI__.core
      .invoke("set_sidecar_format", { format: radio.value })
      .catch((error: unknown) => {
        status.textContent = String(error);
      });
  });
}

autoAdvance.addEventListener("change", () => {
  status.textContent = "";
  window.__TAURI__.core
    .invoke("set_auto_advance", { enabled: autoAdvance.checked })
    .catch((error: unknown) => {
      status.textContent = String(error);
    });
});

debugTiming.addEventListener("change", () => {
  void window.__TAURI__.core.invoke("set_timing_logs", { enabled: debugTiming.checked });
});

clearIndex.addEventListener("click", () => {
  status.textContent = "";
  clearInFlight = true;
  updateClearButton();
  window.__TAURI__.core
    .invoke<boolean>("clear_index")
    .then(async (cleared) => {
      if (cleared) {
        showIndexSize(await window.__TAURI__.core.invoke<string>("index_size"));
      }
    })
    .catch(async (error: unknown) => {
      status.textContent = String(error);
      // The clear can fail after `index-clearing`, so put the figure back.
      showIndexSize(await window.__TAURI__.core.invoke<string>("index_size"));
    })
    .finally(() => {
      clearInFlight = false;
      updateClearButton();
    });
});

(document.getElementById("shortcuts-reset-all") as HTMLButtonElement).addEventListener(
  "click",
  () => {
    void updateShortcuts("reset_shortcuts");
  },
);

// While a row captures, the next key is added to it; Escape cancels.
window.addEventListener("keydown", (event) => {
  if (capturing === null) {
    return;
  }
  const key = keyName(event);
  if (key === null) {
    return;
  }
  event.preventDefault();
  if (key === "escape") {
    capturing = null;
    renderShortcuts();
    return;
  }
  void updateShortcuts("add_shortcut_key", { action: capturing, key });
});

function selectTab(panel: string): void {
  if (capturing !== null) {
    capturing = null;
    renderShortcuts();
  }
  for (const tab of tabs) {
    const selected = tab.getAttribute("aria-controls") === panel;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
    (document.getElementById(tab.getAttribute("aria-controls") ?? "") as HTMLElement).hidden =
      !selected;
    if (selected) {
      tab.focus();
    }
  }
}

for (const tab of tabs) {
  tab.addEventListener("click", () => {
    selectTab(tab.getAttribute("aria-controls") ?? "");
  });
}

tablist.addEventListener("keydown", (event) => {
  const visible = tabs
    .filter((tab) => !tab.hidden)
    .map((tab) => tab.getAttribute("aria-controls") ?? "");
  const current =
    tabs
      .find((tab) => tab.getAttribute("aria-selected") === "true")
      ?.getAttribute("aria-controls") ?? "";
  const target = nextTab(visible, current, event.key);
  if (target !== current) {
    event.preventDefault();
    selectTab(target);
  }
});
