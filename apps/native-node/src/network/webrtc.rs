use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::policy::bundle_policy::RTCBundlePolicy;
use webrtc::peer_connection::policy::ice_transport_policy::RTCIceTransportPolicy;
use webrtc::peer_connection::policy::rtcp_mux_policy::RTCRtcpMuxPolicy;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::commands::{DirectMessage, InternalCommand, WebRTCSignal};
use crate::state::{SharedAppState, WebRTCConnectionState};

// WebRTC configuration matching the TUI node (apps/tui-node)
static WEBRTC_CONFIG: Lazy<RTCConfiguration> = Lazy::new(|| RTCConfiguration {
    ice_servers: vec![
        // Google's public STUN servers
        RTCIceServer {
            urls: vec![
                "stun:stun.l.google.com:19302".to_owned(),
                "stun:stun1.l.google.com:19302".to_owned(),
                "stun:stun2.l.google.com:19302".to_owned(),
                "stun:stun3.l.google.com:19302".to_owned(),
                "stun:stun4.l.google.com:19302".to_owned(),
            ],
            ..Default::default()
        },
        // TURN servers for NAT traversal
        RTCIceServer {
            urls: vec!["turn:numb.viagenie.ca".to_owned()],
            username: "muazkh".to_string(),
            credential: "webrtc@live.com".to_string(),
            ..Default::default()
        },
        RTCIceServer {
            urls: vec!["turn:openrelay.metered.ca:80".to_owned()],
            username: "openrelayproject".to_string(),
            credential: "openrelayproject".to_string(),
            ..Default::default()
        },
    ],
    ice_transport_policy: RTCIceTransportPolicy::All,
    bundle_policy: RTCBundlePolicy::MaxBundle,
    rtcp_mux_policy: RTCRtcpMuxPolicy::Require,
    ice_candidate_pool_size: 10,
    ..Default::default()
});

// WebRTC API instance
static WEBRTC_API: Lazy<webrtc::api::API> = Lazy::new(|| {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs().unwrap();
    
    let interceptor_registry = register_default_interceptors(
        webrtc::api::interceptor_registry::Registry::new(), 
        &mut media_engine
    ).unwrap();
    
    APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(interceptor_registry)
        .build()
});

// Connection manager
pub struct WebRTCManager {
    connections: Arc<Mutex<HashMap<String, PeerConnectionState>>>,
    command_tx: mpsc::Sender<InternalCommand>,
    device_id: String,
}

struct PeerConnectionState {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Option<Arc<RTCDataChannel>>,
    ice_candidates_queue: Vec<RTCIceCandidateInit>,
    remote_description_set: bool,
}

