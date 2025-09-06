import { defineConfig } from "vite";

export default defineConfig({
  // Configure base path via environment variable so branches can deploy
  // to different subdirectories (e.g. dev -> /dev/, main -> /).
  base: process.env.BASE_PATH || "/",
  server: {
    port: 5174,
    fs: {
      allow: [".."],
    },
  },
});
