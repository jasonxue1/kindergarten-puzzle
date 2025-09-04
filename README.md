# kindergarten-puzzle

Rust + WASM puzzle viewer/editor for the browser, plus a Rust CLI that exports
PNG blueprints from JSON specs.

Web app: drag to move, Q/E rotate, F flip. You can export PNG from the toolbar
button or via CLI.

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

Notes on labels

- `shapes.json`: 可为每个形状添加可读文案 `label`（左列展示）。
- `puzzle/*.json`: 可在 `board` 中添加：
  - `label`：外框左列单行文案（适用于规则/简单外框）。
  - `label_lines`：外框左列多行文案（优先于 `label`，适用于不规则外框，描述更完整）。
  - 若两者均未提供，规则外框将回退为自动文案（如“外框 113×123mm（R15）”）。

### Full Pieces (web play layout)

The web app still accepts the original piece‑list JSON with explicit `pieces`
and positions.

## Export PNG

Two ways:

- Method 1: In the web app
  - Click the toolbar button "下载 PNG" to download the current canvas as PNG.

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

导出格式：三列表格（文案｜数量｜图片）。
左列：文字描述（只含关键信息，精确尺寸）；中列：数量；右列：对应图形。
第一行是外框（如：外框 113×123mm（R15）），其后每行一种零件，右侧平铺该组所有轮廓。

## Shell Completions (Nushell)

When using the Nix dev shell (`nix develop`), Nushell autoloads completions for
`just png` and `just png-id` via `flake.nix`. It completes:

- JSON inputs and `shapes.json` paths
- PNG outputs (suggests existing files or `out.png`)
- `px_per_mm` values (4 5 6 8 10 12)
- Puzzle IDs from `puzzle/*.json`

No manual setup needed; run `nu` inside `nix develop`.

## GitHub Pages

- Ensure `pkg/` (generated via `just build`) is present in the repo and committed.
- Push `index.html`, `pkg/`, and `puzzle/` to the default branch.
- In repo Settings → Pages, choose Source: Deploy from branch, Branch: `main`
  (or your default), Folder: `/ (root)`.
- Visit your Pages URL. Use `/?p=k11` to load that puzzle.
