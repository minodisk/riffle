interface TauriGlobal {
  core: {
    invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
  };
}

interface Window {
  __TAURI__: TauriGlobal;
}
