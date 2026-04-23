# FROST MPC TUI Wallet - DKG Flows

## Table of Contents

1. [Overview](#overview)
2. [Online DKG Flow](#online-dkg-flow)
3. [Offline DKG Flow](#offline-dkg-flow)
4. [Hybrid DKG Flow](#hybrid-dkg-flow)
5. [Recovery Procedures](#recovery-procedures)
6. [Security Considerations](#security-considerations)
7. [Troubleshooting](#troubleshooting)

## Overview

Distributed Key Generation (DKG) is the foundational process for creating MPC wallets. The FROST protocol enables multiple parties to jointly generate a key pair where no single party ever has access to the complete private key. This document details both online and offline DKG procedures.

### Key Concepts

- **Threshold (t)**: Minimum number of participants needed to sign
- **Participants (n)**: Total number of key share holders
- **Key Shares**: Individual pieces of the distributed private key
- **Verification Shares**: Public commitments used to verify operations

### DKG Properties

1. **Distributed Trust**: No single point of failure
2. **Verifiable**: All participants can verify correct execution
3. **Robust**: Can complete even if some parties fail (up to n-t failures)
4. **Secure**: Threshold of parties required to reconstruct private key

## Online DKG Flow

The online DKG process uses WebRTC mesh networking for real-time coordination between participants.

### Prerequisites

- All participants online simultaneously for the duration of the
  DKG ceremony
- WebRTC-routable network: STUN is enough for most home NATs
  (full-cone / restricted-cone / port-restricted-cone). Symmetric-NAT
  peers may fail to connect because no TURN server ships with this
  repo. If a participant is behind symmetric NAT, either run the
  ceremony in offline mode over SD card, or stand up your own TURN
  and point the clients at it.

Earlier drafts of this section listed "Synchronized system clocks
(±5 minutes tolerance)" as a prerequisite — FROST DKG is not
time-sensitive; remove that from your checklist.

### Step-by-Step Process

#### 1. Session Initiation

**Coordinator's View:**
```
┌─────────────────────────────────────────────────────┐
│ Create New Wallet - Online DKG                      │
├─────────────────────────────────────────────────────┤
│ Wallet Configuration:                               │
│                                                     │
│ Name: [treasury-wallet_______________]              │
│ Blockchain: [Ethereum (secp256k1)] ▼               │
│ Participants: [3] ▼                                 │
│ Threshold: [2] ▼                                    │
│                                                     │
│ Available Participants (3 online):                  │
│ ☑ alice (coordinator - you)                         │
│ ☑ bob (online - 192.168.1.10)                      │
│ ☑ charlie (online - 192.168.1.11)                  │
│ ☐ dave (offline)                                   │
│                                                     │
│ (No pre-flight NAT/bandwidth check is run by the TUI.
│ The Network Check panel in earlier drafts of this mockup
│ claimed a "Symmetric NAT (WebRTC compatible)" status —
│ backwards: symmetric NAT is the HARDEST case for WebRTC
│ without TURN. Reality is that DKG is attempted directly
│ over the peer mesh once signaling completes; failures
│ surface as peer-connection timeouts, not a pre-flight
│ "bandwidth insufficient" gate.)                     │
│                                                     │
│ [Start DKG] [Test Connection] [Cancel]             │
└─────────────────────────────────────────────────────┘
```

#### 2. Participant Invitation

**Participant's View:**
```
┌─────────────────────────────────────────────────────┐
│ 🔔 DKG Session Invitation                           │
├─────────────────────────────────────────────────────┤
│ Coordinator: alice                                  │
│ Wallet Name: treasury-wallet                        │
│ Type: 2-of-3 Ethereum Wallet                       │
│                                                     │
│ Your Role: Participant #2                           │
│ Other Participants:                                 │
│ • alice (Coordinator)                               │
│ • charlie (Pending)                                 │
│                                                     │
│ Session Details:                                    │
│ • Created: 2024-01-20 10:30:15                     │
│ • Expires: 2024-01-20 10:45:15 (15 min)           │
│ • Protocol: FROST-secp256k1                        │
│                                                     │
│ ⚠️  Joining will start key generation immediately  │
│                                                     │
│ [Accept & Join] [Decline] [View Details]           │
└─────────────────────────────────────────────────────┘
```

#### 3. WebRTC Mesh Formation

**Connection Status Display:**
```
┌─────────────────────────────────────────────────────┐
│ Establishing Secure Connections                     │
├─────────────────────────────────────────────────────┤
│ Building P2P mesh network...                        │
│                                                     │
│ Connections:                                        │
│ • You → bob     [████████████░░░░] Connecting...   │
│ • You → charlie [████████████████] Connected       │
│ • bob → charlie [████████████████] Connected       │
│                                                     │
│ Network Quality:                                    │
│ • Latency: 12ms average                            │
│ • Packet Loss: 0.0%                                │
│ • Encryption: DTLS 1.3                             │
│                                                     │
│ Status: Waiting for all connections...             │
│                                                     │
│ [Details] [Abort]                                  │
└─────────────────────────────────────────────────────┘
```

#### 4. DKG Protocol Execution

**Round 1 - Commitment Generation:**
```
┌─────────────────────────────────────────────────────┐
│ DKG Progress - Round 1 of 2                         │
├─────────────────────────────────────────────────────┤
│ Generating cryptographic commitments...             │
│                                                     │
│ Local Operations:                                   │
│ ✅ Generated secret polynomial                      │
│ ✅ Computed Feldman commitments                     │
│ ✅ Created proof of knowledge                       │
│                                                     │
│ Broadcast Status:                                   │
│ • To bob:     ✅ Sent (confirmed)                  │
│ • To charlie: ✅ Sent (confirmed)                  │
│                                                     │
│ Received Commitments:                               │
│ • From bob:     ✅ Valid                           │
│ • From charlie: ⏳ Waiting...                      │
│                                                     │
│ Round Progress: ▓▓▓▓▓▓▓▓▓▓░░░░░ 66%               │
│                                                     │
│ [View Technical Details] [Pause]                   │
└─────────────────────────────────────────────────────┘
```

**Round 2 - Share Distribution:**
```
┌─────────────────────────────────────────────────────┐
│ DKG Progress - Round 2 of 2                         │
├─────────────────────────────────────────────────────┤
│ Distributing encrypted shares...                    │
│                                                     │
│ Share Generation:                                   │
│ ✅ Computed shares for each participant             │
│ ✅ Encrypted with participant public keys           │
│ ✅ Generated zero-knowledge proofs                  │
│                                                     │
│ Distribution Status:                                │
│ • To bob:     ✅ Delivered & Acknowledged          │
│ • To charlie: ✅ Delivered & Acknowledged          │
│                                                     │
│ Share Verification:                                 │
│ • From bob:     ✅ Valid share received            │
│ • From charlie: ✅ Valid share received            │
│                                                     │
│ Final Verification:                                 │
│ ✅ All shares consistent with commitments           │
│ ✅ Threshold parameters verified                    │
│                                                     │
│ [Complete DKG] [View Shares]                       │
└─────────────────────────────────────────────────────┘
```

#### 5. Wallet Finalization

**Success Screen:**
```
┌─────────────────────────────────────────────────────┐
│ ✅ Wallet Created Successfully!                     │
├─────────────────────────────────────────────────────┤
│ Wallet Details:                                     │
│ • Name: treasury-wallet                            │
│ • Type: 2-of-3 Ethereum Wallet                     │
│ • Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f │
│                                                     │
│ Your Key Share:                                     │
│ • Share Index: 2                                    │
│ • Public Share: 0x04a8b3...                        │
│ • Status: Encrypted and saved                      │
│                                                     │
│ Other Participants:                                 │
│ • alice: Share 1 ✅                                 │
│ • charlie: Share 3 ✅                               │
│                                                     │
│ Next Steps:                                         │
│ 1. Test wallet with small transaction              │
│ 2. Create secure backup                            │
│ 3. Document participant contacts                   │
│                                                     │
│ [View Wallet] [Create Backup] [Done]               │
└─────────────────────────────────────────────────────┘
```

### Online DKG Sequence Diagram

```
Alice (Coordinator)     Bob (Participant)      Charlie (Participant)
        |                       |                       |
        |---- Create Session -->|                       |
        |                       |                       |
        |<--- Accept -------->  |                       |
        |                       |                       |
        |------ Invite -------->|------- Invite ------->|
        |                       |                       |
        |<---- Accept ----------|<----- Accept ---------|
        |                       |                       |
        |==== WebRTC Setup =====|===== WebRTC Setup ====|
        |                       |                       |
        |---- Round 1 Comm ---->|---- Round 1 Comm ---->|
        |<--- Round 1 Comm -----|<--- Round 1 Comm -----|
        |                       |                       |
        |---- Round 2 Share --->|---- Round 2 Share --->|
        |<--- Round 2 Share ----|<--- Round 2 Share ----|
        |                       |                       |
        |===== Verify ==========|====== Verify =========|
        |                       |                       |
        |---- Complete -------->|---- Complete -------->|
```

## Offline DKG Flow

The offline DKG process enables key generation without network connectivity, using removable media for data exchange.

### Prerequisites

- Dedicated, air-gapped machines for each participant
- Removable media (SD cards, USB drives)
- Secure physical channel for media exchange
- Trusted coordinator for orchestration

### Step-by-Step Process

#### 1. Offline Mode Activation

**Each Participant:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Enable Offline Mode                              │
├─────────────────────────────────────────────────────┤
│ Current Status: Online                              │
│                                                     │
│ Offline Mode Checklist:                            │
│ ☑ Network interfaces will be disabled              │
│ ☑ SD card mounted at: /mnt/secure-sd              │
│ ☑ System clock synchronized                        │
│ ☑ Temporary files cleared                          │
│                                                     │
│ Security Verification:                              │
│ • WiFi: Will be disabled                           │
│ • Ethernet: Will be disabled                       │
│ • Bluetooth: Will be disabled                      │
│ • USB: Restricted to storage only                  │
│                                                     │
│ ⚠️  This action cannot be undone without restart   │
│                                                     │
│ [Enable Offline Mode] [Cancel]                     │
└─────────────────────────────────────────────────────┘
```

#### 2. DKG Parameters Exchange

**Coordinator Creates DKG Package:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Create Offline DKG Package                       │
├─────────────────────────────────────────────────────┤
│ DKG Configuration:                                  │
│                                                     │
│ Wallet Name: cold-storage                           │
│ Participants: 3                                     │
│ Threshold: 2                                        │
│ Blockchain: Bitcoin (secp256k1)                    │
│                                                     │
│ Participant Information:                            │
│ 1. alice-airgap (Coordinator)                      │
│ 2. bob-airgap                                      │
│ 3. charlie-airgap                                  │
│                                                     │
│ Package Contents:                                   │
│ • DKG parameters                                    │
│ • Participant identifiers                           │
│ • Session metadata                                  │
│ • Expiration: 48 hours                             │
│                                                     │
│ Export Location: /mnt/secure-sd/dkg-init.json      │
│                                                     │
│ [Generate Package] [Cancel]                         │
└─────────────────────────────────────────────────────┘
```

**Participants Import Package:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Import DKG Package                               │
├─────────────────────────────────────────────────────┤
│ SD Card Status: Mounted                             │
│ Found DKG package: dkg-init.json                    │
│                                                     │
│ Package Details:                                    │
│ • Created by: alice-airgap                         │
│ • Created at: 2024-01-20 10:00:00                 │
│ • Expires at: 2024-01-22 10:00:00                 │
│ • Signature: ✅ Valid                              │
│                                                     │
│ DKG Parameters:                                     │
│ • Wallet: cold-storage                             │
│ • Your Role: Participant #2 (bob-airgap)          │
│ • Threshold: 2 of 3                                │
│                                                     │
│ [Import & Continue] [Reject] [View Raw]            │
└─────────────────────────────────────────────────────┘
```

#### 3. Round 1 - Commitment Generation

**Each Participant Generates Commitments:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Generate DKG Commitments (Offline)               │
├─────────────────────────────────────────────────────┤
│ Round 1 - Local Generation                          │
│                                                     │
│ Operations:                                         │
│ ✅ Generated random polynomial                      │
│ ✅ Computed commitment values                       │
│ ✅ Created cryptographic proofs                     │
│ ✅ Self-verification passed                         │
│                                                     │
│ Commitment Data:                                    │
│ • Size: 2.3 KB                                      │
│ • Format: JSON (signed)                             │
│ • Includes: Public commitments only                │
│                                                     │
│ Ready to export to SD card:                        │
│ /mnt/secure-sd/round1/bob-commitments.json        │
│                                                     │
│ Instructions:                                       │
│ 1. Export your commitments                         │
│ 2. Deliver SD card to coordinator                  │
│ 3. Wait for aggregated commitments                 │
│                                                     │
│ [Export Commitments] [Regenerate]                  │
└─────────────────────────────────────────────────────┘
```

**Coordinator Aggregates Commitments:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Aggregate Round 1 Commitments                    │
├─────────────────────────────────────────────────────┤
│ Commitment Collection Status:                       │
│                                                     │
│ Received Commitments:                               │
│ ✅ alice-airgap: alice-commitments.json           │
│ ✅ bob-airgap: bob-commitments.json               │
│ ⏳ charlie-airgap: Waiting...                      │
│                                                     │
│ Verification Results:                               │
│ • alice: ✅ Valid signature & proofs               │
│ • bob: ✅ Valid signature & proofs                 │
│                                                     │
│ [Refresh] [Import from SD] [Verify All]            │
│                                                     │
│ Once all commitments received:                      │
│ [Create Round 1 Package]                           │
└─────────────────────────────────────────────────────┘
```

#### 4. Round 2 - Share Distribution

**Participants Generate Shares:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Generate Secret Shares (Offline)                 │
├─────────────────────────────────────────────────────┤
│ Round 2 - Share Generation                          │
│                                                     │
│ Imported Round 1 Package: ✅                        │
│ All commitments verified: ✅                        │
│                                                     │
│ Share Generation:                                   │
│ • For alice-airgap: ✅ Encrypted                   │
│ • For charlie-airgap: ✅ Encrypted                 │
│ • Self share: ✅ Stored locally                    │
│                                                     │
│ Export Package Contents:                            │
│ • Encrypted shares for others                      │
│ • Zero-knowledge proofs                            │
│ • Share commitments                                │
│                                                     │
│ Ready to export:                                    │
│ /mnt/secure-sd/round2/bob-shares.json             │
│                                                     │
│ [Export Shares] [Verify] [Back]                    │
└─────────────────────────────────────────────────────┘
```

**Share Verification:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Verify Received Shares                           │
├─────────────────────────────────────────────────────┤
│ Share Import Status:                                │
│                                                     │
│ Received Shares:                                    │
│ • From alice-airgap: ✅ Valid                      │
│ • From charlie-airgap: ✅ Valid                    │
│                                                     │
│ Verification Steps:                                 │
│ ✅ Decrypted shares successfully                    │
│ ✅ Shares match commitment values                   │
│ ✅ Polynomial consistency verified                  │
│ ✅ Zero-knowledge proofs valid                      │
│                                                     │
│ Key Reconstruction Test:                            │
│ ✅ Successfully computed public key                 │
│ ✅ Address derivation successful                    │
│                                                     │
│ Your Key Share: Securely stored                    │
│                                                     │
│ [Complete DKG] [Export Summary]                    │
└─────────────────────────────────────────────────────┘
```

#### 5. Final Verification

**All Participants Confirm:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Offline DKG Complete                             │
├─────────────────────────────────────────────────────┤
│ ✅ Cold Storage Wallet Created                      │
│                                                     │
│ Wallet Summary:                                     │
│ • Name: cold-storage                               │
│ • Type: 2-of-3 Bitcoin Wallet                      │
│ • Address: bc1qxy2kgdygjrsqtzq2n0yrf24...         │
│                                                     │
│ Security Verification:                              │
│ ✅ No network activity detected                     │
│ ✅ All operations performed offline                 │
│ ✅ Key material never exposed                       │
│ ✅ Shares encrypted at rest                         │
│                                                     │
│ Backup Reminder:                                    │
│ ⚠️  Create encrypted backup immediately            │
│ ⚠️  Store backup in separate location              │
│ ⚠️  Test recovery procedure                        │
│                                                     │
│ [Create Backup] [View Details] [Exit]              │
└─────────────────────────────────────────────────────┘
```

### Offline DKG Data Flow

```
Coordinator                 Participant 1              Participant 2
     |                           |                           |
     |-- DKG Parameters -------->|                           |
     |         (SD Card)         |-- DKG Parameters -------->|
     |                           |      (SD Card)            |
     |                           |                           |
     |<-- Round 1 Commitments ---|<-- Round 1 Commitments ---|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |-- Aggregated Commitments->|-- Aggregated Commitments->|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |<-- Round 2 Shares --------|<-- Round 2 Shares --------|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |-- Share Packages -------->|-- Share Packages -------->|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |==== Local Verify =========|==== Local Verify =========|
```

## Hybrid DKG Flow

The hybrid approach combines online coordination with offline key generation for enhanced security.

### Use Cases

1. **High-Value Wallets**: Online coordination, offline key generation
2. **Geographically Distributed Teams**: Mixed online/offline participants
3. **Regulatory Compliance**: Audit trail with air-gapped security

### Process Overview

```
┌─────────────────────────────────────────────────────┐
│ Hybrid DKG Configuration                            │
├─────────────────────────────────────────────────────┤
│ Coordination: Online (WebRTC)                       │
│ Key Generation: Offline (Air-gapped)                │
│                                                     │
│ Participants:                                       │
│ • alice: Online coordination + Offline keygen      │
│ • bob: Fully offline (SD card only)               │
│ • charlie: Online coordination + Offline keygen    │
│                                                     │
│ Workflow:                                           │
│ 1. Online: Establish session parameters            │
│ 2. Offline: Generate commitments                   │
│ 3. Online: Exchange commitments                    │
│ 4. Offline: Generate shares                        │
│ 5. Online: Exchange encrypted shares               │
│ 6. Offline: Verify and store                       │
│                                                     │
│ [Configure Details] [Start Hybrid DKG]             │
└─────────────────────────────────────────────────────┘
```

## Recovery Procedures

### Lost Key Share Recovery

If a participant loses their key share, the only currently-shipping
recovery path is restoring from backup:

- **Restore from backup**: Decrypt an exported keystore file
  (`.json`+`.dat` pair from `~/.frost_keystore/`, or an extension-format
  export) with the original password. Works provided the participant
  kept a copy. The extension/TUI round-trip is test-covered
  (`extension_compat.rs`).

Earlier drafts of this section offered two more recovery methods
that do NOT exist in source today:

- **"Threshold Recovery: generate a new 2-of-3 wallet, Bob gets a new
  share"** — this is cryptographically incoherent. Re-running DKG
  produces a completely new group public key (and therefore a
  completely different on-chain address); it does not reissue a
  lost share for an existing key. A new DKG == a new wallet.
- **"Share Refresh Protocol"** — FROST supports proactive share
  refresh in principle (updating shares so the same group key is
  preserved while old shares become useless), but this crate does
  not implement it. Adding refresh is open work, tracked as a
  future item.

If a share is lost and no backup exists, the participant is out of
the threshold. If the remaining participants still hit the threshold
`t`, the wallet can still sign; if they don't, the funds under that
group key are permanently inaccessible — standard threshold-signature
failure mode.

### Emergency access

The TUI displays no presence / last-seen / timezone information for
participants. Earlier drafts of this section showed a panel with
"Alice: Last seen 2 hours ago" / "Time-Locked Recovery" / "Social
Recovery Protocol: 3 of 4 trustees online" — none of those features
exist. The only emergency options today are:

1. Gather threshold participants (possibly out-of-band) and sign the
   required transaction through the normal signing flow.
2. If threshold participation is impossible, the funds are
   inaccessible. Plan for this by keeping encrypted backups of every
   share in safe places ahead of time.

## Security Considerations

### DKG Security Model

```
┌─────────────────────────────────────────────────────┐
│ Security Properties                                 │
├─────────────────────────────────────────────────────┤
│ ✅ Guaranteed Properties:                           │
│ • No single party has complete key                 │
│ • Threshold parties required for signing           │
│ • Verifiable correct execution                     │
│ • Robust against t-1 malicious parties            │
│                                                     │
│ ⚠️  Assumptions:                                    │
│ • Secure communication channels                    │
│ • Honest majority during DKG                      │
│ • Secure local storage                            │
│ • Trusted execution environment                    │
│                                                     │
│ 🔒 Best Practices:                                  │
│ • Verify participant identities                    │
│ • Use offline DKG for high-value                  │
│ • Regular key share backups                       │
│ • Periodic share refresh                          │
└─────────────────────────────────────────────────────┘
```

### Attack Vectors and Mitigations

| Attack Vector | Impact | Mitigation |
|--------------|--------|------------|
| Malicious participant during DKG | Key compromise | Requires ≥t malicious parties |
| Network eavesdropping | Metadata leak | TLS/DTLS encryption |
| Commitment manipulation | Protocol failure | Cryptographic verification |
| Denial of service | DKG failure | Timeout and retry mechanisms |
| Key share theft | Partial compromise | Encrypted storage (AES-256-GCM + PBKDF2). No HSM integration — earlier drafts claimed HSM support; none exists. |
| Replay attacks | Double signing | FROST nonces are randomly generated per-signing; no separate nonce-tracking or explicit session-id validation layer is applied on top of the protocol. |

## Troubleshooting

### Common DKG Issues

#### "Timeout during Round 1"
```
┌─────────────────────────────────────────────────────┐
│ ⚠️  DKG Timeout Detected                            │
├─────────────────────────────────────────────────────┤
│ Issue: Round 1 timeout (300s exceeded)              │
│ Missing: charlie's commitments                      │
│                                                     │
│ Diagnostics:                                        │
│ • Network: ✅ Connected                             │
│ • Charlie status: 🔴 Disconnected (180s ago)      │
│ • Partial data: 2 of 3 commitments received       │
│                                                     │
│ Options:                                            │
│ 1. Wait for Charlie (extend timeout)               │
│ 2. Restart with available participants             │
│ 3. Switch to offline DKG                          │
│                                                     │
│ [Extend 5 min] [Restart] [Go Offline]             │
└─────────────────────────────────────────────────────┘
```

#### "Verification Failed"
```
┌─────────────────────────────────────────────────────┐
│ ❌ Share Verification Failed                        │
├─────────────────────────────────────────────────────┤
│ Error: Invalid share from participant 'bob'         │
│                                                     │
│ Details:                                            │
│ • Share doesn't match commitment                   │
│ • Polynomial evaluation incorrect                  │
│ • Possible corruption or attack                    │
│                                                     │
│ Automatic Actions Taken:                            │
│ ✅ Notified other participants                      │
│ ✅ Logged incident for audit                        │
│ ✅ Excluded bob from current round                  │
│                                                     │
│ Next Steps:                                         │
│ • Contact bob to verify software                  │
│ • Restart DKG without bob                         │
│ • Consider alternative participant                 │
│                                                     │
│ [View Technical Details] [Restart] [Abort]         │
└─────────────────────────────────────────────────────┘
```

### DKG Best Practices

1. **Pre-DKG Checklist**
   - Verify all participant identities
   - Test network connections
   - Synchronize clocks
   - Clear previous failed attempts

2. **During DKG**
   - Monitor progress actively
   - Keep stable network connection
   - Don't interrupt the process
   - Save all logs for audit

3. **Post-DKG**
   - Test with small transaction
   - Create immediate backup
   - Document participant info
   - Schedule regular health checks

4. **Security Hygiene**
   - Use dedicated devices for high-value wallets
   - Implement proper access controls
   - Regular security audits
   - Practice recovery procedures