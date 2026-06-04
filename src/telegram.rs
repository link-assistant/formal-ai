use std::error::Error;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};

use crate::solver::{ExecutionSurface, SolverConfig, UniversalSolver};

const TEXT_ONLY_MESSAGE: &str = "I can only process Telegram text messages in this implementation. Send a text prompt or a message caption.";
const DEFAULT_API_BASE: &str = "https://api.telegram.org";
/// Crate version advertised by the `/version` bot command. Tracks
/// `Cargo.toml` automatically so every release reports the right number
/// without manual bumps (issue #72).
const FORMAL_AI_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_POLL_TIMEOUT_SECONDS: u32 = 30;
const DEFAULT_POLL_LIMIT: u32 = 100;
const POLL_CONNECT_TIMEOUT_PADDING_SECONDS: u32 = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelegramWebhookReply {
    pub method: &'static str,
    pub chat_id: i64,
    pub text: String,
    pub parse_mode: &'static str,
    pub reply_parameters: TelegramReplyParameters,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelegramReplyParameters {
    pub message_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelegramWebhookError {
    InvalidJson(String),
}

impl Display for TelegramWebhookError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(message) => {
                write!(formatter, "invalid Telegram update JSON: {message}")
            }
        }
    }
}

impl Error for TelegramWebhookError {}

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    #[serde(default)]
    update_id: Option<i64>,
    #[serde(default)]
    message: Option<TelegramMessage>,
    #[serde(default)]
    edited_message: Option<TelegramMessage>,
    #[serde(default)]
    channel_post: Option<TelegramMessage>,
    #[serde(default)]
    edited_channel_post: Option<TelegramMessage>,
}

impl TelegramUpdate {
    fn into_message(self) -> Option<TelegramMessage> {
        self.message
            .or(self.edited_message)
            .or(self.channel_post)
            .or(self.edited_channel_post)
    }
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    message_id: i64,
    chat: TelegramChat,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    caption: Option<String>,
    #[serde(default)]
    entities: Vec<TelegramEntity>,
    #[serde(default)]
    reply_to_message: Option<Box<Self>>,
    #[serde(default)]
    from: Option<TelegramUser>,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramEntity {
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
struct TelegramUser {
    #[serde(default)]
    is_bot: bool,
}

fn message_is_in_public_chat(chat: &TelegramChat) -> bool {
    matches!(
        chat.kind.as_deref(),
        Some("group" | "supergroup" | "channel")
    )
}

fn message_addresses_bot(message: &TelegramMessage) -> bool {
    if message
        .entities
        .iter()
        .any(|entity| entity.kind == "mention" || entity.kind == "bot_command")
    {
        return true;
    }
    if let Some(reply) = &message.reply_to_message {
        if reply.from.as_ref().is_some_and(|user| user.is_bot) {
            return true;
        }
    }
    if let Some(title) = &message.chat.title {
        let lower = title.to_lowercase();
        if lower.contains("formal") || lower.contains("formal_ai") || lower.contains("formal-ai") {
            return true;
        }
    }
    false
}

pub fn handle_telegram_webhook(
    body: &str,
) -> Result<Option<TelegramWebhookReply>, TelegramWebhookError> {
    let update = serde_json::from_str::<TelegramUpdate>(body)
        .map_err(|error| TelegramWebhookError::InvalidJson(error.to_string()))?;
    let Some(message) = update.into_message() else {
        return Ok(None);
    };

    if message_is_in_public_chat(&message.chat) && !message_addresses_bot(&message) {
        return Ok(None);
    }

    Ok(Some(reply_for_message(&message)))
}

fn reply_for_message(message: &TelegramMessage) -> TelegramWebhookReply {
    let raw_text = message.text.as_deref().or(message.caption.as_deref());

    let (reply_text, trace_id) = raw_text.filter(|text| !text.trim().is_empty()).map_or_else(
        || (String::from(TEXT_ONLY_MESSAGE), None),
        |prompt| {
            let trimmed = prompt.trim();
            if is_version_command(trimmed) {
                return (version_reply_text(), None);
            }
            let symbolic = telegram_solver().solve(trimmed);
            let trace = symbolic
                .evidence_links
                .iter()
                .find_map(|link| link.strip_prefix("trace:").map(str::to_owned));
            (symbolic.answer, trace)
        },
    );

    let mut text = telegram_html_from_markdown(&reply_text);
    if let Some(trace) = trace_id {
        text.push_str("\n\n/trace ");
        text.push_str(&trace);
    }

    TelegramWebhookReply {
        method: "sendMessage",
        chat_id: message.chat.id,
        text,
        parse_mode: "HTML",
        reply_parameters: TelegramReplyParameters {
            message_id: message.message_id,
        },
    }
}

fn telegram_solver() -> UniversalSolver {
    let mut config = SolverConfig::from_env();
    config.execution_surface = ExecutionSurface::Telegram;
    UniversalSolver::new(config)
}

fn is_version_command(text: &str) -> bool {
    let first_token = text.split_whitespace().next().unwrap_or("");
    let command = first_token.split('@').next().unwrap_or("");
    command.eq_ignore_ascii_case("/version")
}

fn version_reply_text() -> String {
    format!("formal-ai {FORMAL_AI_VERSION}")
}

#[must_use]
pub fn telegram_html_from_markdown(markdown: &str) -> String {
    let mut rendered = String::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if let Some(language) = trimmed.strip_prefix("```") {
            if in_code_block {
                rendered.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                rendered.push_str(&open_pre_code_tag(language.trim()));
                in_code_block = true;
            }
            continue;
        }

        rendered.push_str(&html_escape(line));
        rendered.push('\n');
    }

    if in_code_block {
        rendered.push_str("</code></pre>\n");
    }

    rendered.trim_end().to_owned()
}

fn open_pre_code_tag(language: &str) -> String {
    language_class(language).map_or_else(
        || String::from("<pre><code>"),
        |class| format!("<pre><code class=\"{class}\">"),
    )
}

fn language_class(language: &str) -> Option<String> {
    if language.is_empty()
        || !language
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return None;
    }

