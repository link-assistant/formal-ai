//! Local WebSocket and WebRTC adapters for the transport-neutral API router.
//!
//! Both adapters carry the same JSON request/response envelope. The envelope
//! preserves the HTTP method, path, headers, status, content type, and body, so
//! all OpenAI-compatible routing, permission checks, and shared-memory writes
//! remain owned by [`crate::server::handle_api_request_with_headers`]. WebRTC
//! uses only host ICE candidates and an application-owned loopback signaling
//! socket; no STUN, TURN, or relay service is configured.

use std::collections::BTreeMap;
use std::error::Error;
use std::future::Future;
use std::io::{Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs as _};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use async_trait::async_trait;
use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tungstenite::Message;
use webrtc::data_channel::{DataChannel, DataChannelEvent};
use webrtc::peer_connection::{
    register_default_interceptors, MediaEngine, PeerConnection, PeerConnectionBuilder,
    PeerConnectionEventHandler, RTCConfigurationBuilder, RTCIceGatheringState,
    RTCPeerConnectionState, RTCSessionDescription, Registry,
};
use webrtc::runtime::{channel, default_runtime, timeout, Runtime, Sender};

use crate::server::handle_api_request_with_headers;

const DATA_CHANNEL_LABEL: &str = "formal-ai";
const BEARER_SCHEME: &str = "Bearer";
const MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
const RTC_CHUNK_BYTES: usize = 12 * 1024;
const SIGNAL_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECTION_TIMEOUT: Duration = Duration::from_mins(1);

/// Error returned while opening a local transport or exchanging one envelope.
pub type TransportError = Box<dyn Error + Send + Sync>;

/// A transport-neutral projection of one HTTP/OpenAI-compatible API request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRequest {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: String,
}

impl TransportRequest {
    /// Build a request with no headers. Headers can be inserted into [`Self::headers`].
    #[must_use]
    pub fn new(
        method: impl Into<String>,
        path: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            headers: BTreeMap::new(),
            body: body.into(),
        }
    }
}

/// A transport-neutral projection of the API router's complete response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportResponse {
    pub status_code: u16,
    pub content_type: String,
    pub body: String,
    #[serde(default)]
    pub deprecated: bool,
}

/// Construct the exact request envelope used by the local CLI chat client.
#[must_use]
pub fn openai_chat_request(prompt: &str, bearer_token: Option<&str>) -> TransportRequest {
    let mut request = TransportRequest::new(
        "POST",
        "/v1/chat/completions",
        json!({
            "model": crate::DEFAULT_MODEL,
            "messages": [{"role": "user", "content": prompt}],
            "stream": false,
        })
        .to_string(),
    );
    request.headers.insert(
        String::from("content-type"),
        String::from("application/json"),
    );
    if let Some(token) = bearer_token {
        request.headers.insert(
            String::from("authorization"),
            [BEARER_SCHEME, token].join(" "),
        );
    }
    request
}

/// Route one transport request through the same handler used by HTTP.
#[must_use]
pub fn dispatch_request(request: &TransportRequest) -> TransportResponse {
    let headers = request
        .headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    let response =
        handle_api_request_with_headers(&request.method, &request.path, &headers, &request.body);
    TransportResponse {
        status_code: response.status_code,
        content_type: response.content_type.to_owned(),
        body: response.body,
        deprecated: response.deprecated,
    }
}

/// Serve transport envelopes over RFC 6455 WebSocket text messages.
pub fn serve_websocket(address: &str, trace: bool) -> Result<(), TransportError> {
    let listener = TcpListener::bind(address)?;
    announce_server("websocket", address);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    if let Err(error) = handle_websocket_connection(stream, trace) {
                        trace_error(trace, "websocket", &error);
                    }
                });
            }
            Err(error) => trace_error(trace, "websocket accept", &error),
        }
    }
    Ok(())
}

/// Send one request envelope to a WebSocket server and receive its response.
pub fn websocket_request(
    endpoint: &str,
    request: &TransportRequest,
) -> Result<TransportResponse, TransportError> {
    let endpoint = websocket_endpoint(endpoint);
    let (mut socket, _) = tungstenite::connect(endpoint.as_str())?;
    socket.send(Message::Text(serde_json::to_string(request)?.into()))?;
    loop {
        match socket.read()? {
            Message::Text(text) => return Ok(serde_json::from_str(&text)?),
            Message::Binary(bytes) => return Ok(serde_json::from_slice(&bytes)?),
            Message::Close(_) => return Err(transport_error("WebSocket closed before a response")),
            Message::Ping(bytes) => socket.send(Message::Pong(bytes))?,
            Message::Pong(_) | Message::Frame(_) => {}
        }
    }
}

