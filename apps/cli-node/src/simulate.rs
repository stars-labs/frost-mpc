//! In-process multi-node DKG simulation (#21).
//!
//! Runs a full N-node FROST DKG inside one process against an embedded
//! signal server on an ephemeral port — real WebRTC over loopback, real
//! crypto, isolated per-node keystores. One self-contained command for CI
//! and LLM smoke-testing; also the shared orchestration the e2e test uses.

use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tui_node::elm::headless::spawn_secp256k1;
use tui_node::elm::model::{WalletConfig, WalletMode};
use tui_node::elm::{Message, Model};

/// Simulation configuration.
pub struct SimulateOpts {
    pub nodes: usize,
    pub threshold: u16,
    pub curve: String,
    /// External signal server; `None` embeds one on an ephemeral port.
    pub signal_url: Option<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Serialize)]
pub struct NodeOutcome {
    pub device_id: String,
    pub wallet_id: String,
    pub group_public_key: String,
}

#[derive(Debug, Serialize)]
pub struct SimulationResult {
    pub nodes: usize,
    pub threshold: u16,
    /// True iff every node finished DKG with the same non-empty group key.
    pub agreed: bool,
    pub group_public_key: String,
    pub outcomes: Vec<NodeOutcome>,
    pub elapsed_ms: u128,
}

impl SimulationResult {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

#[derive(Debug, Clone)]
enum Evt {
    Connected,
    SessionDiscovered(String),
    DkgDone { wallet_id: String, group_key: String },
}

fn watcher() -> (
    Box<dyn Fn(&Model, Option<&Message>) + Send>,
    UnboundedReceiver<Evt>,
) {
    let (tx, rx) = unbounded_channel::<Evt>();
    let closure = move |model: &Model, msg: Option<&Message>| {
        if model.network_state.connected {
            let _ = tx.send(Evt::Connected);
        }
        if let Some(m) = msg {
            match m {
                Message::SessionDiscovered { session } => {
                    let _ = tx.send(Evt::SessionDiscovered(session.session_id.clone()));
                }
                Message::DKGFinalized {
                    wallet_id,
                    group_pubkey_hex,
                    ..
                } => {
                    let _ = tx.send(Evt::DkgDone {
                        wallet_id: wallet_id.clone(),
                        group_key: group_pubkey_hex.clone(),
                    });
                }
                _ => {}
            }
        }
    };
    (Box::new(closure), rx)
}

async fn wait_for<F>(
    rx: &mut UnboundedReceiver<Evt>,
    secs: u64,
    pred: F,
) -> anyhow::Result<Evt>
where
    F: Fn(&Evt) -> bool,
{
    tokio::time::timeout(Duration::from_secs(secs), async {
        loop {
            match rx.recv().await {
                Some(e) if pred(&e) => return Ok(e),
                Some(_) => continue,
                None => anyhow::bail!("event channel closed"),
            }
        }
    })
    .await
    .map_err(|_| anyhow::anyhow!("timed out after {secs}s waiting for event"))?
}

/// Run the simulation and return a structured result.
pub async fn run_simulation(opts: SimulateOpts) -> anyhow::Result<SimulationResult> {
    if opts.curve != "secp256k1" {
        anyhow::bail!("simulate currently supports curve=secp256k1 only");
    }
    if opts.nodes < 2 {
        anyhow::bail!("need at least 2 nodes");
    }
    if opts.threshold < 1 || opts.threshold as usize > opts.nodes {
        anyhow::bail!("threshold must be in 1..=nodes");
    }
    let started = Instant::now();

    // Embedded signal server unless an external one was provided.
    let ws_url = match &opts.signal_url {
        Some(u) => u.clone(),
        None => {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let port = listener.local_addr()?.port();
            tokio::spawn(webrtc_signal_server::run(listener));
            format!("ws://127.0.0.1:{port}")
        }
    };

    // Spawn the runners.
    let mut keystores = Vec::new();
    let mut senders = Vec::new();
    let mut receivers = Vec::new();
    let device_ids: Vec<String> = (0..opts.nodes).map(|i| format!("sim-node-{i}")).collect();
    for device_id in &device_ids {
        let ks = tempfile::TempDir::new()?;
        let (cb, rx) = watcher();
        let tx = spawn_secp256k1(
            device_id.clone(),
            ks.path().to_string_lossy().into_owned(),
            ws_url.clone(),
            cb,
        );
        keystores.push(ks);
        senders.push(tx);
        receivers.push(rx);
    }

    // Connect everyone.
    for tx in &senders {
        let _ = tx.send(Message::TriggerReconnect);
    }
    for rx in &mut receivers {
        wait_for(rx, 15, |e| matches!(e, Evt::Connected)).await?;
    }

    // node 0 creates; the rest join the announced session.
    senders[0].send(Message::HeadlessCreateWallet {
        config: WalletConfig {
            name: "sim".into(),
            total_participants: opts.nodes as u16,
            threshold: opts.threshold,
            mode: WalletMode::Online,
        },
        password: "sim-password-0".into(),
        label: "sim".into(),
    })?;

    for (i, rx) in receivers.iter_mut().enumerate().skip(1) {
        let session_id = match wait_for(rx, 20, |e| matches!(e, Evt::SessionDiscovered(_))).await? {
            Evt::SessionDiscovered(id) => id,
            _ => unreachable!(),
        };
        senders[i].send(Message::HeadlessJoinSession {
            session_id,
            password: format!("sim-password-{i}"),
            label: "sim".into(),
        })?;
    }

    // Collect each node's finalization.
    let mut outcomes = Vec::new();
    for (i, rx) in receivers.iter_mut().enumerate() {
        let done = wait_for(rx, opts.timeout_secs, |e| matches!(e, Evt::DkgDone { .. })).await?;
        if let Evt::DkgDone { wallet_id, group_key } = done {
            outcomes.push(NodeOutcome {
                device_id: device_ids[i].clone(),
                wallet_id,
                group_public_key: group_key,
            });
        }
    }

    let first_key = outcomes.first().map(|o| o.group_public_key.clone()).unwrap_or_default();
    let agreed = !first_key.is_empty()
        && outcomes.iter().all(|o| o.group_public_key == first_key);

    // Keystores must outlive the ceremony.
    drop(keystores);

    Ok(SimulationResult {
        nodes: opts.nodes,
        threshold: opts.threshold,
        agreed,
        group_public_key: first_key,
        outcomes,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "real WebRTC/DKG over loopback; run with --ignored"]
    async fn simulate_2_of_2() {
        let result = run_simulation(SimulateOpts {
            nodes: 2,
            threshold: 2,
            curve: "secp256k1".into(),
            signal_url: None,
            timeout_secs: 90,
        })
        .await
        .expect("simulation ran");
        assert!(result.agreed, "nodes disagreed: {:?}", result.outcomes);
        assert_eq!(result.outcomes.len(), 2);
        assert!(!result.group_public_key.is_empty());
    }
}
