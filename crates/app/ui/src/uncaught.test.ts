import { describe, expect, test } from "vitest";
import { SUPPRESSED_LINE, UncaughtGate, uncaughtLine } from "./uncaught.js";

describe("uncaughtLine", () => {
  test("writes every field on one line", () => {
    expect(
      uncaughtLine("error", "boom", "main.js", 12, 3, "Error: boom\n    at f (main.js:12:3)\n"),
    ).toBe(
      "kind=error message=boom source=main.js line=12 col=3 stack=Error: boom | at f (main.js:12:3)",
    );
  });

  test("omits the fields it was not given", () => {
    expect(uncaughtLine("unhandledrejection", "nope")).toBe("kind=unhandledrejection message=nope");
  });

  test("collapses newlines in the message", () => {
    expect(uncaughtLine("error", "a\r\nb")).toBe("kind=error message=a b");
  });
});

describe("UncaughtGate", () => {
  test("forwards a distinct line once", () => {
    const gate = new UncaughtGate(3);
    expect(gate.pass("a")).toBe("a");
    expect(gate.pass("a")).toBeNull();
    expect(gate.pass("b")).toBe("b");
  });

  test("past the limit, emits one suppression notice and then nothing", () => {
    const gate = new UncaughtGate(2);
    expect(gate.pass("a")).toBe("a");
    expect(gate.pass("b")).toBe("b");
    expect(gate.pass("c")).toBe(SUPPRESSED_LINE);
    expect(gate.pass("d")).toBeNull();
    expect(gate.pass("a")).toBeNull();
  });
});
