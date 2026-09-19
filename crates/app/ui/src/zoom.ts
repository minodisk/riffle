// Where the 1:1 view draws the scaled preview while the crop is missing.
// Everything is in JPEG pixels relative to the focus point, so the caller
// draws the preview at `(-x, -y)` with the returned size.

export interface ZoomFocus {
  sensor_w: number;
  sensor_h: number;
  x: number;
  y: number;
}

export interface PlaceholderRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// `full` is the full JPEG size from a crop header, when one has arrived for
// this file. Without it the full JPEG is taken to be the sensor size, which is
// exact only when the JpgFromRaw is sensor-sized.
export function placeholderRect(
  bitmapWidth: number,
  bitmapHeight: number,
  focus: ZoomFocus,
  full: { width: number; height: number } | null,
): PlaceholderRect {
  const width = full?.width ?? focus.sensor_w;
  const height = full?.height ?? (bitmapHeight * focus.sensor_w) / bitmapWidth;
  return {
    x: (focus.x * width) / focus.sensor_w,
    y: (focus.y * height) / focus.sensor_h,
    width,
    height,
  };
}
