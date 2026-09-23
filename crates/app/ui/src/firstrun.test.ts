import { describe, expect, test, vi } from "vitest";
import { FormatGate } from "./firstrun.js";

describe("FormatGate", () => {
  test("starts closed", () => {
    expect(new FormatGate().isOpen).toBe(false);
  });

  test("defers an action until it opens", () => {
    const gate = new FormatGate();
    const action = vi.fn();
    gate.whenOpen(action);
    expect(action).not.toHaveBeenCalled();
    gate.open();
    expect(gate.isOpen).toBe(true);
    expect(action).toHaveBeenCalledOnce();
  });

  test("runs an action at once when already open", () => {
    const gate = new FormatGate();
    gate.open();
    const action = vi.fn();
    gate.whenOpen(action);
    expect(action).toHaveBeenCalledOnce();
  });

  test("runs the deferred action only once", () => {
    const gate = new FormatGate();
    const action = vi.fn();
    gate.whenOpen(action);
    gate.open();
    gate.open();
    expect(action).toHaveBeenCalledOnce();
  });

  test("opening with nothing deferred", () => {
    const gate = new FormatGate();
    gate.open();
    expect(gate.isOpen).toBe(true);
  });
});
