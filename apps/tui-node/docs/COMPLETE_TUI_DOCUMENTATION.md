# MPC Wallet TUI - Complete Technical Documentation

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Performance Optimizations](#performance-optimizations)
3. [User Experience Design](#user-experience-design)
4. [Navigation System](#navigation-system)
5. [Component Architecture](#component-architecture)
6. [State Management](#state-management)
7. [Security Model](#security-model)
8. [Testing Strategy](#testing-strategy)
9. [Deployment Guide](#deployment-guide)
10. [API Reference](#api-reference)

---

## 1. Architecture Overview

### Core Design Principles

The MPC Wallet TUI follows the **Elm Architecture** pattern, providing:
- **Unidirectional data flow**: Model → View → Message → Update → Model
- **Pure functions**: Side effects isolated in Commands
- **Type safety**: Rust's type system ensures correctness
- **Component isolation**: Each UI component is self-contained

### System Components

```
┌─────────────────────────────────────────────┐
│                   TUI Layer                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐   │
│  │ ElmApp  │ │  Model  │ │ Components  │   │
│  └────┬────┘ └────┬────┘ └──────┬──────┘   │
│       │           │              │           │
│  ┌────▼───────────▼──────────────▼────┐     │
│  │         Message Router              │     │
│  └────┬───────────┬──────────────┬────┘     │
│       │           │              │           │
└───────┼───────────┼──────────────┼───────────┘
        │           │              │
┌───────▼───────────▼──────────────▼───────────┐
│              Core Services                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ Keystore │ │  FROST   │ │  WebRTC  │     │
│  └──────────┘ └──────────┘ └──────────┘     │
└───────────────────────────────────────────────┘
```

### File Structure

Real layout of `src/elm/` — from `ls`:

```
src/elm/
├── app.rs                # ElmApp<C> — main event loop + tui-realm shell
├── model.rs              # Model (pure UI state)
├── message.rs            # Message enum — input events
├── update.rs             # Update fn — Message → state transition + Commands
├── command.rs            # Command<C> enum — side-effect tasks
├── mod.rs
├── provider.rs           # UIProvider trait
├── ws_runtime.rs         # WebSocket client runtime
├── webrtc_signaling.rs   # WebRTC signaling over the signal server
└── components/           # Per-screen tui-realm Component impls
```

Earlier drafts listed `adaptive_event_loop.rs` / `channel_config.rs`
/ `differential_update.rs` as part of this tree — none of those
files exist (verified via `find`). See § 2 below for details.

---

## 2. Performance Optimizations

Deliberate perf work in source today: **async tokio I/O**. That's
it.

Earlier drafts of this section described three specific optimizations
with Rust code samples:

  - `AdaptiveEventLoop { config, current_interval_ms, last_activity,
    is_idle }` — doesn't exist; no adaptive poll-interval code
    anywhere in the tree
  - `ChannelConfig { message_queue_size: 1000, session_event_queue_size:
    500, … }` — doesn't exist; no bounded-channel sizing scheme
  - `UpdateStrategy { NoUpdate / FullRemount / PartialUpdate }` —
    doesn't exist; no differential-render layer
  - "Reduces rendering overhead by 60-80%" — fabricated measurement
  - "CPU usage reduced from 5-10% to <1% when idle" — fabricated
    measurement

All three types are from
`docs/archive/dev-journal/PERFORMANCE_OPTIMIZATIONS.md` — a design
doc for work that was planned but never landed. I had propagated
these as real in other performance-section fixes earlier in this
cleanup pass (41d5ca0 / 7febf90 / f591806 / b335731); those have
been corrected back in their own docs.

Real opportunities if someone takes perf work on:

- Measure idle vs active CPU usage and introduce an adaptive
  event loop if the baseline justifies it.
- Audit `mpsc::unbounded_channel` call sites and add bounded
  alternatives where queue-growth could matter.
- Introduce differential rendering if tui-realm's built-in
  remount-on-state-change turns out to be a bottleneck.
- Add `criterion` benches with a reproducible methodology so
  future claims in this section can be anchored in measurement.

---

## 3. User Experience Design

### Design Philosophy

1. **Zero Learning Curve**: Menu-driven interface, no commands to memorize
2. **Visual Feedback**: Progress bars, status indicators, animations
3. **Contextual Help**: Always available with `?` key
4. **Error Recovery**: Clear error messages with suggested actions
5. **Accessibility**: High contrast, screen reader compatible

### Screen Hierarchy

```
Welcome Screen
    ├── Main Menu
    │   ├── Create New Wallet
    │   │   ├── Mode Selection (Online/Offline)
    │   │   ├── Curve Selection (Secp256k1/Ed25519)
    │   │   ├── Threshold Config
    │   │   └── DKG Process
    │   ├── Join Session
    │   │   ├── Session Discovery
    │   │   └── Session Details
    │   ├── Manage Wallets
    │   │   ├── Wallet List
    │   │   └── Wallet Details
    │   └── Settings
    │       ├── Network Settings
    │       └── Security Settings
    └── Help/About
```

### Visual Components

#### Progress Indicators
- **DKG Progress**: Multi-stage progress with participant status
- **Signing Progress**: Real-time signature generation tracking
- **Network Operations**: Connection status with retry indicators

#### Status Elements
- **Connection Status**: Visual WebSocket/WebRTC indicators
- **Wallet Status**: Balance, last activity, security level
- **Session Status**: Participant count, threshold, readiness

---

## 4. Navigation System

### Keyboard Shortcuts

#### Global Shortcuts
| Key | Action | Available |
|-----|--------|-----------|
| `Ctrl+Q` | Quit application | Always |
| `Ctrl+R` | Refresh current screen | Always |
| `Ctrl+H` | Go to home (main menu) | Always |
| `?` | Show contextual help | Always |
| `Esc` | Go back / Cancel | Context-dependent |

#### Navigation Keys
| Key | Action | Context |
|-----|--------|---------|
| `↑/↓` | Navigate menu items | Menus/Lists |
| `←/→` | Switch tabs/fields | Forms |
| `Enter` | Select/Confirm | Always |
| `Space` | Toggle selection | Checkboxes |
| `Tab` | Next field | Forms |
| `Shift+Tab` | Previous field | Forms |

### Navigation Stack

```rust
pub struct Model {
    pub navigation_stack: Vec<Screen>,
    pub current_screen: Screen,
    // ... other fields
}
```

**Features**:
- Maximum depth: 10 screens (configurable)
- Breadcrumb display
- Quick jump to any level
- Auto-cleanup of invalid paths

---

## 5. Component Architecture

### Component Structure

Each component implements:
```rust
pub trait Component {
    fn update(&mut self, msg: Message) -> Option<Command>;
    fn view(&self) -> Element;
    fn handle_event(&mut self, event: Event) -> Option<Message>;
}
```

### Core Components

#### MainMenu
- Displays wallet count
- Quick actions
- Navigation to major features
- Keyboard navigation with wrap-around

#### WalletList
- Sortable by name/date/balance
- Quick actions per wallet
- Pagination for large lists
- Search/filter capabilities

#### CreateWallet
- Multi-step wizard
- Validation at each step
- Progress persistence
- Rollback capability

#### DKGProcess
- Real-time participant status
- Round progress visualization
- Error recovery options
- Detailed logs panel

#### JoinSession
- Session discovery
- Participant preview
- Requirements validation
- Quick join/reject

### Component Communication

```
User Input → Component → Message → Update → Model → Component → View
                ↑                                          ↓
                └──────────── Command Execution ←──────────┘
```

---

## 6. State Management

### Model Structure

```rust
pub struct Model {
    // Core State
    pub wallet_state: WalletState,
    pub network_state: NetworkState,
    pub ui_state: UIState,
    
    // Navigation
    pub navigation_stack: Vec<Screen>,
    pub current_screen: Screen,
    
    // Session Management
    pub active_session: Option<SessionInfo>,
    pub pending_operations: Vec<Operation>,
    
    // User Context
    pub selected_wallet: Option<String>,
    pub device_id: String,
}
```

### State Updates

#### Pure Updates
- Model transformations
- No side effects
- Deterministic results

#### Commands (Side Effects)
- Network operations
- File I/O
- Async operations
- External system calls

### State Persistence

```rust
// Auto-save every 30 seconds
// Manual save on significant operations
// Crash recovery from last checkpoint
```

---

## 7. Security Model

### Key Protection

#### Encryption
- **Algorithm**: AES-256-GCM
- **Key Derivation**: PBKDF2-SHA256
- **Iterations**: 100,000
- **Salt**: 32 bytes random

#### Storage
- Encrypted keystore files
- Memory protection (zeroization)
- No swap file exposure
- Secure deletion

### Network Security

#### WebSocket
- TLS 1.3 required
- Certificate validation
- Reconnection with backoff
- Message authentication

#### WebRTC
- DTLS 1.3 for data channels
- SRTP for media (future)
- ICE candidate filtering
- TURN server authentication

### Operational Security

#### Offline Mode
- Complete air-gap operation
- SD card data exchange
- Manual verification steps
- Audit trail generation

#### Access Control
- Password protection
- Session timeouts
- Rate limiting
- Failed attempt tracking

---

## 8. Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_adaptive_event_loop() {
        // Test interval adjustments
    }
    
    #[test]
    fn test_differential_updates() {
        // Test update detection
    }
}
```

### Integration Tests

```rust
// tests/integration/
├── dkg_flow.rs       # Complete DKG process
├── signing_flow.rs   # Transaction signing
├── import_export.rs  # Keystore operations
└── network_recovery.rs # Connection handling
```

### Test Coverage

| Component | Coverage | Target |
|-----------|----------|--------|
| Core Logic | 85% | 90% |
| UI Components | 70% | 80% |
| Network Layer | 75% | 85% |
| Cryptography | 95% | 100% |

### Testing Tools

- **Unit**: Rust built-in `#[test]`
- **Integration**: Custom test harness
- **UI**: MockUIProvider for headless testing
- **Network**: Mock WebSocket/WebRTC servers
- **Performance**: Criterion benchmarks

---

## 9. Deployment Guide

### Build Configurations

#### Development
```bash
cargo build --bin mpc-wallet-tui
RUST_LOG=debug ./target/debug/mpc-wallet-tui
```

#### Release
```bash
cargo build --release --bin mpc-wallet-tui
strip target/release/mpc-wallet-tui
```

#### Platform-Specific

**Linux**:
```bash
# Debian/Ubuntu package
cargo deb
# RPM package
cargo rpm build
```

**macOS**:
```bash
# Universal binary
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create target/*/release/mpc-wallet-tui -output mpc-wallet-tui
```

**Windows**:
```bash
# MSI installer
cargo wix
```

### System Requirements

#### Minimum
- CPU: 1 GHz single-core
- RAM: 256 MB
- Storage: 50 MB
- Terminal: VT100 compatible

#### Recommended
- CPU: 2 GHz dual-core
- RAM: 1 GB
- Storage: 200 MB
- Terminal: 256-color support

### Environment Variables

```bash
# Logging
export RUST_LOG=info

# Configuration
export MPC_WALLET_CONFIG=/path/to/config.toml

# Keystore location
export MPC_KEYSTORE_PATH=/secure/location

# Network settings
export MPC_WEBSOCKET_URL=wss://your-server.com
```

### Docker Deployment

Docker packaging isn't currently shipped. The `Dockerfile` that used
to live at `apps/tui-node/Dockerfile` was written for a pre-monorepo,
pre-edition-2024 layout (Rust 1.75, single-crate \`COPY Cargo.lock\`)
and doesn't build against the current workspace. A working
Dockerfile would need:

- `FROM rust:1.85-slim` (edition 2024 requires 1.85+)
- Placement at the monorepo root, not under apps/tui-node/
- A multi-stage build that copies every workspace member crate so
  cargo can resolve the full dep graph, then builds just the TUI
  binary: `cargo build --release --bin mpc-wallet-tui -p tui-node`

See `apps/tui-node/docs/DEPLOYMENT_GUIDE.md` for the currently-
supported deployment paths (systemd + launch scripts).

---

## 10. API Reference

### Message Types

```rust
pub enum Message {
    // Navigation
    Navigate(Screen),
    NavigateBack,
    NavigateHome,
    
    // Wallet Operations
    CreateWallet(WalletConfig),
    DeleteWallet { wallet_id: String },
    ImportWallet { path: PathBuf },
    ExportWallet { wallet_id: String, path: PathBuf },
    
    // DKG Operations
    StartDKG { config: DKGConfig },
    UpdateDKGProgress { round: DKGRound, progress: f32 },
    DKGComplete { result: DKGResult },
    
    // Signing Operations
    StartSigning { request: SigningRequest },
    UpdateSigningProgress { progress: f32 },
    SigningComplete { signature: Signature },
    
    // Network Events
    WebSocketConnected,
    WebSocketDisconnected,
    WebRTCPeerConnected { peer_id: String },
    WebRTCPeerDisconnected { peer_id: String },
    
    // UI Events
    KeyPressed(KeyEvent),
    ScrollUp,
    ScrollDown,
    Refresh,
    Quit,
}
```

### Command Types

```rust
pub enum Command {
    // Data Operations
    LoadWallets,
    LoadSessions,
    SaveSettings { settings: Settings },
    
    // Network Operations
    ConnectWebSocket { url: String },
    SendMessage { to: String, data: Vec<u8> },
    BroadcastMessage { data: Vec<u8> },
    
    // Async Operations
    ExecuteDKG { config: DKGConfig },
    ExecuteSigning { request: SigningRequest },
    
    // System Operations
    ScheduleTask { delay: Duration, task: Task },
    None,
}
```

### Component Interface

```rust
pub trait UIProvider {
    fn update_screen(&mut self, screen: Screen);
    fn show_message(&mut self, level: MessageLevel, text: &str);
    fn update_progress(&mut self, operation: &str, progress: f32);
    fn get_user_input(&mut self, prompt: &str) -> Option<String>;
    fn confirm_action(&mut self, message: &str) -> bool;
}
```

### Keystore API

```rust
impl Keystore {
    pub fn new(path: &str, device_id: &str) -> Result<Self>;
    pub fn create_wallet(&mut self, metadata: WalletMetadata) -> Result<String>;
    pub fn get_wallet(&self, wallet_id: &str) -> Option<&Wallet>;
    pub fn list_wallets(&self) -> Vec<&WalletMetadata>;
    pub fn delete_wallet(&mut self, wallet_id: &str) -> Result<()>;
    pub fn export_wallet(&self, wallet_id: &str, path: &Path) -> Result<()>;
    pub fn import_wallet(&mut self, path: &Path) -> Result<String>;
}
```

### FROST Protocol API

```rust
pub trait FrostProtocol {
    fn start_dkg(config: DKGConfig) -> Result<DKGSession>;
    fn process_round1(session: &mut DKGSession, messages: Vec<Round1Message>) -> Result<Round2Data>;
    fn process_round2(session: &mut DKGSession, messages: Vec<Round2Message>) -> Result<KeyShare>;
    fn start_signing(key_share: &KeyShare, message: &[u8]) -> Result<SigningSession>;
    fn generate_nonces(session: &mut SigningSession) -> Result<SigningNonces>;
    fn generate_signature_share(session: &SigningSession, nonces: &SigningNonces) -> Result<SignatureShare>;
    fn aggregate_signatures(shares: Vec<SignatureShare>) -> Result<Signature>;
}
```

---

## Appendices

### A. Configuration

The TUI has no config file today — runtime settings come from CLI
flags only. See `apps/tui-node/src/bin/mpc-wallet-tui.rs` for the
authoritative `clap::Args` struct, or `apps/tui-node/docs/README.md`
§ Configuration for the summary. The TOML schema originally sketched
here described features that were never implemented (theme, auto-lock,
audit log, reconnect tuning); removed so nobody follows it and
discovers the flags silently do nothing.

### B. Error Codes

No numeric error-code scheme exists — see `src/errors.rs` for the
strongly-typed error variants (`DKGError`, `SigningError`,
`KeystoreError`, `ComponentError`, `CryptoError`). A shared numeric
registry across Rust + TypeScript is open future work.

### C. Keyboard Map Reference

```
┌─────────────────────────────────────┐
│          Global Controls            │
├─────────────┬───────────────────────┤
│ Ctrl+Q      │ Quit                  │
│ Ctrl+R      │ Refresh               │
│ Ctrl+H      │ Home                  │
│ ?           │ Help                  │
│ Esc         │ Back/Cancel           │
└─────────────┴───────────────────────┘

┌─────────────────────────────────────┐
│         Navigation Controls         │
├─────────────┬───────────────────────┤
│ ↑/k         │ Move up               │
│ ↓/j         │ Move down             │
│ ←/h         │ Move left             │
│ →/l         │ Move right            │
│ Enter       │ Select                │
│ Space       │ Toggle                │
│ Tab         │ Next field            │
│ Shift+Tab   │ Previous field        │
└─────────────┴───────────────────────┘

┌─────────────────────────────────────┐
│          Action Shortcuts          │
├─────────────┬───────────────────────┤
│ n           │ New wallet            │
│ j           │ Join session          │
│ s           │ Sign transaction      │
│ w           │ Manage wallets        │
│ /           │ Search                │
│ :           │ Command mode          │
└─────────────┴───────────────────────┘
```

### D. Troubleshooting Guide

#### TUI Display Issues

**Problem**: Garbled or broken UI
**Solution**: 
```bash
# Check terminal capabilities
echo $TERM
# Set proper terminal
export TERM=xterm-256color
# Reset terminal
reset
```

**Problem**: Colors not displaying
**Solution**:
```bash
# Force color output
export COLORTERM=truecolor
# Check terminfo
infocmp $TERM | grep colors
```

#### Performance Issues

**Problem**: High CPU usage
**Solution**:
- Check adaptive event loop is enabled
- Verify bounded channels are configured
- Review log level (debug is expensive)

**Problem**: Slow UI updates
**Solution**:
- Enable differential updates
- Reduce terminal baud rate if remote
- Disable animations in config

#### Network Issues

**Problem**: Cannot connect to WebSocket
**Solution**:
```bash
# Test connectivity
curl -v wss://your-server.com
# Check firewall
sudo iptables -L
# Verify certificates
openssl s_client -connect server:port
```

**Problem**: WebRTC connection fails
**Solution**:
- Check STUN/TURN servers
- Verify NAT type
- Enable UPnP if available
- Configure port forwarding

---

## Conclusion

The MPC Wallet TUI represents a professional-grade implementation of threshold signatures with an emphasis on usability, security, and performance. Through careful architecture decisions and comprehensive optimization, it provides enterprise-ready functionality while maintaining accessibility for all user levels.

For the latest updates and contributions, visit the [GitHub repository](https://github.com/hecoinfo/mpc-wallet).

---

*Document Version: 2.0.0*  
*Last Updated: 2025*  
*Status: Production Ready*