impl WebRTCManager {
    pub fn new(device_id: String, command_tx: mpsc::Sender<InternalCommand>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            command_tx,
            device_id,
        }
    }
    
    pub async fn create_peer_connection(&self, target_device: &str) -> Result<Arc<RTCPeerConnection>> {
        let mut connections = self.connections.lock().await;
        
        // Return existing connection if already created
        if let Some(state) = connections.get(target_device) {
            return Ok(state.peer_connection.clone());
        }
        
        info!("Creating new peer connection for device: {}", target_device);
        
        // Create new peer connection
        let peer_connection = Arc::new(WEBRTC_API.new_peer_connection(WEBRTC_CONFIG.clone()).await?);
        
        // Set up connection state change handler
        let app_state = self.command_tx.clone();
        let target = target_device.to_string();
        peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let tx = app_state.clone();
            let device = target.clone();
            Box::pin(async move {
                info!("Peer connection state changed to {} for {}", s, device);
                
                let state = match s {
                    RTCPeerConnectionState::Connected => WebRTCConnectionState::Connected,
                    RTCPeerConnectionState::Connecting => WebRTCConnectionState::Connecting,
                    RTCPeerConnectionState::Disconnected => WebRTCConnectionState::Disconnected,
                    RTCPeerConnectionState::Failed => WebRTCConnectionState::Failed,
                    _ => WebRTCConnectionState::Disconnected,
                };
                
                let _ = tx.send(InternalCommand::UpdateWebRTCState {
                    device_id: device.clone(),
                    state,
                }).await;
                
                // If connected, check if mesh is ready
                if matches!(s, RTCPeerConnectionState::Connected) {
                    let _ = tx.send(InternalCommand::CheckAndTriggerDkg).await;
                }
            })
        }));
        
        // Set up ICE candidate handler
        let target = target_device.to_string();
        let tx = self.command_tx.clone();
        peer_connection.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
            let target = target.clone();
            let tx = tx.clone();
            Box::pin(async move {
                if let Some(candidate) = candidate {
                    debug!("Local ICE candidate: {}", candidate.to_json().await.unwrap());
                    
                    let _ = tx.send(InternalCommand::SendWebSocketRelay {
                        target_device: target,
                        message: serde_json::json!({
                            "type": "IceCandidate",
                            "candidate": candidate.to_json().await.unwrap().candidate
                        }),
                    }).await;
                }
            })
        }));
        
        // Store the connection
        connections.insert(target_device.to_string(), PeerConnectionState {
            peer_connection: peer_connection.clone(),
            data_channel: None,
            ice_candidates_queue: Vec::new(),
            remote_description_set: false,
        });
        
        Ok(peer_connection)
    }
    
    pub async fn handle_webrtc_signal(
        &self,
        from_device: String,
        signal: WebRTCSignal,
        app_state: SharedAppState,
    ) -> Result<()> {
        match signal {
            WebRTCSignal::Offer { sdp } => {
                self.handle_offer(from_device, sdp, app_state).await?;
            }
            WebRTCSignal::Answer { sdp } => {
                self.handle_answer(from_device, sdp).await?;
            }
            WebRTCSignal::IceCandidate { candidate } => {
                self.handle_ice_candidate(from_device, candidate).await?;
            }
        }
        Ok(())
    }
    
    async fn handle_offer(&self, from_device: String, sdp: String, app_state: SharedAppState) -> Result<()> {
        info!("Handling WebRTC offer from {}", from_device);
        
        let peer_connection = self.create_peer_connection(&from_device).await?;
        
        // Set remote description
        let offer = RTCSessionDescription::offer(sdp)?;
        peer_connection.set_remote_description(offer).await?;
        
        // Mark remote description as set
        {
            let mut connections = self.connections.lock().await;
            if let Some(state) = connections.get_mut(&from_device) {
                state.remote_description_set = true;
                
                // Process queued ICE candidates
                for candidate in state.ice_candidates_queue.drain(..) {
                    peer_connection.add_ice_candidate(candidate).await?;
                }
            }
        }
        
        // Create answer
        let answer = peer_connection.create_answer(None).await?;
        peer_connection.set_local_description(answer.clone()).await?;
        
        // Send answer back
        self.command_tx.send(InternalCommand::SendWebSocketRelay {
            target_device: from_device.clone(),
            message: serde_json::json!({
                "type": "Answer",
                "sdp": answer.sdp
            }),
        }).await?;
        
        // Set up data channel handler for incoming channels
        let from = from_device.clone();
        let tx = self.command_tx.clone();
        let connections = self.connections.clone();
        peer_connection.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
            let from = from.clone();
            let tx = tx.clone();
            let connections = connections.clone();
            Box::pin(async move {
                info!("Data channel '{}' created by {}", dc.label(), from);
                
                // Store data channel
                {
                    let mut conns = connections.lock().await;
                    if let Some(state) = conns.get_mut(&from) {
                        state.data_channel = Some(dc.clone());
                    }
                }
                
                // Set up message handler
                let from_device = from.clone();
                let tx = tx.clone();
                dc.on_message(Box::new(move |msg: DataChannelMessage| {
                    let from = from_device.clone();
                    let tx = tx.clone();
                    Box::pin(async move {
                        if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                            if let Ok(message) = serde_json::from_str::<DirectMessage>(&text) {
                                let _ = tx.send(InternalCommand::ProcessDirectMessage {
                                    from_device: from,
                                    message,
                                }).await;
                            }
                        }
                    })
                }));
            })
        }));
        
        Ok(())
    }
    
    async fn handle_answer(&self, from_device: String, sdp: String) -> Result<()> {
        info!("Handling WebRTC answer from {}", from_device);
        
        let connections = self.connections.lock().await;
        if let Some(state) = connections.get(&from_device) {
            let answer = RTCSessionDescription::answer(sdp)?;
            state.peer_connection.set_remote_description(answer).await?;
            
            drop(connections);
            
            // Mark remote description as set and process queued candidates
            let mut connections = self.connections.lock().await;
            if let Some(state) = connections.get_mut(&from_device) {
                state.remote_description_set = true;
                
                for candidate in state.ice_candidates_queue.drain(..) {
                    state.peer_connection.add_ice_candidate(candidate).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_ice_candidate(&self, from_device: String, candidate: String) -> Result<()> {
        debug!("Handling ICE candidate from {}", from_device);
        
        let candidate_init = RTCIceCandidateInit {
            candidate,
            ..Default::default()
        };
        
        let mut connections = self.connections.lock().await;
        if let Some(state) = connections.get_mut(&from_device) {
            if state.remote_description_set {
                state.peer_connection.add_ice_candidate(candidate_init).await?;
            } else {
                // Queue candidate if remote description not yet set
                state.ice_candidates_queue.push(candidate_init);
            }
        }
        
        Ok(())
    }
    
    pub async fn create_data_channel(&self, target_device: &str) -> Result<()> {
        let peer_connection = self.create_peer_connection(target_device).await?;
        
        // Create data channel
        let dc = peer_connection.create_data_channel("frost-dkg", None).await?;
        
        // Set up message handler
        let from_device = target_device.to_string();
        let tx = self.command_tx.clone();
        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            let from = from_device.clone();
            let tx = tx.clone();
            Box::pin(async move {
                if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                    if let Ok(message) = serde_json::from_str::<DirectMessage>(&text) {
                        let _ = tx.send(InternalCommand::ProcessDirectMessage {
                            from_device: from,
                            message,
                        }).await;
                    }
                }
            })
        }));
        
        // Store data channel
        let mut connections = self.connections.lock().await;
        if let Some(state) = connections.get_mut(target_device) {
            state.data_channel = Some(dc);
        }
        
        Ok(())
    }
    
    pub async fn send_direct_message(&self, target_device: &str, message: DirectMessage) -> Result<()> {
        let connections = self.connections.lock().await;
        
        if let Some(state) = connections.get(target_device) {
            if let Some(dc) = &state.data_channel {
                if dc.ready_state() == webrtc::data_channel::data_channel_state::RTCDataChannelState::Open {
                    let message_str = serde_json::to_string(&message)?;
                    dc.send_text(message_str).await?;
                } else {
                    warn!("Data channel not open for {}", target_device);
                }
            } else {
                warn!("No data channel for {}", target_device);
            }
        } else {
            warn!("No connection to {}", target_device);
        }
        
        Ok(())
    }
    
    pub async fn initiate_offer(&self, target_device: &str) -> Result<()> {
        // Only create offer if we have smaller device ID (polite peer pattern)
        if self.device_id < target_device {
            info!("Creating offer for {}", target_device);
            
            let peer_connection = self.create_peer_connection(target_device).await?;
            
            // Create data channel before creating offer
            self.create_data_channel(target_device).await?;
            
            // Create offer
            let offer = peer_connection.create_offer(None).await?;
            peer_connection.set_local_description(offer.clone()).await?;
            
            // Send offer
            self.command_tx.send(InternalCommand::SendWebSocketRelay {
                target_device: target_device.to_string(),
                message: serde_json::json!({
                    "type": "Offer",
                    "sdp": offer.sdp
                }),
            }).await?;
        }
        
        Ok(())
    }
}

// Helper function to create WebRTC manager
pub fn create_webrtc_manager(
    device_id: String,
    command_tx: mpsc::Sender<InternalCommand>,
) -> Arc<Mutex<WebRTCManager>> {
    Arc::new(Mutex::new(WebRTCManager::new(device_id, command_tx)))
}