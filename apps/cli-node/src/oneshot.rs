//! One-shot subcommands (#24/#25): start a headless runner inline, send a
//! command, block until the correlated terminal event (or timeout), print
//! the result as JSON, and map it to an exit code. Thin wrappers over the
//! same runner + bridge that `serve` uses — for humans/scripts that want a
//! single blocking command instead of the JSONL daemon.
//!
//! Error UX: these commands are the surface investors poke at by hand, so every
//! failure path returns an **actionable** message (what failed + the most likely
//! cause + the next thing to try) rather than a bare "timed out". See
//! `connect_help` and the per-command `wait_outcome` hints.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tui_node::elm::headless::{spawn_ed25519, spawn_secp256k1};
use tui_node::elm::model::{WalletConfig, WalletMode};
use tui_node::elm::{Message, Model};

use crate::bridge::Bridge;
use crate::protocol::CliEvent;

/// Shared configuration for every one-shot command.
pub struct OneShotOpts {
    pub device_id: String,
    pub keystore_path: String,
    pub signal_url: String,
    pub timeout_secs: u64,
    /// Ciphersuite: "secp256k1" (default) or "ed25519". ed25519 produces a
    /// standard RFC-8032 signature that any off-the-shelf verifier (and Solana)
    /// can check — the runner ciphersuite is fixed at spawn.
    pub curve: String,
}

/// Spawn a runner whose events stream to the returned receiver.
fn start(opts: &OneShotOpts) -> (UnboundedSender<Message>, UnboundedReceiver<CliEvent>) {
    let bridge = Arc::new(Mutex::new(Bridge::new()));
    let (ev_tx, ev_rx) = unbounded_channel::<CliEvent>();
    let cb = move |model: &Model, msg: Option<&Message>| {
        let events = bridge.lock().unwrap().on_sync(model, msg);
        for e in events {
            let _ = ev_tx.send(e);
        }
    };
    let tx = if opts.curve == "ed25519" {
        spawn_ed25519(
            opts.device_id.clone(),
            opts.keystore_path.clone(),
            opts.signal_url.clone(),
            cb,
        )
    } else {
        spawn_secp256k1(
            opts.device_id.clone(),
            opts.keystore_path.clone(),
            opts.signal_url.clone(),
            cb,
        )
    };
    (tx, ev_rx)
}

/// Actionable message when the signal-server connection never establishes —
/// the #1 thing a hands-on user trips over (a roomless hosted connection is
/// silently rejected, so it just "hangs" until this 15s timeout).
fn connect_help(opts: &OneShotOpts, secs: u64) -> String {
    let hosted = opts.signal_url.starts_with("wss://");
    let has_room = opts.signal_url.contains("room=");
    let mut s = format!(
        "could not connect to the signal server within {secs}s ({}).",
        opts.signal_url
    );
    if hosted && !has_room {
        s.push_str(
            "\n  → No --room set. The hosted server requires a strong room (≥16 chars); a \
             roomless connection is rejected.\
             \n    Add the SAME value on every device, e.g.  --room \"$(uuidgen | tr -d -)\".",
        );
    } else {
        s.push_str(
            "\n  → Check the server is reachable and you're online. For a LAN/offline demo, run a \
             local server and use  --signal-server ws://<host-ip>:9000  (no room needed).",
        );
    }
    s.push_str(
        "\n  → To prove the cryptography with no server at all:  \
         mpc-wallet-cli simulate --nodes 3 --threshold 2 --sign hello",
    );
    s
}

/// Wait for the signal-server connection; on timeout return [`connect_help`].
/// A server-sent error frame (e.g. weak/missing room) surfaces immediately.
async fn wait_connected(rx: &mut UnboundedReceiver<CliEvent>, opts: &OneShotOpts) -> anyhow::Result<()> {
    const SECS: u64 = 15;
    if opts.signal_url.starts_with("wss://") && !opts.signal_url.contains("room=") {
        eprintln!(
            "note: no --room set — the hosted server requires a strong --room (≥16 chars); \
             if this hangs, that's why."
        );
    }
    let res = tokio::time::timeout(Duration::from_secs(SECS), async {
        loop {
            match rx.recv().await {
                Some(CliEvent::Connection { connected: true }) => return Ok(()),
                Some(CliEvent::Error { code, message, .. }) => anyhow::bail!("{code}: {message}"),
                Some(_) => continue,
                None => anyhow::bail!("the runner stopped before it could connect"),
            }
        }
    })
    .await;
    match res {
        Ok(inner) => inner,
        Err(_) => Err(anyhow::anyhow!(connect_help(opts, SECS))),
    }
}

/// Wait for a terminal outcome. On a runtime error, surface it verbatim; on
/// timeout, append `hint` (what to check) to a clear "timed out waiting for X".
async fn wait_outcome<P>(
    rx: &mut UnboundedReceiver<CliEvent>,
    secs: u64,
    waiting_for: &str,
    hint: &str,
    pred: P,
) -> anyhow::Result<CliEvent>
where
    P: Fn(&CliEvent) -> bool,
{
    let res = tokio::time::timeout(Duration::from_secs(secs), async {
        loop {
            match rx.recv().await {
                Some(e) if pred(&e) => return Ok(e),
                Some(CliEvent::Error { code, message, .. }) => anyhow::bail!("{code}: {message}"),
                Some(_) => continue,
                None => anyhow::bail!("the runner stopped before {waiting_for}"),
            }
        }
    })
    .await;
    match res {
        Ok(inner) => inner,
        Err(_) => Err(anyhow::anyhow!("timed out after {secs}s waiting for {waiting_for}.{hint}")),
    }
}