fn handle_websocket_connection(stream: TcpStream, trace: bool) -> Result<(), TransportError> {
    let peer = stream.peer_addr().ok();
    let mut socket = tungstenite::accept(stream)?;
    trace_event(trace, "websocket", peer, "open");
    loop {
        let message = match socket.read() {
            Ok(message) => message,
            Err(tungstenite::Error::ConnectionClosed) => break,
            Err(error) => return Err(error.into()),
        };
        let request = match message {
            Message::Text(text) => serde_json::from_str(&text),
            Message::Binary(bytes) => serde_json::from_slice(&bytes),
            Message::Close(_) => break,
            Message::Ping(bytes) => {
                socket.send(Message::Pong(bytes))?;
                continue;
            }
            Message::Pong(_) | Message::Frame(_) => continue,
        };
        let response = match request {
            Ok(request) => dispatch_request(&request),
            Err(error) => invalid_envelope(&error.to_string()),
        };
        socket.send(Message::Text(serde_json::to_string(&response)?.into()))?;
    }
    trace_event(trace, "websocket", peer, "close");
    Ok(())
}

/// Serve local-first WebRTC data channels with loopback TCP offer/answer signaling.
pub fn serve_webrtc(address: &str, trace: bool) -> Result<(), TransportError> {
    let listener = TcpListener::bind(address)?;
    announce_server("webrtc", address);
    let udp_address = loopback_udp_address(listener.local_addr()?);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let udp_address = udp_address.clone();
                std::thread::spawn(move || {
                    if let Err(error) =
                        handle_webrtc_connection(stream, udp_address.as_str(), trace)
                    {
                        trace_error(trace, "webrtc", &error);
                    }
                });
            }
            Err(error) => trace_error(trace, "webrtc signaling accept", &error),
        }
    }
    Ok(())
}

/// Complete local signaling and exchange one request/response over a WebRTC data channel.
pub fn webrtc_request(
    endpoint: &str,
    request: &TransportRequest,
) -> Result<TransportResponse, TransportError> {
    let address = resolve_endpoint(endpoint)?;
    let mut signaling = TcpStream::connect_timeout(&address, SIGNAL_TIMEOUT)?;
    signaling.set_read_timeout(Some(SIGNAL_TIMEOUT))?;
    signaling.set_write_timeout(Some(SIGNAL_TIMEOUT))?;
    let request_bytes = serde_json::to_vec(request)?;
    if request_bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(transport_error("WebRTC request exceeds the 16 MiB limit"));
    }

    block_on(async move {
        let runtime = rtc_runtime();
        let (gather_tx, mut gather_rx) = channel(1);
        let (done_tx, _done_rx) = channel(1);
        let handler = Arc::new(ClientPeerHandler {
            gather_complete_tx: gather_tx,
            done_tx,
        });
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;
        let registry = register_default_interceptors(Registry::new(), &mut media_engine)?;
        let peer = Box::pin(
            PeerConnectionBuilder::new()
                .with_configuration(RTCConfigurationBuilder::new().build())
                .with_media_engine(media_engine)
                .with_interceptor_registry(registry)
                .with_handler(handler)
                .with_runtime(runtime.clone())
                .with_udp_addrs(vec![String::from("127.0.0.1:0")])
                .build(),
        )
        .await?;
        let data_channel = peer.create_data_channel(DATA_CHANNEL_LABEL, None).await?;

        let offer = peer.create_offer(None).await?;
        peer.set_local_description(offer).await?;
        wait_for_signal(&runtime, &mut gather_rx, "client ICE gathering").await?;
        let local = peer
            .local_description()
            .await
            .ok_or_else(|| transport_error("WebRTC client has no local description"))?;
        write_signal_frame(&mut signaling, &local)?;
        let answer: RTCSessionDescription = read_signal_frame(&mut signaling)?;
        peer.set_remote_description(answer).await?;

        let exchange = async {
            let mut sent = false;
            let mut response_bytes = Vec::new();
            while let Some(event) = data_channel.poll().await {
                match event {
                    DataChannelEvent::OnOpen if !sent => {
                        send_data_payload(&data_channel, &request_bytes).await?;
                        sent = true;
                    }
                    DataChannelEvent::OnMessage(message) => {
                        if append_data_chunk(&mut response_bytes, &message.data)? {
                            return Ok(serde_json::from_slice(&response_bytes)?);
                        }
                    }
                    DataChannelEvent::OnError => {
                        return Err(transport_error("WebRTC data channel reported an error"));
                    }
                    DataChannelEvent::OnClose => {
                        return Err(transport_error(
                            "WebRTC data channel closed before a response",
                        ));
                    }
                    _ => {}
                }
            }
            Err(transport_error(
                "WebRTC data channel ended before a response",
            ))
        };
        let response = timeout(&*runtime, CONNECTION_TIMEOUT, exchange)
            .await
            .map_err(|_| transport_error("timed out waiting for WebRTC response"))??;
        let _ = data_channel.close().await;
        peer.close().await?;
        Ok(response)
    })
}

