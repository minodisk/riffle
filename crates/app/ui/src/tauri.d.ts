interface TauriEvent<T> {
  event: string;
  payload: T;
}

type TauriDownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

interface TauriUpdate {
  version: string;
  downloadAndInstall(
    onEvent?: (progress: TauriDownloadEvent) => void,
  ): Promise<void>;
}

interface TauriGlobal {
  core: {
    invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
  };
  event: {
    listen<T>(
      event: string,
      handler: (event: TauriEvent<T>) => void,
    ): Promise<() => void>;
  };
  window: {
    getCurrentWindow(): { setTitle(title: string): Promise<void> };
  };
  updater: {
    check(): Promise<TauriUpdate | null>;
  };
  process: {
    relaunch(): Promise<void>;
  };
}

interface Window {
  __TAURI__: TauriGlobal;
}
