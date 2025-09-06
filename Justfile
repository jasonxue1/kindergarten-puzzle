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
    # Remove all files that are matched by .gitignore across the repo
    git clean -fdX

# Serve the app locally using pnpm (modern web UI)
serve:
    cd web && pnpm install && pnpm dev

# Rebuild on changes (requires watchexec; optional)
watch:
    watchexec -e rs -w {{ CRATE_DIR }} -r -- just build

# CLI PNG export removed; use the web UI toolbar to export PNG.

# Format all code and content
fmt:
    cargo fmt --all
    taplo format
    prettier --write .
    mdsf format .
    alejandra -q .

# Lint sources
lint:
    taplo check
    cargo check --workspace
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    mado check
    deadnix --fail .
