// The frame size from a JPEG's first SOFn marker, read without decoding.
// `null` when the bytes are not a JPEG or no SOF precedes SOS or the end.
export function jpegSize(bytes: Uint8Array): { width: number; height: number } | null {
  if (bytes.length < 2 || bytes[0] !== 0xff || bytes[1] !== 0xd8) {
    return null;
  }
  let i = 2;
  while (i + 1 < bytes.length) {
    if (bytes[i] !== 0xff) {
      return null;
    }
    const marker = bytes[i + 1];
    if (marker === 0xff) {
      i += 1;
      continue;
    }
    if ((marker >= 0xd0 && marker <= 0xd8) || marker === 0x01) {
      i += 2;
      continue;
    }
    if (marker === 0xda || marker === 0xd9) {
      return null;
    }
    if (marker >= 0xc0 && marker <= 0xcf && marker !== 0xc4 && marker !== 0xc8 && marker !== 0xcc) {
      if (i + 9 > bytes.length) {
        return null;
      }
      return {
        height: (bytes[i + 5] << 8) | bytes[i + 6],
        width: (bytes[i + 7] << 8) | bytes[i + 8],
      };
    }
    if (i + 4 > bytes.length) {
      return null;
    }
    i += 2 + ((bytes[i + 2] << 8) | bytes[i + 3]);
  }
  return null;
}

// The largest same-aspect size at or under `maxPixels`, or `null` when the
// image already fits.
export function fitWithin(
  width: number,
  height: number,
  maxPixels: number,
): { resizeWidth: number; resizeHeight: number } | null {
  if (width * height <= maxPixels) {
    return null;
  }
  const scale = Math.sqrt(maxPixels / (width * height));
  return {
    resizeWidth: Math.max(1, Math.floor(width * scale)),
    resizeHeight: Math.max(1, Math.floor(height * scale)),
  };
}
