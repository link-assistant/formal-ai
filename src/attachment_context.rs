//! Shared attachment-context builder for every interface surface.
//!
//! Issue #535 asks that attached files be supported in Desktop, Telegram bot,
//! Web app, and other interface surfaces.
//!
//! The web app already prepends an `Attached files:` block to the prompt so the
//! solver's [`document_originality`](crate::solver_handlers) handler can
//! recognise and ground the request. This module lifts that block-building into
//! one canonical, testable place so every surface — Telegram included —
//! produces the exact same textual context the handler already parses.
//!
//! The rendered block matches the format the solver consumes:
//!
//! ```text
//! Attached files:
//! 1. report.txt (text/plain, 8.0 KB)
//! Text excerpt: The tower opened in 1889.
//! ```

use std::fmt::Write as _;

/// One attached file's metadata plus an optional extracted text excerpt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// Display file name (e.g. `report.txt`).
    pub name: String,
    /// MIME type (e.g. `text/plain`); empty when unknown.
    pub mime_type: String,
    /// File size in bytes, when the surface reports it.
    pub size_bytes: Option<u64>,
    /// A short text excerpt / OCR sample, when the surface could extract one.
    pub text_excerpt: Option<String>,
}

impl Attachment {
    /// Construct an attachment from name and MIME type, with no size or excerpt.
    #[must_use]
    pub fn new(name: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mime_type: mime_type.into(),
            size_bytes: None,
            text_excerpt: None,
        }
    }

    /// Attach a known size in bytes.
    #[must_use]
    pub const fn with_size(mut self, size_bytes: u64) -> Self {
        self.size_bytes = Some(size_bytes);
        self
    }

    /// Attach an extracted text excerpt.
    #[must_use]
    pub fn with_excerpt(mut self, excerpt: impl Into<String>) -> Self {
        let excerpt = excerpt.into();
        if !excerpt.trim().is_empty() {
            self.text_excerpt = Some(excerpt);
        }
        self
    }

    /// Render the parenthetical `(type, size)` descriptor for this file's line.
    fn descriptor(&self) -> String {
        let mime = if self.mime_type.trim().is_empty() {
            "application/octet-stream"
        } else {
            self.mime_type.trim()
        };
        self.size_bytes.map_or_else(
            || mime.to_owned(),
            |size| format!("{mime}, {}", human_readable_size(size)),
        )
    }
}

/// Render an `Attached files:` context block for `attachments`, or `None` when
/// there is nothing to attach. The block is safe to prepend to any prompt/caption.
#[must_use]
pub fn build_attachment_context(attachments: &[Attachment]) -> Option<String> {
    if attachments.is_empty() {
        return None;
    }
    let mut block = String::from("Attached files:");
    for (index, attachment) in attachments.iter().enumerate() {
        // `write!` into a `String` is infallible, so the `Result` is discarded.
        let _ = write!(
            block,
            "\n{}. {} ({})",
            index + 1,
            attachment.name,
            attachment.descriptor(),
        );
        if let Some(excerpt) = &attachment.text_excerpt {
            let _ = write!(block, "\nText excerpt: {}", excerpt.trim());
        }
    }
    Some(block)
}

/// Combine an optional user message with an attachment context block.
///
/// The solver sees the message first and the `Attached files:` block after a
/// blank line — the same shape the web app produces. Returns `None` only when
/// there is neither text nor any attachment.
#[must_use]
pub fn compose_prompt_with_attachments(
    message: Option<&str>,
    attachments: &[Attachment],
) -> Option<String> {
    let message = message.map(str::trim).filter(|text| !text.is_empty());
    let context = build_attachment_context(attachments);
    match (message, context) {
        (Some(message), Some(context)) => Some(format!("{message}\n\n{context}")),
        (Some(message), None) => Some(message.to_owned()),
        (None, Some(context)) => Some(context),
        (None, None) => None,
    }
}

/// Format a byte count as a compact human-readable size (`8.0 KB`, `1.5 MB`).
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn human_readable_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    let value = bytes as f64;
    if value < KIB {
        format!("{bytes} B")
    } else if value < MIB {
        format!("{:.1} KB", value / KIB)
    } else if value < GIB {
        format!("{:.1} MB", value / MIB)
    } else {
        format!("{:.1} GB", value / GIB)
    }
}