    Some(format!("language-{language}"))
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Options that control how the long-polling loop talks to Telegram.
///
/// `api_base` is the Telegram Bot API root (`<https://api.telegram.org>` by default).
/// `token` is the bot token without the leading `bot` prefix.
/// `timeout_seconds` is forwarded to Telegram's `getUpdates` long-polling timeout.
/// `limit` is forwarded to Telegram's `getUpdates` limit parameter (1-100).
/// `allowed_updates` is forwarded as JSON to Telegram so the bot can restrict the
/// update types it receives.
#[derive(Debug, Clone)]
pub struct TelegramPollingConfig {
    pub api_base: String,
    pub token: String,
    pub timeout_seconds: u32,
    pub limit: u32,
    pub allowed_updates: Vec<String>,
}

impl TelegramPollingConfig {
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            api_base: String::from(DEFAULT_API_BASE),
            token: token.into(),
            timeout_seconds: DEFAULT_POLL_TIMEOUT_SECONDS,
            limit: DEFAULT_POLL_LIMIT,
            allowed_updates: Vec::new(),
        }
    }

    #[must_use]
    pub fn get_updates_url(&self, offset: Option<i64>) -> String {
        let mut url = format!(
            "{}/bot{}/getUpdates?timeout={}&limit={}",
            self.api_base.trim_end_matches('/'),
            self.token,
            self.timeout_seconds,
            self.limit
        );
        if let Some(offset_value) = offset {
            let _ = write!(url, "&offset={offset_value}");
        }
        if !self.allowed_updates.is_empty() {
            let encoded = url_encode(&serialize_string_array(&self.allowed_updates));
            let _ = write!(url, "&allowed_updates={encoded}");
        }
        url
    }

    #[must_use]
    pub fn send_message_url(&self) -> String {
        format!(
            "{}/bot{}/sendMessage",
            self.api_base.trim_end_matches('/'),
            self.token
        )
    }

    #[must_use]
    pub const fn http_timeout_seconds(&self) -> u32 {
        self.timeout_seconds
            .saturating_add(POLL_CONNECT_TIMEOUT_PADDING_SECONDS)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelegramPollingError {
    InvalidJson(String),
    UnexpectedResponse(String),
}

impl Display for TelegramPollingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(message) => {
                write!(formatter, "invalid Telegram getUpdates JSON: {message}")
            }
            Self::UnexpectedResponse(message) => {
                write!(
                    formatter,
                    "unexpected Telegram getUpdates payload: {message}"
                )
            }
        }
    }
}

