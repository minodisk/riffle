interface TauriEvent<T> {
  event: string;
  payload: T;
}

interface TauriGlobal {
  core: {
    invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
  };
  event: {
    listen<T>(event: string, handler: (event: TauriEvent<T>) => void): Promise<() => void>;
  };
  window: {
    getCurrentWindow(): { setTitle(title: string): Promise<void> };
  };
}

interface Window {
  __TAURI__: TauriGlobal;
}
