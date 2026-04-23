# Signal and WebRTC Message Types

This document defines the JSON message types and protocol flow for negotiating and creating an MPC wallet using a signaling server. Nodes communicate via a CLI application that supports both Ed25519 (Solana) and Secp256k1 (Ethereum) cryptographic curves. The signaling server coordinates device discovery and WebRTC connection setup; all MPC protocol messages are exchanged over WebRTC.

---

## Protocol Overview

1. **Node Registration:**  
   Each node connects to the signaling server via WebSocket and registers with a unique `device_id`.

2. **Discovery:**  
   Nodes query the signaling server for available devices.

3. **Session Negotiation & Mesh Formation:**
   Nodes coordinate session parameters (e.g., total participants,
   threshold, session ID) and build the peer mesh themselves. The
   signaling server tracks registered `device_id`s and session
   announcements in memory (standalone server) or Durable Object
   storage (Cloudflare Worker variant), and relays opaque peer-to-peer
   envelopes — but it does not inspect or intermediate DKG/signing
   protocol messages.

4. **Signaling Exchange:**  
   Nodes exchange WebRTC signaling data (SDP offers/answers, ICE candidates) via the signaling server to establish direct device-to-device connections.

5. **MPC Wallet Creation:**  
   Once WebRTC connections are established, nodes exchange MPC protocol messages (commitments, shares, etc.) directly.

6. **Threshold Signing:**  
   After wallet creation, nodes can participate in threshold signing processes using the FROST protocol.

---

## WebSocket (Signaling Server) Message Types

### 1. Registration

**Client → Server**
```json
{ "type": "register", "device_id": "<device_id>" }
```
Registers the client with the signaling server using a unique `device_id`.

---

### 2. Device Discovery

**Client → Server**
```json
{ "type": "list_devices" }
```
Requests a list of currently registered devices.

**Server → Client**
```json
{ "type": "devices", "devices": ["device1", "device2", ...] }
```
Returns the list of available devices.

---

### 3. Signaling Relay

**Client → Server**
```json
{ "type": "relay", "to": "<device_id>", "data": { ... } }
```
Sends signaling data (SDP offer/answer, ICE candidate, etc.) to another device via the server.

**Server → Client**
```json
{ "type": "relay", "from": "<device_id>", "data": { ... } }
```
Relays signaling data from another device.

---

### 4. Error

**Server → Client**
```json
{ "type": "error", "error": "<description>" }
```
Sent if an error occurs (e.g., unknown device).

### 5. Session discovery

The signal server also knows about session announcements — this
enables cold-start rejoin + peer discovery without needing every
participant to be online simultaneously at announce time.
Authoritative enum: `ClientMsg` / `ServerMsg` in
`apps/signal-server/server/src/lib.rs`.

**Client → Server**
```json
{ "type": "announce_session", "session_info": { ... } }
{ "type": "request_active_sessions" }
{ "type": "session_status_update", "session_info": { ... } }
{ "type": "query_my_active_sessions" }
```

**Server → Clients**
```json
{ "type": "session_available", "session_info": { ... } }
{ "type": "sessions_for_device", "sessions": [ ... ] }
{ "type": "session_list_request", "from": "<device_id>" }
{ "type": "session_removed", "session_id": "<id>", "reason": "<text>" }
```

The `session_info` payload is a JSON blob whose shape is defined
by the sender (browser extension, TUI, native-node all share it
via `packages/@mpc-wallet/types/src/session.ts` on the TS side
and ad-hoc on the Rust side). The signal server treats the inner
payload as opaque.

---

## WebRTC (Device-to-Device) Message Types

Once a direct WebRTC connection is established, nodes exchange application-level messages for the MPC protocol.

### 1. Session Management

```json
{
  "type": "session_proposal",
  "payload": {
    "session_id": "<id>",
    "total": 3,
    "threshold": 2,
    "participants": ["device1", "device2", "device3"]
  }
}
```
Proposes a new MPC session with specified parameters.

```json
{
  "type": "session_response",
  "payload": {
    "session_id": "<id>",
    "accepted": true
  }
}
```
Responds to a session proposal.

---

### 2. Mesh Formation