impl Error for TelegramPollingError {}

#[derive(Debug, Deserialize)]
struct GetUpdatesResponse {
    ok: bool,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    result: Option<Vec<serde_json::Value>>,
}

/// A reply built from a Telegram update that should be sent back through `sendMessage`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelegramPollingReply {
    pub chat_id: i64,
    pub text: String,
    pub parse_mode: &'static str,
    pub reply_parameters: TelegramReplyParameters,
}

impl TelegramPollingReply {
    /// Encode the reply as the JSON body Telegram's `sendMessage` expects.
    #[must_use]
    pub fn to_send_message_body(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| String::from("{}"))
    }
}

/// Parsed slice of a Telegram `getUpdates` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUpdatesBatch {
    pub replies: Vec<TelegramPollingReply>,
    pub next_offset: Option<i64>,
}

/// Parse a Telegram `getUpdates` response and convert each update into a reply.
///
/// The returned `next_offset` is `max(update_id) + 1` so the next `getUpdates`
/// call only returns new updates.
pub fn parse_get_updates_response(body: &str) -> Result<ParsedUpdatesBatch, TelegramPollingError> {
    let response = serde_json::from_str::<GetUpdatesResponse>(body)
        .map_err(|error| TelegramPollingError::InvalidJson(error.to_string()))?;

    if !response.ok {
        let description = response
            .description
            .unwrap_or_else(|| String::from("Telegram reported ok=false"));
        return Err(TelegramPollingError::UnexpectedResponse(description));
    }

    let updates = response.result.unwrap_or_default();
    let mut replies = Vec::new();
    let mut highest_update_id: Option<i64> = None;

    for raw in updates {
        let update_text = raw.to_string();
        let parsed = serde_json::from_value::<TelegramUpdate>(raw)
            .map_err(|error| TelegramPollingError::InvalidJson(error.to_string()))?;
        if let Some(id) = parsed.update_id {
            highest_update_id =
                Some(highest_update_id.map_or(id, |existing| std::cmp::max(existing, id)));
        }

        if let Some(message) = parsed.into_message() {
            replies.push(reply_for_polling_message(&message));
        } else {
            eprintln!(
                "telegram-poll: ignoring update without a supported message field: {update_text}"
            );
        }
    }

    Ok(ParsedUpdatesBatch {
        replies,
        next_offset: highest_update_id.map(|id| id + 1),
    })
}

fn reply_for_polling_message(message: &TelegramMessage) -> TelegramPollingReply {
    let webhook = reply_for_message(message);
    TelegramPollingReply {
        chat_id: webhook.chat_id,
        text: webhook.text,
        parse_mode: webhook.parse_mode,
        reply_parameters: webhook.reply_parameters,
    }
}

fn serialize_string_array(values: &[String]) -> String {
    let mut buffer = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            buffer.push(',');
        }
        buffer.push('"');
        for character in value.chars() {
            match character {
                '"' | '\\' => {
                    buffer.push('\\');
                    buffer.push(character);
                }
                _ => buffer.push(character),
            }
        }
        buffer.push('"');
    }
    buffer.push(']');
    buffer
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        let character = byte as char;
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '~') {
            encoded.push(character);
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}
