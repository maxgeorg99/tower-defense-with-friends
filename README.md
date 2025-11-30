# Rust Projects Collection

This repository contains multiple Rust projects including a chat TUI and a tower defense game.

## Projects

### 1. Chat TUI (`quickstart-chat`)

A terminal user interface (TUI) client for [the `quickstart-chat` module](/modules/quickstart-chat).

This client is described in-depth by [the SpacetimeDB Rust client quickstart](https://spacetimedb.com/docs/sdks/rust/quickstart).

### 2. Tower Defense Game (`bevy-demo`)

A tower defense game built with Bevy 0.17, featuring:
- **Multiple Tower Types**: Archer towers and catapult towers with different stats
- **Radial Wheel Menu**: Left-click to open a radial menu for tower selection
- **Config-Driven**: All tower types defined in `towers.toml` for easy customization
- **Hot-Reloading**: Changes to config files are applied in real-time
- **Wave System**: Progressive enemy waves with configurable spawn patterns

#### Running the Tower Defense Game

```bash
cargo run --bin bevy-demo --features bevy-demo
```

#### Tower Configuration

Edit `towers.toml` to customize tower stats:
- `cost`: Gold required to build
- `range`: Attack range in pixels
- `damage`: Damage per shot
- `fire_rate`: Seconds between shots
- `projectile_speed`: Speed of projectiles in pixels/second

#### Tower Manager TUI

Manage and edit tower configurations with the TUI:

```bash
cargo run --bin tower-manager
```

**Controls:**
- `↑/↓`: Navigate towers (in tower list) or navigate fields (in tower details)
- `Tab`: Switch panels between tower list and details
- `Enter`: Start editing selected field (in tower details) or save changes (while editing)
- `Esc`: Cancel editing (while editing)
- `a`: Add new tower
- `x`: Delete selected tower
- `w`: Save changes to file
- `q`: Quit

**How to edit:**
1. Use `↑/↓` to select a tower
2. Press `Tab` to switch to tower details
3. Use `↑/↓` to navigate to the field you want to edit (indicated by `>>`)
4. Press `Enter` to start editing
5. Type your changes
6. Press `Enter` to save or `Esc` to cancel

## Features

- **Ratatui TUI**: Clean terminal interface with message history and user list
- **Auto-Restart on Changes**: Uses cargo-watch for fast development iteration
- **Multi-user Testing**: Easy environment variable to spawn multiple clients

## Running the Client

### Normal Mode

```bash
cargo run --bin chat-tui
```

### With Auto-Restart on Changes (Development)

For the best development experience:

```bash
./simple-hotreload.sh
```

This uses `cargo-watch` to automatically restart the TUI when you save changes. Simple and it works!

**Or install cargo-watch manually:**
```bash
cargo install cargo-watch
cargo watch -x 'run --bin chat-tui' -w src
```

See [QUICKSTART.md](QUICKSTART.md) for more details, or [HOT_RELOAD_STATUS.md](HOT_RELOAD_STATUS.md) for why Subsecond hot-patching doesn't work yet.

### Testing Multiple Users

```bash
# Terminal 1 - First user (saved identity)
cargo run --bin chat-tui

# Terminal 2 - Second user (fresh identity)
FRESH_IDENTITY=1 cargo run --bin chat-tui
```

See [TESTING_MULTIPLE_USERS.md](TESTING_MULTIPLE_USERS.md) for more details.
