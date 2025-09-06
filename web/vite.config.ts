import { defineConfig } from "vite";

// Allow CI to override base path for GitHub Pages per-branch deploys.
// Falls back to the repo root path used on main.
const base = process.env.BASE_PATH || "/kindergarten-puzzle/";

export default defineConfig({
  base,
  server: {
    port: 5174,
    fs: {
      allow: [".."],
    },
  },
});
