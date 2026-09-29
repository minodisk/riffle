// Reads a `:root` token from the stylesheet for canvas drawing, which cannot
// use `var()`. A missing or misspelled property resolves to an empty string,
// which the canvas would ignore and keep its previous color, so the caller
// names a fallback.
export function token(
  style: Pick<CSSStyleDeclaration, "getPropertyValue">,
  name: string,
  fallback: string,
): string {
  const value = style.getPropertyValue(name).trim();
  return value === "" ? fallback : value;
}
