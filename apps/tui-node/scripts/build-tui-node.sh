#!/usr/bin/env bash
# Build script for MPC Wallet TUI Node.
# Invoke from anywhere — resolves its own paths.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Building MPC Wallet TUI Node..."
echo "Project directory: $PROJECT_ROOT"

cd "$PROJECT_ROOT"

cargo build --release --bin mpc-wallet-tui

BINARY_PATH="$PROJECT_ROOT/target/release/mpc-wallet-tui"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH" >&2
    exit 1
fi

echo "TUI node built successfully at: $BINARY_PATH"

# Optionally copy to deployment directory
DEPLOY_DIR="/opt/mpc-wallet"
if [ -d "$DEPLOY_DIR" ] && [ -w "$DEPLOY_DIR" ]; then
    echo "Copying TUI node to deployment directory..."
    cp "$BINARY_PATH" "$DEPLOY_DIR/mpc-wallet-tui"
    chmod +x "$DEPLOY_DIR/mpc-wallet-tui"
    echo "TUI node deployed to: $DEPLOY_DIR/mpc-wallet-tui"
fi

echo "Build complete!"
