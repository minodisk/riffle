// Decodes preview JPEG bytes into an `ImageBitmap`. `invoke` is unavailable
// here, so the main thread fetches the bytes and this worker only decodes.
interface DecodeRequest {
  seq: number;
  jpeg: ArrayBuffer;
}

// The DOM lib has no worker global scope type, and pulling in the "webworker"
// lib alongside "dom" clashes on the shared globals, so declare what is used.
interface WorkerScope {
  addEventListener(type: "message", listener: (event: MessageEvent<DecodeRequest>) => void): void;
  postMessage(message: unknown, transfer?: Transferable[]): void;
}

const ctx = self as unknown as WorkerScope;

ctx.addEventListener("message", (event: MessageEvent<DecodeRequest>) => {
  const { seq, jpeg } = event.data;
  createImageBitmap(new Blob([jpeg], { type: "image/jpeg" }))
    .then((bitmap) => {
      ctx.postMessage({ seq, bitmap }, [bitmap]);
    })
    .catch((err: unknown) => {
      ctx.postMessage({ seq, error: String(err) });
    });
});
