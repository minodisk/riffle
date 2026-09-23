// Decodes preview JPEG bytes into an `ImageBitmap`. `invoke` is unavailable
// here, so the main thread fetches the bytes and this worker only decodes.
import { fitWithin, jpegSize } from "./decode.js";

interface DecodeRequest {
  seq: number;
  jpeg: ArrayBuffer;
  maxPixels: number | null;
}

// The DOM lib has no worker global scope type, and pulling in the "webworker"
// lib alongside "dom" clashes on the shared globals, so declare what is used.
interface WorkerScope {
  addEventListener(type: "message", listener: (event: MessageEvent<DecodeRequest>) => void): void;
  postMessage(message: unknown, transfer?: Transferable[]): void;
}

const ctx = self as unknown as WorkerScope;

ctx.addEventListener("message", (event: MessageEvent<DecodeRequest>) => {
  const { seq, jpeg, maxPixels } = event.data;
  decode(jpeg, maxPixels)
    .then((bitmap) => {
      ctx.postMessage({ seq, bitmap }, [bitmap]);
    })
    .catch((err: unknown) => {
      ctx.postMessage({ seq, error: String(err) });
    });
});

// Only a preview over `maxPixels` gets resize options; everything else takes
// the plain call, as does a JPEG whose size cannot be read.
function decode(jpeg: ArrayBuffer, maxPixels: number | null): Promise<ImageBitmap> {
  const blob = new Blob([jpeg], { type: "image/jpeg" });
  if (maxPixels === null) {
    return createImageBitmap(blob);
  }
  const size = jpegSize(new Uint8Array(jpeg));
  const fit = size === null ? null : fitWithin(size.width, size.height, maxPixels);
  if (fit === null) {
    return createImageBitmap(blob);
  }
  return createImageBitmap(blob, { ...fit, resizeQuality: "high" });
}
