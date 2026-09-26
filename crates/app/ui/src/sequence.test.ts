import { describe, expect, test } from "vitest";
import {
  type SequenceDone,
  SequenceFlow,
  changedLine,
  doneStatus,
  failureText,
  progressStatus,
  rebuildNotice,
  revealAfter,
  rowText,
} from "./sequence.js";

const row = (path: string, changed: boolean) => ({
  path,
  old: "2024:01:02 03:04:58",
  new: changed ? "2024:01:02 03:04:59" : "2024:01:02 03:04:58",
  changed,
});

const done = (over: Partial<SequenceDone> = {}): SequenceDone => ({
  run_id: 1,
  dir: "/x/export",
  output_dir: "/x/export-sequenced",
  written: 3,
  total: 3,
  failed: [],
  canceled: false,
  ...over,
});

describe("text", () => {
  test("a row names the file and its old and new time", () => {
    expect(rowText(row("/x/export/DSC00001.jpg", true))).toBe(
      "DSC00001.jpg  2024:01:02 03:04:58 -> 2024:01:02 03:04:59",
    );
    expect(rowText(row("C:\\x\\L1000001.JPG", false))).toBe(
      "L1000001.JPG  2024:01:02 03:04:58 -> 2024:01:02 03:04:58",
    );
  });

  test("the rebuild notice shows only for an existing output folder", () => {
    const preview = { output_dir: "/x/export-sequenced", rows: [], failed: [] };
    expect(rebuildNotice({ ...preview, output_exists: true })).toBe(
      "will be rebuilt: its JPEG files are replaced",
    );
    expect(rebuildNotice({ ...preview, output_exists: false })).toBeNull();
  });

  test("counts the files that get a new time", () => {
    expect(changedLine([row("/a.jpg", false), row("/b.jpg", true), row("/c.jpg", true)])).toBe(
      "2 of 3 files get a new time",
    );
    expect(changedLine([row("/a.jpg", false)])).toBe("0 of 1 file gets a new time");
  });

  test("a failure names the file", () => {
    expect(failureText({ path: "/x/export/broken.jpg", message: "no EXIF" })).toBe(
      "broken.jpg: no EXIF",
    );
  });

  test("progress", () => {
    expect(progressStatus(2, 5)).toBe("sequencing 2 / 5");
  });

  test("the done status", () => {
    expect(doneStatus(done())).toBe("Wrote 3 of 3 files to /x/export-sequenced");
    expect(
      doneStatus(done({ written: 2, failed: [{ path: "/x/export/b.jpg", message: "denied" }] })),
    ).toBe("Wrote 2 of 3 files to /x/export-sequenced, 1 failed");
    expect(doneStatus(done({ written: 1, canceled: true }))).toBe("canceled, 1 of 3 written");
    expect(
      doneStatus(
        done({
          written: 0,
          total: 0,
          failed: [{ path: "/x/export", message: "no JPEG files" }],
        }),
      ),
    ).toBe("Wrote 0 of 0 files to /x/export-sequenced, 1 failed");
  });

  test.each([
    ["a run that wrote every file", done(), "/x/export-sequenced"],
    [
      "a run with a per-file failure",
      done({ written: 2, failed: [{ path: "/x/export/b.jpg", message: "denied" }] }),
      "/x/export-sequenced",
    ],
    ["a canceled run", done({ written: 1, canceled: true }), null],
    ["a run that wrote nothing", done({ written: 0 }), null],
    [
      "a folder-level error",
      done({ written: 0, total: 0, failed: [{ path: "/x/export", message: "no JPEG files" }] }),
      null,
    ],
  ])("reveals the output folder after %s", (_name, payload, expected) => {
    expect(revealAfter(payload)).toBe(expected);
  });
});

function running(): SequenceFlow {
  const flow = new SequenceFlow();
  flow.start();
  flow.picked("/x/export");
  flow.previewed();
  flow.run();
  return flow;
}

describe("SequenceFlow", () => {
  test("walks picking, previewing, previewed, running and done", () => {
    const flow = new SequenceFlow();
    expect(flow.start()).toBe(true);
    expect(flow.phase).toBe("picking");
    expect(flow.isOpen).toBe(false);
    expect(flow.picked("/x/export")).toBe(true);
    expect(flow.phase).toBe("previewing");
    expect(flow.previewed()).toBe(true);
    expect(flow.phase).toBe("previewed");
    expect(flow.isOpen).toBe(true);
    expect(flow.run()).toBe("/x/export");
    expect(flow.phase).toBe("running");
    expect(flow.started(1)).toEqual({ done: null, cancel: false });
    expect(flow.accepts(1)).toBe(true);
    expect(flow.done(done())).toBe(true);
    expect(flow.phase).toBe("done");
    expect(flow.isOpen).toBe(false);
    expect(flow.start()).toBe(true);
  });

  test("busy is true from picking through previewing, unlike isOpen", () => {
    const flow = new SequenceFlow();
    expect(flow.busy).toBe(false);
    flow.start();
    expect(flow.phase).toBe("picking");
    expect(flow.isOpen).toBe(false);
    expect(flow.busy).toBe(true);
    flow.picked("/x/export");
    expect(flow.phase).toBe("previewing");
    expect(flow.isOpen).toBe(false);
    expect(flow.busy).toBe(true);
    flow.previewed();
    expect(flow.busy).toBe(true);
    flow.run();
    flow.started(1);
    flow.done(done());
    expect(flow.phase).toBe("done");
    expect(flow.busy).toBe(false);
  });

  test("does not start again while under way", () => {
    const flow = new SequenceFlow();
    flow.start();
    expect(flow.start()).toBe(false);
    expect(running().start()).toBe(false);
  });

  test("a dismissed picker ends the flow", () => {
    const flow = new SequenceFlow();
    flow.start();
    expect(flow.picked(null)).toBe(false);
    expect(flow.phase).toBe("idle");
  });

  test("a failed preview ends the flow", () => {
    const flow = new SequenceFlow();
    flow.start();
    flow.picked("/x/export");
    flow.fail();
    expect(flow.phase).toBe("idle");
    expect(flow.previewed()).toBe(false);
  });

  test("dismissing a preview closes it; there is nothing to run", () => {
    const flow = new SequenceFlow();
    flow.start();
    flow.picked("/x/export");
    flow.previewed();
    expect(flow.dismiss()).toEqual({ kind: "close" });
    expect(flow.isOpen).toBe(false);
    expect(flow.run()).toBeNull();
  });

  test("dismissing a run cancels it and keeps the dialog open", () => {
    const flow = running();
    flow.started(4);
    expect(flow.dismiss()).toEqual({ kind: "cancel", runId: 4 });
    expect(flow.isOpen).toBe(true);
  });

  test("ignores the events of another run", () => {
    const flow = running();
    flow.started(4);
    expect(flow.accepts(3)).toBe(false);
    expect(flow.done(done({ run_id: 3 }))).toBe(false);
    expect(flow.phase).toBe("running");
  });

  test("a done that comes before the run id is kept until started", () => {
    const flow = running();
    expect(flow.accepts(1)).toBe(false);
    expect(flow.done(done())).toBe(false);
    expect(flow.started(1)).toEqual({ done: done(), cancel: false });
  });

  test("a cancel asked for before the run id is passed on by started", () => {
    const flow = running();
    expect(flow.dismiss()).toEqual({ kind: "cancel", runId: null });
    expect(flow.started(1)).toEqual({ done: null, cancel: true });
  });
});
