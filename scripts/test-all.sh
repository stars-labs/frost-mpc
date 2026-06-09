#!/usr/bin/env bash
# Run tests for all packages in the monorepo

set -e

echo "🧪 Testing MPC Wallet Monorepo..."

# Test browser extension
echo "🌐 Testing browser extension..."
cd apps/browser-extension
bun test
cd ../..

# Test Rust workspace — tui-node + frost-core + signal-server.
# Exclude native-node: its binary pulls the graphics-stack feature
# set which is inappropriate for a headless test run. (The crate
# still gets `cargo build`'d in build-all.sh on a workstation.)
#
# `--lib --tests` covers both the per-crate unit tests (67 in
# tui-node::lib) AND the separate integration-test binaries under
# apps/tui-node/tests/ (component_rendering.rs: 13 tests;
# update_transitions.rs: 88 tests). Without `--tests` those 101
# tests get silently skipped.
echo "🦀 Testing Rust workspace..."
cargo test --workspace --lib --tests --exclude frost-mpc-native

echo "✅ All tests complete!"