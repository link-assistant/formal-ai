use std::error::Error;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};

use crate::attachment_context::{compose_prompt_with_attachments, Attachment};
use crate::engine::{naturalize_thinking_step, ThinkingStep};
use crate::solver::{ExecutionSurface, SolverConfig, UniversalSolver};

const TEXT_ONLY_MESSAGE: &str = "I can only process Telegram text messages in this implementation. Send a text prompt or a message caption.";
const DEFAULT_API_BASE: &str = "https://api.telegram.org";
/// Telegram rejects `sendMessage` payloads whose text exceeds 4096 characters,
/// so the supplementary thinking blockquote (issue #488) is only appended when
/// the whole reply still fits within this budget.
const TELEGRAM_MAX_MESSAGE_LEN: usize = 4096;
/// Crate version advertised by the `/version` bot command. Tracks
/// `Cargo.toml` automatically so every release reports the right number
/// without manual bumps (issue #72).
const FORMAL_AI_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_POLL_TIMEOUT_SECONDS: u32 = 30;
const DEFAULT_POLL_LIMIT: u32 = 100;
const POLL_CONNECT_TIMEOUT_PADDING_SECONDS: u32 = 10;
/// Minimum gap between consecutive `editMessageText` calls on the same chat
/// (issue #488). Telegram throttles bots to roughly one message-affecting
/// operation per chat per second; this debounce keeps the progressive thinking
/// stream within that budget without surprising the user with bursts.
const TELEGRAM_THINKING_EDIT_DEBOUNCE_MS: u64 = 1_200;
/// Cap on the number of intermediate edits we stream while the answer is being
/// composed (issue #488). With the debounce above this keeps the visible
/// "thinking" phase under ~5 s, matching the upper bound the issue asks for.
const TELEGRAM_THINKING_MAX_EDITS: usize = 4;
/// Initial placeholder shown the moment the thinking bubble appears on
/// Telegram, before the first thinking step is rendered. Mirrors the web UI's
/// "Reading the request…" pending phase (issue #488).
const TELEGRAM_THINKING_INITIAL_PLACEHOLDER: &str = "<i>\u{1F4AD} Reading the request…</i>";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelegramWebhookReply {
    pub method: &'static str,
    pub chat_id: i64,
    pub text: String,
    pub parse_mode: &'static str,
    pub reply_parameters: TelegramReplyParameters,
}

/// One debounced `editMessageText` call that progressively reveals the
/// solver's thinking inside Telegram (issue #488).
///
/// Each edit replaces the live thinking message with a richer snapshot of the
/// solver's reasoning; the runtime sleeps `delay_before_ms` between edits so
/// the stream stays within Telegram's per-chat rate limits and still feels
/// "alive" rather than a single instant dump.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramThinkingEdit {
    pub text: String,
    pub parse_mode: &'static str,
    pub delay_before_ms: u64,
}

impl TelegramThinkingEdit {
    /// Encode the edit as the JSON body Telegram's `editMessageText` expects.
    /// `chat_id` and `message_id` are supplied by the runtime once the initial
    /// `sendMessage` reports the live thinking message's id.
    #[must_use]
    pub fn to_edit_message_body(&self, chat_id: i64, message_id: i64) -> String {
        let mut value = serde_json::Map::new();
        value.insert(String::from("chat_id"), serde_json::Value::from(chat_id));
        value.insert(
            String::from("message_id"),
            serde_json::Value::from(message_id),
        );
        value.insert(
            String::from("text"),
            serde_json::Value::from(self.text.clone()),
        );
        value.insert(
            String::from("parse_mode"),
            serde_json::Value::from(self.parse_mode),
        );
        serde_json::to_string(&serde_json::Value::Object(value))
            .unwrap_or_else(|_| String::from("{}"))
    }
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
    #[serde(default)]
    document: Option<TelegramDocument>,
    #[serde(default)]
    photo: Vec<TelegramPhotoSize>,
    #[serde(default)]
    audio: Option<TelegramDocument>,
    #[serde(default)]
    voice: Option<TelegramDocument>,
    #[serde(default)]
    video: Option<TelegramDocument>,
}

