# Module name used for both local and maincloud
MODULE_NAME := "td-mmo"
MAINCLOUD_URI := "https://maincloud.spacetimedb.com"

# SpacetimeAuth client ID (set this to your actual client ID)
AUTH_CLIENT_ID := env_var_or_default("SPACETIMEDB_CLIENT_ID", "client_032BJ1Hcqe1lvzV379770F")

# === Server ===

# Generate client bindings from server
generate:
    spacetime generate --lang rust --out-dir src/module_bindings --project-path spacetimedb

# Publish to local SpacetimeDB (run `spacetime start` first)
publish-local:
    spacetime publish {{MODULE_NAME}} --project-path spacetimedb

# Publish to maincloud (keeps existing data)
publish-maincloud:
    spacetime publish {{MODULE_NAME}} --server maincloud --project-path spacetimedb

# Publish to maincloud and reset all data
publish-maincloud-reset:
    spacetime publish {{MODULE_NAME}} --server maincloud --project-path spacetimedb --delete-data

# Build + generate + publish locally
deploy: generate publish-local

# Run SpacetimeDB dev server
dev:
    spacetime dev

# === Client ===

# Run game connected to local SpacetimeDB
bevy:
    SPACETIMEDB_URI="http://127.0.0.1:3000" SPACETIMEDB_MODULE={{MODULE_NAME}} SPACETIMEDB_CLIENT_ID={{AUTH_CLIENT_ID}} SPACETIMEDB_REQUIRE_AUTH=1 cargo run --features bevy-demo --bin game

# Run game connected to maincloud (with login screen)
bevy-live:
    SPACETIMEDB_URI={{MAINCLOUD_URI}} SPACETIMEDB_MODULE={{MODULE_NAME}} SPACETIMEDB_CLIENT_ID={{AUTH_CLIENT_ID}} SPACETIMEDB_REQUIRE_AUTH=1 cargo run --features bevy-demo --bin game

# Run with hot reloading (local)
bevy-hot:
    SPACETIMEDB_URI="http://127.0.0.1:3000" SPACETIMEDB_MODULE={{MODULE_NAME}} BEVY_ASSET_ROOT="." dx serve --bin game --hot-patch --features "bevy-hotpatch"

# === Tools ===

wave-manager:
    cargo run --bin wave-manager

tower-manager:
    cargo run --bin tower-manager

# === WASM ===

# Build WASM debug (requires: rustup target add wasm32-unknown-unknown && cargo install trunk)
wasm-build:
    trunk build

# Build WASM release
wasm-release:
    trunk build --release

# Run WASM dev server (hot reload, debug build)
wasm:
    trunk serve --open

# Run WASM release server
wasm-live:
    trunk serve --release --open

# Clean WASM build artifacts
wasm-clean:
    rm -rf dist/.stage target/wasm-opt

# === Bindings ===

# Regenerate bindings and apply WASM patches
regenerate: generate patch-bindings

# Apply WASM patches to module_bindings (run after `just generate`)
patch-bindings:
    @echo "Patching module_bindings for WASM compatibility..."
    ./patch_module_bindings.sh
    @echo "Done!"
