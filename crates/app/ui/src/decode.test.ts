import { describe, expect, test } from "vitest";
import { fitWithin, jpegSize } from "./decode.js";

const SOI = [0xff, 0xd8];
const APP0 = [0xff, 0xe0, 0x00, 0x06, 0x4a, 0x46, 0x49, 0x46];
const SOS = [0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00];

function sof(marker: number, width: number, height: number): number[] {
  return [
    0xff,
    marker,
    0x00,
    0x0b,
    0x08,
    height >> 8,
    height & 0xff,
    width >> 8,
    width & 0xff,
    0x01,
    0x01,
    0x11,
    0x00,
  ];
}

describe("jpegSize", () => {
  test("reads a baseline SOF0 after APP0", () => {
    const bytes = new Uint8Array([...SOI, ...APP0, ...sof(0xc0, 9520, 6328), ...SOS]);
    expect(jpegSize(bytes)).toEqual({ width: 9520, height: 6328 });
  });

  test("reads a progressive SOF2", () => {
    const bytes = new Uint8Array([...SOI, ...APP0, ...sof(0xc2, 3200, 2127), ...SOS]);
    expect(jpegSize(bytes)).toEqual({ width: 3200, height: 2127 });
  });

  test("skips DHT and standalone markers", () => {
    const dht = [0xff, 0xc4, 0x00, 0x03, 0x00];
    const rst = [0xff, 0xd0];
    const bytes = new Uint8Array([...SOI, ...rst, ...dht, ...sof(0xc1, 640, 480), ...SOS]);
    expect(jpegSize(bytes)).toEqual({ width: 640, height: 480 });
  });

  test("is null without an SOF before SOS", () => {
    const bytes = new Uint8Array([...SOI, ...APP0, ...SOS, ...sof(0xc0, 640, 480)]);
    expect(jpegSize(bytes)).toBeNull();
  });

  test("is null for a truncated buffer", () => {
    const bytes = new Uint8Array([...SOI, ...APP0, ...sof(0xc0, 640, 480).slice(0, 6)]);
    expect(jpegSize(bytes)).toBeNull();
  });

  test("is null for non-JPEG bytes", () => {
    expect(jpegSize(new Uint8Array([0x89, 0x50, 0x4e, 0x47]))).toBeNull();
  });
});

describe("fitWithin", () => {
  test("fits a 60 MP frame under 12 MP and keeps the ratio", () => {
    const size = fitWithin(9520, 6328, 12_000_000);
    expect(size).not.toBeNull();
    const { resizeWidth, resizeHeight } = size!;
    expect(resizeWidth * resizeHeight).toBeLessThanOrEqual(12_000_000);
    expect(Math.abs(resizeHeight - (resizeWidth * 6328) / 9520)).toBeLessThanOrEqual(1);
  });

  test("resizes a 24 MP frame", () => {
    const size = fitWithin(6000, 4000, 12_000_000);
    expect(size).not.toBeNull();
    expect(size!.resizeWidth * size!.resizeHeight).toBeLessThanOrEqual(12_000_000);
  });

  test("leaves a frame at or under the limit alone", () => {
    expect(fitWithin(4000, 3000, 12_000_000)).toBeNull();
    expect(fitWithin(1600, 1067, 12_000_000)).toBeNull();
  });
});
