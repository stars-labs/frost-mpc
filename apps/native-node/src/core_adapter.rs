//! Adapter to integrate TUI node's shared core with native UI

use slint::Weak;
use std::sync::Arc;
use tui_node::core::{
    connection_manager::ConnectionManager,
    dkg_manager::DkgManager,
    offline_manager::OfflineManager,
    session_manager::SessionManager,
    signing_manager::SigningManager,
    wallet_manager::WalletManager,
    CoreState, UICallback,
};

use crate::slint_generatedMainWindow::MainWindow;
use crate::ui_callback::NativeUICallback;

/// Core adapter that manages all the shared business logic
pub struct CoreAdapter {
    pub state: Arc<CoreState>,
    pub connection_manager: Arc<ConnectionManager>,
    pub session_manager: Arc<SessionManager>,
    pub dkg_manager: Arc<DkgManager>,
    pub wallet_manager: Arc<WalletManager>,
    pub offline_manager: Arc<OfflineManager>,
    pub signing_manager: Arc<SigningManager>,
    ui_callback: Arc<dyn UICallback>,
}

impl CoreAdapter {
    /// Create new core adapter with native UI callback
    pub fn new(window: Weak<MainWindow>) -> Self {
        let state = Arc::new(CoreState::new());
        let ui_callback: Arc<dyn UICallback> = Arc::new(NativeUICallback::new(window));
        
        Self {
            connection_manager: Arc::new(ConnectionManager::new(state.clone(), ui_callback.clone())),
            session_manager: Arc::new(SessionManager::new(state.clone(), ui_callback.clone())),
            dkg_manager: Arc::new(DkgManager::new(state.clone(), ui_callback.clone())),
            wallet_manager: Arc::new(WalletManager::new(state.clone(), ui_callback.clone())),
            offline_manager: Arc::new(OfflineManager::new(state.clone(), ui_callback.clone())),
            signing_manager: Arc::new(SigningManager::new(state.clone(), ui_callback.clone())),
            state,
            ui_callback,
        }
    }

    /// Create a new signing request. Typically called from a
    /// "Sign Message" button in Settings; opens the confirm modal
    /// via `UICallback::update_signing_request`. Returns the
    /// request id so the caller can pair approve/reject later.
    pub async fn request_signing(
        &self,
        message_hex: String,
        chain: String,
        display_label: Option<String>,
    ) -> Result<String, String> {
        self.signing_manager
            .request_signing(message_hex, chain, display_label)
            .await
            .map_err(|e| e.to_string())
    }

    /// User approved the pending signing request from the confirm
    /// modal. Drives state through commitment / share / aggregate.
    pub async fn approve_signing(&self, request_id: String) -> Result<(), String> {
        self.signing_manager
            .approve(&request_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// User rejected the pending signing request.
    pub async fn reject_signing(&self, request_id: String) -> Result<(), String> {
        self.signing_manager
            .reject(&request_id)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Connect to WebSocket server
    pub async fn connect_websocket(&self, url: String) -> Result<(), String> {
        self.connection_manager
            .connect_websocket(url)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Create a new wallet
    pub async fn create_wallet(&self) -> Result<(), String> {
        // For demo, create with default parameters
        self.wallet_manager
            .create_wallet(
                "New Wallet".to_string(),
                2,
                vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()],
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    /// Import a keystore from disk. Opens a native file-picker for
    /// the `.dat` path, then passes the user-supplied password to
    /// `WalletManager::import_wallet` for decryption. Pass `""` if
    /// the keystore is unencrypted.
    pub async fn import_wallet(&self, password: String) -> Result<(), String> {
        // `rfd::AsyncFileDialog` is async-friendly but its await
        // point runs on the GUI thread; keep this on tokio's
        // blocking scheduler to avoid blocking the Slint event loop.
        let Some(handle) = tokio::task::spawn_blocking(|| {
            rfd::FileDialog::new()
                .add_filter("MPC keystore", &["dat", "json"])
                .set_title("Import MPC keystore")
                .pick_file()
        })
        .await
        .map_err(|e| e.to_string())?
        else {
            self.ui_callback
                .show_message("Import cancelled".to_string(), false)
                .await;
            return Ok(());
        };

        let path = handle.to_string_lossy().into_owned();
        let msg = if password.is_empty() {
            format!("Importing keystore from {path} (no password)...")
        } else {
            format!("Importing keystore from {path}...")
        };
        self.ui_callback.show_message(msg, false).await;

        self.wallet_manager
            .import_wallet(path, password)
            .await
            .map_err(|e| e.to_string())
    }

    /// Export the active wallet to a keystore file. Opens a native
    /// save dialog for the destination path; the user-supplied
    /// password is used to encrypt the output. Pass `""` to write
    /// an unencrypted keystore.
    pub async fn export_wallet(&self, password: String) -> Result<(), String> {
        let Some(handle) = tokio::task::spawn_blocking(|| {
            rfd::FileDialog::new()
                .add_filter("MPC keystore", &["dat"])
                .set_title("Export MPC keystore")
                .set_file_name("mpc-wallet.dat")
                .save_file()
        })
        .await
        .map_err(|e| e.to_string())?
        else {
            self.ui_callback
                .show_message("Export cancelled".to_string(), false)
                .await;
            return Ok(());
        };

        let path = handle.to_string_lossy().into_owned();

        // Export the currently-active wallet. CoreState tracks the
        // active index alongside the wallet list.
        let active_index = *self.state.active_wallet_index.lock().await;
        let msg = if password.is_empty() {
            format!("Exporting wallet to {path} (unencrypted)...")
        } else {
            format!("Exporting wallet to {path}...")
        };
        self.ui_callback.show_message(msg, false).await;

        self.wallet_manager
            .export_wallet(active_index, path, password)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Create a new session
    pub async fn create_session(&self) -> Result<(), String> {
        // Get device ID (would be from config in real app)
        let device_id = "native-node-001".to_string();
        
        self.session_manager
            .create_session(device_id, 2, 3)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    /// Join an existing session
    pub async fn join_session(&self, session_id: String) -> Result<(), String> {
        let device_id = "native-node-001".to_string();
        
        self.session_manager
            .join_session(session_id, device_id)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Leave current session
    pub async fn leave_session(&self) -> Result<(), String> {
        let device_id = "native-node-001".to_string();
        
        self.session_manager
            .leave_session(device_id)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Refresh available sessions
    pub async fn refresh_sessions(&self) -> Result<(), String> {
        self.session_manager
            .refresh_sessions()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Toggle offline mode
    pub async fn toggle_offline_mode(&self) -> Result<(), String> {
        self.offline_manager
            .toggle_offline_mode()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Start DKG process
    pub async fn start_dkg(&self) -> Result<(), String> {
        // Get active session
        let session = self.session_manager
            .get_active_session()
            .await
            .ok_or_else(|| "No active session".to_string())?;
        
        // Start DKG with session participants
        self.dkg_manager
            .start_dkg(session.threshold.0, session.participants)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Abort DKG process
    pub async fn abort_dkg(&self) -> Result<(), String> {
        self.dkg_manager
            .abort_dkg()
            .await
            .map_err(|e| e.to_string())
    }
}