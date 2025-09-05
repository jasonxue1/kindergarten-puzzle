// Loaded directly by the browser from /public (no Vite transforms).
// Exposes the wasm-pack init function on window so the React app can call it
// without importing from /public inside source code.
import init from "./pkg/puzzle_wasm.js";
window.__puzzleWasmInit = init;
