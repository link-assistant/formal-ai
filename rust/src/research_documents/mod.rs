//! Downloadable research artifacts over the upstream document concept layer.
//!
//! Research acquisition and synthesis happen before this boundary. Only supplied
//! claims and captured source references enter the report; rendering performs no
//! search and cannot manufacture evidence. Consumers deliver `bytes` with the
//! supplied MIME type and safe filename. No filesystem writes occur here.

/// A captured source that supports the synthesized report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchReference {
    pub id: String,
    pub title: String,
    pub url: String,
    pub captured_at: String,
}

/// A statement and the source identifiers that support it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchStatement {
    pub text: String,
    pub source_ids: Vec<String>,
}

/// Localized content supplied by the research synthesis stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchDocument {
    pub title: String,
    pub language: String,
    pub statements: Vec<ResearchStatement>,
    pub references: Vec<ResearchReference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactFormat {
    Markdown,
    Pdf,
    Docx,
}

impl ArtifactFormat {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Pdf => "pdf",
            Self::Docx => "docx",
        }
    }

    #[must_use]
    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Markdown => "text/markdown; charset=utf-8",
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchArtifact {
    pub filename: String,
    pub mime_type: &'static str,
    pub language: String,
    pub bytes: Vec<u8>,
}

/// Explicit failures prevent unsupported output being advertised as a download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactError {
    EmptyDocument,
    InvalidContent,
    InvalidReference,
    MissingEvidence,
    UnsafeFilename,
    ConversionUnavailable,
    /// Upstream PDF uses standard Type 1 fonts without Unicode shaping.
    PdfRequiresUnicodeFonts,
    /// Upstream PDF has one page and no line wrapping. Use DOCX or Markdown.
    PdfRequiresPagination,
}

impl ResearchDocument {
    fn validate(&self) -> Result<(), ArtifactError> {
        if self.title.trim().is_empty()
            || self.language.trim().is_empty()
            || self.statements.is_empty()
            || self
                .statements
                .iter()
                .any(|statement| statement.text.trim().is_empty())
        {
            return Err(ArtifactError::EmptyDocument);
        }
        let mut ids = std::collections::BTreeSet::new();
        for reference in &self.references {
            if reference.id.is_empty()
                || !reference
                    .id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-')
                || !ids.insert(reference.id.as_str())
                || reference.title.trim().is_empty()
                || reference.captured_at.trim().is_empty()
                || !reference.url.starts_with("https://")
                || reference.url.chars().any(char::is_whitespace)
            {
                return Err(ArtifactError::InvalidReference);
            }
        }
        if self.statements.iter().any(|statement| {
            statement.source_ids.is_empty()
                || statement
                    .source_ids
                    .iter()
                    .any(|id| !ids.contains(id.as_str()))
        }) {
            return Err(ArtifactError::MissingEvidence);
        }
        Ok(())
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = vec![self.title.clone()];
        lines.extend(
            self.statements.iter().map(|statement| {
                format!("{} [{}]", statement.text, statement.source_ids.join(", "))
            }),
        );
        lines.extend(self.references.iter().map(|reference| {
            format!(
                "[{}] {} - {} ({})",
                reference.id, reference.title, reference.url, reference.captured_at
            )
        }));
        lines
    }

    /// Render a fully synthesized report. `stem` is a download name, never a path.
    pub fn render(
        &self,
        format: ArtifactFormat,
        stem: &str,
    ) -> Result<ResearchArtifact, ArtifactError> {
        self.validate()?;
        if stem.is_empty()
            || stem.len() > 100
            || !stem
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            return Err(ArtifactError::UnsafeFilename);
        }
        let lines = self.lines();
        if lines.iter().any(|line| {
            line.chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        }) {
            return Err(ArtifactError::InvalidContent);
        }
        let bytes = if format == ArtifactFormat::Markdown {
            lines.join("\n\n").into_bytes()
        } else {
            if format == ArtifactFormat::Pdf {
                if lines.iter().any(|line| !line.is_ascii()) {
                    return Err(ArtifactError::PdfRequiresUnicodeFonts);
                }
                if lines.len() > 28
                    || lines.iter().enumerate().any(|(index, line)| {
                        // The upstream PDF profile emits one unwrapped text
                        // run per block. Estimate Helvetica's standard
                        // character widths to keep each block in the page's
                        // 468 point line area, including the larger title.
                        let font_size = if index == 0 { 24 } else { 12 };
                        pdf_text_width_units(line) * font_size > 468_000
                            || line.contains(['\n', '\r'])
                    })
                {
                    return Err(ArtifactError::PdfRequiresPagination);
                }
            }
            let html = lines
                .iter()
                .enumerate()
                .fold(String::new(), |mut html, (index, line)| {
                    use std::fmt::Write as _;
                    let tag = if index == 0 { "h1" } else { "p" };
                    let escaped = line
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    let _ = write!(html, "<{tag}>{escaped}</{tag}>");
                    html
                });
            let target = if format == ArtifactFormat::Pdf {
                "PDF"
            } else {
                "DOCX"
            };
            let conversion =
                crate::document_formats::convert_document_format("HTML", target, &html)
                    .ok_or(ArtifactError::ConversionUnavailable)?;
            conversion
                .package_bytes
                .ok_or(ArtifactError::ConversionUnavailable)?
        };
        Ok(ResearchArtifact {
            filename: format!("{stem}.{}", format.extension()),
            mime_type: format.mime_type(),
            language: self.language.clone(),
            bytes,
        })
    }
}

/// Standard Helvetica glyph widths in thousandths of an em for ASCII text.
/// The report renderer emits regular Helvetica runs only.
fn pdf_text_width_units(text: &str) -> usize {
    text.bytes()
        .map(|character| match character {
            b' ' | b'!' | b'.' | b',' | b':' | b';' | b'I' => 278,
            b'"' => 355,
            b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'0'..=b'9'
            | b'?'
            | b'L'
            | b'_'
            | b'a'
            | b'b'
            | b'd'
            | b'e'
            | b'g'
            | b'h'
            | b'n'
            | b'o'
            | b'p'
            | b'q'
            | b'u' => 556,
            b'@' => 1_015,
            b'A' | b'B' | b'E' | b'P' | b'R' | b'S' | b'X' | b'K' | b'V' | b'Y' => 667,
            b'C' | b'D' | b'H' | b'N' | b'U' | b'w' => 722,
            b'F' | b'T' | b'Z' => 611,
            b'G' | b'O' | b'Q' => 778,
            b'J' | b'c' | b's' | b'v' | b'x' | b'z' | b'k' | b'y' => 500,
            b'M' | b'm' => 833,
            b'W' => 944,
            b'\'' | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'-' | b'/' | b'\\' | b'f' | b'r'
            | b't' => 333,
            b'*' | b'+' | b'<' | b'=' | b'>' => 584,
            b'i' | b'j' | b'l' => 222,
            _ => 1_000,
        })
        .sum()
}
