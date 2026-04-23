# Testing Guide

Covers the browser-extension test suite. For the Rust side, see
`cargo test --workspace` plus `apps/tui-node/docs/RUN_TEST_INSTRUCTIONS.md`
for the 3-node manual mesh smoke test.

## Where tests live

The extension test tree is rooted at
`apps/browser-extension/tests/` and has colocated `.test.ts` files
next to the modules they exercise (e.g.
`src/services/walletClient.test.ts`).

```
apps/browser-extension/
├── tests/
│   ├── config/            # Configuration tests
│   ├── entrypoints/
│   │   ├── background/    # Service-worker regression suites
│   │   └── offscreen/     # WebRTC + FROST / WASM tests
│   ├── integration/       # Cross-component integration
│   ├── services/          # Service-layer unit tests
│   ├── utils/             # Test helpers
│   ├── __mocks__/         # Manual mocks
│   ├── setup-bun.ts       # Bun test setup + global mocks
│   ├── wxt-imports-mock.ts
│   └── README.md          # Test-tree doc
└── src/**/*.test.ts       # Colocated unit tests
```

## Test runner

This repo uses **Bun's built-in test runner**, not Vitest, not Jest.
Test files import from `bun:test`:

```ts
import { describe, expect, test, beforeEach } from "bun:test";
```

Coverage configuration is in `bunfig.toml` at the extension root;
see [COVERAGE.md](COVERAGE.md) for the caveats about Bun's
coverage-exclusion limitations.

## Running tests

From the repo root:

```bash
bun run test              # -> ./scripts/test-all.sh (all workspace tests)
bun run test:extension    # -> cd apps/browser-extension && bun test
```

From inside `apps/browser-extension/`:

```bash
bun test                                   # full suite
bun test tests/services/walletClient.test.ts
bun run test:watch                         # watch mode
bun run test:coverage                      # coverage report
bun run test:unit                          # tests/services + tests/config
bun run test:integration                   # tests/integration
bun run test:webrtc                        # tests/entrypoints/offscreen/webrtc.*.test.ts
```

No `test:e2e` or `test:ui` script exists — earlier drafts of this
doc mentioned them.

## Writing tests

1. Place test files under `apps/browser-extension/tests/` (or colocate
   next to the module as `<name>.test.ts`).
2. Import from `bun:test`, not `vitest` or `@jest/globals`.
3. Use existing mock patterns from `tests/__mocks__/` and
   `tests/setup-bun.ts`.
4. For WebRTC / WASM-touching tests, mirror the patterns in
   `src/entrypoints/offscreen/webrtc.test.ts` and
   `tests/entrypoints/offscreen/`.
5. For signing / DKG regression suites, see
   `tests/entrypoints/background/` — existing suites cover
   `dkgAutoTrigger`, `signingAutoTrigger`, `signingNotification`,
   `dappSignatureApproval`, and `signingDecline`.

## Svelte type checking

Separate from tests. Run from inside the extension directory:

```bash
cd apps/browser-extension && bun run check
```

## Live signal-server smoke tests

No automated harness exercises the full FROST + WebRTC pairing against
a real signal server — that needs three browser instances driving the
extension. See `apps/browser-extension/tests/README.md` for the
current status and `docs/testing/E2E_TEST_IMPLEMENTATION_PLAN.md` for
the open plan to harness it.
