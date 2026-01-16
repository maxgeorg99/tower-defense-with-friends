# Generate client bindings from server
generate:
    spacetime generate --lang rust --out-dir src/module_bindings --project-path spacetimedb

# Publish to local SpacetimeDB (run `spacetime start` first)
publish-local:
    spacetime publish rust --project-path spacetimedb

# Publish to maincloud
publish-maincloud name:
    spacetime publish {{name}} --server maincloud --project-path spacetimedb

# Build + generate + publish locally
deploy: generate publish-local

# Run SpacetimeDB dev server
dev:
    spacetime dev --project-path spacetimedb

# == Client ==

bevy:
    BEVY_ASSET_ROOT="." dx serve --bin bevy-demo --hot-patch --features "bevy-hotpatch"

# === Tools ===

wave-manager:
    cargo run --bin wave-manager

tower-manager:
    cargo run --bin tower-manager