/// A Telegram `document` attachment (any uploaded file).
#[derive(Debug, Deserialize)]
struct TelegramDocument {
    #[serde(default)]
    file_name: Option<String>,
    #[serde(default)]
    mime_type: Option<String>,
    #[serde(default)]
    file_size: Option<u64>,
}

/// One size of a Telegram `photo`. Photos arrive as an array of increasing
/// resolutions; the largest carries the most representative size metadata.
#[derive(Debug, Deserialize)]
struct TelegramPhotoSize {
    #[serde(default)]
    file_size: Option<u64>,
}

impl TelegramMessage {
    /// Collect every attached file on this message as shared
    /// [`Attachment`](crate::attachment_context::Attachment) metadata, so the
    /// solver receives the same `Attached files:` context every other surface
    /// builds. Telegram delivers the file's own caption separately, so no text
    /// excerpt is available here — the solver still recognises the attachment by
    /// name and MIME type and requests the local file text.
    fn attachments(&self) -> Vec<Attachment> {
        let mut attachments = Vec::new();
        if let Some(document) = &self.document {
            attachments.push(document.as_attachment("document", self.message_id, ""));
        }
        if let Some(audio) = &self.audio {
            attachments.push(audio.as_attachment("audio", self.message_id, "audio/mpeg"));
        }
        if let Some(voice) = &self.voice {
            attachments.push(voice.as_attachment("voice", self.message_id, "audio/ogg"));
        }
        if let Some(video) = &self.video {
            attachments.push(video.as_attachment("video", self.message_id, "video/mp4"));
        }
        if !self.photo.is_empty() {
            let largest = self
                .photo
                .iter()
                .max_by_key(|size| size.file_size.unwrap_or(0));
            let mut attachment =
                Attachment::new(format!("photo_{}.jpg", self.message_id), "image/jpeg");
            if let Some(size) = largest.and_then(|size| size.file_size) {
                attachment = attachment.with_size(size);
            }
            attachments.push(attachment);
        }
        attachments
    }
}

impl TelegramDocument {
    /// Convert a Telegram file object into a shared [`Attachment`], defaulting a
    /// missing name to `<kind>_<message_id>` and a missing MIME type to
    /// `fallback_mime` (or octet-stream when that is empty too).
    fn as_attachment(&self, kind: &str, message_id: i64, fallback_mime: &str) -> Attachment {
        let name = self
            .file_name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| format!("{kind}_{message_id}"));
        let mime = self
            .mime_type
            .clone()
            .filter(|mime| !mime.trim().is_empty())
            .unwrap_or_else(|| fallback_mime.to_owned());
        let mut attachment = Attachment::new(name, mime);
        if let Some(size) = self.file_size {
            attachment = attachment.with_size(size);
        }
        attachment
    }
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
    compose_telegram_reply(message).reply
}

/// Final reply text plus the thinking steps that produced it (issue #488).
///
/// The polling code needs both the rendered answer (final edit) and the
/// thinking steps (progressive intermediate edits), so this struct keeps them
/// together and lets the solver run once per Telegram update.
struct TelegramReplyBundle {
    reply: TelegramWebhookReply,
    thinking_steps: Vec<ThinkingStep>,
}

