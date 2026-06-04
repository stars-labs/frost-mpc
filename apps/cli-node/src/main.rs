//! `mpc-wallet-cli` — headless, scriptable front-end for the MPC wallet.
//!
//! Drives the same Elm core as the TUI/native via
//! `tui_node::elm::HeadlessRunner`, exposing a newline-delimited JSON
//! protocol on stdin/stdout (see [`protocol`]). Built for LLM/agent
//! control and automated end-to-end testing.
//!
//! IMPORTANT: stdout carries ONLY protocol JSON. All logs go to stderr.

use clap::{Parser, Subcommand};
use mpc_wallet_cli::protocol;
use mpc_wallet_cli::serve::{self, ServeOpts};
use mpc_wallet_cli::simulate::{self, SimulateOpts};

#[derive(Parser)]
#[command(name = "mpc-wallet-cli", version, about = "Headless MPC wallet CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the JSONL daemon: read commands on stdin, emit events on stdout.
    Serve(ServeArgs),
    /// Run a full N-node DKG in one process (embedded signal server) and
    /// print a JSON summary. Self-contained — ideal for CI / smoke tests.
    Simulate(SimulateArgs),
    /// Print the command/event protocol catalog as JSON (self-discovery).
    Schema,
}

#[derive(clap::Args)]
struct SimulateArgs {
    /// Number of participants (devices).
    #[arg(long, default_value_t = 2)]
    nodes: usize,
    /// Signers required (K of N). Defaults to N (all).
    #[arg(long)]
    threshold: Option<u16>,
    /// Ciphersuite (currently secp256k1).
    #[arg(long, default_value = "secp256k1")]
    curve: String,
    /// Overall timeout in seconds.
    #[arg(long, default_value_t = 90)]
    timeout: u64,
    /// If set, after DKG sign this message with a quorum and verify it.
    #[arg(long)]
    sign: Option<String>,
    /// tracing filter (stderr); empty to silence.
    #[arg(long, default_value = "")]
    log_level: String,
}

#[derive(clap::Args)]
struct ServeArgs {
    /// Stable identity for this node (used in the DKG participant set).
    #[arg(long, default_value = "cli-node")]
    device_id: String,
    /// Keystore directory. Use an isolated dir per node when testing.
    #[arg(long, default_value = "~/.frost_keystore")]
    keystore: String,
    /// Signal server URL.
    #[arg(long, default_value = "wss://panda.qzz.io")]
    signal_server: String,
    /// Ciphersuite (P1: secp256k1 only).
    #[arg(long, default_value = "secp256k1")]
    curve: String,
    /// tracing filter (stderr).
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Schema => {
            println!("{}", protocol::schema_json());
            Ok(())
        }
        Command::Simulate(args) => {
            if !args.log_level.is_empty() {
                let _ = tracing_subscriber::fmt()
                    .with_writer(std::io::stderr)
                    .with_env_filter(
                        tracing_subscriber::EnvFilter::try_new(&args.log_level)
                            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                    )
                    .with_ansi(false)
                    .try_init();
            }
            let threshold = args.threshold.unwrap_or(args.nodes as u16);
            let opts = SimulateOpts {
                nodes: args.nodes,
                threshold,
                curve: args.curve,
                signal_url: None,
                timeout_secs: args.timeout,
            };
            let ok = if let Some(msg) = args.sign {
                let r = simulate::run_signing_simulation(opts, &msg).await?;
                println!("{}", r.to_json());
                r.verified
            } else {
                let r = simulate::run_simulation(opts).await?;
                println!("{}", r.to_json());
                r.agreed
            };
            if ok {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Command::Serve(args) => {
            // Logs MUST go to stderr so stdout stays pure JSONL.
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_new(&args.log_level)
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .with_ansi(false)
                .init();

            let keystore_path = expand_tilde(&args.keystore);
            serve::serve(ServeOpts {
                device_id: args.device_id,
                keystore_path,
                signal_url: args.signal_server,
                curve: args.curve,
            })
            .await
        }
    }
}

/// Expand a leading `~/` to the user's home dir.
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}
