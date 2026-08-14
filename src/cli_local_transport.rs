//! CLI bindings for the local HTTP, WebSocket, and WebRTC server/client modes.

use std::error::Error;

use clap::{Args as ClapArgs, ValueEnum};
use formal_ai::local_transport::{
    openai_chat_request, serve_webrtc, serve_websocket, webrtc_request, websocket_request,
};
use serde_json::Value;

#[derive(Debug, ClapArgs)]
#[allow(clippy::struct_excessive_bools)] // Independent command-line switches are intentional.
pub struct ServeArgs {
    #[arg(long, env = "FORMAL_AI_HOST", default_value = "127.0.0.1")]
    host: String,

    #[arg(long, env = "FORMAL_AI_PORT", default_value_t = 8080)]
    port: u16,

    /// Allow OpenAI-compatible agent clients to receive tool calls. Equivalent
    /// to `FORMAL_AI_AGENT_MODE=1` on every transport.
    #[arg(long, default_value_t = false)]
    agent_mode: bool,

    /// Serve the transport-neutral API envelope over WebSocket.
    #[arg(long, conflicts_with = "webrtc", default_value_t = false)]
    ws: bool,

    /// Serve local-first WebRTC data channels with loopback offer/answer signaling.
    #[arg(long, conflicts_with = "ws", default_value_t = false)]
    webrtc: bool,

    /// Print local transport connection, signaling, and lifecycle diagnostics.
    #[arg(long, default_value_t = false)]
    transport_trace: bool,
}

#[derive(Debug, ClapArgs)]
pub struct ConnectArgs {
    /// Local transport used for this OpenAI-compatible chat request.
    #[arg(long, value_enum)]
    transport: ClientTransport,

    /// WebSocket URL or WebRTC signaling address.
    #[arg(long, default_value = "127.0.0.1:8080")]
    endpoint: String,

    #[arg(long, env = "FORMAL_AI_PROMPT")]
    prompt: String,

    /// Bearer token forwarded to the shared API permission router.
    #[arg(long)]
    api_key: Option<String>,

    #[arg(long, value_enum, default_value_t = ConnectFormat::Text)]
    format: ConnectFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ClientTransport {
    #[value(alias = "ws")]
    Websocket,
    Webrtc,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ConnectFormat {
    Text,
    Json,
}

pub fn run_serve(args: &ServeArgs) -> Result<(), Box<dyn Error>> {
    if args.agent_mode {
        formal_ai::enable_http_agent_mode_for_current_process();
    }
    let address = format!("{}:{}", args.host, args.port);
    if args.ws {
        serve_websocket(&address, args.transport_trace).map_err(|error| transport_error(&error))?;
    } else if args.webrtc {
        serve_webrtc(&address, args.transport_trace).map_err(|error| transport_error(&error))?;
    } else {
        formal_ai::serve(&address)?;
    }
    Ok(())
}

pub fn run_connect(args: &ConnectArgs) -> Result<(), Box<dyn Error>> {
    let token = args.api_key.clone().or_else(api_key_from_env);
    let request = openai_chat_request(&args.prompt, token.as_deref());
    let response = match args.transport {
        ClientTransport::Websocket => {
            websocket_request(&args.endpoint, &request).map_err(|error| transport_error(&error))?
        }
        ClientTransport::Webrtc => {
            webrtc_request(&args.endpoint, &request).map_err(|error| transport_error(&error))?
        }
    };
    if !(200..300).contains(&response.status_code) {
        return Err(format!(
            "local transport returned status {}: {}",
            response.status_code, response.body
        )
        .into());
    }
    match args.format {
        ConnectFormat::Json => println!("{}", response.body),
        ConnectFormat::Text => println!("{}", assistant_content(&response.body)?),
    }
    Ok(())
}

fn assistant_content(body: &str) -> Result<String, Box<dyn Error>> {
    let response: Value = serde_json::from_str(body)?;
    response["choices"][0]["message"]["content"]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| String::from("response did not contain an assistant message").into())
}

fn api_key_from_env() -> Option<String> {
    for name in [
        "FORMAL_AI_API_BEARER_TOKEN",
        "FORMAL_AI_HTTP_BEARER_TOKEN",
        "FORMAL_AI_API_TOKEN",
    ] {
        if let Some(value) = std::env::var_os(name) {
            if !value.is_empty() {
                return Some(value.to_string_lossy().into_owned());
            }
        }
    }
    None
}

fn transport_error(error: &formal_ai::local_transport::TransportError) -> Box<dyn Error> {
    error.to_string().into()
}
