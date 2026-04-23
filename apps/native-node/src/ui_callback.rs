//! UICallback implementation for native Slint UI.
//!
//! Slint's `MainWindow` is `!Send` (its generated struct holds
//! `Cell` / `UnsafeCell` fields), so capturing a strong handle into
//! closures passed to `slint::invoke_from_event_loop` — which
//! requires `Send` — does not compile. We therefore clone the
//! `Weak<MainWindow>` (which IS `Send`) before each closure and
//! upgrade inside, bailing silently if the window is already gone.

use async_trait::async_trait;
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak};
use tui_node::core::{
    ConnectionInfo, ConnectionStatus, OperationMode, ParticipantInfo, ParticipantStatus,
    SDCardOperation, SessionInfo, SessionStatus, SigningRequest, SigningState, UICallback,
    WalletInfo,
};

use crate::slint_generatedMainWindow::{
    AppState, ConnectionInfo as SlintConnectionInfo, MainWindow,
    Participant as SlintParticipant, SDCardOperation as SlintSDCardOperation,
    SessionInfo as SlintSessionInfo, WalletInfo as SlintWalletInfo,
};

/// Native UI callback implementation using Slint
pub struct NativeUICallback {
    window: Weak<MainWindow>,
}

impl NativeUICallback {
    pub fn new(window: Weak<MainWindow>) -> Self {
        Self { window }
    }

    /// Convert core ConnectionInfo to Slint ConnectionInfo
    fn to_slint_connection(conn: &ConnectionInfo) -> SlintConnectionInfo {
        SlintConnectionInfo {
            peer_id: conn.peer_id.clone().into(),
            status: match conn.status {
                ConnectionStatus::Connected => "connected".into(),
                ConnectionStatus::Connecting => "connecting".into(),
                ConnectionStatus::Disconnected => "disconnected".into(),
                ConnectionStatus::Failed => "failed".into(),
            },
            latency_ms: conn.latency_ms as i32,
            quality: conn.quality,
        }
    }

    /// Convert core WalletInfo to Slint WalletInfo
    fn to_slint_wallet(wallet: &WalletInfo) -> SlintWalletInfo {
        // Produce a short-form address for display since Slint 1.x
        // strings can't be sliced inside the UI. Prefix + suffix is
        // the idiomatic wallet address rendering.
        let display_address = if wallet.address.len() > 20 {
            format!(
                "{}...{}",
                &wallet.address[..6],
                &wallet.address[wallet.address.len() - 4..]
            )
        } else {
            wallet.address.clone()
        };
        SlintWalletInfo {
            id: wallet.id.clone().into(),
            name: wallet.name.clone().into(),
            address: display_address.into(),
            balance: wallet.balance.clone().into(),
            chain: wallet.chain.clone().into(),
            threshold: wallet.threshold.clone().into(),
        }
    }

    /// Convert core SessionInfo to Slint SessionInfo
    fn to_slint_session(session: &SessionInfo) -> SlintSessionInfo {
        SlintSessionInfo {
            session_id: session.session_id.clone().into(),
            initiator: session.initiator.clone().into(),
            participants: session.participants.len() as i32,
            threshold: format!("{}/{}", session.threshold.0, session.threshold.1).into(),
            status: match session.status {
                SessionStatus::Waiting => "waiting".into(),
                SessionStatus::InProgress => "in_progress".into(),
                SessionStatus::Completed => "completed".into(),
                SessionStatus::Failed => "failed".into(),
            },
            created_at: session.created_at.clone().into(),
        }
    }

    /// Convert core ParticipantInfo to Slint Participant
    fn to_slint_participant(participant: &ParticipantInfo) -> SlintParticipant {
        SlintParticipant {
            id: participant.id.clone().into(),
            name: participant.name.clone().into(),
            status: match participant.status {
                ParticipantStatus::Ready => "ready".into(),
                ParticipantStatus::Processing => "processing".into(),
                ParticipantStatus::Completed => "completed".into(),
                ParticipantStatus::Failed => "failed".into(),
                ParticipantStatus::Offline => "offline".into(),
            },
            round_completed: participant.round_completed as i32,
        }
    }

    /// Convert core SDCardOperation to Slint SDCardOperation
    fn to_slint_sd_operation(op: &SDCardOperation) -> SlintSDCardOperation {
        SlintSDCardOperation {
            operation_type: match op.operation_type {
                tui_node::core::SDOperationType::Export => "export".into(),
                tui_node::core::SDOperationType::Import => "import".into(),
            },
            data_type: op.data_type.clone().into(),
            participant: op.participant.clone().into(),
            timestamp: op.timestamp.clone().into(),
        }
    }
}

/// Helper: run a UI closure on the Slint event loop, upgrading the
/// weak handle inside the closure so the body can remain !Send-free.
fn dispatch<F>(window_weak: Weak<MainWindow>, body: F)
where
    F: FnOnce(MainWindow) + Send + 'static,
{
    if let Err(e) = slint::invoke_from_event_loop(move || {
        if let Some(window) = window_weak.upgrade() {
            body(window);
        }
    }) {
        eprintln!("[NativeUI] invoke_from_event_loop failed: {e}");
    }
}

