import { describe, expect, test } from "vitest";
import { IdleGate } from "./idle.js";

describe("IdleGate", () => {
  test("runs at once when idle", () => {
    const gate = new IdleGate(() => false);
    const ran: string[] = [];
    gate.request("a", () => ran.push("a"));
    expect(ran).toEqual(["a"]);
    expect(gate.waiting).toBeNull();
  });

  test("holds while busy and reports the label", () => {
    const gate = new IdleGate(() => true);
    const ran: string[] = [];
    gate.request("a", () => ran.push("a"));
    expect(ran).toEqual([]);
    expect(gate.waiting).toBe("a");
  });

  test("drain runs the held operation exactly once and clears it", () => {
    const gate = new IdleGate(() => true);
    const ran: string[] = [];
    gate.request("a", () => ran.push("a"));
    gate.drain();
    gate.drain();
    expect(ran).toEqual(["a"]);
    expect(gate.waiting).toBeNull();
  });

  test("a second request while busy replaces the first", () => {
    const gate = new IdleGate(() => true);
    const ran: string[] = [];
    gate.request("a", () => ran.push("a"));
    gate.request("b", () => ran.push("b"));
    expect(gate.waiting).toBe("b");
    gate.drain();
    expect(ran).toEqual(["b"]);
  });

  test("discard drops the held operation", () => {
    const gate = new IdleGate(() => true);
    const ran: string[] = [];
    gate.request("a", () => ran.push("a"));
    gate.discard();
    expect(gate.waiting).toBeNull();
    gate.drain();
    expect(ran).toEqual([]);
  });

  test("drain with nothing held is a no-op", () => {
    const gate = new IdleGate(() => false);
    gate.drain();
    expect(gate.waiting).toBeNull();
  });
});
