# WebRTC Signal Server — Cloudflare Worker

Rust-over-WASM Cloudflare Worker backing the signal-server side of
the MPC Wallet. Uses a single `Devices` Durable Object class to hold
connected-device + session-announcement state.

## Features

- **WebSocket-based signaling** for WebRTC device discovery + P2P
  relay
- **Durable Object** persistence (`Devices` class) for consistent
  device + session tracking across Worker invocations
- Compatible with Cloudflare's free plan (uses SQLite-backed
  Durable Objects via the `new_sqlite_classes` migration flag)

## Protocol

The wire protocol is shared with the standalone Rust signal server
under `../server/` — authoritative enum definitions live at
`apps/signal-server/server/src/lib.rs` (`ClientMsg` and `ServerMsg`).
Full message-type matrix + session-discovery semantics are in
[`docs/deployment/CLOUDFLARE_DEPLOYMENT.md`](../../../docs/deployment/CLOUDFLARE_DEPLOYMENT.md)
(workspace-level, rewritten in commit 1841904). Abbreviated
mini-reference:

### Client → Server

- `{ "type": "register", "device_id": "<id>" }`
- `{ "type": "list_devices" }`
- `{ "type": "relay", "to": "<id>", "data": <any JSON> }`
- `{ "type": "announce_session", "session_info": { … } }`
- `{ "type": "request_active_sessions" }`
- `{ "type": "session_status_update", "session_info": { … } }`
- `{ "type": "query_my_active_sessions" }`

### Server → Client

- `{ "type": "devices", "devices": [ … ] }`
- `{ "type": "relay", "from": "<id>", "data": <any JSON> }`
- `{ "type": "error", "error": "<message>" }`
- `{ "type": "session_available", "session_info": { … } }`
- `{ "type": "sessions_for_device", "sessions": [ … ] }`
- `{ "type": "session_list_request", "from": "<id>" }`
- `{ "type": "session_removed", "session_id": "<id>", "reason": "<text>" }`

## Project Structure

- `src/lib.rs` — Worker entry + `Devices` Durable Object impl
- `wrangler.toml` — Worker + Durable Object configuration
- `Cargo.toml` — `cdylib` + `rlib` crate-type, builds via
  `worker-build --release`

## Deploying to Cloudflare

```bash
# 1. Install wrangler + worker-build
npm install -g wrangler          # or: bun add -g wrangler
cargo install worker-build

# 2. Log in once per machine
wrangler login

# 3. Edit wrangler.toml with YOUR account_id + routes
#    (the committed config is bound to the upstream
#    maintainer's `xiongchenyu.dpdns.org` route)

# 4. Deploy — wrangler deploy handles the build via the
#    `[build] command = "worker-build --release"` entry in
#    wrangler.toml; no separate `wrangler build` step.
wrangler deploy
```

Older `wrangler publish` is deprecated in current Wrangler
versions — use `wrangler deploy`.

## Durable Object migration notes

The `Devices` Durable Object class was renamed from an earlier
`Peers` name; the committed `wrangler.toml` has:

```toml
[[migrations]]
tag = "v2"
renamed_classes = [{ from = "Peers", to = "Devices" }]
```

A fresh-account deployment that has never shipped `Peers` can use a
simpler migration:

```toml
[[migrations]]
tag = "v1"
new_sqlite_classes = ["Devices"]
```

(`new_sqlite_classes` is the free-plan-compatible route.)

## References

- Canonical deployment reference:
  [`docs/deployment/CLOUDFLARE_DEPLOYMENT.md`](../../../docs/deployment/CLOUDFLARE_DEPLOYMENT.md)
  + [`docs/signal-server/docs/deployment/cloudflare-deployment.md`](../docs/deployment/cloudflare-deployment.md)
- [Cloudflare Durable Objects docs](https://developers.cloudflare.com/durable-objects/)
- [worker-rs](https://github.com/cloudflare/workers-rs) — the Rust
  SDK this crate uses
