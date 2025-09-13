// Global types for WASM bridge exposed on window
export type PuzzleWasm = {
  load_puzzle_from_text: (txt: string) => Promise<void>;
};

declare global {
  interface Window {
    __puzzleWasmInit?: (wasmUrl: string) => Promise<PuzzleWasm>;
    __puzzleWasm?: PuzzleWasm;
    __BASE_URL?: string;
  }
}

export {};
