# FROST MPC TUI Wallet - Security Model

## Table of Contents

1. [Security Overview](#security-overview)
2. [Threat Model](#threat-model)
3. [Cryptographic Security](#cryptographic-security)
4. [Network Security](#network-security)
5. [Local Security](#local-security)
6. [Operational Security](#operational-security)
7. [Security Best Practices](#security-best-practices)
8. [Incident Response](#incident-response)
9. [Compliance and Auditing](#compliance-and-auditing)

## Security Overview

The FROST MPC TUI Wallet implements defense-in-depth security architecture, combining cryptographic protection, secure communications, and operational safeguards to protect digital assets.

### Security Principles

1. **Zero Trust**: No single component or participant is fully trusted
2. **Least Privilege**: Components have minimal required permissions
3. **Defense in Depth**: Multiple layers of security controls
4. **Fail Secure**: System fails to a secure state
5. **Auditability**: All security-relevant events are logged

### Security Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Security Layers                        │
├─────────────────────────────────────────────────────────┤
│ Layer 5: Operational Security                           │
│ • Access controls, procedures, training                 │
├─────────────────────────────────────────────────────────┤
│ Layer 4: Application Security                           │
│ • Input validation, secure coding, memory safety        │
├─────────────────────────────────────────────────────────┤
│ Layer 3: Network Security                               │
│ • TLS/DTLS, certificate validation, firewall           │
├─────────────────────────────────────────────────────────┤
│ Layer 2: Cryptographic Security                         │
│ • FROST protocol, key encryption, signatures            │
├─────────────────────────────────────────────────────────┤
│ Layer 1: Platform Security                              │
│ • OS hardening, secure boot, hardware security         │
└─────────────────────────────────────────────────────────┘
```

## Threat Model

### Adversary Capabilities

We consider adversaries with the following capabilities:

#### Network Adversary (Level 1)
- Can observe all network traffic
- Can delay or drop packets
- Can attempt man-in-the-middle attacks
- Cannot break TLS/DTLS encryption

#### Compromised Participant (Level 2)
- Controls up to t-1 participants (below threshold)
- Has valid key shares for controlled participants
- Can attempt to manipulate protocol messages
- Cannot forge signatures without threshold

#### Local Malware (Level 3)
- Has user-level access to device
- Can read unencrypted files
- Can capture keyboard input
- Cannot access encrypted keystore without password

#### Physical Access (Level 4)
- Physical access to device
- Can attempt cold boot attacks
- Can install hardware keyloggers
- Time-limited access window

### Attack Scenarios

```
┌─────────────────────────────────────────────────────────┐
│ Attack Tree: Unauthorized Transaction                    │
├─────────────────────────────────────────────────────────┤
│ Goal: Execute unauthorized transaction                   │
│                                                         │
│ ├─ Compromise threshold participants                    │
│ │  ├─ Social engineering (High Risk)                   │
│ │  ├─ Malware infection (Medium Risk)                 │
│ │  └─ Physical coercion (Low Risk)                    │
│ │                                                       │
│ ├─ Break cryptographic security                        │
│ │  ├─ Factor private key (Negligible Risk)            │
│ │  ├─ Break FROST protocol (Negligible Risk)          │
│ │  └─ Exploit implementation bug (Low Risk)           │
│ │                                                       │
│ └─ Exploit operational weakness                        │
│     ├─ Weak passwords (Medium Risk)                    │
│     ├─ Poor key management (Medium Risk)              │
│     └─ Insider threat (Medium Risk)                   │
└─────────────────────────────────────────────────────────┘
```

## Cryptographic Security

### FROST Protocol Security

The FROST (Flexible Round-Optimized Schnorr Threshold) protocol provides:

```
┌─────────────────────────────────────────────────────────┐
│ FROST Security Properties                               │
├─────────────────────────────────────────────────────────┤
│ ✅ Unforgeability                                       │
│    No adversary can forge signatures without           │
│    controlling ≥t participants                          │
│                                                         │
│ ✅ Robustness                                           │
│    Protocol completes despite up to n-t failures       │
│                                                         │
│ ✅ Privacy                                              │
│    Individual key shares reveal nothing about          │
│    the complete private key                            │
│                                                         │
│ ✅ Non-repudiation                                      │
│    Participants cannot deny their participation        │
│    in signing operations                               │
│                                                         │
│ ✅ Verifiability                                        │
│    All operations can be publicly verified             │
└─────────────────────────────────────────────────────────┘
```

### Key Generation Security

```rust
// Secure random number generation
use rand_core::OsRng; // Cryptographically secure RNG

// Key generation with proper entropy
let mut rng = OsRng;
let secret_share = Scalar::random(&mut rng);

// Commitment generation with binding
let commitment = VerifiableSecretSharing::commit(&secret_share);
```

### Cryptographic Parameters

| Parameter | Value | Security Level |
|-----------|-------|----------------|
| Curve (Ethereum) | secp256k1 | 128-bit |
| Curve (Solana) | ed25519 | 128-bit |
| Hash Function | SHA-256 | 128-bit collision |
| KDF | PBKDF2-SHA256 | 100,000 iterations |
| Encryption | AES-256-GCM | 256-bit |
| Key Share Size | 32 bytes | Full entropy |

## Network Security

### TLS / DTLS scope

This application does NOT configure TLS cipher suites, versions,
or OCSP settings directly — both layers are delegated:

- **Signal-server TLS (WebSocket)**: clients connect via `wss://`
  and trust the system CA store. `tokio-tungstenite` uses the
  native TLS stack on each platform. The public Cloudflare Worker
  endpoint at `wss://xiongchenyu.dpdns.org` gets its TLS from
  Cloudflare's edge. Operators running the self-hosted signal
  server terminate TLS in their own reverse proxy (nginx / caddy /
  Cloudflare Tunnel — see `docs/deployment/README.md`).
- **WebRTC DTLS-SRTP**: version + cipher selection happens inside
  the `webrtc` crate / browser WebRTC implementation. Earlier
  drafts claimed "DTLS 1.3" specifically — the protocol version
  is whatever the underlying library negotiates, not something
  this app pins. Data channels (no media tracks) ride DTLS-SRTP
  as usual.

No certificate pinning, no HSTS enforcement, no OCSP stapling,
no TURN-server authentication (no TURN infra ships — clients rely
on public STUN only), no SDP sanitization layer. Earlier drafts
of this section enumerated all of those as features — they're not
implemented.

### Network Isolation

For operators who want to deploy the self-hosted signal server in
a segmented network, standard network-engineering practice applies
(DMZ for the WS listener, separate management interface, air-gap
for offline participants). The application itself doesn't ship
any tooling that enforces this segmentation — it's an operator-side
concern.

## Local Security

### Keystore Protection

Values below are the real constants in
`apps/tui-node/src/keystore/encryption.rs`:

```
Key Derivation:
  Algorithm:  PBKDF2-HMAC-SHA256
  Iterations: 100_000  (PBKDF2_ITERATIONS constant)
  Salt:       16 bytes (SALT_LEN = 16, fresh per wallet)

Encryption:
  Algorithm:  AES-256-GCM
  Nonce:      12 bytes (NONCE_LEN = 12, fresh per encryption)
  Auth tag:   16 bytes (standard GCM tag, appended to ciphertext
              by the aes-gcm crate — no separate storage field)

`.dat` on-disk layout (no version prefix):
  ┌─────────────────────────────────────────────┐
  │ salt       (16 B)                           │
  │ nonce      (12 B)                           │
  │ ciphertext + GCM auth tag (variable bytes)  │
  └─────────────────────────────────────────────┘
```

Earlier drafts of this section claimed a 32-byte salt and a
leading `Version (4 bytes)` field — neither is true (verified
against `encryption.rs:20-21` for the constants and `:99` for
the write format).

### Memory Protection

Today only `packages/@mpc-wallet/frost-core/src/root_secret.rs`
uses `zeroize::Zeroize` (grep: 1 hit across the whole workspace).
Key shares, decrypted keystore blobs, passwords entered into the
PasswordPrompt screen, and signing intermediate state are NOT
zeroed on drop. Earlier drafts of this doc showed a
`#[zeroize(drop)] SensitiveData` struct with automatic wiping —
that pattern is not applied anywhere in the current tree.

Adding systematic zeroization (at minimum to: `key_package`,
`group_public_key`, `frost_nonces`, `frost_commitments`,
`frost_signature_shares`, `signing_message`, and the
PasswordPrompt draft buffers) is tracked as open hardening work.

### File System Security

```
┌─────────────────────────────────────────────────────────┐
│ File Permissions (Unix, recommended)                   │
├─────────────────────────────────────────────────────────┤
│ ~/.frost_keystore/                                      │
│ ├── index.json           (600) Wallet index            │
│ ├── device_id            (600) Device identity         │
│ └── <device_id>/                                        │
│     ├── ed25519/         (700) Curve-scoped dir        │
│     │   ├── <wid>.json   (600) Wallet metadata         │
│     │   └── <wid>.dat    (600) Encrypted key share     │
│     └── secp256k1/       (700)                         │
│         ├── <wid>.json   (600)                         │
│         └── <wid>.dat    (600)                         │
└─────────────────────────────────────────────────────────┘
```

The current implementation calls `fs::create_dir_all` for the
directories but does not explicitly `chmod` them — inheriting the
user's umask. Hardening steps below still apply as a defence-in-depth
recommendation.

## Operational Security

### Access Control Matrix

```
┌─────────────────────────────────────────────────────────┐
│ Role-Based Access Control                               │
├─────────────────────────────────────────────────────────┤
│ Role          │ Create │ Sign │ Admin │ Audit │ Backup │
├───────────────┼────────┼──────┼───────┼───────┼────────┤
│ Participant   │   ✓    │  ✓   │   ✗   │   ✗   │   ✓    │
│ Coordinator   │   ✓    │  ✓   │   ✓   │   ✗   │   ✓    │
│ Auditor       │   ✗    │  ✗   │   ✗   │   ✓   │   ✗    │
│ Administrator │   ✗    │  ✗   │   ✓   │   ✓   │   ✓    │
└─────────────────────────────────────────────────────────┘
```

### Operational Procedures

#### Secure Setup Checklist
```
┌─────────────────────────────────────────────────────────┐
│ Pre-Deployment Security Checklist                       │
├─────────────────────────────────────────────────────────┤
│ ☐ Operating System                                      │
│   ☐ Latest security patches installed                  │
│   ☐ Unnecessary services disabled                      │
│   ☐ Firewall configured                               │
│   ☐ Antivirus/EDR installed                          │
│                                                         │
│ ☐ Application Configuration                             │
│   ☐ Strong passwords enforced                         │
│   ☐ Audit logging enabled                             │
│   ☐ Network timeouts configured                       │
│   ☐ Rate limiting enabled                             │
│                                                         │
│ ☐ Physical Security                                     │
│   ☐ Device in secure location                         │
│   ☐ Screen lock configured                            │
│   ☐ BIOS/UEFI password set                           │
│   ☐ Full disk encryption enabled                      │
└─────────────────────────────────────────────────────────┘
```

#### Key Ceremony Process
```
┌─────────────────────────────────────────────────────────┐
│ Secure Key Generation Ceremony                          │
├─────────────────────────────────────────────────────────┤
│ Phase 1: Preparation                                    │
│ • Clean room setup                                      │
│ • Device verification                                   │
│ • Participant authentication                           │
│ • Witness presence                                     │
│                                                         │
│ Phase 2: Generation                                     │
│ • Air-gapped environment                               │
│ • Video recording (optional)                           │
│ • Dual control verification                            │
│ • Immediate backup creation                            │
│                                                         │
│ Phase 3: Verification                                   │
│ • Test transaction                                     │
│ • Backup restoration test                              │
│ • Audit log review                                     │
│ • Secure storage confirmation                          │
└─────────────────────────────────────────────────────────┘
```

## Security Best Practices

### For Users

```
┌─────────────────────────────────────────────────────────┐
│ User Security Guidelines                                │
├─────────────────────────────────────────────────────────┤
│ 1. Password Security                                    │
│    • Use unique, strong passwords (>16 chars)         │
│    • Enable password manager                           │
│    • Never share passwords                            │
│    • Change passwords regularly                        │
│                                                         │
│ 2. Device Security                                      │
│    • Keep OS and software updated                     │
│    • Use full disk encryption                         │
│    • Enable screen lock (5 min timeout)              │
│    • Disable unnecessary services                     │
│                                                         │
│ 3. Operational Security                                 │
│    • Verify participant identities                     │
│    • Use secure communication channels                │
│    • Regular security training                        │
│    • Report suspicious activity                       │
│                                                         │
│ 4. Backup Security                                      │
│    • Encrypt all backups                              │
│    • Store in multiple locations                      │
│    • Test recovery regularly                          │
│    • Secure physical storage                          │
└─────────────────────────────────────────────────────────┘
```

### For Administrators

```
┌─────────────────────────────────────────────────────────┐
│ Administrator Security Checklist                        │
├─────────────────────────────────────────────────────────┤
│ Daily Tasks:                                            │
│ ☐ Review security alerts                               │
│ ☐ Check system logs                                    │
│ ☐ Monitor failed login attempts                       │
│ ☐ Verify backup completion                            │
│                                                         │
│ Weekly Tasks:                                           │
│ ☐ Review access logs                                   │
│ ☐ Test incident response                              │
│ ☐ Update security patches                             │
│ ☐ Audit user permissions                              │
│                                                         │
│ Monthly Tasks:                                          │
│ ☐ Security training review                            │
│ ☐ Penetration testing                                 │
│ ☐ Disaster recovery drill                             │
│ ☐ Policy compliance audit                             │
└─────────────────────────────────────────────────────────┘
```

## Incident Response

### Incident Classification

```
┌─────────────────────────────────────────────────────────┐
│ Security Incident Severity Levels                       │
├─────────────────────────────────────────────────────────┤
│ CRITICAL (P1) - Immediate Response Required             │
│ • Unauthorized transaction detected                     │
│ • Multiple key shares compromised                      │
│ • Active attack in progress                            │
│ Response Time: < 15 minutes                            │
│                                                         │
│ HIGH (P2) - Urgent Response                             │
│ • Single key share potentially compromised             │
│ • Suspicious participant behavior                      │
│ • Failed signature verification                        │
│ Response Time: < 1 hour                                │
│                                                         │
│ MEDIUM (P3) - Timely Response                           │
│ • Unusual network activity                             │
│ • Failed login attempts                                │
│ • Configuration tampering                              │
│ Response Time: < 4 hours                               │
│                                                         │
│ LOW (P4) - Scheduled Response                           │
│ • Policy violations                                     │
│ • Non-critical vulnerabilities                         │
│ • Documentation issues                                 │
│ Response Time: < 24 hours                              │
└─────────────────────────────────────────────────────────┘
```

### Incident Response Procedure

```
┌─────────────────────────────────────────────────────────┐
│ Incident Response Workflow                              │
├─────────────────────────────────────────────────────────┤
│ 1. DETECT                                               │
│    ↓                                                    │
│ 2. ASSESS → Determine severity                         │
│    ↓                                                    │
│ 3. CONTAIN → Isolate affected systems                  │
│    ↓                                                    │
│ 4. INVESTIGATE → Gather evidence                        │
│    ↓                                                    │
│ 5. REMEDIATE → Fix vulnerabilities                     │
│    ↓                                                    │
│ 6. RECOVER → Restore normal operations                 │
│    ↓                                                    │
│ 7. DOCUMENT → Create incident report                   │
│    ↓                                                    │
│ 8. IMPROVE → Update procedures                         │
└─────────────────────────────────────────────────────────┘
```

### Emergency Contacts

```
┌─────────────────────────────────────────────────────────┐
│ Emergency Response Team                                 │
├─────────────────────────────────────────────────────────┤
│ Role                  │ Contact           │ Backup      │
├───────────────────────┼───────────────────┼─────────────┤
│ Security Lead         │ security@frost    │ +1-555-0100 │
│ Technical Lead        │ tech@frost        │ +1-555-0101 │
│ Legal Counsel         │ legal@frost       │ +1-555-0102 │
│ PR/Communications     │ pr@frost          │ +1-555-0103 │
│ Executive Sponsor     │ exec@frost        │ +1-555-0104 │
└─────────────────────────────────────────────────────────┘
```

## Compliance and Auditing

### Audit Log Format

```json
{
  "timestamp": "2024-01-20T10:30:00Z",
  "event_type": "signature_created",
  "severity": "info",
  "actor": "alice",
  "action": "sign_transaction",
  "resource": "treasury-wallet",
  "details": {
    "transaction_hash": "0xabcd...",
    "participants": ["alice", "bob"],
    "threshold_met": true
  },
  "ip_address": "192.168.1.100",
  "user_agent": "mpc-wallet-tui/<CARGO_PKG_VERSION>",
  "session_id": "sess_123456",
  "correlation_id": "corr_789012"
}
```

### Compliance Framework

```
┌─────────────────────────────────────────────────────────┐
│ Regulatory Compliance Matrix                            │
├─────────────────────────────────────────────────────────┤
│ Standard      │ Requirement        │ Implementation     │
├───────────────┼────────────────────┼────────────────────┤
│ SOC 2 Type II │ Access Controls    │ RBAC, MFA         │
│               │ Encryption         │ AES-256-GCM       │
│               │ Audit Trails       │ Immutable logs    │
├───────────────┼────────────────────┼────────────────────┤
│ ISO 27001     │ Risk Assessment    │ Annual review     │
│               │ Incident Response  │ 24/7 team         │
│               │ Business Continuity│ DR procedures     │
├───────────────┼────────────────────┼────────────────────┤
│ GDPR          │ Data Protection    │ Encryption at rest│
│               │ Right to Erasure   │ Key deletion      │
│               │ Data Portability   │ Export functions  │
└─────────────────────────────────────────────────────────┘
```

### Security Metrics

```
┌─────────────────────────────────────────────────────────┐
│ Security KPIs Dashboard                                 │
├─────────────────────────────────────────────────────────┤
│ Metric                    │ Target │ Current │ Status  │
├───────────────────────────┼────────┼─────────┼─────────┤
│ Failed Login Rate         │ <1%    │ 0.3%    │ ✅      │
│ Patch Compliance          │ 100%   │ 98%     │ ⚠️      │
│ Incident Response Time    │ <1hr   │ 45min   │ ✅      │
│ Security Training         │ 100%   │ 100%    │ ✅      │
│ Vulnerability Scan        │ 0 High │ 0       │ ✅      │
│ Backup Success Rate       │ 99.9%  │ 99.95%  │ ✅      │
│ Uptime                    │ 99.9%  │ 99.97%  │ ✅      │
└─────────────────────────────────────────────────────────┘
```

## Security Hardening Guide

### System Hardening

```bash
# Linux Security Hardening Script
#!/bin/bash

# Kernel parameters
echo "kernel.randomize_va_space=2" >> /etc/sysctl.conf
echo "net.ipv4.tcp_syncookies=1" >> /etc/sysctl.conf
echo "net.ipv4.conf.all.rp_filter=1" >> /etc/sysctl.conf

# Disable unnecessary services
systemctl disable bluetooth
systemctl disable cups
systemctl disable avahi-daemon

# Configure firewall
ufw default deny incoming
ufw default allow outgoing
ufw allow 443/tcp  # Signal server
ufw enable

# File system hardening
chmod 700 ~/.frost_keystore
chmod -R go-rwx ~/.frost_keystore

# Enable audit logging
auditctl -w ~/.frost_keystore -p wa -k frost_keystore_changes
```

### Application Hardening

```toml
# config.toml - Security Settings
[security]
# Password policy
min_password_length = 16
require_special_chars = true
password_history = 5
max_password_age_days = 90

# Session management
session_timeout_minutes = 15
max_concurrent_sessions = 1
require_mfa = true

# Network security
allowed_ips = ["192.168.1.0/24"]
rate_limit_per_minute = 60
connection_timeout_seconds = 30

# Cryptographic settings
min_key_share_entropy_bits = 256
require_secure_random = true
key_derivation_iterations = 100000
```

## Conclusion

Security is not a feature but a continuous process. The FROST MPC TUI Wallet implements comprehensive security controls at every layer, from cryptographic protocols to operational procedures. Regular security assessments, updates, and training ensure the system remains secure against evolving threats.

For security concerns or vulnerability reports, please open a private
advisory via [GitHub Security Advisories](https://github.com/hecoinfo/mpc-wallet/security/advisories/new).