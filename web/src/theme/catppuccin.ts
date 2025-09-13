/* Generate CSS variables from @catppuccin/palette and inject into <head>. */
import { flavors } from "@catppuccin/palette";

type AnyColor = string | { hex?: string } | undefined;

function getHex(c: AnyColor, fallback: string): string {
  if (!c) return fallback;
  if (typeof c === "string") return c;
  if (typeof c.hex === "string") return c.hex;
  return fallback;
}

type Palette = { colors?: Record<string, { hex?: string } | undefined> };

export function injectCatppuccinVariables() {
  const { latte, mocha } = flavors as unknown as { latte?: Palette; mocha?: Palette };
  if (!latte || !mocha) return;
  const cssParts: string[] = [":root{"];
  for (const [name, val] of Object.entries(latte.colors ?? {}) as Array<
    [string, { hex?: string } | undefined]
  >) {
    cssParts.push(`--ctp-latte-${name}:${getHex(val?.hex, "#000000")};`);
  }
  for (const [name, val] of Object.entries(mocha.colors ?? {}) as Array<
    [string, { hex?: string } | undefined]
  >) {
    cssParts.push(`--ctp-mocha-${name}:${getHex(val?.hex, "#000000")};`);
  }
  cssParts.push("}");
  const css = cssParts.join("");

  let style = document.getElementById("catppuccin-palette-vars") as HTMLStyleElement | null;
  if (!style) {
    style = document.createElement("style");
    style.id = "catppuccin-palette-vars";
    document.head.appendChild(style);
  }
  style.textContent = css;
}
