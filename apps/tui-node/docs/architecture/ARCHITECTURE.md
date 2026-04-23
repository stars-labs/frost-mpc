# FROST MPC TUI Wallet - Architecture

## Table of Contents

1. [System Overview](#system-overview)
2. [Core Components](#core-components)
3. [TUI Architecture](#tui-architecture)
4. [Network Layer](#network-layer)
5. [Cryptographic Core](#cryptographic-core)
6. [Storage System](#storage-system)
7. [Security Architecture](#security-architecture)
8. [Performance Considerations](#performance-considerations)
9. [Extension Points](#extension-points)

## System Overview

The FROST MPC TUI Wallet is built as a modular, event-driven system that provides enterprise-grade multi-party computation through a terminal interface. The architecture prioritizes security, usability, and extensibility.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Terminal UI Layer                       │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Ratatui   │  │ UI Provider  │  │  Event Handler   │  │
│  │  Framework  │  │  Interface   │  │     System       │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Business Logic Layer                      │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Session   │  │    Wallet    │  │   Transaction    │  │
│  │  Manager    │  │   Manager    │  │     Engine       │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      Network Layer                           │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  WebSocket  │  │    WebRTC    │  │    Offline       │  │
│  │   Client    │  │     Mesh     │  │    Handler       │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Cryptographic Core                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │    FROST    │  │   Keystore   │  │   Threshold      │  │
│  │   Protocol  │  │  Encryption  │  │    Signing       │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Separation of Concerns**: Clear boundaries between UI, business logic, and cryptography
2. **Event-Driven Architecture**: Asynchronous message passing between components
3. **Security by Design**: Cryptographic operations isolated in secure modules
4. **User-Centric Interface**: TUI designed for ease of use without sacrificing functionality
5. **Network Resilience**: Support for both online and offline operations

## Core Components

### Application Entry (`elm/app.rs`)

The real entry struct is `ElmApp<C>`, not a named `AppRunner`.
Earlier drafts of this doc referenced an `AppRunner` type that
never existed in source.

```rust
// src/elm/app.rs
pub struct ElmApp<C: frost_core::Ciphersuite> {
    model: Model,                                // pure UI state
    app: Application<Id, Message, UserEvent>,    // tui-realm shell
    terminal: CrosstermTerminalAdapter,
    message_tx: UnboundedSender<Message>,
    message_rx: UnboundedReceiver<Message>,
    app_state: Arc<Mutex<AppState<C>>>,          // shared with non-Elm managers
    should_quit: bool,
}
```

See [`ELM_ARCHITECTURE.md`](./ELM_ARCHITECTURE.md) for the
Model/Update/View breakdown.

### UI Provider System (`elm/provider.rs`)

Trait abstracting UI backends so non-Elm managers (the `core::*Manager`
types reused by native-node) can push state without knowing whether
they're driving a Ratatui TUI, a Slint GUI, or a test harness:

```rust
#[async_trait]
pub trait UIProvider: Send + Sync {
    // Connection + device list
    async fn set_connection_status(&self, connected: bool);
    async fn set_device_id(&self, device_id: String);
    async fn update_device_list(&self, devices: Vec<String>);

    // Session / DKG / signing updates
    async fn update_session_status(&self, status: String);
    async fn add_session_invite(&self, invite: SessionInfo);
    async fn update_dkg_status(&self, status: String);
    async fn add_signing_request(&self, request: PendingSigningRequest);
    async fn set_signature_result(&self, signing_id: String, signature: Vec<u8>);

    // Wallet list + logs + mesh status + error/progress
    async fn update_wallet_list(&self, wallets: Vec<WalletDisplayInfo>);
    async fn add_log(&self, message: String);
    async fn update_mesh_status(&self, ready: usize, total: usize);
    async fn show_error(&self, error: String);
    async fn set_busy(&self, busy: bool);
    // …etc., see provider.rs for the full surface
}
```

**Real implementations:**
- `NoOpUIProvider` (`elm/provider.rs`) — no-op for tests / headless
- The TUI itself drives UI updates through the tui-realm Elm loop
  rather than implementing `UIProvider` directly. (Earlier drafts
  of this doc listed `TuiProvider` / `CliProvider` / `TestProvider`
  implementations that don't exist in source — removed.)

### State Management (`utils/appstate_compat.rs`)

`AppState<C: Ciphersuite>` is the shared state container — a
thread-safe (`Arc<Mutex<AppState<C>>>`) blob holding the pieces
that the Elm `Model` doesn't own: peer connections, ICE candidates,
DKG/signing FROST state, etc.

Key fields (abbreviated — full struct in `utils/appstate_compat.rs`):

```rust
pub struct AppState<C: Ciphersuite> {
    // Identity + network
    pub device_id: String,
    pub signal_server_url: String,
    pub devices: Vec<String>,

    // Session
    pub session: Option<SessionInfo>,
    pub invites: Vec<SessionInfo>,
    pub available_sessions: Vec<SessionAnnouncement>,

    // Keystore + blockchain surface
    pub keystore: Option<Arc<Keystore>>,
    pub blockchain_addresses: Vec<BlockchainInfo>,
    pub current_wallet_id: Option<String>,

    // WebRTC mesh (per-peer tables)
    pub device_connections: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    pub data_channels: HashMap<String, Arc<RTCDataChannel>>,
    pub device_statuses: HashMap<String, RTCPeerConnectionState>,
    pub pending_ice_candidates: HashMap<String, Vec<RTCIceCandidateInit>>,

    // DKG state machine + packages
    pub mesh_status: MeshStatus,
    pub dkg_state: DkgState,
    pub dkg_round1_packages: BTreeMap<Identifier<C>, round1::Package<C>>,
    pub dkg_round2_packages: BTreeMap<Identifier<C>, round2::Package<C>>,
    pub key_package: Option<KeyPackage<C>>,
    pub group_public_key: Option<VerifyingKey<C>>,

    // Signing state machine + FROST intermediates
    pub signing_state: SigningState<C>,
    pub pending_signing_requests: Vec<PendingSigningRequest>,
    pub frost_commitments: BTreeMap<Identifier<C>, SigningCommitments<C>>,
    pub frost_signature_shares: BTreeMap<Identifier<C>, SignatureShare<C>>,
    pub frost_nonces: Option<SigningNonces<C>>,
    pub signing_message: Option<Vec<u8>>,

    pub log: Vec<String>,
}
```

No `curve_type`, `wallets: HashMap<…>`, `pending_operations: VecDeque<…>`,
`network_status`, or `offline_mode` fields — those were listed in
earlier drafts of this doc. Curve is per-wallet (lives in the
keystore's `WalletMetadata`), wallets live in the keystore, signing
requests queue through `pending_signing_requests`, and offline-mode
is set at startup via the `--offline` CLI flag (no runtime toggle
field).

## TUI Architecture

### Ratatui Integration

The TUI is built on Ratatui, providing a responsive terminal interface:

```rust
pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    ui_state: UIState,
    event_receiver: mpsc::Receiver<Event>,
    command_sender: mpsc::Sender<Command>,
}
```

### UI Components

#### Main Layout
```
┌─ Title Bar ─────────────────────────┐
├─ Menu/Content Area ─────────────────┤
│                                      │
├─ Activity Log ──────────────────────┤
│                                      │
├─ Status Bar ────────────────────────┤
```

#### Component Tree
```
App
├── TitleBar
│   ├── AppName
│   ├── DeviceId
│   └── ConnectionStatus
├── ContentArea
│   ├── MainMenu
│   ├── WalletList
│   ├── SessionView
│   └── PopupManager
├── ActivityLog
│   └── LogEntries
└── StatusBar
    ├── NetworkIndicator
    ├── WalletCount
    └── NotificationBadge
```

### Event System

Events flow through a centralized event bus:

```rust
pub enum UIEvent {
    KeyPress(KeyEvent),
    MenuSelection(MenuItem),
    NetworkUpdate(NetworkStatus),
    SessionUpdate(SessionInfo),
    WalletUpdate(WalletInfo),
    Notification(NotificationType),
}
```

### Rendering Pipeline

1. **Event Collection**: Keyboard and system events
2. **State Update**: Modify application state based on events
3. **View Calculation**: Determine what to render
4. **Buffer Rendering**: Draw to terminal buffer
5. **Terminal Update**: Flush buffer to screen

## Network Layer

### WebSocket Communication

Maintains persistent connection to signaling server:

```rust
pub struct WebSocketClient {
    url: String,
    connection: Option<WebSocketStream>,
    message_handler: Arc<dyn MessageHandler>,
    reconnect_strategy: ReconnectStrategy,
}
```

**Features:**
- Automatic reconnection with exponential backoff
- Message queuing during disconnections
- TLS certificate validation
- Binary and text message support

### WebRTC Mesh Network

Peer-to-peer communication for DKG and signing:

```rust
pub struct WebRTCManager {
    local_peer: PeerConnection,
    remote_peers: HashMap<String, PeerConnection>,
    data_channels: HashMap<String, DataChannel>,
    ice_servers: Vec<IceServer>,
}
```

**Mesh Formation Process:**
1. Signal server facilitates peer discovery
2. ICE candidates exchanged via signaling
3. Direct P2P connections established
4. Data channels created for protocol messages

### Offline Data Transfer

Support for air-gapped operations:

```rust
pub struct OfflineHandler {
    import_queue: VecDeque<OfflinePacket>,
    export_queue: VecDeque<OfflinePacket>,
    sd_card_monitor: FileSystemWatcher,
}
```

## Cryptographic Core

### FROST Protocol Implementation

Threshold signature scheme with distributed key generation:

```rust
pub struct FrostProtocol<C: Ciphersuite> {
    round: ProtocolRound,
    participant_index: u16,
    threshold: u16,
    total_participants: u16,
    commitments: HashMap<u16, Commitment<C>>,
    shares: HashMap<u16, Share<C>>,
}
```

**Protocol Rounds:**
1. **DKG Round 1**: Generate and broadcast commitments
2. **DKG Round 2**: Generate and send secret shares
3. **Key Derivation**: Compute public key from shares
4. **Signing Round 1**: Generate nonce commitments
5. **Signing Round 2**: Create signature shares
6. **Aggregation**: Combine shares into final signature

### Keystore Architecture

Secure storage for cryptographic materials:

```rust
pub struct Keystore {
    encryption_key: DerivedKey,
    wallets: HashMap<String, EncryptedWallet>,
    metadata: KeystoreMetadata,
}

pub struct EncryptedWallet {
    encrypted_share: Vec<u8>,
    nonce: [u8; 12],
    wallet_info: WalletInfo,
    participant_info: ParticipantInfo,
}
```

**Encryption Scheme:**
- Key Derivation: PBKDF2-SHA256 (100,000 iterations)
- Encryption: AES-256-GCM
- Authentication: Built into GCM mode
- Backup Format: Compatible with the browser extension keystore format (PBKDF2 + AES-256-GCM round-trip tested)

## Storage System

### Directory Structure

The structure is partitioned by device_id and curve (see
`src/keystore/storage.rs`):

```
~/.frost_keystore/
├── index.json                    # Wallet index (device_id × curve → wallet list)
├── device_id                     # This node's device_id
└── <device_id>/
    ├── ed25519/
    │   ├── <wallet_id>.json      # Wallet metadata (threshold, participants, etc.)
    │   └── <wallet_id>.dat       # Encrypted FROST key share (AES-256-GCM)
    └── secp256k1/
        ├── <wallet_id>.json
        └── <wallet_id>.dat
```

The TUI currently has no config-file, session-history, log-archive, or
automated-backup functionality — all runtime config goes through CLI
flags (see `apps/tui-node/src/bin/mpc-wallet-tui.rs`), and logs stream
to the path passed via `--log-location`.

### Data Persistence

```rust
pub trait StorageBackend {
    fn save_wallet(&self, wallet: &EncryptedWallet) -> Result<()>;
    fn load_wallet(&self, name: &str) -> Result<EncryptedWallet>;
    fn list_wallets(&self) -> Result<Vec<WalletInfo>>;
    fn delete_wallet(&self, name: &str) -> Result<()>;
}
```

**Implementations:**
- `FileSystemBackend`: Default local storage
- `MemoryBackend`: For testing
- `RemoteBackend`: Future cloud backup support

## Security Architecture

### Threat Model

1. **Network Adversary**: Can observe and modify network traffic
2. **Compromised Participant**: One or more malicious participants
3. **Local Malware**: Malicious software on user's machine
4. **Physical Access**: Attacker with device access

### Security Measures

#### Cryptographic Security
- FROST protocol provides threshold security
- No single party holds complete private key
- Signatures require threshold participation

#### Network Security
- TLS for all WebSocket connections
- DTLS for WebRTC data channels
- Certificate pinning for known servers

#### Local Security
- Keystore encryption at rest
- Memory protection for sensitive data
- Secure random number generation

#### Operational Security
- Offline mode for air-gapped signing
- Session timeouts and expiration
- Audit logs for all operations

### Security Boundaries

```
┌─────────────────────────────────────┐
│         Untrusted Zone              │
│  - Network Communication            │
│  - Signal Server                    │
│  - Other Participants               │
├─────────────────────────────────────┤
│      Trust Boundary                 │
├─────────────────────────────────────┤
│         Trusted Zone                │
│  - Local Keystore                   │
│  - FROST Protocol Core              │
│  - UI Event Handler                 │
└─────────────────────────────────────┘
```

## Performance Considerations

### Optimization Strategies

1. **Async I/O**: All network operations are non-blocking
2. **Message Batching**: Combine multiple protocol messages
3. **Connection Pooling**: Reuse WebRTC connections
4. **Lazy Loading**: Load wallets on demand
5. **Efficient Rendering**: Only redraw changed UI sections

### Resource Management

```rust
pub struct ResourceManager {
    connection_pool: ConnectionPool,
    message_batcher: MessageBatcher,
    state_cache: StateCache,
    render_throttle: RenderThrottle,
}
```

### Performance Metrics

- **DKG Completion**: < 5 seconds for 3-party setup
- **Signing Time**: < 2 seconds with all parties online
- **UI Responsiveness**: < 50ms for user interactions
- **Memory Usage**: < 100MB typical, < 500MB peak

## Extension Points

### Plugin System (Future)

```rust
pub trait WalletPlugin {
    fn name(&self) -> &str;
    fn supported_chains(&self) -> Vec<Blockchain>;
    fn create_transaction(&self, params: TxParams) -> Result<Transaction>;
    fn verify_address(&self, address: &str) -> Result<bool>;
}
```

### Custom UI Themes

```rust
pub struct Theme {
    pub colors: ColorScheme,
    pub borders: BorderStyle,
    pub symbols: SymbolSet,
}
```

### Protocol Extensions

- Support for additional curves
- Custom threshold schemes
- Multi-signature protocols
- Hardware wallet integration

### Integration APIs

```rust
// REST API for external integration
pub trait ExternalAPI {
    fn create_wallet(&self, params: WalletParams) -> Result<WalletId>;
    fn sign_transaction(&self, wallet: WalletId, tx: Transaction) -> Result<Signature>;
    fn get_wallet_info(&self, wallet: WalletId) -> Result<WalletInfo>;
}
```

## Development Guidelines

### Module Organization

```
src/
├── app_runner.rs       # Application orchestration
├── ui/                 # Terminal UI components
│   ├── mod.rs
│   ├── tui.rs         # Main TUI implementation
│   ├── provider.rs    # UI abstraction
│   └── widgets/       # Custom widgets
├── network/           # Networking code
│   ├── websocket.rs
│   ├── webrtc.rs
│   └── offline.rs
├── protocol/          # FROST implementation
│   ├── dkg.rs
│   ├── signing.rs
│   └── types.rs
├── keystore/          # Secure storage
│   ├── encryption.rs
│   ├── storage.rs
│   └── models.rs
└── handlers/          # Business logic
    ├── session_handler.rs
    ├── wallet_handler.rs
    └── transaction_handler.rs
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
```

### Testing Strategy

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Multi-component interaction
3. **Protocol Tests**: FROST protocol compliance
4. **UI Tests**: Terminal UI behavior
5. **Security Tests**: Penetration testing scenarios

### Future Enhancements

1. **Hardware Security Module Support**
   - Integration with HSMs for key storage
   - PKCS#11 interface support

2. **Multi-Protocol Support**
   - Additional threshold signature schemes
   - Post-quantum cryptography preparation

3. **Enterprise Features**
   - LDAP/Active Directory integration
   - Compliance reporting
   - Advanced audit trails

4. **Cloud Integration**
   - Encrypted cloud backup
   - Multi-device synchronization
   - Remote signing capabilities