```json
{
  "type": "channel_open",
  "payload": {
    "device_id": "<device_id>"
  }
}
```
Notifies other devices when a data channel is opened.

```json
{
  "type": "mesh_ready",
  "payload": {
    "session_id": "<id>",
    "device_id": "<device_id>"
  }
}
```
Indicates a device has established connections to all other participants.

---

### 3. Distributed Key Generation (DKG)

Authoritative enum: `WebRTCMessage<C>` in
`apps/tui-node/src/protocal/signal.rs:204`. Serde tags use
`snake_case` on the enum variant name — note the `_package` suffix.

```json
{
  "type": "dkg_round1_package",
  "payload": {
    "package": "<frost-core::keys::dkg::round1::Package>"
  }
}
```
Sends DKG round 1 package (commitments) to other devices.

```json
{
  "type": "dkg_round2_package",
  "payload": {
    "package": "<frost-core::keys::dkg::round2::Package>"
  }
}
```
Sends DKG round 2 package (encrypted shares) to other devices.

DKG finalization is entirely local — there is no `dkg_complete`
wire message. Each participant runs `dkg::part3` on its received
packages to produce its own `KeyPackage` + the shared
`VerifyingKey` (group public key). Participants can cross-check
the resulting group key out-of-band if needed. Earlier drafts of
this doc listed a `dkg_complete` message broadcasting the
group_pubkey — no such type exists in `WebRTCMessage`.

---

### 4. Threshold Signing

```json
{
  "type": "signing_request",
  "payload": {
    "signing_id": "<id>",
    "transaction_data": "<hex-data>",
    "required_signers": 2
  }
}
```
Initiates a signing request for specified transaction data.

```json
{
  "type": "signing_acceptance",
  "payload": {
    "signing_id": "<id>",
    "accepted": true
  }
}
```
Responds to a signing request.

```json
{
  "type": "signer_selection",
  "payload": {
    "signing_id": "<id>",
    "selected_signers": ["<frost-identifier-1>", "<frost-identifier-2>"]
  }
}
```
Announces which participants will be involved in signing.

```json
{
  "type": "signing_commitment",
  "payload": {
    "signing_id": "<id>",
    "sender_identifier": "<frost-identifier>",
    "commitment": "<serialized-commitment>"
  }
}
```
Sends a FROST round 1 commitment for signing.

```json
{
  "type": "signature_share",
  "payload": {
    "signing_id": "<id>",
    "sender_identifier": "<frost-identifier>",
    "share": "<serialized-share>"
  }
}
```
Sends a FROST round 2 signature share.

```json
{
  "type": "aggregated_signature",
  "payload": {
    "signing_id": "<id>",
    "signature": "<hex-encoded-signature>"
  }
}
```
Broadcasts the final aggregated signature.

---

## Protocol Flow

### 1. Registration & Discovery

- Each node connects to the signaling server and registers with a unique `device_id`.
- Nodes may request a list of available devices.

### 2. Session Negotiation & Mesh Formation

- One node (initiator) proposes a session with specific parameters (session ID, total participants, threshold).
- Other nodes accept the session proposal.
- All nodes establish WebRTC connections with each other through signaling exchange.
- Each node tracks the status of its connections and reports readiness when all connections are established.

### 3. Distributed Key Generation (DKG)

- When all nodes report mesh readiness, the DKG process begins automatically:
  - **Round 1:** Each node sends commitments to all other nodes.
  - **Round 2:** Each node sends encrypted shares to all other nodes.
  - **Finalization:** Nodes verify shares and compute their final key shares.
- After successful DKG, each node has:
  - A key package with its signing share
  - The group public key for the distributed wallet

### 4. Threshold Signing

- Any node can initiate a signing request by specifying transaction data.
- Other nodes can accept the request.
- Once enough nodes accept (meeting or exceeding the threshold):
  - The initiator selects which accepted nodes will participate (exactly threshold number).
  - Selected signers exchange FROST round 1 commitments.
  - Selected signers exchange FROST round 2 signature shares.
  - One node aggregates the shares into a final signature and broadcasts it to all participants.

### 5. Completion

- The final signature can be used as appropriate for the blockchain (Ethereum or Solana).
- The session and WebRTC connections remain active for future signing operations.

---

