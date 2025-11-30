# Run the chat TUI with a fresh identity
chat:
    FRESH_IDENTITY=1 cargo run --bin chat-tui

dev:
    spacetime dev --database TD-with-friends

# Run the Bevy demo with hot reloading
bevy:
    BEVY_ASSET_ROOT="." dx serve --bin bevy-demo --hot-patch --features "bevy-hotpatch"

wave-manager:
    cargo run --bin wave-manager

tower-manager:
    cargo run --bin tower-manager
