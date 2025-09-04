# Simple task runner for building and serving the WASM app

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

CRATE_DIR := "puzzle-wasm"
OUT_DIR := "pkg"

default: build

# Build optimized WASM bundle to ./pkg for GitHub Pages
build:
    # wasm-pack uses the crate dir as the base for --out-dir; use ../ to place pkg at repo root
    wasm-pack build --target web --release --out-dir ../{{ OUT_DIR }} {{ CRATE_DIR }}

# Build a debug/dev bundle (faster compile, bigger output)
build-dev:
    # Place dev build at repo-root ./pkg as well
    wasm-pack build --target web --dev --out-dir ../{{ OUT_DIR }} {{ CRATE_DIR }}

# Clean generated artifacts
clean:
    rm -rf {{ OUT_DIR }}

# Serve the repository root locally at http://localhost:5173
serve:
    python3 -m http.server 5173

# Rebuild on changes (requires watchexec; optional)
watch:
    watchexec -e rs -w {{ CRATE_DIR }} -r -- just build

# Export blueprint PNG from a JSON file

# Usage: just png path/to/puzzle.json out.png [px_per_mm] [shapes.json]
png json out="out.png" px_per_mm="6" shapes_path="":
    out_path="{{ out }}"; \
    rm -f "${out_path}"; \
    if [ -n "{{ shapes_path }}" ]; then \
      cargo run --release --manifest-path blueprint/Cargo.toml -- {{ json }} "${out_path}" {{ px_per_mm }} {{ shapes_path }}; \
    else \
      cargo run --release --manifest-path blueprint/Cargo.toml -- {{ json }} "${out_path}" {{ px_per_mm }}; \
    fi

# Export by id: reads puzzle/<id>.json to out.png

# Usage: just png-id k11 out.png [px_per_mm]
png-id id out="out.png" px_per_mm="6":
    just png "puzzle/{{ id }}.json" "{{ out }}" {{ px_per_mm }}

# Format all code and content
fmt:
    # 1) Rust
    cargo fmt --all
    # 2) TOML (Cargo.toml, etc.)
    taplo format
    # 3) Web assets (HTML/JS/CSS/JSON/YML) via prettierd
    prettier --write .
    # 4) Markdown via mdsf (already in PATH)
    mdsf format .
    # 5) Nix files via alejandra
    alejandra -q .

# Lint sources
lint:
    # 1) TOML lint
    taplo check
    # 2) Rust lint
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    # 3) Rust type-check
    cargo check --workspace
    # 4) Markdown lint (mado.toml)
    mado check
    # Lint Nix code: report unused expressions/bindings
    deadnix --fail .
    # Lint Nix via nixd (LSP). This target verifies availability.
