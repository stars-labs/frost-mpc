# docs/

Cross-cutting documentation for the MPC Wallet monorepo. The project
landing page is [/README.md](../README.md) at the repo root; this
directory collects documentation that doesn't belong to a single
workspace member.

## Contents

- [`MONOREPO_ARCHITECTURE.md`](MONOREPO_ARCHITECTURE.md) — workspace
  layout, how the Rust + Bun workspaces fit together, build order.
- [`MPC_WALLET_TECHNICAL_DOCUMENTATION.md`](MPC_WALLET_TECHNICAL_DOCUMENTATION.md)
  — comprehensive technical reference (architecture, protocol,
  crypto details, deployment scenarios). Several hundred pages.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — code-of-conduct, branching
  model, commit format, PR guidelines.
- [`CHANGELOG.md`](CHANGELOG.md) — release history.

### Subdirectories

- [`deployment/`](deployment/) — production deployment guide
  (Docker, K8s, systemd, signal-server deployment).
  Currently a single-file `README.md` with inline recipes + the
  [`CLOUDFLARE_DEPLOYMENT.md`](deployment/CLOUDFLARE_DEPLOYMENT.md)
  guide for the Worker variant.
- [`implementation/`](implementation/) — deep-dives on specific
  cross-cutting implementation choices. Notable:
  [`EIP-6963-IMPLEMENTATION.md`](implementation/EIP-6963-IMPLEMENTATION.md)
  (wallet provider discovery) and
  [`MULTI_LAYER2_SUPPORT.md`](implementation/MULTI_LAYER2_SUPPORT.md).
- [`testing/`](testing/) — testing strategy + harness docs. See
  [`testing/README.md`](testing/README.md) as the index; the
  [`RUN_TEST_INSTRUCTIONS.md`](testing/RUN_TEST_INSTRUCTIONS.md)
  is the practical how-to.

## Per-workspace-member docs

Each app and package has its own docs subtree:

- [`apps/tui-node/docs/`](../apps/tui-node/docs/) — largest
  subtree; architecture, protocol, keyboard handling, keystore
  internals, many historical phase-summary docs.
- [`apps/browser-extension/docs/`](../apps/browser-extension/docs/)
- [`apps/native-node/docs/`](../apps/native-node/docs/) — deferrs
  most content to the parent [`apps/native-node/README.md`](../apps/native-node/README.md).
- [`apps/signal-server/docs/`](../apps/signal-server/docs/)
- (Rust library crates don't have docs subtrees; their public API
  is documented via `///` rustdoc.)
