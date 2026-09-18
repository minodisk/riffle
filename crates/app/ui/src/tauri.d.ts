interface TauriEvent<T> {
  event: string;
  payload: T;
}

interface TauriUpdate {
  version: string;
  downloadAndInstall(): Promise<void>;
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
