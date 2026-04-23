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
- `eth_requestAccounts` - Request permission to access accounts
- `eth_accounts` - Get currently connected accounts
- `eth_chainId` - Get the current chain ID (default: 0x1 for Ethereum mainnet)
- `net_version` - Get the network version
- `eth_getBalance` - Get account balance
- `eth_sendTransaction` - Send transactions
- `personal_sign` - Sign messages
- And more standard Ethereum RPC methods

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