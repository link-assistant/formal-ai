use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::engine::FormalAiEngine;

const TEXT_ONLY_MESSAGE: &str = "I can only process Telegram text messages in this prototype. Send a text prompt or a message caption.";

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
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
}

pub fn handle_telegram_webhook(
    body: &str,
) -> Result<Option<TelegramWebhookReply>, TelegramWebhookError> {
    let update = serde_json::from_str::<TelegramUpdate>(body)
        .map_err(|error| TelegramWebhookError::InvalidJson(error.to_string()))?;
    let Some(message) = update.into_message() else {
        return Ok(None);
    };

    Ok(Some(reply_for_message(&message)))
}

fn reply_for_message(message: &TelegramMessage) -> TelegramWebhookReply {
    let reply_text = message
        .text
        .as_deref()
        .or(message.caption.as_deref())
        .filter(|text| !text.trim().is_empty())
        .map_or_else(
            || String::from(TEXT_ONLY_MESSAGE),
            |prompt| FormalAiEngine.answer(prompt.trim()).answer,
        );

    TelegramWebhookReply {
        method: "sendMessage",
        chat_id: message.chat.id,
        text: telegram_html_from_markdown(&reply_text),
        parse_mode: "HTML",
        reply_parameters: TelegramReplyParameters {
            message_id: message.message_id,
        },
    }
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
