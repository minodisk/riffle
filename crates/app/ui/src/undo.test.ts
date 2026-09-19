import { describe, expect, test } from "vitest";
import { History } from "./undo.js";

describe("History", () => {
  test("pops in reverse push order and is empty afterwards", () => {
    const history = new History<number>(10);
    history.push(1);
    history.push(2);
    expect(history.pop()).toBe(2);
    expect(history.pop()).toBe(1);
    expect(history.pop()).toBeUndefined();
  });

  test("drops the oldest entry past the limit", () => {
    const history = new History<number>(2);
    history.push(1);
    history.push(2);
    history.push(3);
    expect(history.pop()).toBe(3);
    expect(history.pop()).toBe(2);
    expect(history.pop()).toBeUndefined();
  });

  test("removes an entry by identity, keeping later ones", () => {
    const history = new History<{ n: number }>(10);
    const a = { n: 1 };
    const b = { n: 2 };
    history.push(a);
    history.push(b);
    history.remove(a);
    history.remove({ n: 2 });
    expect(history.pop()).toBe(b);
    expect(history.pop()).toBeUndefined();
  });

  test("clear empties it", () => {
    const history = new History<number>(10);
    history.push(1);
    history.clear();
    expect(history.pop()).toBeUndefined();
  });
});
