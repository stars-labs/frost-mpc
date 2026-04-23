# EIP-6963 Implementation Summary

## Overview
The MPC Wallet has been successfully updated to implement the EIP-6963 standard for multi-injected provider discovery. This allows dApps to discover and connect to the wallet alongside other wallet extensions.

## Implementation Details

### 1. Provider Injection (`src/entrypoints/injected/index.ts`)
- Creates a `PageProvider` class that implements the Ethereum provider API
- Exposes the provider **only** as `window.starlabEthereum`, never
  as `window.ethereum` — the injection code explicitly scopes
  itself to the namespaced property so we never overwrite another
  wallet extension's `window.ethereum`. dApps discover us via the
  EIP-6963 announcement, not a `window.ethereum` prototype chain.
- Implements all required EIP-1193 methods via a `Proxy` around
  `PageProvider` to pass through legacy MetaMask-specific
  properties.

### 2. EIP-6963 Provider Announcement
- Listens for `eip6963:requestProvider` events
- Responds with `eip6963:announceProvider` events containing:
  - UUID: Unique identifier for each provider instance
  - Name: "MPC Wallet"
  - Icon: Base64-encoded SVG logo
  - RDNS: "org.starlab.wallet"
  - Description: Wallet description

### 3. Supported RPC Methods

Real dispatch table in
`src/entrypoints/background/rpcHandler.ts:100-135`:

- `eth_requestAccounts` / `eth_accounts` — permission request / query
- `eth_chainId` — returns the current network id via
  `NetworkService.getCurrentNetwork()`. Earlier drafts claimed a
  `default: 0x1 for Ethereum mainnet` fallback; the real handler
  throws `"No current network found"` when no network is set (see
  `handleChainIdRequest` at line 204). The `0x1` hardcode only
  appears inside `eth_requestAccounts` at line 186 as the chainId
  recorded in the permission ledger when no network has been
  selected yet — not as a chainId-query default.
- `net_version` — numeric-string network id (same source)
- `eth_getBalance` / `eth_getTransactionCount` / `eth_gasPrice` /
  `eth_estimateGas` — forwarded to the RPC provider
- `eth_sendTransaction` — wraps the MPC signing flow
- `eth_signMessage` / `personal_sign` — EIP-191 message signing
  (threshold-signed via FROST, ecrecover-compatible)
- Any other method: if `isReadOnlyMethod()` returns true, it's
  forwarded to the RPC provider; otherwise the handler throws
  `"Unsupported method: <method>"`.

### 4. Key Features
- **Auto-connection**: Smooth connection flow for better UX
- **Multi-provider support**: Works alongside MetaMask and other wallets
- **Session persistence**: Accounts are cached in sessionStorage
- **Fallback addresses**: Provides deterministic addresses when wallet is locked
- **Legacy compatibility**: Supports both modern and legacy dApp interfaces

## Testing

### Manual Testing Steps
1. Build the extension: `bun run build` (from `apps/browser-extension/`)
2. Load the extension in Chrome:
   - Navigate to `chrome://extensions`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `.output/chrome-mv3/` directory
3. Visit any EIP-6963-aware dApp (e.g. the Uniswap interface, or
   a local Rabby Kit / Wagmi `createConfig` test harness) and confirm
   "MPC Wallet" appears in the wallet-discovery list alongside any
   other installed wallet extension.

Earlier drafts of this section referenced bundled test harness files
(`/test-dapp.html`, `/test-extension-loaded.html`, `/test-eip6963.js`)
that never shipped in-tree — removed rather than kept as broken
pointers. A dedicated test-dApp fixture is open work.

### Expected Behavior
1. "MPC Wallet" shows up in the dApp's EIP-6963 "Discovered Wallets" list.
2. Clicking "Connect" invokes the MPC-wallet `eth_requestAccounts` flow.
3. Default RPC replies (before a wallet is unlocked):
   - `eth_chainId`: `"0x1"` (Ethereum mainnet)
   - `net_version`: `"1"`
   - `eth_accounts`: `[]` until a wallet is selected + unlocked.

## Integration with dApps
The wallet will automatically work with any dApp that:
1. Supports EIP-6963 provider discovery
2. Uses standard `window.ethereum` interface
3. Implements EIP-1193 request format

Popular dApps like Uniswap, OpenSea, and others that support multi-wallet discovery will automatically detect and display the MPC Wallet as an option for users to connect.