fn print(ev: &CliEvent) {
    println!(
        "{}",
        serde_json::to_string_pretty(ev).unwrap_or_else(|_| ev.to_line())
    );
}

/// `wallet list` — read the keystore (no network).
pub async fn wallet_list(opts: OneShotOpts) -> anyhow::Result<bool> {
    let (tx, mut rx) = start(&opts);
    tx.send(Message::ListWallets)?;
    let ev = wait_outcome(
        &mut rx,
        5,
        "the wallet list",
        &format!("\n  → Check the --keystore path is readable ({}).", opts.keystore_path),
        |e| matches!(e, CliEvent::Wallets { .. }),
    )
    .await?;
    print(&ev);
    Ok(true)
}

/// `wallet create` — announce a DKG and block until it completes.
pub async fn wallet_create(
    opts: OneShotOpts,
    name: String,
    threshold: u16,
    total: u16,
    password: String,
) -> anyhow::Result<bool> {
    if total < 2 {
        anyhow::bail!(
            "--total must be ≥ 2 (got {total}). A shared wallet needs at least two devices; \
             the classic demo is --total 3 --threshold 2 (2-of-3)."
        );
    }
    if threshold < 1 || threshold > total {
        anyhow::bail!(
            "--threshold must be between 1 and --total ({total}); got {threshold}. \
             Tip: 2-of-3 = --threshold 2 --total 3."
        );
    }
    let (tx, mut rx) = start(&opts);
    tx.send(Message::TriggerReconnect)?;
    wait_connected(&mut rx, &opts).await?;
    eprintln!(
        "note: announcing a {threshold}-of-{total} DKG as device '{}'. Waiting up to {}s for the \
         other {} participant(s) to join (same --room, a UNIQUE --device-id each)…",
        opts.device_id,
        opts.timeout_secs,
        total - 1
    );
    tx.send(Message::HeadlessCreateWallet {
        config: WalletConfig {
            name: name.clone(),
            total_participants: total,
            threshold,
            mode: WalletMode::Online,
        },
        password,
        label: name,
    })?;
    let ev = wait_outcome(
        &mut rx,
        opts.timeout_secs,
        "the DKG to complete",
        "\n  → DKG needs ALL participants online together. Did the other devices run \
         `mpc-wallet-cli session join` with the SAME --room and a unique --device-id each?",
        |e| matches!(e, CliEvent::DkgComplete { .. }),
    )
    .await?;
    print(&ev);
    Ok(true)
}

/// `session join` — join a discovered DKG/signing session by id.
pub async fn session_join(
    opts: OneShotOpts,
    session_id: String,
    password: String,
) -> anyhow::Result<bool> {
    let (tx, mut rx) = start(&opts);
    tx.send(Message::TriggerReconnect)?;
    wait_connected(&mut rx, &opts).await?;
    // Give the server a moment to replay the session, then join.
    tokio::time::sleep(Duration::from_secs(3)).await;
    tx.send(Message::HeadlessJoinSession {
        session_id,
        password,
        label: String::new(),
    })?;
    let ev = wait_outcome(
        &mut rx,
        opts.timeout_secs,
        "the session to complete",
        "\n  → Is the session id correct and the creator still online in the SAME --room? \
         Everyone must share --room and --signal-server; the password must match this wallet.",
        |e| {
            matches!(
                e,
                CliEvent::DkgComplete { .. }
                    | CliEvent::SignatureComplete { .. }
                    | CliEvent::ReshareComplete { .. }
            )
        },
    )
    .await?;
    print(&ev);
    Ok(true)
}

/// `reshare` — initiate a share refresh/resharing of an existing wallet and
/// block until it completes. The group public key (address) is preserved; the
/// refreshed share replaces the old one on disk. Retained co-signers approve by
/// running `session join` (or `serve`) on the announced reshare session.
pub async fn reshare(
    opts: OneShotOpts,
    wallet_id: String,
    password: String,
) -> anyhow::Result<bool> {
    let (tx, mut rx) = start(&opts);
    tx.send(Message::TriggerReconnect)?;
    wait_connected(&mut rx, &opts).await?;
    tx.send(Message::HeadlessReshare {
        wallet_id,
        password,
        keystore_path: opts.keystore_path.clone(),
    })?;
    let ev = wait_outcome(
        &mut rx,
        opts.timeout_secs,
        "the reshare to complete",
        "\n  → Reshare needs the retained signers to join the announced session in the SAME \
         --room (via `session join`, or `serve --auto-approve`). Check --wallet-id exists \
         (`wallet list`) and the password matches.",
        |e| matches!(e, CliEvent::ReshareComplete { .. }),
    )
    .await?;
    print(&ev);
    Ok(true)
}

/// `sign` — initiate a threshold signing and block until it completes.
pub async fn sign(
    opts: OneShotOpts,
    wallet_id: String,
    message: String,
    encoding: String,
    password: String,
) -> anyhow::Result<bool> {
    let (tx, mut rx) = start(&opts);
    tx.send(Message::TriggerReconnect)?;
    wait_connected(&mut rx, &opts).await?;
    tx.send(Message::HeadlessSign {
        wallet_id,
        message,
        encoding,
        password,
    })?;
    let ev = wait_outcome(
        &mut rx,
        opts.timeout_secs,
        "the signature",
        "\n  → Signing needs a quorum to approve. Did a co-signer run `session join` (or \
         `serve --auto-approve`) on the announced session in the SAME --room? Check --wallet-id \
         exists (`wallet list`) and the password matches.",
        |e| matches!(e, CliEvent::SignatureComplete { .. }),
    )
    .await?;
    print(&ev);
    Ok(true)
}