fn compose_telegram_reply(message: &TelegramMessage) -> TelegramReplyBundle {
    let raw_text = message.text.as_deref().or(message.caption.as_deref());
    // Fold any attached document/photo/media into the same `Attached files:`
    // context every other surface builds, so an attachment-only message (or a
    // caption plus a file) reaches the solver as a grounded prompt (issue #535).
    let attachments = message.attachments();
    let prompt = compose_prompt_with_attachments(raw_text, &attachments);

    let (reply_text, trace_id, thinking, steps) =
        prompt.filter(|text| !text.trim().is_empty()).map_or_else(
            || (String::from(TEXT_ONLY_MESSAGE), None, None, Vec::new()),
            |prompt| {
                let trimmed = prompt.trim();
                if is_version_command(trimmed) {
                    return (version_reply_text(), None, None, Vec::new());
                }
                let symbolic = telegram_solver().solve(trimmed);
                let trace = symbolic
                    .evidence_links
                    .iter()
                    .find_map(|link| link.strip_prefix("trace:").map(str::to_owned));
                let thinking = telegram_thinking_blockquote(&symbolic.thinking_steps);
                (symbolic.answer, trace, thinking, symbolic.thinking_steps)
            },
        );

    let mut text = telegram_html_from_markdown(&reply_text);
    let trace_footer = trace_id.map(|trace| format!("\n\n/trace {trace}"));

    // Append the collapsed thinking blockquote after the answer, but only while
    // the whole reply (answer + thinking + trace footer) still fits inside
    // Telegram's 4096-character limit; the reasoning is supplementary, so it is
    // dropped rather than risk a rejected `sendMessage` (issue #488).
    if let Some(thinking) = thinking {
        let trace_len = trace_footer.as_deref().map_or(0, str::len);
        if text.len() + "\n\n".len() + thinking.len() + trace_len <= TELEGRAM_MAX_MESSAGE_LEN {
            text.push_str("\n\n");
            text.push_str(&thinking);
        }
    }
    if let Some(footer) = trace_footer {
        text.push_str(&footer);
    }

    TelegramReplyBundle {
        reply: TelegramWebhookReply {
            method: "sendMessage",
            chat_id: message.chat.id,
            text,
            parse_mode: "HTML",
            reply_parameters: TelegramReplyParameters {
                message_id: message.message_id,
            },
        },
        thinking_steps: steps,
    }
}

/// Render the solver's concrete thinking steps as a Telegram expandable
/// blockquote (issue #488).
///
/// Telegram's native `<blockquote expandable>` is collapsed by default and
/// expands on tap, which is a direct, native fit for the issue's "show the
/// reasoning, collapsed, with an expand affordance" requirement on a non-UI
/// surface. The lines are the same concrete English meta-language descriptions
/// every other surface renders (the CLI `--thinking` trace, the OpenAI and
/// Anthropic APIs); the browser UI additionally translates them into the user's
/// language through its i18n catalog. Returns `None` when there is nothing to
/// show so callers can skip the separator entirely.
fn telegram_thinking_blockquote(steps: &[ThinkingStep]) -> Option<String> {
    if steps.is_empty() {
        return None;
    }
    let mut body = String::new();
    for step in steps {
        let sentence = if step.summary.is_empty() {
            naturalize_thinking_step(&step.step, &step.detail)
        } else {
            step.summary.clone()
        };
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str(&html_escape(&sentence));
    }
    Some(format!("<blockquote expandable>💭 {body}</blockquote>"))
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
    pub fn edit_message_text_url(&self) -> String {
        format!(
            "{}/bot{}/editMessageText",
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
///
/// On chats where the bot can post follow-up messages (polling mode), the
/// initial `sendMessage` posts the live thinking placeholder; the
/// `progressive_edits` chain then progressively reveals more reasoning via
/// `editMessageText` until the final edit replaces the bubble with the
/// composed answer (issue #488). The runtime sleeps each edit's
/// `delay_before_ms` between calls so the stream stays within Telegram's
/// per-chat rate limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelegramPollingReply {
    pub chat_id: i64,
    pub text: String,
    pub parse_mode: &'static str,
    pub reply_parameters: TelegramReplyParameters,
    /// Progressive `editMessageText` chain that walks the user through the
    /// solver's reasoning before the final answer lands. Skipped during
    /// serialization because it is runtime metadata, not a Telegram API field.
    #[serde(skip)]
    pub progressive_edits: Vec<TelegramThinkingEdit>,
}

impl TelegramPollingReply {
    /// Encode the reply as the JSON body Telegram's `sendMessage` expects.
    #[must_use]
    pub fn to_send_message_body(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| String::from("{}"))
    }
}

/// Extract `result.message_id` from a Telegram `sendMessage` response body.
///
/// The runtime targets the extracted id with follow-up `editMessageText`
/// calls (issue #488). Returns `None` when the response is missing/malformed —
/// callers then skip the progressive thinking stream and keep the initial
/// placeholder rather than risk garbled state.
#[must_use]
pub fn extract_sent_message_id(body: &str) -> Option<i64> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value.get("ok").and_then(serde_json::Value::as_bool)?;
    value
        .get("result")
        .and_then(|result| result.get("message_id"))
        .and_then(serde_json::Value::as_i64)
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
    let bundle = compose_telegram_reply(message);
    let progressive_edits = build_progressive_thinking_edits(
        &bundle.thinking_steps,
        bundle.reply.text.clone(),
        bundle.reply.parse_mode,
    );
    TelegramPollingReply {
        chat_id: bundle.reply.chat_id,
        // Issue #488: on the polling surface the live thinking placeholder is
        // the initial message; the rendered answer arrives via the final
        // `editMessageText`. When progressive edits are present the placeholder
        // replaces the answer text in the initial `sendMessage` body.
        text: if progressive_edits.is_empty() {
            bundle.reply.text
        } else {
            String::from(TELEGRAM_THINKING_INITIAL_PLACEHOLDER)
        },
        parse_mode: bundle.reply.parse_mode,
        reply_parameters: bundle.reply.reply_parameters,
        progressive_edits,
    }
}

