import { cp, mkdir, stat } from "node:fs/promises";
import path from "node:path";

const cwd = process.cwd();
const repoRoot = path.resolve(cwd, "..");
const publicDir = path.join(cwd, "public");

async function exists(p) {
  try {
    await stat(p);
    return true;
  } catch {
    return false;
  }
}

async function main() {
  await mkdir(publicDir, { recursive: true });
  const assets = [
    {
      src: path.join(repoRoot, "puzzles.json"),
      dst: path.join(publicDir, "puzzles.json"),
    },
    {
      src: path.join(repoRoot, "shapes.json"),
      dst: path.join(publicDir, "shapes.json"),
    },
  ];
  for (const a of assets) {
    if (await exists(a.src)) {
      await cp(a.src, a.dst);
    }
  }
  const puzzleSrc = path.join(repoRoot, "puzzle");
  if (await exists(puzzleSrc)) {
    await cp(puzzleSrc, path.join(publicDir, "puzzle"), {
      recursive: true,
      force: true,
    });
  }
  const pkgSrc = path.join(repoRoot, "pkg");
  if (await exists(pkgSrc)) {
    await cp(pkgSrc, path.join(publicDir, "pkg"), {
      recursive: true,
      force: true,
    });
  } else {
    console.warn(
      "[dev-prepare] pkg/ not found. Run `just build` first to generate WASM bundle.",
    );
  }
}

main().catch((e) => {
  console.error("[dev-prepare] Failed:", e);
  process.exit(1);
});
