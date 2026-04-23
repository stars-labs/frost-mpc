# Native Desktop Application Documentation

Documentation for the MPC Wallet native desktop application built
with the Slint UI framework.

## Primary reference

The definitive "state of native-node" lives one level up at
[apps/native-node/README.md](../README.md) — architecture
diagram, feature-parity matrix vs TUI + browser extension, build
instructions, and the list of next-step work. Read that first.

## Also here

- [REFACTOR_PLAN.md](./REFACTOR_PLAN.md) — historical refactor
  plan from the initial Slint rehabilitation pass.

## Overview

The native desktop application provides:

- Cross-platform desktop interface (Linux, macOS, Windows)
- Native performance with the Slint UI framework
- Shared core functionality with `tui-node::core::{*Manager, CoreState}`
- Modern, responsive UI design
- Real-time status updates

## Features

- **Session Management** — create and join DKG sessions
- **Keystore Operations** — import/export encrypted keystores
- **Network Monitoring** — WebRTC/WebSocket status
- **Multi-chain Support** — Ethereum and Solana
- **Threshold Signing** — approve/reject modal with FROST round
  scaffolding (see parent README for FROST hookup status)
- **SD-card air-gap** — export/import/clear via `rfd` folder picker

## Architecture

The native app reuses the `tui-node` library's `core` module as its
business-logic backend. Slint UI events are bridged through the
`UICallback` trait (implemented by `NativeUICallback` in
`src/ui_callback.rs`) which posts closures onto the Slint event
loop via `Weak<MainWindow>` + `slint::invoke_from_event_loop`.
Closures must be `Send`; `MainWindow` itself is `!Send`, so each
callback clones the `Weak` and upgrades inside the closure.

See the parent [README.md](../README.md#architecture) for the
full component diagram.
