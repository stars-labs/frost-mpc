# Phase C — FROST Threshold Signing on an Existing Wallet

**Purpose**: After Phase A, every participant has a persisted key share on disk. Phase C lets the user actually *use* that share: pick a wallet, enter the password, submit a message/transaction, coordinate FROST sign rounds across threshold-many participants, and see the resulting signature.

**Non-goals this phase**:
- Transaction construction for specific chains (ETH RLP, Solana msg-v0, BTC PSBT). Phase C signs a raw byte slice and returns a signature — chain-specific wrapping is a Phase D concern.
- Fee estimation, nonce management, gas, broadcast. This phase **does not touch the network** past the MPC mesh.
- Running ed25519 + secp256k1 concurrently (deferred follow-up; this phase uses whichever curve the wallet was generated with).
- Signing-request approval queue UX (the plan's `pending_signing_requests` list). We do synchronous "sign now" in one ceremony; approval-queue is Phase E.

**Architecture reminder**: the TUI binary is generic over `C: Ciphersuite` at type level, resolved to `Secp256K1Sha256` at `mpc-wallet-tui.rs`. Multi-curve would require two binary instances or a runtime dispatch — out of scope.

**Remove this file** once all 5 stages are `Complete`.

---

## Stage 1: Keystore hydration — load `KeyPackage<C>` back from disk

**Goal**: A new `Command::UnlockWallet { wallet_id, password }` reads the encrypted wallet file, decrypts it with the password, deserializes into a `KeyPackage<C>` + `PublicKeyPackage<C>`, and writes them onto `AppState` so the signing layer has everything FROST needs. Emits `Message::WalletUnlocked { wallet_id }` on success, `Message::WalletUnlockFailed { error }` on decrypt/deserialize failure.

**Rationale**: the DKG flow populates AppState from live FROST output. A signing ceremony that starts from a cold wallet needs the same state repopulated from disk — otherwise every signing Command would have to re-implement the keystore load, and the cleartext `KeyPackage` would leak through too many call sites.

**Success Criteria**:
- Command takes `&mut Keystore` path (fresh construction, same pattern as Stage A.2 for write)
- `Keystore::load_wallet_file(wallet_id, password)` returns `Vec<u8>` (exists at `keystore/storage.rs:242`). That byte stream is the FROST `KeyPackage<C>` serialization produced by Stage A.2's `.serialize()`. Use `KeyPackage::<C>::deserialize(&bytes)` to round-trip.
- `PublicKeyPackage<C>` is NOT in the decrypted bytes — only the private share is encrypted per-device. But we need it for signing. **Decision needed**: either (a) serialize both into the key share blob when we write, (b) re-derive the pubkey package from `KeyPackage.verifying_shares()` + `KeyPackage.verifying_key()`, or (c) store it cleartext alongside the wallet file. `(b)` is the clean answer — FROST exposes `PublicKeyPackage::new(verifying_shares, verifying_key)` and we already have both on `KeyPackage`.

  *Update during implementation if (b) turns out not to work — both `(a)` and `(c)` are reasonable fallbacks.*
- AppState fields populated: `key_package`, `public_key_package`, `current_wallet_id` (same three Stage A.2 used on the write side)
- Wrong password → `Message::WalletUnlockFailed { error: "wrong password" }`. No panic.

**Files touched**:
- `apps/tui-node/src/elm/message.rs` — `WalletUnlocked { wallet_id }`, `WalletUnlockFailed { error }`
- `apps/tui-node/src/elm/command.rs` — `UnlockWallet { wallet_id, password, keystore_path }` (same shape as `FinalizeWalletFromDkg`)
- `apps/tui-node/src/elm/update.rs` — handlers for both Messages (stash wallet_id, push notification)
- Possibly `apps/tui-node/src/keystore/storage.rs` — thin helper `load_and_deserialize<C>(id, password) -> Result<(KeyPackage<C>, PublicKeyPackage<C>)>` to keep the deserialize plumbing in one place

**Tests**:
- Write a wallet via existing keystore API, read it back via the new Command, assert the deserialized `KeyPackage.identifier()` matches what was written
- Wrong password → `WalletUnlockFailed` (no panic)
- Unknown wallet_id → `WalletUnlockFailed` with "not found"

**Status**: Not Started

---

## Stage 2: Protocol layer — `protocal/signing.rs`

**Goal**: Mirror the structure of `protocal/dkg.rs` for signing. Three entry points:
- `handle_start_signing<C>(state, message: Vec<u8>)` — the coordinator-side ceremony kickoff; runs Round 1 locally, broadcasts commitments
- `process_signing_round1<C>(state, from, commitment_bytes)` — accumulate peer commitments, trigger Round 2 when threshold reached
- `process_signing_round2<C>(state, from, share_bytes)` — accumulate signature shares, run aggregate once threshold reached, emit final signature

Based on the working reference at `packages/@mpc-wallet/frost-core/examples/unified_dkg.rs:125–227`.

**Success Criteria**:
- `frost_core::round1::commit(key_package.signing_share(), rng)` produces `(SigningNonces, SigningCommitments)` — stash nonces on AppState, broadcast commitments
- Round 1 complete when `state.frost_commitments.len() >= threshold` (note: **threshold**, not total — signing only requires threshold-many participants)
- `SigningPackage::new(commitments, &message)` + `round2::sign(signing_package, nonces, key_package)` produces `SignatureShare`
- Round 2 complete when `state.frost_signature_shares.len() >= threshold`
- `aggregate(signing_package, &shares, pubkey_package)` returns the final `Signature<C>`. Serialize via `.serialize()` → hex for the UI.
- All error surfaces transition `signing_state = SigningState::Failed(reason)` rather than panic (same defensive pattern we used across the DKG layer)
- Uses the same `canonical_identifier::<C>` helper from `dkg.rs` so identifiers are consistent across DKG and signing ceremonies

**Files touched**:
- `apps/tui-node/src/protocal/signing.rs` (NEW, ~500 lines following the `dkg.rs` template)
- `apps/tui-node/src/protocal/mod.rs` — `pub mod signing;`

**Tests**:
- **Integration**: an all-in-one test that (a) does a 3-of-3 DKG in-memory, (b) feeds the resulting KeyPackages into `handle_start_signing` + the two `process_*` entry points across 3 simulated peers, (c) asserts the aggregated signature verifies under the group public key. This is a big test but worth it — if it passes we've exercised the whole protocol without needing the UI layer.
- Unit: `handle_start_signing` with missing `key_package` → `signing_state = Failed("no key_package")`

**Status**: Not Started

---

## Stage 3: SignTransaction screen — pick message, unlock, kick off

**Goal**: The user navigates `MainMenu → Manage Wallets → select wallet → Sign`, enters their password (if the wallet isn't already unlocked this session), enters the message bytes (hex or ASCII toggle), and submits. Two new screens + reuse of `PasswordPrompt`.

**Success Criteria**:
- `Screen::WalletDetail` gains a "Sign Transaction" action button (the screen already exists, just needs the new row)
- `Screen::SignTransaction { wallet_id }` renders:
  - Wallet header (id + short group pubkey)
  - Input field for the message to sign (single-line for now; multi-line comes with Phase D)
  - Hex/ASCII toggle so users can paste either
  - "Sign" button at the bottom
- Enter on "Sign" → push `Screen::PasswordPrompt` (reuse Stage A.1's screen) → on submit, route to `Message::InitiateSigning { message_bytes, wallet_id, password }`
- The InitiateSigning handler:
  1. Dispatches `Command::UnlockWallet { wallet_id, password, keystore_path }` (Stage C.1)
  2. Once `WalletUnlocked` fires, dispatches `Command::StartSigning { message_bytes }`
- `Command::StartSigning` → announces the signing session on the wire (like `Command::StartDKG` does for DKG) AND kicks off `handle_start_signing`

**Files touched**:
- `apps/tui-node/src/elm/components/sign_transaction.rs` (NEW, ~250 lines, pattern off `create_wallet.rs`)
- `apps/tui-node/src/elm/components/mod.rs` — register + export
- `apps/tui-node/src/elm/components/wallet_detail.rs` — add the "Sign" action row
- `apps/tui-node/src/elm/app.rs` — mount branch, render routing, keyboard arm for the new screen
- `apps/tui-node/src/elm/update.rs` — handlers for `InitiateSigning`, `WalletUnlocked`
- `apps/tui-node/src/elm/model.rs` — `Model.signing_request: Option<SigningRequestDraft>` carrying `{ wallet_id, message_bytes, input_mode: Hex|Ascii }` while the user fills it in

**Tests**:
- Transition: `InitiateSigning` with a mock Model state produces the expected Command sequence (UnlockWallet → StartSigning)
- Render: the component shows the wallet id, message input field, hex/ASCII toggle, and Sign button
- Input validation: empty message → inline error, hex-mode with non-hex chars → inline error

**Status**: Not Started

---

## Stage 4: Joiner side — accept signing session, run rounds

**Goal**: Joiners see an incoming signing session in the JoinSession → Signing tab, accept it, enter their password, and run the signing ceremony. The component already exists (`join_session.rs`); we need to wire the signing-tab path end-to-end.

**Success Criteria**:
- Creator's `Command::StartSigning` broadcasts an `AnnounceSession { session_type: "signing", ... }` over the signal server — existing code in `Command::StartDKG` at `command.rs:375` is the template
- Signal server / primary reader routes `SessionType::Signing` announcements into `Message::SessionDiscovered { session }` (may already work; verify)
- On `JoinSession`'s Signing tab, pressing Enter on a row routes to `PasswordPrompt` → on submit, dispatches `Command::UnlockWallet` followed by `Command::JoinSigning { session_id, message_bytes }`
- `Command::JoinSigning` mirrors `Command::JoinDKG`: record the session on AppState, ensure data-channel mesh is up, then call `handle_start_signing` (same entry point as the creator — the function is symmetric in FROST signing, unlike DKG)
- SigningProgress screen shows round status + mesh status (reuse `DKGProgressComponent`'s participant-list rendering if practical)

**Files touched**:
- `apps/tui-node/src/elm/command.rs` — new `Command::StartSigning`, `Command::JoinSigning` executors
- `apps/tui-node/src/elm/update.rs` — joiner-side `InitiateSigning` via the JoinSession tab
- `apps/tui-node/src/elm/components/signing_progress.rs` (NEW, maybe just shells DKGProgress rendering — TBD)
- `apps/tui-node/src/elm/app.rs` — mount branch for `Screen::SigningProgress`
- `apps/tui-node/src/protocal/signal.rs` — verify SessionType::Signing parse path (may just work)

**Tests**:
- Integration: 3-node smoke where mpc-1 creates+signs, mpc-2 & mpc-3 join. Assert all 3 logs contain "Signature aggregated" and the signature bytes match across nodes.
- Unit: `Command::JoinSigning` with missing `active_session` → `SigningFailed`

**Status**: Not Started

---

## Stage 5: End-to-end wiring + SignatureComplete screen

**Goal**: Once `handle_start_signing`'s `aggregate` step produces a signature, the whole flow needs to terminate on a visible SignatureComplete screen, LoadWallets refresh, and a success notification.

**Success Criteria**:
- `Message::SigningComplete { request_id, signature: Vec<u8> }` handler:
  1. Clears `signing_state` back to `Idle`
  2. Clears `pending_password` (if UnlockWallet used it)
  3. Stashes a `CompletedSignatureInfo { wallet_id, message_bytes, signature_hex, verified: bool }` on Model — parallel to `CompletedWalletInfo` from Phase A
  4. Navigates `push_screen(Screen::SignatureComplete { signature })`
  5. Pushes a success notification
- New `SignatureCompleteComponent` renders:
  - Wallet id, group pubkey (short)
  - Message (hex or ASCII depending on mode)
  - Signature hex (copyable — full hex block)
  - "Verified: yes/no" (run `verifying_key.verify(&message, &signature)` as a sanity check — if it fails we definitely shouldn't show the signature as valid)
  - Enter / Esc → `NavigateBack` to MainMenu
- `Message::SigningFailed { request_id, error }` navigates back to WalletDetail with the error modal

**Files touched**:
- `apps/tui-node/src/elm/components/signature_complete.rs` (NEW, pattern off `wallet_complete.rs`)
- `apps/tui-node/src/elm/model.rs` — `CompletedSignatureInfo`, `WalletState.last_finalized_signature`
- `apps/tui-node/src/elm/update.rs` — `SigningComplete` + `SigningFailed` handlers
- `apps/tui-node/src/elm/app.rs` — mount + keyboard for SignatureComplete
- Possibly `apps/tui-node/src/elm/components/wallet_detail.rs` — pop the user back to Detail after SignatureComplete, showing the updated "last sign timestamp"

**Tests**:
- Transition: `SigningComplete` pushes SignatureComplete + stashes info + clears signing_state
- Render: component shows signature hex, verified status, message preview
- E2E smoke: 3 nodes sign a fixed test message (`"hello world"`), assert SignatureComplete renders on all three with the same signature hex and `Verified: yes`

**Status**: Not Started

---

## Execution order

C.1 → C.2 → C.3 → C.4 → C.5. C.1 and C.2 are parallel-safe (different files), but C.2 is more work and has more risk, so tackle C.1 first to unblock the downstream UI work.

Each stage ships as its own commit (or small set). Every commit must pass `cargo check -p tui-node` + the existing 89-test suite. The 3-node DKG smoke must keep passing after every commit.

After Stage 5 is green, **delete this file**.

---

## Open questions

1. **Session acceptance UX** — does the creator wait for `threshold - 1` joiners to accept before kicking off Round 1, or does it proceed the instant anyone is available? DKG required all N; signing only needs threshold-many. **Proposal**: wait for threshold-many acceptances with a 60s timeout, then either proceed or fail. Re-examine at Stage C.4.
2. **Message input format** — single-line text is fine for a demo but real transactions are multi-KB blobs. Phase D will need a "paste raw bytes" or "load from file" path. Don't over-scope Phase C.
3. **Hot-loaded `KeyPackage` lifetime** — once a wallet is unlocked, how long does its `KeyPackage` stay in memory? Until process exit? Until an idle timer? **Proposal**: keep until explicit re-lock or process exit for this phase; auto-lock timer is a Phase E chore.
