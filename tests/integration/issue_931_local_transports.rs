use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use formal_ai::local_transport::{
    openai_chat_request, webrtc_request, websocket_request, TransportRequest,
};
use serde_json::Value;

use crate::http_server::{http_request, reserve_loopback_port, spawn_formal_ai_server_with_env};

const TOKEN: &str = "sk-local-agentic-tools";

#[derive(Clone, Copy)]
enum Transport {
    WebSocket,
    WebRtc,
}

impl Transport {
    const fn server_flag(self) -> &'static str {
        match self {
            Self::WebSocket => "--ws",
            Self::WebRtc => "--webrtc",
        }
    }

    const fn client_value(self) -> &'static str {
        match self {
            Self::WebSocket => "websocket",
            Self::WebRtc => "webrtc",
        }
    }
}

struct LocalTransportServer {
    child: Child,
    memory_path: PathBuf,
}

impl LocalTransportServer {
    fn spawn(transport: Transport, port: u16) -> Self {
        let memory_path = std::env::temp_dir().join(format!(
            "formal-ai-issue-931-{}-{port}.lino",
            std::process::id()
        ));
        let child = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "--silent",
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                transport.server_flag(),
            ])
            .env("FORMAL_AI_API_BEARER_TOKEN", TOKEN)
            .env("FORMAL_AI_MEMORY_PATH", &memory_path)
            .env("FORMAL_AI_DREAMING", "0")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn local transport server");
        let mut server = Self { child, memory_path };
        server.wait_until_listening(port);
        server
    }

    fn wait_until_listening(&mut self, port: u16) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait().expect("inspect server") {
                panic!("local transport server exited early: {status}");
            }
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!("local transport server did not listen on port {port}");
    }

    fn memory_contains(&self, needle: &str) -> bool {
        std::fs::read_to_string(&self.memory_path).is_ok_and(|memory| memory.contains(needle))
    }
}

impl Drop for LocalTransportServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.memory_path);
        let _ = std::fs::remove_file(self.memory_path.with_extension("lino.lock"));
    }
}

fn answer_bytes(body: &str) -> Vec<u8> {
    let response: Value = serde_json::from_str(body).expect("OpenAI-compatible JSON response");
    response["choices"][0]["message"]["content"]
        .as_str()
        .expect("assistant answer")
        .as_bytes()
        .to_vec()
}

#[test]
fn issue_931_websocket_and_webrtc_answers_match_http_byte_for_byte() {
    let http_port = reserve_loopback_port();
    let ws_port = reserve_loopback_port();
    let rtc_port = reserve_loopback_port();
    let _http = spawn_formal_ai_server_with_env(
        http_port,
        &[
            ("FORMAL_AI_API_BEARER_TOKEN", TOKEN),
            ("FORMAL_AI_DREAMING", "0"),
        ],
    );
    let ws = LocalTransportServer::spawn(Transport::WebSocket, ws_port);
    let rtc = LocalTransportServer::spawn(Transport::WebRtc, rtc_port);

    let unauthorized = TransportRequest::new("GET", "/v1/models", "");
    assert_eq!(
        websocket_request(&format!("ws://127.0.0.1:{ws_port}"), &unauthorized)
            .expect("unauthorized WebSocket response")
            .status_code,
        401
    );
    assert_eq!(
        webrtc_request(&format!("127.0.0.1:{rtc_port}"), &unauthorized)
            .expect("unauthorized WebRTC response")
            .status_code,
        401
    );

    for (language, prompt) in [
        ("English", "Hi"),
        ("Russian", "Привет"),
        ("Hindi", "नमस्ते"),
        ("Chinese", "你好"),
    ] {
        let request = openai_chat_request(prompt, Some(TOKEN));
        let http = http_request(
            "POST",
            http_port,
            "/v1/chat/completions",
            Some(TOKEN),
            Some(&request.body),
        )
        .expect("HTTP response");
        let websocket = websocket_request(&format!("ws://127.0.0.1:{ws_port}"), &request)
            .expect("WebSocket response");
        let webrtc =
            webrtc_request(&format!("127.0.0.1:{rtc_port}"), &request).expect("WebRTC response");

        assert_eq!(http.status_code, 200, "{language}: {}", http.body);
        assert_eq!(websocket.status_code, http.status_code, "{language}");
        assert_eq!(webrtc.status_code, http.status_code, "{language}");
        assert_eq!(websocket.content_type, http.content_type, "{language}");
        assert_eq!(webrtc.content_type, http.content_type, "{language}");
        assert_eq!(
            answer_bytes(&websocket.body),
            answer_bytes(&http.body),
            "{language}"
        );
        assert_eq!(
            answer_bytes(&webrtc.body),
            answer_bytes(&http.body),
            "{language}"
        );
    }

    // The data-channel adapter chunks above WebRTC's 16 KiB message ceiling in
    // both directions: this envelope carries a 20 KiB request body and the
    // network endpoint returns a complete response larger than one chunk.
    let mut large_request = TransportRequest::new("GET", "/v1/network", "x".repeat(20 * 1024));
    large_request
        .headers
        .insert(String::from("authorization"), format!("Bearer {TOKEN}"));
    let http_network = http_request("GET", http_port, "/v1/network", Some(TOKEN), None)
        .expect("HTTP network response");
    let websocket_network = websocket_request(&format!("ws://127.0.0.1:{ws_port}"), &large_request)
        .expect("large WebSocket response");
    let webrtc_network = webrtc_request(&format!("127.0.0.1:{rtc_port}"), &large_request)
        .expect("chunked WebRTC response");
    assert!(http_network.body.len() > 16 * 1024);
    assert_eq!(
        websocket_network.body.as_bytes(),
        http_network.body.as_bytes()
    );
    assert_eq!(webrtc_network.body.as_bytes(), http_network.body.as_bytes());

    assert!(ws.memory_contains("role \"user\""));
    assert!(rtc.memory_contains("role \"user\""));
}

#[test]
fn issue_931_whole_task_uses_the_same_cli_as_server_and_client() {
    for transport in [Transport::WebSocket, Transport::WebRtc] {
        let port = reserve_loopback_port();
        let _server = LocalTransportServer::spawn(transport, port);
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "--silent",
                "connect",
                "--transport",
                transport.client_value(),
                "--endpoint",
                &format!("127.0.0.1:{port}"),
                "--api-key",
                TOKEN,
                "--prompt",
                "Hi",
            ])
            .output()
            .expect("run formal-ai client");

        assert!(
            output.status.success(),
            "{} client failed: {}",
            transport.client_value(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "Hi, how may I help you?\n"
        );
    }
}