#[async_trait]
impl UICallback for NativeUICallback {
    async fn update_connection_status(&self, websocket: bool, webrtc: bool) {
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_websocket_connected(websocket);
            state.set_webrtc_connected(webrtc);
        });
    }

    async fn update_mesh_connections(&self, connections: Vec<ConnectionInfo>) {
        let slint_connections: Vec<SlintConnectionInfo> = connections
            .iter()
            .map(Self::to_slint_connection)
            .collect();
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            let model = ModelRc::new(VecModel::from(slint_connections));
            state.set_mesh_connections(model);
        });
    }

    async fn update_operation_mode(&self, mode: OperationMode) {
        let mode_str: &'static str = match mode {
            OperationMode::Online => "online",
            OperationMode::Offline => "offline",
            OperationMode::Hybrid => "hybrid",
        };
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_operation_mode(mode_str.into());
        });
    }

    async fn update_wallets(&self, wallets: Vec<WalletInfo>) {
        let slint_wallets: Vec<SlintWalletInfo> = wallets
            .iter()
            .map(Self::to_slint_wallet)
            .collect();
        let has_keystore = !wallets.is_empty();
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            let model = ModelRc::new(VecModel::from(slint_wallets));
            state.set_wallets(model);
            state.set_has_keystore(has_keystore);
        });
    }

    async fn update_active_wallet(&self, index: usize) {
        let idx = index as i32;
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_active_wallet_index(idx);
        });
    }

    async fn update_available_sessions(&self, sessions: Vec<SessionInfo>) {
        let slint_sessions: Vec<SlintSessionInfo> = sessions
            .iter()
            .map(Self::to_slint_session)
            .collect();
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            let model = ModelRc::new(VecModel::from(slint_sessions));
            state.set_available_sessions(model);
        });
    }

    async fn update_active_session(&self, session: Option<SessionInfo>) {
        let slint_session = session.as_ref().map(Self::to_slint_session);
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            if let Some(s) = slint_session {
                state.set_active_session(s);
                state.set_has_active_session(true);
            } else {
                state.set_has_active_session(false);
            }
        });
    }

    async fn update_dkg_status(&self, active: bool, round: u8, progress: f32) {
        let r = round as i32;
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_dkg_active(active);
            state.set_dkg_current_round(r);
            state.set_dkg_progress(progress);
        });
    }

    async fn update_dkg_participants(&self, participants: Vec<ParticipantInfo>) {
        let slint_participants: Vec<SlintParticipant> = participants
            .iter()
            .map(Self::to_slint_participant)
            .collect();
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            let model = ModelRc::new(VecModel::from(slint_participants));
            state.set_dkg_participants(model);
        });
    }

    async fn update_offline_status(&self, enabled: bool, sd_card_detected: bool) {
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_offline_enabled(enabled);
            state.set_sd_card_detected(sd_card_detected);
        });
    }

    async fn update_sd_operations(&self, operations: Vec<SDCardOperation>) {
        let slint_operations: Vec<SlintSDCardOperation> = operations
            .iter()
            .map(Self::to_slint_sd_operation)
            .collect();
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            let model = ModelRc::new(VecModel::from(slint_operations));
            state.set_pending_sd_operations(model);
        });
    }

    async fn update_signing_request(&self, request: Option<SigningRequest>) {
        // Slint doesn't have Option — flatten into a bool flag
        // + scalar fields. When None, clear everything back to
        // empty so the confirm modal (which gates on
        // has_signing_request) closes cleanly.
        let (has_request, id, preview, chain, label) = match request {
            Some(r) => {
                // Truncate the message preview for display —
                // full hex strings are unreadable in the modal,
                // and pushing the whole payload to Slint wastes
                // memory on every state update.
                let preview = if r.message_hex.len() > 120 {
                    format!("{}…", &r.message_hex[..120])
                } else {
                    r.message_hex
                };
                (
                    true,
                    r.id,
                    preview,
                    r.chain,
                    r.display_label.unwrap_or_default(),
                )
            }
            None => (false, String::new(), String::new(), String::new(), String::new()),
        };
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_has_signing_request(has_request);
            state.set_signing_request_id(id.into());
            state.set_signing_message_preview(preview.into());
            state.set_signing_chain(chain.into());
            state.set_signing_label(label.into());
        });
    }

    async fn update_signing_state(&self, state_enum: SigningState) {
        // Stringify to match the Slint-side lowercase discriminants.
        let s: &'static str = match state_enum {
            SigningState::Idle => "idle",
            SigningState::AwaitingApproval => "awaiting_approval",
            SigningState::Commitment => "commitment",
            SigningState::Share => "share",
            SigningState::Aggregating => "aggregating",
            SigningState::Complete => "complete",
            SigningState::Failed(_) => "failed",
        };
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_signing_state(s.into());
        });
    }

    async fn update_signing_complete(&self, signature_hex: String) {
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_last_signature(signature_hex.into());
        });
    }

    async fn show_message(&self, message: String, is_error: bool) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let prefix = if is_error { "[ERROR]" } else { "[INFO]" };
        let log_entry = format!("{timestamp} {prefix} {message}");
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_status_message(message.clone().into());

            // Model trait exposes row_count/row_data — materialize into a
            // Vec so we can push + drain, then re-wrap into a VecModel.
            let existing = state.get_log_messages();
            let mut logs: Vec<slint::SharedString> = (0..existing.row_count())
                .filter_map(|i| existing.row_data(i))
                .collect();
            logs.push(log_entry.into());
            // Keep only last 100 messages
            let overflow = logs.len().saturating_sub(100);
            if overflow > 0 {
                logs.drain(0..overflow);
            }
            let model = ModelRc::new(VecModel::from(logs));
            state.set_log_messages(model);
        });
    }

    async fn show_progress(&self, title: String, progress: f32) {
        let text = format!("{}: {:.0}%", title, progress * 100.0);
        dispatch(self.window.clone(), move |window| {
            let state = window.global::<AppState>();
            state.set_status_message(text.into());
        });
    }

    async fn request_confirmation(&self, _message: String) -> bool {
        // For now, auto-confirm. In a real implementation, this would
        // show a modal and await the user's response via a oneshot
        // channel.
        true
    }
}
