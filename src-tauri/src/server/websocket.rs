use crate::server::servers::{get_server_by_id, get_server_token};
use futures_util::{SinkExt, StreamExt};
use http::{HeaderMap, HeaderName, HeaderValue};
use std::sync::Arc;
use tauri::Emitter;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::handshake::client::generate_key;

#[derive(Default)]
pub struct WebSocketManager {
    ws_connection: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    cancel_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

// Function to convert the HTTP endpoint to WebSocket endpoint
fn convert_to_websocket(endpoint: &str) -> Result<String, String> {
    let url = url::Url::parse(endpoint).map_err(|e| format!("Invalid URL: {}", e))?;

    // Determine WebSocket protocol based on the scheme
    let ws_protocol = if url.scheme() == "https" { "wss://" } else { "ws://" };

    // Extract host and port (if present)
    let host = url.host_str().ok_or_else(|| "No host found in URL")?;
    let port = url.port_or_known_default().unwrap_or(if url.scheme() == "https" { 443 } else { 80 });

    // Build WebSocket URL, include the port if not the default
    let ws_endpoint = if port == 80 || port == 443 {
        format!("{}{}{}", ws_protocol, host, "/ws")
    } else {
        format!("{}{}:{}/ws", ws_protocol, host, port)
    };

    Ok(ws_endpoint)
}

// Function to build a HeaderMap from a vector of key-value pairs
fn build_header_map(headers: Vec<(String, String)>) -> Result<HeaderMap, String> {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        let header_name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| format!("Invalid header name: {}", e))?;
        let header_value = HeaderValue::from_str(&value)
            .map_err(|e| format!("Invalid header value: {}", e))?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}

#[tauri::command]
pub async fn connect_to_server(
    id: String,
    state: tauri::State<'_, WebSocketManager>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    dbg!("Connecting to server",id.as_str());

    // Disconnect any existing connection first
    disconnect(state.clone()).await?;

    dbg!("Disconnected from previous server",id.as_str());

    // Retrieve server details
    let server = get_server_by_id(id.as_str())
        .ok_or_else(|| format!("Server with ID {} not found", id))?;
    let mut endpoint = convert_to_websocket(server.endpoint.as_str())?;

    dbg!("Server endpoint",endpoint.as_str());

    // Retrieve the token for the server (token is optional)
    let token = get_server_token(id.as_str()).map(|t| t.access_token.clone());

    dbg!("Server token",token.as_ref().unwrap_or(&"".to_string()).as_str());

    // Create the WebSocket request
    let mut request = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(
        &endpoint
    ).map_err(|e| format!("Failed to create WebSocket request: {}", e))?;

    dbg!("WebSocket request");

    // Add necessary headers
    request.headers_mut().insert("Connection", "Upgrade".parse().unwrap());
    request.headers_mut().insert("Upgrade", "websocket".parse().unwrap());
    request.headers_mut().insert("Sec-WebSocket-Version", "13".parse().unwrap());
    request.headers_mut().insert(
        "Sec-WebSocket-Key",
        generate_key().parse().unwrap(),
    );

    dbg!("WebSocket headers",request.headers().iter().map(|(k, v)| format!("{}: {}", k.as_str(), v.to_str().unwrap())).collect::<Vec<String>>().join("\n"));

    // If a token exists, add it to the headers
    if let Some(token) = token {
        request.headers_mut().insert("X-API-TOKEN", token.parse().unwrap());
    }

    dbg!("WebSocket headers with token",request.headers().iter().map(|(k, v)| format!("{}: {}", k.as_str(), v.to_str().unwrap())).collect::<Vec<String>>().join("\n"));

    // Establish the WebSocket connection
    dbg!(&request);
    let (mut ws_remote, _) = connect_async(request).await
        .map_err(|e| {
            dbg!("WebSocket connection error",&e);
            match e {
                Error::ConnectionClosed => "WebSocket connection was closed".to_string(),
                Error::Protocol(protocol_error) => format!("Protocol error: {}", protocol_error),
                Error::Utf8 => "UTF-8 error in WebSocket data".to_string(),
                _ => format!("Unknown error: {:?}", e),
            }
        })?;

    dbg!("Connected to server1234",id.as_str());

    // Create cancellation channel
    let (cancel_tx, mut cancel_rx) = mpsc::channel(1);

    dbg!("try to lock Connected to server",id.as_str());

    // Store connection and cancellation sender
    *state.ws_connection.lock().await = Some(ws_remote);
    *state.cancel_tx.lock().await = Some(cancel_tx);

    dbg!("locked Connected to server",id.as_str());

    // Spawn listener task with cancellation
    let app_handle_clone = app_handle.clone();
    let connection_clone = state.ws_connection.clone();
    tokio::spawn(async move {
        let mut connection = connection_clone.lock().await;
        if let Some(ws) = connection.as_mut() {
            loop {
                dbg!("try to select Connected to server",id.as_str());

                tokio::select! {
                    msg = ws.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                let _ = app_handle_clone.emit("ws-message", text);
                            },
                            Some(Err(_)) | None => break,
                            _ => continue,
                        }
                    }
                    _ = cancel_rx.recv() => {
                        break;
                    }
                }
            }
        }
    });

    dbg!("END Connected to server");

    Ok(())
}

#[tauri::command]
pub async fn disconnect(state: tauri::State<'_, WebSocketManager>) -> Result<(), String> {
    // Send cancellation signal
    if let Some(cancel_tx) = state.cancel_tx.lock().await.take() {
        let _ = cancel_tx.send(()).await;
    }

    // Close connection
    let mut connection = state.ws_connection.lock().await;
    if let Some(mut ws) = connection.take() {
        let _ = ws.close(None).await;
    }

    Ok(())
}