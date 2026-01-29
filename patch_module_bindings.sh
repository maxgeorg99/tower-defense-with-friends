#!/bin/bash
# Run this after: spacetime generate --lang rust --out-dir src/module_bindings --project-path spacetimedb
# Or use: just regenerate

set -e

FILE="src/module_bindings/mod.rs"

if [ ! -f "$FILE" ]; then
    echo "Error: $FILE not found"
    exit 1
fi

echo "Patching $FILE for WASM compatibility..."

# Create a backup
cp "$FILE" "${FILE}.bak"

# Use a temp file for safer editing
TEMP_FILE=$(mktemp)

# Read the file and apply patches using awk for more reliable multi-line editing
awk '
# Track if we are inside run_threaded function
/pub fn run_threaded\(&self\) -> std::thread::JoinHandle<\(\)>/ {
    # Print native version
    print "    /// Spawn a thread which processes WebSocket messages as they are received."
    print "    #[cfg(not(target_arch = \"wasm32\"))]"
    print "    pub fn run_threaded(&self) -> std::thread::JoinHandle<()> {"
    print "        let imp = self.imp.clone();"
    print "        std::thread::spawn(move || {"
    print "            futures::executor::block_on(imp.run_async()).ok();"
    print "        })"
    print "    }"
    print ""
    print "    /// Spawn a background task which processes WebSocket messages as they are received."
    print "    ///"
    print "    /// This is the wasm32 equivalent of `run_threaded`."
    print "    /// On wasm, this spawns an async task via wasm-bindgen-futures."
    print "    /// Note: The bevy_spacetimedb plugin must NOT call this - connection runs automatically."
    print "    #[cfg(target_arch = \"wasm32\")]"
    print "    pub fn run_threaded(&self) -> std::thread::JoinHandle<()> {"
    print "        use wasm_bindgen_futures::spawn_local;"
    print "        let imp = self.imp.clone();"
    print "        spawn_local(async move {"
    print "            let _ = imp.run_async().await;"
    print "        });"
    print "        // This is a hack - we need bevy_spacetimedb to NOT call run_fn on wasm"
    print "        // For now, panic with a clear message"
    print "        panic!(\"run_threaded should not be called on wasm - use with_delayed_connect and the connection runs automatically\")"
    print "    }"

    # Skip original function - read until closing brace
    skip = 1
    brace_count = 1
    next
}

# If skipping run_threaded, count braces
skip == 1 {
    for (i = 1; i <= length($0); i++) {
        c = substr($0, i, 1)
        if (c == "{") brace_count++
        if (c == "}") brace_count--
    }
    if (brace_count == 0) {
        skip = 0
    }
    next
}

# Patch advance_one_message_blocking - add #[cfg] and WASM version
/pub fn advance_one_message_blocking\(&self\) -> __sdk::Result<\(\)>/ {
    # Check if already patched
    if (prev_line ~ /#\[cfg\(not\(target_arch/) {
        print
        next
    }

    # Print native version with cfg
    print "    #[cfg(not(target_arch = \"wasm32\"))]"
    print "    pub fn advance_one_message_blocking(&self) -> __sdk::Result<()> {"
    print "        futures::executor::block_on(self.imp.advance_one_message_async())"
    print "    }"
    print ""
    print "    /// Process one WebSocket message, `await`ing until one is received."
    print "    ///"
    print "    /// This is the wasm32 equivalent of `advance_one_message_blocking`."
    print "    #[cfg(target_arch = \"wasm32\")]"
    print "    pub async fn advance_one_message_blocking(&self) -> __sdk::Result<()> {"
    print "        self.imp.advance_one_message_async().await"
    print "    }"

    # Skip original function
    skip_blocking = 1
    brace_count_blocking = 1
    next
}

# Skip original advance_one_message_blocking body
skip_blocking == 1 {
    for (i = 1; i <= length($0); i++) {
        c = substr($0, i, 1)
        if (c == "{") brace_count_blocking++
        if (c == "}") brace_count_blocking--
    }
    if (brace_count_blocking == 0) {
        skip_blocking = 0
    }
    next
}

# Store previous line for context checking
{ prev_line = current_line; current_line = $0 }

# Print all other lines
{ print }
' "$FILE" > "$TEMP_FILE"

# Replace original with patched version
mv "$TEMP_FILE" "$FILE"

# Remove backup if successful
rm -f "${FILE}.bak"

echo "Successfully patched $FILE for WASM compatibility!"
echo ""
echo "Changes applied:"
echo "  - run_threaded: Added #[cfg] guards and WASM version with spawn_local"
echo "  - advance_one_message_blocking: Added #[cfg] guards and async WASM version"
