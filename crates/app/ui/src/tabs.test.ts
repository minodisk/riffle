import { describe, expect, test } from "vitest";
import { nextTab } from "./tabs.js";

const tabs = ["sidecar", "culling", "shortcuts"];
const withDebug = [...tabs, "debug"];

describe("nextTab horizontal", () => {
  test("ArrowRight moves to the next tab and wraps", () => {
    expect(nextTab(tabs, "sidecar", "ArrowRight", "horizontal")).toBe("culling");
    expect(nextTab(tabs, "shortcuts", "ArrowRight", "horizontal")).toBe("sidecar");
    expect(nextTab(withDebug, "shortcuts", "ArrowRight", "horizontal")).toBe("debug");
    expect(nextTab(withDebug, "debug", "ArrowRight", "horizontal")).toBe("sidecar");
  });

  test("ArrowLeft moves to the previous tab and wraps", () => {
    expect(nextTab(tabs, "culling", "ArrowLeft", "horizontal")).toBe("sidecar");
    expect(nextTab(tabs, "sidecar", "ArrowLeft", "horizontal")).toBe("shortcuts");
    expect(nextTab(withDebug, "sidecar", "ArrowLeft", "horizontal")).toBe("debug");
  });

  test("Home and End move to the first and last tab", () => {
    expect(nextTab(tabs, "culling", "Home", "horizontal")).toBe("sidecar");
    expect(nextTab(tabs, "culling", "End", "horizontal")).toBe("shortcuts");
    expect(nextTab(withDebug, "culling", "End", "horizontal")).toBe("debug");
  });

  test("other keys keep the current tab", () => {
    expect(nextTab(withDebug, "culling", "Enter", "horizontal")).toBe("culling");
    expect(nextTab(tabs, "culling", "ArrowUp", "horizontal")).toBe("culling");
    expect(nextTab(tabs, "culling", "ArrowDown", "horizontal")).toBe("culling");
  });
});

describe("nextTab vertical", () => {
  test("ArrowDown moves to the next tab and wraps", () => {
    expect(nextTab(tabs, "sidecar", "ArrowDown", "vertical")).toBe("culling");
    expect(nextTab(tabs, "shortcuts", "ArrowDown", "vertical")).toBe("sidecar");
  });

  test("ArrowUp moves to the previous tab and wraps", () => {
    expect(nextTab(tabs, "culling", "ArrowUp", "vertical")).toBe("sidecar");
    expect(nextTab(tabs, "sidecar", "ArrowUp", "vertical")).toBe("shortcuts");
  });

  test("Home and End move to the first and last tab", () => {
    expect(nextTab(tabs, "culling", "Home", "vertical")).toBe("sidecar");
    expect(nextTab(tabs, "culling", "End", "vertical")).toBe("shortcuts");
  });

  test("the visible Debug tab joins the cycle; a hidden one is not in the list", () => {
    expect(nextTab(withDebug, "shortcuts", "ArrowDown", "vertical")).toBe("debug");
    expect(nextTab(withDebug, "debug", "ArrowDown", "vertical")).toBe("sidecar");
    expect(nextTab(withDebug, "sidecar", "ArrowUp", "vertical")).toBe("debug");
    expect(nextTab(withDebug, "culling", "End", "vertical")).toBe("debug");
    expect(nextTab(tabs, "shortcuts", "ArrowDown", "vertical")).toBe("sidecar");
  });

  test("ArrowLeft and ArrowRight keep the current tab", () => {
    expect(nextTab(tabs, "culling", "ArrowLeft", "vertical")).toBe("culling");
    expect(nextTab(tabs, "culling", "ArrowRight", "vertical")).toBe("culling");
  });
});
