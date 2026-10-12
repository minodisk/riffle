// The payload of the `ai-state` event and the `ai_enabled` command.
// `downloaded` is the bytes so far of a running model download, `null` when
// none runs.
export interface AiState {
  enabled: boolean;
  ready: boolean;
  downloaded: number | null;
  total: number;
  error: string | null;
}

const MB = 1_000_000;

// The settings modal's status line under the checkbox.
export function aiStatus({ enabled, ready, downloaded, total, error }: AiState): string {
  if (error !== null) return error;
  if (downloaded !== null) {
    return `Downloading the model: ${Math.floor(downloaded / MB)} / ${Math.round(total / MB)} MB`;
  }
  if (!enabled) {
    return ready ? "Off" : `Off; turning it on downloads the model (${Math.round(total / MB)} MB)`;
  }
  return ready ? "On" : "The model is not downloaded";
}

// Whether the third pass can run: AI is on and its model is in place.
export function aiActive(state: AiState): boolean {
  return state.enabled && state.ready;
}
