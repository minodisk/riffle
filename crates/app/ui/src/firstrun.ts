import type { LabelPreset } from "./labels.js";

// Folder opens wait on the first-launch sidecar format choice. The gate starts
// closed; `open` lets folders open from then on and runs the one action that
// was deferred while it was closed (the startup restore of the last folder).
export class FormatGate {
  private opened = false;
  private deferred: (() => void) | null = null;

  get isOpen(): boolean {
    return this.opened;
  }

  whenOpen(action: () => void): void {
    if (this.opened) {
      action();
      return;
    }
    this.deferred = action;
  }

  open(): void {
    if (this.opened) {
      return;
    }
    this.opened = true;
    const action = this.deferred;
    this.deferred = null;
    action?.();
  }
}

// Whether the chosen format writes XMP, whose `xmp:Label` names must match
// Lightroom's UI language, so the dialog asks for that language before closing.
export function asksLanguage(format: string): boolean {
  return format === "xmp" || format === "both";
}

// The preset whose code is the browser language with its likely script
// (`zh-Hant` for `zh-TW`), else the one whose code is its primary subtag
// (`ja` for `ja-JP`), else the first one (English). An empty or invalid tag,
// or a webview without `Intl.Locale`, skips the script match.
export function defaultPreset(
  presets: LabelPreset[],
  navigatorLanguage: string,
): LabelPreset | undefined {
  try {
    const { language, script } = new Intl.Locale(navigatorLanguage).maximize();
    if (script) {
      const scripted = `${language}-${script}`.toLowerCase();
      const preset = presets.find((preset) => preset.code.toLowerCase() === scripted);
      if (preset) {
        return preset;
      }
    }
  } catch {
    // Fall through to the primary subtag match.
  }
  const primary = navigatorLanguage.split("-")[0].toLowerCase();
  return presets.find((preset) => preset.code === primary) ?? presets[0];
}
