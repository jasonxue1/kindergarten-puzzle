# Kindergarten Puzzle

Interactive, browser‑based puzzle viewer/editor.
Data lives in JSON files (`shapes.json` + `puzzle/*.json`).
The app is bilingual (English/中文), and the browser loads `shapes.json`
directly from the repo at runtime. No local PNG export is required.

Controls: drag to move, Q/E rotate, F flip.
Language can be switched in the UI.

## Repo Layout

- `shapes.json`: Shape catalog (mm units).
  Only `label_en` and `label_zh` are used.
- `puzzle/`: Puzzle specs (counts + board). Notes are `note_en`/`note_zh`.
- `puzzle-wasm/`: Rust crate compiled to WebAssembly
  (runtime + physics + UI glue).
- `puzzle-core/`: Shared Rust code for the web runtime.
- `blueprint-core/`: Optional PNG blueprint renderer used by the web export.
- `web/`: Vite + React front‑end. `web/public/` is generated during build/dev.
- `pkg/`: Generated WASM bundle from `wasm-pack` (created by `just build`).

## Build and Run

Prereqs: Rust + wasm‑pack, Node.js ≥ 20, pnpm ≥ 8. Nix is optional but recommended.

Using the provided Justfile:

```bash
# Optional: enter Nix dev shell (provides tools)
nix develop

# Build optimized WASM to ./pkg
just build

# Start the web app (copies JSON + pkg into web/public)
just serve
```

Open <http://localhost:5174/>.

Notes:

- `web/public/` is not tracked by git. The script `web/scripts/dev-prepare.mjs`
  creates it and copies:
  - `puzzles.json`, `shapes.json`, `puzzle/` from repo root
  - `pkg/` from repo root (WASM bundle)
  - plus a small `wasm-bridge.js` used to load the WASM module
- Production build: `cd web && pnpm install && pnpm build`
  (runs the same prepare step automatically and outputs `web/dist/`).

## Runtime Loading

- Default behavior: if the URL has `?p=<id>`, the app fetches
  `puzzle/<id>.json`. Otherwise it also fetches the default `puzzle/k11.json`.
- If a puzzle JSON is in counts format, the browser fetches `shapes.json` from
  the server first, and only falls back to the embedded copy if the request
  fails.
- `puzzles.json` is used by the chooser. It does not contain `desc`; each
  entry includes `id`, optional `title`, and `path`.

## JSON Formats

### shapes.json (catalog)

Each shape defines its geometry and bilingual label fields.
Only `label_en` and `label_zh` are supported.

```json
{
  "shapes": [
    {
      "id": "circle_d30",
      "type": "circle",
      "d": 30,
      "label_en": "Circle (diameter 30 mm)",
      "label_zh": "圆（直径 30mm）"
    }
  ]
}
```

Types include `rect`, `equilateral_triangle`, `right_triangle`,
`regular_polygon`, `circle`, `isosceles_trapezoid`, `parallelogram`, `polygon`.

### puzzle/`<id>`.json (counts + board)

Recommended format for shareable puzzles. Example:

```json
{
  "units": "mm",
    "board": {
      "type": "polygon",
      "polygons": [
        [
          [0, 0],
          [113, 0],
          [113, 108],
          [113, 123, 15],
          [0, 123]
        ]
      ]
     },
  "counts": {
    "circle_d30": 1,
    "square_30": 1
  },
  "note_en": "Place all pieces inside the frame. No overlaps. Leaving gaps is allowed.",
  "note_zh": "请将所有拼块放入外框内，彼此不可重叠；允许留空，不必完全填满。"
}
```

Optional: a counts JSON may set `shapes_file` to a custom catalog path.
If absent, the browser loads `shapes.json` from the server.

### Full piece layout (optional)

The app also accepts a full `pieces` list with explicit positions and
rotations for interactive play.

## UI Behavior

- Language: English default. Toggle to 中文 in the toolbar. Notes prefer
  `note_zh` when language is 中文, otherwise `note_en`.
- Colors: a stable cycling palette is assigned deterministically by input order.

## Development

- Format and lint all:

```bash
just fmt
just lint
```

The Markdown in this repo follows mado rules (e.g., list indentation).
README has been validated by `mado check`.

## License

MIT
