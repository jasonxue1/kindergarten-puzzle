# kindergarten-puzzle

Rust + WASM puzzle viewer/editor for the browser, plus a Rust CLI that exports
printable blueprints (PDF) from JSON specs.

Web app: drag to move, Q/E rotate, F flip, S save JSON. Exporting images is
handled by the CLI, not the web app.

## Project Layout

- `shapes.json`: Shared shape catalog (IDs and dimensions; units mm)
- `puzzle/k11.json`: Example puzzle with board and counts per shape ID
- `puzzle-wasm/`: Rust crate compiled to WebAssembly
- `blueprint/`: Rust CLI to export blueprint PDF from JSON
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

Open <http://localhost:5173/>

## Usage (Web)

- Default loads a built‑in puzzle expanded from `puzzle/k11.json` and
  `shapes.json`.
- To load another JSON from `puzzle/`, use `?p=<name>` where the app fetches
  `puzzle/<name>.json`. Example: `/?p=k11`.
- The file picker can load any local `*.json` in either format (full pieces, or
  counts+board).
- Button: 保存JSON — downloads the current state as `puzzle.json`.

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

### Full Pieces (web play layout)

The web app still accepts the original piece‑list JSON with explicit `pieces`
and positions.

## Export Blueprints (CLI)

Use the Rust exporter in `blueprint/` to generate a flat, printable PNG with
labels and dimensions.

PNG from counts + shapes (uses `shapes.json` by default). Output defaults to `out.png`:

```bash
just export puzzle/k11.json
```

Custom output name:

```bash
just export puzzle/k11.json out.png
```

Resolution (px per mm, default 6; higher is sharper):

```bash
just export puzzle/k11.json out.png 8
```

Custom shapes catalog path:

```bash
just export puzzle/k11.json out.png 6 shapes.json
```

Layout: board on top (标注“外框”+总宽/总高+R半径)；下方每行一种形状，左侧大号标签如
“直径30圆*1”“直角 30×60 三角形*2”，右侧平铺该组所有轮廓。

## GitHub Pages

- Ensure `pkg/` (generated via `just build`) is present in the repo and committed.
- Push `index.html`, `pkg/`, and `puzzle/` to the default branch.
- In repo Settings → Pages, choose Source: Deploy from branch, Branch: `main`
  (or your default), Folder: `/ (root)`.
- Visit your Pages URL. Use `/?p=k11` to load that puzzle.