## Message Flow Examples

### Session Creation

1. Device `mpc-1` sends proposal to `mpc-2` and `mpc-3`:
   ```json
   {
     "type": "session_proposal",
     "payload": {
       "session_id": "wallet_2of3",
       "total": 3,
       "threshold": 2,
       "participants": ["mpc-1", "mpc-2", "mpc-3"]
     }
   }
   ```

2. Devices `mpc-2` and `mpc-3` send acceptance:
   ```json
   {
     "type": "session_response",
     "payload": {
       "session_id": "wallet_2of3",
       "accepted": true
     }
   }
   ```

### Mesh Formation

1. Data channel opened between `mpc-1` and `mpc-2`:
   ```json
   {
     "type": "channel_open",
     "payload": {
       "device_id": "mpc-2"
     }
   }
   ```

2. All connections established for `mpc-1`:
   ```json
   {
     "type": "mesh_ready",
     "payload": {
       "session_id": "wallet_2of3",
       "device_id": "mpc-1"
     }
   }
   ```

### DKG Process

1. `mpc-1` sends Round 1 package to all devices:
   ```json
   {
     "type": "dkg_round1",
     "payload": {
       "package": "<serialized-package>"
     }
   }
   ```

2. After all Round 1 packages are received, `mpc-1` sends Round 2 package:
   ```json
   {
     "type": "dkg_round2",
     "payload": {
       "package": "<serialized-package>"
     }
   }
   ```

### Signing Process

1. `mpc-1` sends signing request:
   ```json
   {
     "type": "signing_request",
     "payload": {
       "signing_id": "sign_mpc-1_1624511234",
       "transaction_data": "0x123456789abcdef",
       "required_signers": 2
     }
   }
   ```

2. `mpc-2` accepts the request:
   ```json
   {
     "type": "signing_acceptance",
     "payload": {
       "signing_id": "sign_mpc-1_1624511234",
       "accepted": true
     }
   }
   ```

3. `mpc-1` selects signers (itself and `mpc-2`):
   ```json
   {
     "type": "signer_selection",
     "payload": {
       "signing_id": "sign_mpc-1_1624511234",
       "selected_signers": ["<mpc-1-identifier>", "<mpc-2-identifier>"]
     }
   }
   ```

4. Selected signers exchange commitments and shares, then aggregate the final signature.

---

## Mesh Formation Details

Establishing a complete WebRTC mesh involves:

1. **Connection Establishment:**
   - Each node attempts to establish WebRTC connections to all other participants
   - Connections use SDP offers/answers and ICE candidates exchanged via the signaling server

2. **Data Channel Tracking:**
   - Nodes track when a data channel is successfully opened with each device
   - A `channel_open` message is sent when each data channel opens

3. **Readiness Notification:**
   - When a node has established data channels to all other participants, it broadcasts a `mesh_ready` message
   - Each node tracks which devices have reported mesh readiness
   - The mesh is considered fully ready when all nodes have reported readiness

4. **Automatic Recovery:**
   - If connections fail, nodes automatically attempt reconnection with backoff
   - The `/mesh_ready` command can be used manually if automatic detection fails

---

## Troubleshooting

- **Device Discovery Issues:** Use the `/list` command to refresh the device list from the signaling server.
- **WebRTC Connection Failures:**
  - Ensure you're not behind a restrictive firewall blocking WebRTC.
  - Check the logs for ICE connectivity errors.
  - Try restarting the affected nodes.
- **DKG Failures:**
  - Verify that all nodes are using the same cryptographic curve.
  - Check that the mesh is fully ready before DKG begins.
  - Examine the logs for cryptographic errors in package processing.
- **Signing Issues:**
  - Ensure DKG completed successfully for all nodes.
  - Verify that enough participants have accepted the signing request.
  - Check that selected signers can communicate with each other.

---

## Implementation Notes

- The protocol uses FROST (Flexible Round-Optimized Schnorr Threshold signatures) for threshold signing.
- The implementation supports both Ed25519 (for Solana) and Secp256k1 (for Ethereum) curves.
- WebRTC is used for secure device-to-device communication without requiring a central server after initial connection setup.
- The signaling server is stateless and only facilitates connection establishment, not the cryptographic protocol.