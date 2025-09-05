# kindergarten-puzzle

Rust + WASM puzzle viewer/editor for the browser, plus a Rust CLI that exports
PNG blueprints from JSON specs.

Web app: drag to move, Q/E rotate, F flip. Export PNG from the toolbar or via CLI.
Language: English by default, switchable to Chinese in the UI.

## Project Layout

- `shapes.json`: Shared shape catalog (IDs and dimensions; units mm)
- `puzzle/k11.json`: Example puzzle with board and counts per shape ID
- `puzzle-wasm/`: Rust crate compiled to WebAssembly
- `puzzle-core/`: Shared Rust library for reusable logic
- `blueprint/`: Rust CLI to export PNG from JSON
- `pkg/`: Generated WASM bundle (created by `just build`)

## Build (with Nix + just)

Using the included flake and Justfile:

- Enter dev shell:

```bash
nix develop
```

- Build optimized WASM to `pkg/` (or use `just build-dev` for debug):

```bash
just build
```

- Serve locally:

```bash
just serve
```

Open <http://localhost:5174/>

## Usage (Web)

- Default loads a built‑in puzzle expanded from `puzzle/k11.json` and `shapes.json`.
- To load another JSON from `puzzle/`, use `?p=<name>`; the app fetches `puzzle/<name>.json`.
  Example: `/?p=k11`.
- The file picker can load any local `*.json` in either format (full pieces, or counts+board).
- Save JSON button downloads the current state as `puzzle.json`.

Notes and Tutor

- NOTE: Optional per‑puzzle note fields `note_en` and `note_zh` are supported
  and shown under the toolbar.
- TUTOR: A global help panel lists each toolbar button’s function and hotkeys.
  Toggle via the Tutor button.

Coloring

- Pieces use a fixed 8‑color palette in this order: red, orange, yellow,
  green, cyan, blue, purple, pink.
- Colors are assigned deterministically by first grouping all pieces by type,
  then assigning colors by the grouped order with `index % 8`.
- Each piece’s color is stable during interaction (drag/rotate) and doesn’t
  change when bringing pieces to front.

## JSON Formats

Two supported inputs:

### Counts + Board (recommended for sharing between puzzles)

```json
{
  "shapes": [
    { "id": "circle_d30", "type": "circle", "d": 30 },
    { "id": "square_30", "type": "rect", "w": 30, "h": 30 },
    { "id": "rect_30x60", "type": "rect", "w": 30, "h": 60 },
    { "id": "hex_side30", "type": "regular_polygon", "n": 6, "side": 30 },
    { "id": "pent_side30", "type": "regular_polygon", "n": 5, "side": 30 },
    { "id": "rt_30x60", "type": "right_triangle", "a": 30, "b": 60 },
    {
      "id": "trap_60_30_30",
      "type": "isosceles_trapezoid",
      "base_bottom": 60,
      "base_top": 30,
      "height": 30
    },
    {
      "id": "para_45_15_30",
      "type": "parallelogram",
      "base": 45,
      "offset_top": 15,
      "height": 30
    },
    { "id": "tri_eq_30", "type": "equilateral_triangle", "side": 30 }
  ]
}
```

```json
{
  "units": "mm",
  "board": {
    "type": "rect_with_quarter_round_cut",
    "w": 113,
    "h": 123,
    "cut_corner": "topright",
    "r": 15
  },
  "counts": {
    "circle_d30": 1,
    "square_30": 1,
    "rect_30x60": 1,
    "hex_side30": 1,
    "pent_side30": 1,
    "rt_30x60": 2,
    "trap_60_30_30": 1,
    "para_45_15_30": 1,
    "tri_eq_30": 2
  }
}
```

Notes on labels

- `shapes.json`: You can set human‑readable labels per shape for export.
  - For bilingual output, prefer `label_en` and/or `label_zh`.
  - If only `label` is provided, it is used for Chinese; English exports will
    fall back to auto labels.
- `puzzle/*.json`: In `board`, you may include:
  - `label`: Single‑line text for the board (left column).
  - `label_lines`: Multi‑line text for the board (takes precedence).
  - If neither is provided, a default auto label is generated, e.g.
    "Board 113×123 mm (R15)".

### Full Pieces (web play layout)

The web app still accepts the original piece‑list JSON with explicit `pieces`
and positions.

## Export PNG

Two ways:

- Method 1: In the web app
  - Click the toolbar button "Download PNG" to download the current canvas as
    PNG. The export uses the current UI language for labels.

- Method 2: Via CLI (just)
  - From a JSON file (uses `shapes.json` by default):

  ```bash
  just png puzzle/k11.json
  ```

  - Custom output name:

  ```bash
  just png puzzle/k11.json out.png
  ```

  - Resolution (px per mm, default 6; higher is sharper):

  ```bash
  just png puzzle/k11.json out.png 8
  ```

  - Custom shapes catalog path:

  ```bash
  just png puzzle/k11.json out.png 6 shapes.json
  ```

  - By ID (reads `puzzle/<id>.json`):

  ```bash
  just png-id k11 out.png 6
  ```

Export format: 3 columns (label | count | shapes).
Left: concise text with exact dimensions; Middle: piece count; Right: outlines
tiled for each group. The first row is the board (e.g., "Board 113×123 mm
(R15)"); subsequent rows list each part type.

## Shell Completions (Nushell)

When using the Nix dev shell (`nix develop`), Nushell autoloads completions for
`just png` and `just png-id` via `flake.nix`. It completes:

- JSON inputs and `shapes.json` paths
- PNG outputs (suggests existing files or `out.png`)
- `px_per_mm` values (4 5 6 8 10 12)
- Puzzle IDs from `puzzle/*.json`

No manual setup needed; run `nu` inside `nix develop`.

## GitHub Pages

- Build WASM: `just build` (outputs `pkg/`).
- Build modern UI: `cd web && pnpm install && pnpm build` (outputs `web/dist`).
- Deploy the contents of `web/dist` to Pages. Ensure `pkg/` and `puzzle/` are
  also published at paths your app expects.
- Visit your Pages URL. Use `/?p=k11` to load that puzzle.
  Modern Web UI (React + Vite)

- A modern React + Vite + TypeScript UI is scaffolded under `web/` for a more
  modular architecture.
- It renders the same element IDs so the existing WASM logic continues to work.
- The UI includes a Toolbar, Status bar, Canvas, Note area, and a Tutor toggle
  point for future help overlays.

Run with the same two commands (pnpm required for serving):

- Build WASM bundle to `pkg/`:

  just build

- Serve locally (starts the modern UI via pnpm):

  just serve

Notes:

- `just serve` runs `pnpm install && pnpm dev` in `web/`. Ensure pnpm is
  installed.
- During development, the dev server copies `puzzles.json`, `shapes.json`, and
  the `puzzle/` folder into `web/public/` so the chooser can list and open
  puzzles as before.
- The React app sets the language via the existing `#langSel` element so the
  Rust side stays in sync for exports.
