# Multi-tenant isolation — decision (#31)

**Decision:** for now, **one signal-server instance per tenant** (Option 1, zero
code). Implement **room namespacing** (Option 2) only when a single hosted
endpoint must serve multiple tenants. Cloudflare DO-per-room (Option 3) is the
scale path for the hosted product.

---

## Current state (the problem)

`apps/signal-server/server/src/lib.rs` is a **single flat namespace** — verified
by code review:

- One global `devices: HashMap<device_id, sender>`; `device_id` must be globally
  unique (duplicate `Register` is rejected, l.101).
- `Devices` roster is broadcast to **everyone** on every (de)register (l.110-114).
- `announce_session` → `session_available` broadcast to **all** connected
  devices (l.273).
- `RequestActiveSessions` → returns **all** stored sessions to any requester
  (l.284).
- `Relay { to }` → delivered to a specific device by id (l.219).

So on a shared server, every tenant **sees every other tenant's** device ids and
session announcements. Funds stay safe (you can only contribute a usable share to
a session you were enrolled in, and identifiers come from the participant set),
but it leaks metadata (who's online, what ceremonies exist) and is noisy. Not
acceptable for separate investor cohorts / customers on one endpoint.

---

## Options

| | Isolation | Code change | Ops | When |
|---|---|---|---|---|
| **1. Instance per tenant** | Full (separate process/port/host) | **none** | run N servers | **now / demos / few tenants** |
| **2. Room namespacing** | Full, logical, on one process | moderate, localized to `lib.rs` + `Register` | one server | single hosted endpoint, many tenants |
| **3. Cloudflare Worker, DO per room** | Full, serverless, autoscaling | port Option 2 logic into the Worker (`apps/signal-server/cloudflare-worker`) | managed edge | hosted product at scale |

### Why Option 1 now
- Zero code, ships today, full isolation (different process ⇒ no cross-tenant
  visibility at all). Device-id collisions only matter *within* a tenant.
- Perfect for investor demos: give each cohort its own URL.
```bash
MPC_SIGNAL_BIND=0.0.0.0:9001 webrtc-signal-server   # tenant / cohort A
MPC_SIGNAL_BIND=0.0.0.0:9002 webrtc-signal-server   # tenant / cohort B
```
Clients: `--signal-server ws://<host>:9001` (CLI/TUI) or the extension's signal
setting.

### Conventions (apply regardless of option)
- **Device-id scheme:** `"<tenant>-<role>-<n>"` (e.g. `acme-cli-a`) → globally
  unique, human-traceable, collision-proof (ties into #29 hygiene).
- **Keystores:** one directory per `(tenant, device)`; never shared across tenants.
- **Sessions:** include the tenant in the wallet name; never reuse session ids
  across tenants.

---

## Option 2 implementation plan (when a single endpoint is needed)

Localized change in `apps/signal-server/server/src/lib.rs` (+ the `ClientMsg`
type in the shared crate). A "room" is the tenant boundary.

1. **Register carries a room.** Add `room: Option<String>` to
   `ClientMsg::Register` (default `"default"` for backward compat). Track the
   socket's room alongside its `device_id`.
2. **Scope the device map by room.** Either
   `HashMap<Room, HashMap<DeviceId, Sender>>` or key by `(room, device_id)`.
   Duplicate-id rejection becomes per-room (two tenants may both have `cli-a`).
3. **Scope the four fan-out sites to the sender's room:**
   - `Devices` roster broadcast (l.110) → only same-room devices.
   - `session_available` on announce (l.273) → only same-room devices.
   - `RequestActiveSessions` reply (l.284) → only sessions whose room matches.
   - `SessionListRequest` broadcast (l.299) → only same-room devices.
4. **Scope sessions.** Store `room` in `StoredSession`; key the sessions map by
   `(room, session_id)` (or filter by room on read). Cleanup unchanged.
5. **Relay** (l.219): resolve `to` within the sender's room only; `to == "*"`
   broadcasts within the room. Cross-room relay is refused.
6. **Clients pass the room:** add `--room`/`MPC_ROOM` to CLI/TUI and a room field
   to the extension's connect. Absent ⇒ `"default"`.

**Acceptance / test:** an L1-style test — two rooms on one embedded server; a
node in room A never receives room B's `session_available`/`Devices`, and a relay
A→(B's device) is refused. Reuse the `webrtc_signal_server::run` in-process
harness already used by `simulate`/e2e.

**Effort:** ~half a day; risk low (additive, default-room keeps existing clients
working). Not started — gated on the "single hosted endpoint" requirement.

---

## Recommendation summary

- **Today (demos, ≤ handful of tenants):** Option 1 — one instance per tenant.
  No code; do it via deployment/ops.
- **Hosted multi-tenant endpoint:** implement Option 2 (plan above); then port to
  Option 3 (Cloudflare DO-per-room) for scale.
- Either way, adopt the `<tenant>-<role>-<n>` device-id convention now — it's free
  and prevents the collision class (#29).

Tracking: issue #31. Implement Option 2 when the single-endpoint requirement is
real — say the word and it's ~half a day's work.
