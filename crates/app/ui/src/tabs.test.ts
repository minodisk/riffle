import { describe, expect, test } from "vitest";
import { nextTab } from "./tabs.js";

const tabs = ["sidecar", "culling", "shortcuts"];
const withDebug = [...tabs, "debug"];

describe("nextTab", () => {
  test("ArrowRight moves to the next tab and wraps", () => {
    expect(nextTab(tabs, "sidecar", "ArrowRight")).toBe("culling");
    expect(nextTab(tabs, "shortcuts", "ArrowRight")).toBe("sidecar");
    expect(nextTab(withDebug, "shortcuts", "ArrowRight")).toBe("debug");
    expect(nextTab(withDebug, "debug", "ArrowRight")).toBe("sidecar");
  });

  test("ArrowLeft moves to the previous tab and wraps", () => {
    expect(nextTab(tabs, "culling", "ArrowLeft")).toBe("sidecar");
    expect(nextTab(tabs, "sidecar", "ArrowLeft")).toBe("shortcuts");
    expect(nextTab(withDebug, "sidecar", "ArrowLeft")).toBe("debug");
  });

  test("Home and End move to the first and last tab", () => {
    expect(nextTab(tabs, "culling", "Home")).toBe("sidecar");
    expect(nextTab(tabs, "culling", "End")).toBe("shortcuts");
    expect(nextTab(withDebug, "culling", "End")).toBe("debug");
  });

  test("other keys keep the current tab", () => {
    expect(nextTab(withDebug, "culling", "Enter")).toBe("culling");
    expect(nextTab(tabs, "culling", "ArrowUp")).toBe("culling");
  });
});
