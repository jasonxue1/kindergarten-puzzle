import { defineConfig } from "vite";

// Select base path per hosting target.
// - Cloudflare Pages: root domain per env -> base '/'
// - GitHub Pages: project subpath -> set BASE_PATH or default to '/kindergarten-puzzle/'
export default defineConfig(({ mode }) => {
  const isCF = !!process.env.CF_PAGES;
  const base = isCF ? "/" : process.env.BASE_PATH || "/kindergarten-puzzle/";
  return {
    base,
    server: {
      port: 5174,
      fs: {
        allow: [".."],
      },
    },
  };
});
