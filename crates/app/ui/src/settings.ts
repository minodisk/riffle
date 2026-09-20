import { type Binding, keyName } from "./keys.js";
import { nextTab } from "./tabs.js";

const shortcutLabels: Record<string, string> = {
  previous: "Previous",
  next: "Next",
  open: "Open in DxO PhotoLab",
  focus: "Focus mark",
  zoom: "1:1 zoom",
  rate1: "1 star",
  rate2: "2 stars",
  rate3: "3 stars",
  rate4: "4 stars",
  rate5: "5 stars",
  reject: "Reject",
  pick: "Pick",
  unflag: "Un-reject / un-pick",
  clear: "Clear",
  red: "Red label",
  orange: "Orange label",
  yellow: "Yellow label",
  green: "Green label",
  blue: "Blue label",
  pink: "Pink label",
  purple: "Purple label",
  clearlabel: "Clear label",
};

const shortcutsRows = document.getElementById("shortcuts-rows") as HTMLTableElement;
const status = document.getElementById("status") as HTMLDivElement;
const sidecarRadios = document.querySelectorAll<HTMLInputElement>('input[name="sidecar-format"]');
const autoAdvance = document.getElementById("auto-advance") as HTMLInputElement;
const debugTiming = document.getElementById("debug-timing") as HTMLInputElement;
const tablist = document.getElementById("tabs") as HTMLDivElement;
const tabs = [...tablist.querySelectorAll<HTMLButtonElement>('[role="tab"]')];
let shortcutBindings: Binding[] = [];
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
      const display = (key: string) => (key === "space" ? "Space" : key);
      const resetCell = document.createElement("td");
      row.append(label, keysCell, resetCell);
      for (const key of keys) {
        const chip = document.createElement("span");
        chip.className = "chip";
        chip.textContent = display(key);
        const remove = document.createElement("button");
        remove.type = "button";
        remove.textContent = "×";
        remove.setAttribute("aria-label", `Remove ${display(key)}`);
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