/// Build the progressive `editMessageText` chain (issue #488).
///
/// Splits the solver's thinking steps into at most
/// `TELEGRAM_THINKING_MAX_EDITS` snapshots, where each snapshot reveals one
/// more group of steps than the previous, then ends with a final edit that
/// replaces the live bubble with the fully composed answer (which already
/// carries the collapsed expandable blockquote when it fits within
/// Telegram's 4096-character budget).
///
/// Returns an empty vector when there are no steps to stream so callers can
/// stay on the single-`sendMessage` fast path.
fn build_progressive_thinking_edits(
    steps: &[ThinkingStep],
    final_text: String,
    parse_mode: &'static str,
) -> Vec<TelegramThinkingEdit> {
    if steps.is_empty() {
        return Vec::new();
    }

    let mut edits = Vec::new();
    let total = steps.len();
    let cap = TELEGRAM_THINKING_MAX_EDITS.min(total);
    let mut last_visible = 0;
    for snapshot_index in 0..cap {
        let visible = ((snapshot_index + 1) * total).div_ceil(cap).min(total);
        if visible == last_visible {
            continue;
        }
        last_visible = visible;
        let Some(text) = telegram_thinking_blockquote(&steps[..visible]) else {
            continue;
        };
        // Edits whose text would exceed Telegram's 4096-char limit are
        // dropped; the live bubble keeps the latest legal snapshot until the
        // final answer lands.
        if text.len() > TELEGRAM_MAX_MESSAGE_LEN {
            continue;
        }
        edits.push(TelegramThinkingEdit {
            text,
            parse_mode,
            delay_before_ms: TELEGRAM_THINKING_EDIT_DEBOUNCE_MS,
        });
    }
    // Final edit: hand off from the live thinking bubble to the composed
    // answer (which already includes the collapsed thinking blockquote when
    // it fits within Telegram's 4096-char budget).
    edits.push(TelegramThinkingEdit {
        text: final_text,
        parse_mode,
        delay_before_ms: TELEGRAM_THINKING_EDIT_DEBOUNCE_MS,
    });
    edits
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
