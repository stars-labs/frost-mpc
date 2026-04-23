use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc; // Add this import for signal handling

use tokio_tungstenite::{accept_async, tungstenite::Message};

// Import shared types from the library crate

use webrtc_signal_server::{ClientMsg, ServerMsg};

type DeviceSender = mpsc::UnboundedSender<Message>;
type DeviceMap = Arc<Mutex<HashMap<String, DeviceSender>>>;

// KISS: Store minimal session info - just the announcement
#[derive(Clone)]
struct StoredSession {
    session_info: serde_json::Value,  // The full announcement as-is
    active_participants: Vec<String>,  // Currently online participants
    last_active: std::time::Instant,  // Updated when participants leave; used for grace period
}

type SessionMap = Arc<Mutex<HashMap<String, StoredSession>>>;
// Map device_id to list of session_ids they're participating in
type DeviceSessionsMap = Arc<Mutex<HashMap<String, Vec<String>>>>;

#[tokio::main]
async fn main() {
    let devices: DeviceMap = Arc::new(Mutex::new(HashMap::new()));
    let sessions: SessionMap = Arc::new(Mutex::new(HashMap::new()));
    let device_sessions: DeviceSessionsMap = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();
    println!("Signal server listening on 0.0.0.0:9000");
    
    // Periodic cleanup: expire sessions that have had no active participants for >5 minutes
    let sessions_cleanup = sessions.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let mut guard = sessions_cleanup.lock().unwrap();
            guard.retain(|id, session| {
                let age = session.last_active.elapsed();
                let keep = !session.active_participants.is_empty()
                    || age < std::time::Duration::from_secs(300);
                if !keep {
                    println!("🗑️ Expiring session '{}' (no active participants for {:?})", id, age);
                }
                keep
            });
        }
    });

    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        println!("Shutdown signal received. Terminating...");
    };

    let server = async {
        while let Ok((stream, _)) = listener.accept().await {
            let devices = devices.clone();
            let sessions = sessions.clone();
            let device_sessions = device_sessions.clone();

            tokio::spawn(async move {
                // Handle WebSocket handshake errors gracefully
                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        // This is likely a connection test or non-WebSocket connection
                        eprintln!("WebSocket handshake failed (this is normal for connection tests): {:?}", e);
                        return;
                    }
                };
                let (mut ws_sink, mut ws_stream) = ws_stream.split();
                let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
                let mut device_id: Option<String> = None;

                // Task to forward messages from rx to ws_sink
                let ws_sink_task = tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        if ws_sink.send(msg).await.is_err() {
                            break;
                        }
                    }
                });

                loop {
                    tokio::select! {
                        Some(msg) = ws_stream.next() => {
                            let msg = match msg {
                                Ok(m) if m.is_ping() => {
                                    let _ = tx.send(Message::Pong(m.into_data()));
                                    continue;
                                }
                                Ok(m) if m.is_text() => m.into_text().unwrap(),
                                Ok(m) if m.is_close() => break,
                                _ => continue,
                            };

                            let parsed: Result<ClientMsg, _> = serde_json::from_str(&msg);

                            match parsed {
                                Ok(ClientMsg::Register { device_id: reg_id }) => {
                                    let mut devices_guard = devices.lock().unwrap();
                                    if devices_guard.contains_key(&reg_id) {
                                        let err = ServerMsg::Error { error: "device_id already registered".to_string() };
                                        let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                                        break;
                                    }
                                    device_id = Some(reg_id.clone());
                                    devices_guard.insert(reg_id.clone(), tx.clone());
                                    println!("Registered device: {}", reg_id);

                                    // Broadcast updated device list to all devices (owned Vec)
                                    let device_list: Vec<String> = devices_guard.keys().cloned().collect();
                                    let msg = ServerMsg::Devices { devices: device_list.clone() };
                                    let msg_txt = serde_json::to_string(&msg).unwrap();
                                    for (_id, ptx) in devices_guard.iter() {
                                        let _ = ptx.send(Message::Text(msg_txt.clone().into()));
                                    }
                                }
                                Ok(ClientMsg::ListDevices) => {
                                    let devices_guard = devices.lock().unwrap();
                                    let device_list: Vec<String> = devices_guard.keys().cloned().collect();
                                    let msg = ServerMsg::Devices { devices: device_list };
                                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                                }
                                Ok(ClientMsg::Relay { to, data }) => {
                                    // Check if this is a SessionProposal to update session participants
                                    if data.get("websocket_msg_type").and_then(|v| v.as_str()) == Some("SessionProposal") {
                                        if let (Some(session_id), Some(participants)) = (
                                            data.get("session_id").and_then(|v| v.as_str()),
                                            data.get("participants").and_then(|v| v.as_array())
                                        ) {
                                            // Update existing session with participant information
                                            let mut sessions_guard = sessions.lock().unwrap();
                                            if let Some(session) = sessions_guard.get_mut(session_id) {
                                                // Update stored session_info to include participants
                                                session.session_info = data.clone();
                                                
                                                // Update active participants based on who's currently connected
                                                session.active_participants.clear();
                                                let devices_guard = devices.lock().unwrap();
                                                for p in participants {
                                                    if let Some(participant_id) = p.as_str() {
                                                        // Check if this device is currently connected
                                                        if devices_guard.contains_key(participant_id) {
                                                            session.active_participants.push(participant_id.to_string());
                                                        }
                                                    }
                                                }
                                                drop(devices_guard);
                                                println!("Updated session '{}' with participants: {:?} (active: {:?})", 
                                                    session_id, participants, session.active_participants);
                                            }
                                            drop(sessions_guard);
                                            
                                            // Update device sessions map for all participants
                                            let mut device_sessions_guard = device_sessions.lock().unwrap();
                                            for p in participants {
                                                if let Some(participant_id) = p.as_str() {
                                                    let entry = device_sessions_guard
                                                        .entry(participant_id.to_string())
                                                        .or_default();
                                                    if !entry.contains(&session_id.to_string()) {
                                                        entry.push(session_id.to_string());
                                                        println!("Added session '{}' to device '{}' session list", session_id, participant_id);
                                                    }
                                                }
                                            }
                                            drop(device_sessions_guard);
                                        }
                                    }
                                    
                                    // Check if this is a SessionUpdate to track active participants  
                                    if data.get("websocket_msg_type").and_then(|v| v.as_str()) == Some("SessionUpdate") {
                                        if let (Some(session_id), Some(accepted_devices)) = (
                                            data.get("session_id").and_then(|v| v.as_str()),
                                            data.get("accepted_devices").and_then(|v| v.as_array())
                                        ) {
                                            // Update session's active participants
                                            let mut sessions_guard = sessions.lock().unwrap();
                                            if let Some(session) = sessions_guard.get_mut(session_id) {
                                                // Update active participants based on who's in the accepted_devices and currently connected
                                                session.active_participants.clear();
                                                let devices_guard = devices.lock().unwrap();
                                                for p in accepted_devices {
                                                    if let Some(participant_id) = p.as_str() {
                                                        // Check if this device is currently connected
                                                        if devices_guard.contains_key(participant_id) {
                                                            session.active_participants.push(participant_id.to_string());
                                                        }
                                                    }
                                                }
                                                drop(devices_guard);
                                                println!("Updated active participants for session '{}': {:?}", 
                                                    session_id, session.active_participants);
                                                
                                                // Update the stored session_info to include accepted_devices
                                                if session.session_info.get("participants").and_then(|v| v.as_array()).is_some() {
                                                    // Only update if we have participants info, otherwise preserve original session_info
                                                    let mut updated_info = session.session_info.clone();
                                                    updated_info.as_object_mut().unwrap().insert("accepted_devices".to_string(), serde_json::Value::Array(accepted_devices.clone()));
                                                    session.session_info = updated_info;
                                                }
                                            }
                                            drop(sessions_guard);
                                            
                                            // Update device sessions map for accepted devices
                                            let mut device_sessions_guard = device_sessions.lock().unwrap();
                                            for p in accepted_devices {
                                                if let Some(participant_id) = p.as_str() {
                                                    let entry = device_sessions_guard
                                                        .entry(participant_id.to_string())
                                                        .or_default();
                                                    if !entry.contains(&session_id.to_string()) {
                                                        entry.push(session_id.to_string());
                                                    }
                                                }
                                            }
                                            drop(device_sessions_guard);
                                        }
                                    }
                                    
                                    let devices_guard = devices.lock().unwrap();
                                    
                                    // Handle broadcast relay to all devices
                                    if to == "*" {
                                        let relay = ServerMsg::Relay {
                                            from: device_id.as_deref().unwrap_or_default().to_string(),
                                            data: data.clone(),
                                        };
                                        let relay_text = serde_json::to_string(&relay).unwrap();
                                        
                                        println!("Broadcasting relay from {} to all devices: {:?}", 
                                            device_id.as_deref().unwrap_or("unknown"), data);
                                        
                                        // Send to all devices except the sender
                                        for (id, device_tx) in devices_guard.iter() {
                                            if Some(id) != device_id.as_ref() {
                                                let _ = device_tx.send(Message::Text(relay_text.clone().into()));
                                            }
                                        }
                                    } else {
                                        // Handle targeted relay to specific device
                                        if let Some(device_tx) = devices_guard.get(&to) {
                                            let relay = ServerMsg::Relay {
                                                from: device_id.as_deref().unwrap_or_default().to_string(),
                                                data: data.clone(), // Clone data for the message
                                            };
                                            // Log the relay action
                                            println!("Relaying message from {} to {}: {:?}", device_id.as_deref().unwrap_or("unknown"), to, data);
                                            let _ = device_tx.send(Message::Text(serde_json::to_string(&relay).unwrap().into()));
                                        } else {
                                            println!("Relay failed: unknown device {}", to);
                                            let err = ServerMsg::Error { error: format!("unknown device: {}", to) };
                                            let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                                        }
                                    }
                                    // Explicitly drop the lock
                                    drop(devices_guard);
                                }
                                Ok(ClientMsg::AnnounceSession { session_info }) => {
                                    // Store the session for later discovery
                                    if let Some(ref device) = device_id {
                                        // Extract session ID from the announcement
                                        // Check for both session_id and session_code for compatibility
                                        let session_key = if let Some(id) = session_info.get("session_id")
                                            .and_then(|v| v.as_str()) {
                                            id.to_string()
                                        } else if let Some(code) = session_info.get("session_code")
                                            .and_then(|v| v.as_str()) {
                                            code.to_string()
                                        } else {
                                            // Fallback to using device_id + timestamp as key
                                            format!("{}-{}", device, SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_millis())
                                        };
                                        
                                        // Store session with creator as first active participant
                                        let stored_session = StoredSession {
                                            session_info: session_info.clone(),
                                            active_participants: vec![device.clone()], // Creator is first participant
                                            last_active: std::time::Instant::now(),
                                        };
                                        
                                        let mut sessions_guard = sessions.lock().unwrap();
                                        sessions_guard.insert(session_key.clone(), stored_session);
                                        drop(sessions_guard);
                                        
                                        // Track that this device is in this session
                                        let mut device_sessions_guard = device_sessions.lock().unwrap();
                                        device_sessions_guard
                                            .entry(device.clone())
                                            .or_default()
                                            .push(session_key.clone());
                                        drop(device_sessions_guard);
                                        
                                        println!("Stored session '{}' from device '{}'", session_key, device);
                                    }
                                    
                                    // Broadcast session announcement to all connected devices
                                    let devices_guard = devices.lock().unwrap();
                                    let msg = ServerMsg::SessionAvailable { session_info };
                                    let msg_txt = serde_json::to_string(&msg).unwrap();
                                    println!("Broadcasting session announcement from {}", device_id.as_deref().unwrap_or("unknown"));
                                    for (id, device_tx) in devices_guard.iter() {
                                        if Some(id) != device_id.as_ref() {  // Don't send back to announcer
                                            let _ = device_tx.send(Message::Text(msg_txt.clone().into()));
                                        }
                                    }
                                    drop(devices_guard);
                                }
                                Ok(ClientMsg::RequestActiveSessions) => {
                                    println!("Session list request from {}", device_id.as_deref().unwrap_or("unknown"));
                                    
                                    // Send all stored sessions to the requester
                                    let sessions_guard = sessions.lock().unwrap();
                                    println!("Found {} active sessions", sessions_guard.len());
                                    
                                    for (session_key, stored_session) in sessions_guard.iter() {
                                        let msg = ServerMsg::SessionAvailable { 
                                            session_info: stored_session.session_info.clone() 
                                        };
                                        let msg_txt = serde_json::to_string(&msg).unwrap();
                                        println!("Sending stored session '{}' to requester", session_key);
                                        let _ = tx.send(Message::Text(msg_txt.into()));
                                    }
                                    drop(sessions_guard);
                                    
                                    // Also broadcast request to get fresh updates from active creators
                                    let devices_guard = devices.lock().unwrap();
                                    let msg = ServerMsg::SessionListRequest {
                                        from: device_id.as_deref().unwrap_or_default().to_string(),
                                    };
                                    let msg_txt = serde_json::to_string(&msg).unwrap();
                                    for (id, device_tx) in devices_guard.iter() {
                                        if Some(id) != device_id.as_ref() {  // Don't send back to requester
                                            let _ = device_tx.send(Message::Text(msg_txt.clone().into()));
                                        }
                                    }
                                    drop(devices_guard);
                                }
                                Ok(ClientMsg::SessionStatusUpdate { session_info }) => {
                                    println!("Session status update from {}: {:?}", device_id.as_deref().unwrap_or("unknown"), session_info);
                                    
                                    // Handle participant joining a session
                                    if let Some(participant_joined) = session_info.get("participant_joined")
                                        .and_then(|v| v.as_str()) {
                                        
                                        if let Some(session_id) = session_info.get("session_id")
                                            .and_then(|v| v.as_str()) {
                                            
                                            // Update the stored session with new participant
                                            let mut sessions_guard = sessions.lock().unwrap();
                                            if let Some(stored_session) = sessions_guard.get_mut(session_id) {
                                                // Add participant to the participants array in session_info
                                                if let Some(participants) = stored_session.session_info
                                                    .get_mut("participants")
                                                    .and_then(|v| v.as_array_mut()) {
                                                    
                                                    // Check if participant isn't already in list
                                                    let already_joined = participants.iter()
                                                        .any(|p| p.as_str() == Some(participant_joined));
                                                    
                                                    if !already_joined {
                                                        participants.push(serde_json::Value::String(participant_joined.to_string()));
                                                        println!("Added {} to session {} participants", participant_joined, session_id);
                                                        
                                                        // Also update active_participants
                                                        if !stored_session.active_participants.contains(&participant_joined.to_string()) {
                                                            stored_session.active_participants.push(participant_joined.to_string());
                                                        }
                                                    }
                                                }
                                                
                                                // Broadcast updated session to all participants
                                                let updated_session_info = stored_session.session_info.clone();
                                                let participant_count = updated_session_info.get("participants")
                                                    .and_then(|v| v.as_array())
                                                    .map(|arr| arr.len())
                                                    .unwrap_or(0);
                                                drop(sessions_guard);
                                                
                                                // Broadcast updated session using Relay to trigger WebRTC
                                                // Send a special relay message that will be recognized as a participant update
                                                let update_msg = serde_json::json!({
                                                    "type": "participant_update",
                                                    "session_id": session_id,
                                                    "session_info": updated_session_info.clone(),
                                                });
                                                
                                                let devices_guard = devices.lock().unwrap();
                                                println!("Broadcasting participant update for session {} with {} participants", 
                                                    session_id, participant_count);
                                                
                                                // Relay to all devices to trigger WebRTC connections
                                                for (id, device_tx) in devices_guard.iter() {
                                                    let relay = ServerMsg::Relay {
                                                        from: "server".to_string(),
                                                        data: update_msg.clone(),
                                                    };
                                                    let msg_txt = serde_json::to_string(&relay).unwrap();
                                                    let _ = device_tx.send(Message::Text(msg_txt.into()));
                                                    println!("Sent participant update to device: {}", id);
                                                }
                                                drop(devices_guard);
                                            } else {
                                                println!("Session {} not found for participant update", session_id);
                                            }
                                        }
                                    }
                                }
                                Ok(ClientMsg::QueryMyActiveSessions) => {
                                    // Client asks "what sessions am I in?"
                                    if let Some(ref dev_id) = device_id {
                                        println!("Device '{}' querying for active sessions", dev_id);
                                        
                                        let mut sessions_guard = sessions.lock().unwrap();
                                        let mut my_sessions = Vec::new();
                                        let mut session_keys_to_track = Vec::new();
                                        
                                        // Check active participants list and update it
                                        for (key, session) in sessions_guard.iter_mut() {
                                            // Check if device is in participants array
                                            if let Some(participants) = session.session_info.get("participants")
                                                .and_then(|v| v.as_array()) {
                                                let is_participant = participants.iter()
                                                    .any(|p| p.as_str() == Some(dev_id.as_str()));
                                                if is_participant {
                                                    // Add to active participants if not already there (rejoin case)
                                                    if !session.active_participants.contains(dev_id) {
                                                        session.active_participants.push(dev_id.clone());
                                                        println!("Added '{}' back to active participants for session '{}'", dev_id, key);
                                                    }
                                                    my_sessions.push(session.session_info.clone());
                                                    session_keys_to_track.push(key.clone());
                                                }
                                            }
                                        }
                                        drop(sessions_guard);
                                        
                                        // Update device sessions map with all sessions this device is in
                                        let mut device_sessions_guard = device_sessions.lock().unwrap();
                                        device_sessions_guard.insert(dev_id.clone(), session_keys_to_track);
                                        drop(device_sessions_guard);
                                        
                                        // Send response with list of sessions
                                        let response = ServerMsg::SessionsForDevice {
                                            sessions: my_sessions.clone(),
                                        };
                                        let msg_txt = serde_json::to_string(&response).unwrap();
                                        println!("Found {} sessions for device '{}'", my_sessions.len(), dev_id);
                                        let _ = tx.send(Message::Text(msg_txt.into()));
                                    }
                                }
                                Err(_) => {
                                    let err = ServerMsg::Error { error: "invalid message".to_string() };
                                    let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                                }
                            }
                        }
                        else => break,
                    }
                }

                // Cleanup on disconnect
                if let Some(my_id) = device_id {
                    // Remove device from active participants in sessions
                    let device_sessions_guard = device_sessions.lock().unwrap();
                    if let Some(session_ids) = device_sessions_guard.get(&my_id) {
                        let mut sessions_guard = sessions.lock().unwrap();

                        for session_id in session_ids {
                            if let Some(session) = sessions_guard.get_mut(session_id) {
                                // Remove from active participants
                                session.active_participants.retain(|p| p != &my_id);
                                println!("Removed '{}' from active participants in session '{}'", my_id, session_id);

                                // Keep session even when all participants disconnect — periodic
                                // cleanup will expire it after a grace period so rejoining works.
                                if session.active_participants.is_empty() {
                                    session.last_active = std::time::Instant::now();
                                    println!("Session '{}' has no active participants, keeping for grace period", session_id);
                                } else {
                                    println!("Session '{}' continues with {} active participants",
                                        session_id, session.active_participants.len());
                                }
                            }
                        }

                        drop(sessions_guard);
                    }
                    drop(device_sessions_guard);
                    
                    // Clean up device sessions map
                    let mut device_sessions_guard = device_sessions.lock().unwrap();
                    device_sessions_guard.remove(&my_id);
                    drop(device_sessions_guard);
                    
                    // Now remove device from active list
                    let mut devices_guard = devices.lock().unwrap();
                    devices_guard.remove(&my_id);
                    println!("Device {} disconnected", my_id);

                    // Broadcast updated device list to all devices (owned Vec)
                    let device_list: Vec<String> = devices_guard.keys().cloned().collect();
                    let msg = ServerMsg::Devices {
                        devices: device_list.clone(),
                    };
                    let msg_txt = serde_json::to_string(&msg).unwrap();
                    for (_id, ptx) in devices_guard.iter() {
                        let _ = ptx.send(Message::Text(msg_txt.clone().into()));
                    }
                }
                ws_sink_task.abort();
            });
        }
    };

    tokio::select! {
        _ = server => {},
        _ = shutdown_signal => {},
    }

    println!("Server has shut down.");
}
