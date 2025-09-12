import { cp, mkdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";

const cwd = process.cwd();
const repoRoot = path.resolve(cwd, "..");
const publicDir = path.join(cwd, "public");

const exists = (p) =>
  stat(p)
    .then(() => true)
    .catch(() => false);

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
    {
      src: path.join(repoRoot, "LICENSE"),
      dst: path.join(publicDir, "LICENSE"),
    },
  ];
  for (const a of assets) {
    if (await exists(a.src)) {
      await cp(a.src, a.dst);
    }
  }
  // Ensure wasm bridge exists in public so App.tsx can load it without Vite bundling
  const bridgePath = path.join(publicDir, "wasm-bridge.js");
  await writeFile(
    bridgePath,
    "import init from './pkg/puzzle_wasm.js';\nwindow.__puzzleWasmInit = init;\n",
    "utf8",
  );
  const puzzleSrc = path.join(repoRoot, "puzzle");
  if (await exists(puzzleSrc)) {
    await cp(puzzleSrc, path.join(publicDir, "puzzle"), {
      recursive: true,
      force: true,
    });
  }
  // WASM bundle should already be placed in web/public/pkg by the build step (see Justfile)
}

main().catch((e) => {
  console.error("[dev-prepare] Failed:", e);
  process.exit(1);
});
