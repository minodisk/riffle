// Where the focus mark goes on the unrotated preview, drawn `drawWidth` x
// `drawHeight` and centered on the origin. Free of DOM and Tauri so it is
// tested without mocks.

export interface MarkFocus {
  sensor_w: number;
  sensor_h: number;
  x: number;
  y: number;
  frame: { width: number; height: number } | null;
  manual_focus: boolean;
}

export interface FocusMark {
  x: number;
  y: number;
  rect: { x: number; y: number; width: number; height: number } | null;
}

// `null` for a manual-focus shot, whose recorded point is not trusted.
export function focusMark(
  focus: MarkFocus | null | undefined,
  drawWidth: number,
  drawHeight: number,
): FocusMark | null {
  if (focus === null || focus === undefined || focus.manual_focus) {
    return null;
  }
  const x = -drawWidth / 2 + (focus.x * drawWidth) / focus.sensor_w;
  const y = -drawHeight / 2 + (focus.y * drawHeight) / focus.sensor_h;
  if (focus.frame === null) {
    return { x, y, rect: null };
  }
  const width = (focus.frame.width * drawWidth) / focus.sensor_w;
  const height = (focus.frame.height * drawHeight) / focus.sensor_h;
  return { x, y, rect: { x: x - width / 2, y: y - height / 2, width, height } };
}