fn handle_webrtc_connection(
    mut signaling: TcpStream,
    udp_address: &str,
    trace: bool,
) -> Result<(), TransportError> {
    signaling.set_read_timeout(Some(SIGNAL_TIMEOUT))?;
    signaling.set_write_timeout(Some(SIGNAL_TIMEOUT))?;
    let peer_address = signaling.peer_addr().ok();
    let offer: RTCSessionDescription = read_signal_frame(&mut signaling)?;
    trace_event(trace, "webrtc", peer_address, "offer");

    block_on(async move {
        let runtime = rtc_runtime();
        let (gather_tx, mut gather_rx) = channel(1);
        let (done_tx, mut done_rx) = channel(1);
        let handler = Arc::new(ServerPeerHandler {
            runtime: runtime.clone(),
            gather_complete_tx: gather_tx,
            done_tx,
            trace,
        });
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;
        let registry = register_default_interceptors(Registry::new(), &mut media_engine)?;
        let peer = Box::pin(
            PeerConnectionBuilder::new()
                .with_configuration(RTCConfigurationBuilder::new().build())
                .with_media_engine(media_engine)
                .with_interceptor_registry(registry)
                .with_handler(handler)
                .with_runtime(runtime.clone())
                .with_udp_addrs(vec![udp_address.to_owned()])
                .build(),
        )
        .await?;
        peer.set_remote_description(offer).await?;
        let answer = peer.create_answer(None).await?;
        peer.set_local_description(answer).await?;
        wait_for_signal(&runtime, &mut gather_rx, "server ICE gathering").await?;
        let local = peer
            .local_description()
            .await
            .ok_or_else(|| transport_error("WebRTC server has no local description"))?;
        write_signal_frame(&mut signaling, &local)?;
        trace_event(trace, "webrtc", peer_address, "answer");
        let _ = timeout(&*runtime, CONNECTION_TIMEOUT, done_rx.recv()).await;
        peer.close().await?;
        trace_event(trace, "webrtc", peer_address, "close");
        Ok(())
    })
}

#[derive(Clone)]
struct ClientPeerHandler {
    gather_complete_tx: Sender<()>,
    done_tx: Sender<()>,
}

#[async_trait]
impl PeerConnectionEventHandler for ClientPeerHandler {
    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            let _ = self.gather_complete_tx.try_send(());
        }
    }

    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        if matches!(
            state,
            RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed
        ) {
            let _ = self.done_tx.try_send(());
        }
    }
}

#[derive(Clone)]
struct ServerPeerHandler {
    runtime: Arc<dyn Runtime>,
    gather_complete_tx: Sender<()>,
    done_tx: Sender<()>,
    trace: bool,
}

#[async_trait]
impl PeerConnectionEventHandler for ServerPeerHandler {
    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            let _ = self.gather_complete_tx.try_send(());
        }
    }

    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        if self.trace {
            eprintln!("[local-transport] webrtc peer state={state}");
        }
        if matches!(
            state,
            RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed
        ) {
            let _ = self.done_tx.try_send(());
        }
    }

    async fn on_data_channel(&self, data_channel: Arc<dyn DataChannel>) {
        let done_tx = self.done_tx.clone();
        let trace = self.trace;
        self.runtime.spawn(Box::pin(async move {
            let mut request_bytes = Vec::new();
            while let Some(event) = data_channel.poll().await {
                match event {
                    DataChannelEvent::OnOpen => {
                        if trace {
                            eprintln!("[local-transport] webrtc data-channel open");
                        }
                    }
                    DataChannelEvent::OnMessage(message) => {
                        let response = match append_data_chunk(&mut request_bytes, &message.data) {
                            Ok(true) => {
                                let response = match serde_json::from_slice(&request_bytes) {
                                    Ok(request) => dispatch_request(&request),
                                    Err(error) => invalid_envelope(&error.to_string()),
                                };
                                request_bytes.clear();
                                Some(response)
                            }
                            Ok(false) => None,
                            Err(error) => {
                                request_bytes.clear();
                                Some(invalid_envelope(&error.to_string()))
                            }
                        };
                        if let Some(response) = response {
                            match serde_json::to_vec(&response) {
                                Ok(bytes) => {
                                    if let Err(error) =
                                        send_data_payload(&data_channel, &bytes).await
                                    {
                                        trace_error(trace, "webrtc response", &error);
                                        break;
                                    }
                                }
                                Err(error) => {
                                    trace_error(trace, "webrtc response encoding", &error);
                                    break;
                                }
                            }
                        }
                    }
                    DataChannelEvent::OnClose | DataChannelEvent::OnError => break,
                    _ => {}
                }
            }
            let _ = done_tx.try_send(());
        }));
    }
}

