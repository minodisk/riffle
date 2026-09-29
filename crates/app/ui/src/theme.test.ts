import { describe, expect, test } from "vitest";
import { token } from "./theme.js";

function style(props: Record<string, string>): Pick<CSSStyleDeclaration, "getPropertyValue"> {
  return { getPropertyValue: (name) => props[name] ?? "" };
}

describe("token", () => {
  test("trims the custom property's value", () => {
    expect(token(style({ "--card": " gray " }), "--card", "black")).toBe("gray");
  });

  test("falls back when the property is missing", () => {
    expect(token(style({}), "--card", "black")).toBe("black");
  });

  test("falls back when the property is blank", () => {
    expect(token(style({ "--card": "  " }), "--card", "black")).toBe("black");
  });
});
