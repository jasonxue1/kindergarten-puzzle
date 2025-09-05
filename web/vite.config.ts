import { defineConfig } from "vite";

export default defineConfig({
  // Use repo name as base so built assets resolve on GitHub Pages
  // If you deploy to a custom domain or user site root, adjust this to "/".
  base: "/kindergarten-puzzle/",
  server: {
    port: 5174,
    fs: {
      allow: [".."],
    },
  },
});
