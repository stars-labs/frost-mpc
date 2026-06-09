# MPC Wallet Monorepo Architecture

## Overview

The MPC Wallet has been restructured as a monorepo to support multiple platforms while sharing code efficiently. This document describes the architecture and development practices.

## Directory Structure

```
frost-mpc/
├── apps/                        # All applications
│   ├── browser-extension/       # Chrome/Firefox extension (WXT + Svelte 5)
│   ├── tui-node/                # Terminal UI (Ratatui) + shared tui-node::core library
│   ├── native-node/             # Desktop GUI (Slint 1.x), reuses tui-node::core
│   └── signal-server/           # WebRTC signaling servers
│       ├── server/              # Standard WebSocket server
│       └── cloudflare-worker/   # Edge deployment
│
├── packages/@frost-mpc/        # Shared packages
│   ├── frost-core/              # Core FROST cryptography (Rust)
│   ├── core-wasm/               # WebAssembly bindings (Rust → JS)
│   ├── blockchain/              # Chain integrations (solana-sdk only; Ethereum/Bitcoin hand-rolled over sha2/sha3/bs58)
│   └── types/                   # TypeScript type definitions
│
├── scripts/                     # Monorepo build / test / clean scripts
├── Cargo.toml                   # Rust workspace root
├── package.json                 # Bun workspace root
└── flake.nix                    # Nix development environment
```

## Applications

### Browser Extension (`apps/browser-extension/`)
- **Technology**: TypeScript, Svelte 5 (legacy reactivity, not runes), WXT framework
- **Features**: Web3 wallet, FROST MPC, multi-chain support, EIP-1193 provider injection
- **Build**: `bun run build` (from apps/browser-extension)
- **Dev**: `bun run dev` (from apps/browser-extension)

### Terminal UI Node (`apps/tui-node/`)
- **Technology**: Rust, Ratatui (terminal UI), tui-realm (Elm architecture), WebRTC
- **Features**: Terminal UI, offline/SD-card mode, keystore management
- **Build**: `cargo build -p tui-node`
- **Run**: `cargo run --bin frost-mpc-tui -p tui-node -- --device-id mpc-1`
- **Library**: Exposes `tui-node::core::{WalletManager, SessionManager, DkgManager, OfflineManager, ConnectionManager, SigningManager}` for reuse by native-node.

### Native Node (`apps/native-node/`)
- **Technology**: Rust, Slint 1.x UI framework, `rfd` for native file dialogs
- **Features**: Desktop GUI reusing tui-node::core business logic; session management, DKG, encrypted keystore import/export, signing modal, SD-card export/import
- **Build**: `cargo build -p frost-mpc-native`
- **Run**: `cargo run --bin frost-mpc-native`

### Signal Servers (`apps/signal-server/`)
- **Standard Server**: Rust WebSocket server for development
- **Cloudflare Worker**: Edge deployment for production

## Shared Packages

### `@frost-mpc/frost-core`
Core FROST implementation in Rust, shared between TUI, native, and the WASM bindings:
- DKG (Distributed Key Generation)
- Threshold signing
- Keystore management
- Multi-curve support (secp256k1, ed25519)

### `@frost-mpc/core-wasm`
Thin WebAssembly wrapper around frost-core:
- Browser-compatible cryptography
- Async/await interface
- TypeScript bindings

### `@frost-mpc/types`
Centralized TypeScript type definitions:
- Message types for all communication
- State management interfaces
- Keystore formats
- Network protocols

## Development Workflow

### Prerequisites
```bash
# Install Bun
curl -fsSL https://bun.sh/install | bash

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or use Nix
nix develop
```

### Building Everything
```bash
# Build all packages and apps
./scripts/build-all.sh

# Or individually:
bun install                    # Install JS dependencies
bun run build:wasm            # Build WASM package
bun run build                 # Build browser extension
cargo build                   # Build all Rust apps
```

### Testing
```bash
# Run all tests
./scripts/test-all.sh

# Or individually:
bun test                      # JS/TS tests
cargo test                    # Rust tests
```

### Development Tips

1. **Shared Types**: Always define types in `@frost-mpc/types`
2. **Crypto Code**: Implement in `frost-core`, not in apps
3. **Import Paths**: Use `@frost-mpc/types` not relative paths
4. **Workspace Commands**: Run from root, not subdirectories

## Architecture Principles

### 1. Code Sharing
- Cryptographic operations in `packages/@frost-mpc/frost-core`
- Business logic (WalletManager / SessionManager / DkgManager /
  OfflineManager / ConnectionManager / SigningManager) lives in
  `apps/tui-node/src/core/` and is re-exported via `tui-node::lib.rs`.
  native-node consumes this as a Cargo dependency on `tui-node`.
- UI-specific code in respective apps

### 2. Type Safety
- Single source of truth for types
- Consistent interfaces across platforms
- Strong typing for all messages

### 3. Platform Independence
- Core logic independent of runtime
- Platform-specific code isolated
- Shared protocols and formats

### 4. Modularity
- Each app can be developed independently
- Clear dependency boundaries

All workspace crates are currently at version `0.1.0` (except
`webrtc-signal-server` at `0.1.1` from its pre-monorepo crates.io
life). No independent versioning has been introduced yet — all
crates move together on `main`.

## Communication Flow

```
Browser Extension          TUI Node              Native Node
       |                      |                      |
       |------WebSocket-------|------WebSocket------|
                              |
                        Signal Server
                              |
       |------WebRTC----------|------WebRTC---------|
       
All apps use the same:
- Message types (@frost-mpc/types)
- Cryptography (@frost-mpc/frost-core)
- Network protocols
```

## Adding New Features

1. **Define types** in `@frost-mpc/types`
2. **Implement crypto** in `frost-core` if needed
3. **Add to apps** with platform-specific UI
4. **Test across platforms** to ensure compatibility

## Future Expansion

Not currently in development, but the monorepo structure wouldn't
block any of these:

- Mobile application targets
- Additional blockchain integrations
- Hardware-wallet co-signer integration
- Improved offline-mode ergonomics

There is no scheduled roadmap for these — they're noted here to
clarify what the layout can accommodate, not what's under way.

## Troubleshooting

### Common Issues

1. **Import errors**: Ensure `@frost-mpc/types` is built
2. **WASM not found**: Run `bun run build:wasm` first
3. **Type conflicts**: Check for duplicate type definitions
4. **Build failures**: Clean and rebuild from root

### Build Order
1. `packages/@frost-mpc/types`
2. `packages/@frost-mpc/frost-core`
3. `packages/@frost-mpc/core-wasm`
4. Applications

## Contributing

When contributing:
1. Follow the monorepo structure
2. Add tests for shared packages
3. Update types when adding features
4. Ensure cross-platform compatibility
5. Document platform-specific code

This architecture provides a solid foundation for the MPC Wallet ecosystem while maintaining code quality and developer experience.