async fn wait_for_signal(
    runtime: &Arc<dyn Runtime>,
    receiver: &mut webrtc::runtime::Receiver<()>,
    phase: &str,
) -> Result<(), TransportError> {
    timeout(&**runtime, SIGNAL_TIMEOUT, receiver.recv())
        .await
        .map_err(|_| transport_error(format!("local_transport_timeout:{phase}")))?;
    Ok(())
}

async fn send_data_payload(
    data_channel: &Arc<dyn DataChannel>,
    payload: &[u8],
) -> webrtc::error::Result<()> {
    let chunks = payload.chunks(RTC_CHUNK_BYTES);
    let chunk_count = chunks.len();
    for (index, chunk) in chunks.enumerate() {
        let mut frame = Vec::with_capacity(chunk.len() + 1);
        frame.push(u8::from(index + 1 == chunk_count));
        frame.extend_from_slice(chunk);
        data_channel.send(BytesMut::from(frame.as_slice())).await?;
    }
    Ok(())
}

fn append_data_chunk(buffer: &mut Vec<u8>, frame: &[u8]) -> Result<bool, TransportError> {
    let Some((&final_marker, payload)) = frame.split_first() else {
        return Err(transport_error("empty WebRTC transport frame"));
    };
    if final_marker > 1 {
        return Err(transport_error("invalid WebRTC transport frame marker"));
    }
    if buffer.len().saturating_add(payload.len()) > MAX_PAYLOAD_BYTES {
        return Err(transport_error("WebRTC payload exceeds the 16 MiB limit"));
    }
    buffer.extend_from_slice(payload);
    Ok(final_marker == 1)
}

fn write_signal_frame<T: Serialize>(
    stream: &mut TcpStream,
    value: &T,
) -> Result<(), TransportError> {
    let payload = serde_json::to_vec(value)?;
    let length = u32::try_from(payload.len())?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

fn read_signal_frame<T: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
) -> Result<T, TransportError> {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_PAYLOAD_BYTES {
        return Err(transport_error(
            "WebRTC signaling frame exceeds the 16 MiB limit",
        ));
    }
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload)?;
    Ok(serde_json::from_slice(&payload)?)
}

fn rtc_runtime() -> Arc<dyn Runtime> {
    static RUNTIME: OnceLock<Arc<dyn Runtime>> = OnceLock::new();
    Arc::clone(
        RUNTIME
            .get_or_init(|| default_runtime().expect("webrtc was built without a default runtime")),
    )
}

fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    let mut output = None;
    {
        let slot = &mut output;
        rtc_runtime().block_on(Box::pin(async move {
            *slot = Some(future.await);
        }));
    }
    output.expect("WebRTC runtime did not drive the future")
}

fn announce_server(transport: &str, address: &str) {
    crate::dreaming_runtime::start_core_dreaming();
    eprintln!(
        "formal-ai shared memory: {}",
        crate::shared_memory::shared_memory_path().display()
    );
    eprintln!("formal-ai server listening on {transport}://{address}");
}

fn websocket_endpoint(endpoint: &str) -> String {
    if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
        endpoint.to_owned()
    } else {
        format!("ws://{endpoint}")
    }
}

fn resolve_endpoint(endpoint: &str) -> Result<SocketAddr, TransportError> {
    endpoint
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| transport_error(format!("local_transport_resolve:{endpoint}")))
}

fn loopback_udp_address(signaling_address: SocketAddr) -> String {
    match signaling_address.ip() {
        std::net::IpAddr::V4(address) if address.is_unspecified() => String::from("0.0.0.0:0"),
        std::net::IpAddr::V6(address) if address.is_unspecified() => String::from("[::]:0"),
        std::net::IpAddr::V4(address) => format!("{address}:0"),
        std::net::IpAddr::V6(address) => format!("[{address}]:0"),
    }
}

fn invalid_envelope(detail: &str) -> TransportResponse {
    TransportResponse {
        status_code: 400,
        content_type: String::from("application/json"),
        body: json!({
            "error": {
                "message": format!("invalid_transport_envelope:{detail}"),
                "type": "invalid_request_error",
            }
        })
        .to_string(),
        deprecated: false,
    }
}

fn transport_error(message: impl Into<String>) -> TransportError {
    std::io::Error::other(message.into()).into()
}

fn trace_event(trace: bool, transport: &str, peer: Option<SocketAddr>, event: &str) {
    if trace {
        eprintln!("[local-transport] {transport} peer={peer:?} event={event}");
    }
}

fn trace_error(trace: bool, phase: &str, error: &dyn std::fmt::Display) {
    if trace {
        eprintln!("[local-transport] {phase} failed: {error}");
    }
}
