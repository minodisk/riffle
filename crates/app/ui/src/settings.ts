import { type Binding, displayKey, keyName } from "./keys.js";
import { LABEL_COLORS, type LabelNames, englishLabelNames, labelNamesPayload } from "./labels.js";
import { type McpState, mcpEndpoint, mcpExamples, mcpStatus } from "./mcp.js";
import { SettingsModal, cycleFocus } from "./modal.js";
import { nextTab } from "./tabs.js";

const shortcutLabels: Record<string, string> = {
  previous: "Previous",
  next: "Next",
  burstPrevious: "Previous burst",
  burstNext: "Next burst",
  burstFramePrevious: "Previous frame in burst",
  burstFrameNext: "Next frame in burst",
  extendPrevious: "Extend selection left",
  extendNext: "Extend selection right",
  open: "Open folder",
  undo: "Undo",
  redo: "Redo",
  toggleLeft: "Show / hide the left pane",
  toggleRight: "Show / hide the right pane",
  toggleStrip: "Show / hide the filmstrip",
  toggleSides: "Show / hide both side panes",
  focus: "Focus mark",
  zoom: "1:1 zoom",
  grayscale: "Grayscale",
  compare: "Compare frames",
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

// The settings modal in the main window. `main.ts` hands it every keydown
// while it is open.
export type Settings = {
  readonly isOpen: boolean;
  open(): void;
  close(): void;
  keydown(event: KeyboardEvent): void;
};

// Looks up the modal's elements and loads its values; call it once.
export function initSettings(): Settings {
  const dialog = document.getElementById("settings-dialog") as HTMLDivElement;
  const box = document.getElementById("settings-box") as HTMLDivElement;
  const modal = new SettingsModal();
  let returnFocus: HTMLElement | null = null;
  const shortcutsRows = document.getElementById("shortcuts-rows") as HTMLTableElement;
  const status = document.getElementById("settings-status") as HTMLDivElement;
  const sidecarRadios = document.querySelectorAll<HTMLInputElement>('input[name="sidecar-format"]');
  const labelNamesBlock = document.getElementById("label-names") as HTMLDivElement;
  const labelNameInput = (color: string): HTMLInputElement =>
    document.getElementById(`label-name-${color}`) as HTMLInputElement;
  let japaneseLabelNames: LabelNames | null = null;
  const autoAdvance = document.getElementById("auto-advance") as HTMLInputElement;
  const mcpEnabled = document.getElementById("mcp-enabled") as HTMLInputElement;
  const mcpStatusLine = document.getElementById("mcp-status") as HTMLParagraphElement;
  const mcpEndpointText = document.getElementById("mcp-endpoint") as HTMLPreElement;
  const mcpExamplesBlock = document.getElementById("mcp-examples") as HTMLDivElement;
  const debugTiming = document.getElementById("debug-timing") as HTMLInputElement;
  const indexSize = document.getElementById("index-size") as HTMLParagraphElement;
  const clearIndex = document.getElementById("clear-index") as HTMLButtonElement;
  const clearIndexNote = document.getElementById("clear-index-note") as HTMLParagraphElement;
  const tablist = document.getElementById("settings-tabs") as HTMLDivElement;
  const tabs = [...tablist.querySelectorAll<HTMLButtonElement>('[role="tab"]')];
  let shortcutBindings: Binding[] = [];
  let scanRunning = false;
  let clearInFlight = false;
  // Set by the first `scan-state`, so the initial `scan_running` answer is not
  // applied over a newer state that arrived while it was in flight.
  let scanStateSeen = false;

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
        if (modal.capturing === action) {
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
            modal.capturing = action;
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

  // The backend also emits `shortcuts-changed`, which the main view culls by.
  async function updateShortcuts(command: string, args?: Record<string, unknown>): Promise<void> {
    modal.capturing = null;
    try {
      shortcutBindings = await window.__TAURI__.core.invoke<Binding[]>(command, args);
      status.textContent = "";
    } catch (error) {
      status.textContent = String(error);
    }
    renderShortcuts();
  }

  function writesXmp(format: string): boolean {
    return format === "xmp" || format === "both";
  }

  function showSidecarFormat(format: string): void {
    for (const radio of sidecarRadios) {
      radio.checked = radio.value === format;
    }
    labelNamesBlock.hidden = !writesXmp(format);
  }

  function showLabelNames(names: LabelNames): void {
    for (const color of LABEL_COLORS) {
      labelNameInput(color).value = names[color];
    }
  }

  function saveLabelNames(names: LabelNames): void {
    status.textContent = "";
    window.__TAURI__.core
      .invoke<LabelNames>("set_label_names", { names })
      .then(showLabelNames)
      .catch((error: unknown) => {
        status.textContent = String(error);
      });
  }

  void window.__TAURI__.core.invoke<Binding[]>("shortcuts").then((bindings) => {
    shortcutBindings = bindings;
    renderShortcuts();
  });
  void window.__TAURI__.core.invoke<string>("sidecar_format").then(showSidecarFormat);
  void window.__TAURI__.event.listen<string>("sidecar-format", ({ payload }) => {
    showSidecarFormat(payload);
  });
  void window.__TAURI__.core
    .invoke<{ names: LabelNames; japanese: LabelNames }>("label_names")
    .then(({ names, japanese }) => {
      japaneseLabelNames = japanese;
      showLabelNames(names);
    });
  void window.__TAURI__.event.listen<LabelNames>("label-names", ({ payload }) => {
    showLabelNames(payload);
  });
  void window.__TAURI__.core.invoke<boolean>("auto_advance").then((enabled) => {
    autoAdvance.checked = enabled;
  });
  void window.__TAURI__.event.listen<boolean>("auto-advance", ({ payload }) => {
    autoAdvance.checked = payload;
  });
  function copyButton(text: HTMLElement): HTMLButtonElement {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = "Copy";
    button.addEventListener("click", () => {
      copyText(text);
    });
    return button;
  }

  // When the webview refuses clipboard access, the text is selected instead so
  // the platform's copy shortcut takes it.
  function copyText(text: HTMLElement): void {
    status.textContent = "";
    navigator.clipboard.writeText(text.textContent ?? "").catch(() => {
      const range = document.createRange();
      range.selectNodeContents(text);
      const selection = window.getSelection();
      selection?.removeAllRanges();
      selection?.addRange(range);
      status.textContent = "Could not copy; the text is selected, so copy it with the keyboard.";
    });
  }

  function showMcpState(state: McpState): void {
    mcpEnabled.checked = state.enabled;
    mcpStatusLine.textContent = mcpStatus(state);
    mcpEndpointText.textContent = mcpEndpoint(state.port);
    mcpExamplesBlock.replaceChildren(
      ...mcpExamples(state.port).map(({ client, text }) => {
        const block = document.createElement("div");
        const title = document.createElement("p");
        title.textContent = `For example, ${client}:`;
        const copyable = document.createElement("div");
        copyable.className = "copyable";
        const pre = document.createElement("pre");
        pre.textContent = text;
        copyable.append(pre, copyButton(pre));
        block.append(title, copyable);
        return block;
      }),
    );
  }

  void window.__TAURI__.core.invoke<McpState>("mcp_enabled").then(showMcpState);
  void window.__TAURI__.event.listen<McpState>("mcp-state", ({ payload }) => {
    showMcpState(payload);
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
      labelNamesBlock.hidden = !writesXmp(radio.value);
      window.__TAURI__.core
        .invoke("set_sidecar_format", { format: radio.value })
        .catch((error: unknown) => {
          status.textContent = String(error);
        });
    });
  }

  for (const color of LABEL_COLORS) {
    labelNameInput(color).addEventListener("change", () => {
      saveLabelNames(labelNamesPayload((c) => labelNameInput(c).value));
    });
  }

  (document.getElementById("label-names-japanese") as HTMLButtonElement).addEventListener(
    "click",
    () => {
      if (japaneseLabelNames !== null) saveLabelNames(japaneseLabelNames);
    },
  );

  (document.getElementById("label-names-english") as HTMLButtonElement).addEventListener(
    "click",
    () => {
      saveLabelNames(englishLabelNames());
    },
  );

  autoAdvance.addEventListener("change", () => {
    status.textContent = "";
    window.__TAURI__.core
      .invoke("set_auto_advance", { enabled: autoAdvance.checked })
      .catch((error: unknown) => {
        status.textContent = String(error);
      });
  });

  mcpEnabled.addEventListener("change", () => {
    status.textContent = "";
    window.__TAURI__.core
      .invoke("set_mcp_enabled", { enabled: mcpEnabled.checked })
      .catch((error: unknown) => {
        status.textContent = String(error);
      });
  });

  (document.getElementById("mcp-endpoint-copy") as HTMLButtonElement).addEventListener(
    "click",
    () => {
      copyText(mcpEndpointText);
    },
  );

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

  function selectTab(panel: string): void {
    if (modal.capturing !== null) {
      modal.capturing = null;
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

  function focusables(): HTMLElement[] {
    return [...box.querySelectorAll<HTMLElement>("button, input, [tabindex]")].filter(
      (el) =>
        el.tabIndex >= 0 &&
        !(el as HTMLButtonElement).disabled &&
        !(el instanceof HTMLInputElement && el.type === "radio" && !el.checked) &&
        el.getClientRects().length > 0,
    );
  }

  function open(): void {
    if (!modal.open()) {
      return;
    }
    returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    status.textContent = "";
    void window.__TAURI__.core.invoke<string>("sidecar_format").then(showSidecarFormat);
    if (!clearInFlight) {
      void window.__TAURI__.core.invoke<string>("index_size").then((size) => {
        if (!clearInFlight) showIndexSize(size);
      });
    }
    renderShortcuts();
    dialog.hidden = false;
    tabs.find((tab) => tab.getAttribute("aria-selected") === "true")?.focus();
  }

  function close(): void {
    if (!modal.isOpen) {
      return;
    }
    modal.close();
    renderShortcuts();
    dialog.hidden = true;
    returnFocus?.focus();
    returnFocus = null;
  }

  // Every key stops here while the modal is open, so none reaches the culling
  // keymap; the ones left `native` do what the focused control does with them.
  function keydown(event: KeyboardEvent): void {
    const decision = modal.key(keyName(event));
    switch (decision.kind) {
      case "add":
        event.preventDefault();
        void updateShortcuts("add_shortcut_key", { action: modal.capturing, key: decision.key });
        return;
      case "cancel":
        event.preventDefault();
        modal.capturing = null;
        renderShortcuts();
        return;
      case "close":
        event.preventDefault();
        close();
        return;
      case "focus": {
        event.preventDefault();
        const elements = focusables();
        if (elements.length === 0) {
          return;
        }
        const from = elements.indexOf(document.activeElement as HTMLElement);
        elements[cycleFocus(elements.length, from, decision.step)].focus();
        return;
      }
      case "native":
        if (tablist.contains(event.target as Node)) {
          tabKey(event);
        }
    }
  }

  function tabKey(event: KeyboardEvent): void {
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
  }

  (document.getElementById("settings-close") as HTMLButtonElement).addEventListener("click", close);

  // A click on the backdrop, outside the box, closes as well.
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) {
      close();
    }
  });

  return {
    get isOpen() {
      return modal.isOpen;
    },
    open,
    close,
    keydown,
  };
}
