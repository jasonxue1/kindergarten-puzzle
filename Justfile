# Simple task runner for building and serving the WASM app

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

CRATE_DIR := "puzzle-wasm"
OUT_DIR := "web/public/pkg"
TOML_FILES := "Cargo.toml wrangler.toml mado.toml taplo.toml blueprint-core/Cargo.toml fonts/Cargo.toml puzzle-wasm/Cargo.toml"

default: build

# Build optimized WASM bundle to ./pkg for GitHub Pages
build:
    # wasm-pack uses the crate dir as the base for --out-dir; use ../ to place output under repo root
    wasm-pack build --target web --release --out-dir ../{{ OUT_DIR }} {{ CRATE_DIR }}

# Clean generated artifacts
clean:
    # Remove all files that are matched by .gitignore across the repo
    git clean -fdX

# Serve the app locally using pnpm (modern web UI)
serve:
    pnpm -C web install
    pnpm -C web dev

# Format all code and content
fmt:
    cargo fmt --all
    # Limit Taplo to concrete TOML files to avoid slow repo-wide scanning
    taplo format --config taplo.toml {{ TOML_FILES }}
    # Use pnpm-managed Prettier in web/
    pnpm -C web install
    pnpm -C web format
    mdsf format .
    alejandra -q .

# Lint sources
lint:
    # Check only specific TOML files for speed
    taplo check --config taplo.toml {{ TOML_FILES }}
    cargo check --workspace
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    mado check
    deadnix --fail .
    # Run ESLint via pnpm in web/
    pnpm -C web install
    pnpm -C web lint
