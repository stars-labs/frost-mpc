# Cloudflare Worker Deployment Guide

## Prerequisites

1. Install Wrangler CLI:
```bash
npm install -g wrangler
```

2. Login to Cloudflare:
```bash
wrangler login
```

## Build and Deploy

1. Build the Worker:
```bash
cd apps/signal-server/cloudflare-worker
wrangler build
```

2. Deploy to Cloudflare:
```bash
wrangler publish
```

## Configuration

Make sure your `wrangler.toml` is configured with:
- Your Cloudflare account ID
- The worker name
- Durable Objects binding

## Features Added

The Cloudflare Worker now supports:

### Session Discovery
- **AnnounceSession**: When a wallet creator starts a session, it's stored in Durable Object storage
- **RequestActiveSessions**: New nodes can request all active sessions on connection
- **SessionAvailable**: Broadcasts session announcements to all connected devices
- **SessionListRequest**: Requests fresh session updates from active creators
- **SessionStatusUpdate**: Updates session status in storage

### Storage
- Sessions are stored with `session:` prefix in Durable Object storage
- Sessions persist across reconnections
- Sessions are cleaned up when the creator disconnects

### How It Works

1. **Creator starts session**: Sends `AnnounceSession` message
2. **Worker stores session**: Saves to Durable Object storage
3. **Worker broadcasts**: Sends `SessionAvailable` to all connected devices
4. **New node connects**: Sends `RequestActiveSessions` 
5. **Worker responds**: Returns all stored sessions
6. **Discovery complete**: New node sees available sessions

## Testing

After deployment, test with:
```bash
cargo run --bin mpc-wallet-tui -p tui-node -- --device-id mpc-1
# Create a session

cargo run --bin mpc-wallet-tui -p tui-node -- --device-id mpc-2
# Should see the session from mpc-1
```

## Monitoring

View logs in Cloudflare dashboard:
1. Go to Workers & Pages
2. Select your worker
3. View Logs tab

## Rollback

If issues occur, you can rollback:
```bash
wrangler rollback
```