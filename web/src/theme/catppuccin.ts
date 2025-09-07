/* Generate CSS variables from @catppuccin/palette and inject into <head>. */
import { flavors } from "@catppuccin/palette";

type AnyColor = string | { hex?: string } | undefined;

function getHex(c: AnyColor, fallback: string): string {
  if (!c) return fallback;
  if (typeof c === "string") return c;
  if (typeof c.hex === "string") return c.hex;
  return fallback;
}

function alpha(hex: string, a: number): string {
  const m = /^#?([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i.exec(hex);
  if (!m) return hex;
  const r = parseInt(m[1], 16);
  const g = parseInt(m[2], 16);
  const b = parseInt(m[3], 16);
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

export function injectCatppuccinVariables() {
  const latte: any = (flavors as any)?.latte;
  const mocha: any = (flavors as any)?.mocha;
  if (!latte || !mocha) return;
  const cssParts: string[] = [":root{"];
  for (const [name, val] of Object.entries<any>(latte.colors ?? {})) {
    cssParts.push(`--ctp-latte-${name}:${getHex(val?.hex, "#000000")};`);
  }
  for (const [name, val] of Object.entries<any>(mocha.colors ?? {})) {
    cssParts.push(`--ctp-mocha-${name}:${getHex(val?.hex, "#000000")};`);
  }
  cssParts.push("}");
  const css = cssParts.join("");

  let style = document.getElementById(
    "catppuccin-palette-vars",
  ) as HTMLStyleElement | null;
  if (!style) {
    style = document.createElement("style");
    style.id = "catppuccin-palette-vars";
    document.head.appendChild(style);
  }
  style.textContent = css;
}
