//! Wiktionary API client + minimal wikitext parser.
//!
//! Wiktionary's translation tables are encoded with a small set of
//! templates:
//!
//! ```wikitext
//! * Russian: {{t+|ru|привет}}, {{t|ru|здравствуйте}}
//! * Hindi: {{t|hi|नमस्ते}}, {{t|hi|नमस्कार}}
//! * Chinese:
//! *: Mandarin: {{t+|cmn|你好|tr=nǐhǎo}}, {{t+|zh|您好}}
//! ```
//!
//! The patterns are stable enough that a parser is preferable to a
//! generic wikitext renderer. We only need to extract the candidate
//! surface forms per language; everything else (qualifiers,
//! transliterations) is discarded.
//!
//! Reverse direction (RU → EN) is symmetric: the Russian Wiktionary
//! page for `как дела` records translations under `=== Перевод ===`
//! using `{{перев-блок}}` and `|en=[[how do you do]], [[how are you]]`.

use super::http::{HttpClient, HttpError};

/// Wiktionary candidate returned by [`Wiktionary::translations`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiktionaryCandidate {
    /// Target-language surface form, with wiki markup stripped.
    pub surface: String,
    /// Optional qualifier such as `informal`, `formal`, `archaic`. We
    /// keep this for traceability — the demo currently prefers
    /// unqualified or `informal` matches.
    pub qualifier: Option<String>,
}

/// Wiktionary client. Always constructed against a specific edition
/// (`en` for English Wiktionary, `ru` for Russian, etc).
pub struct Wiktionary<'a, T: HttpClient + ?Sized> {
    edition: String,
    http: &'a T,
}

impl<'a, T: HttpClient + ?Sized> Wiktionary<'a, T> {
    /// Construct a client for a specific Wiktionary edition.
    pub fn new(edition: &str, http: &'a T) -> Self {
        Self {
            edition: edition.to_owned(),
            http,
        }
    }

    /// Fetch the raw wikitext for `page` on this edition of Wiktionary.
    /// Returns the body verbatim — callers do their own parsing.
    pub fn wikitext(&self, page: &str) -> Result<String, HttpError> {
        let url = format!(
            "https://{edition}.wiktionary.org/w/api.php?action=parse&page={page}\
             &prop=wikitext&formatversion=2&format=json&redirects=1",
            edition = self.edition,
            page = url_encode(page),
        );
        let body = self.http.get(&url)?;
        Ok(extract_wikitext_from_parse_response(&body).unwrap_or(body))
    }

    /// Look up translations of `page` into `target_lang` using this
    /// edition's translation tables.
    ///
    /// Strategy:
    /// 1. Fetch `page` wikitext.
    /// 2. Locate the `Translations` section (English Wiktionary) or
    ///    `=== Перевод ===` block (Russian Wiktionary).
    /// 3. Extract `{{t|...}}` / `{{t+|...}}` / `|<lang>=[[...]]`
    ///    candidates for the requested target language.
    pub fn translations(
        &self,
        page: &str,
        target_lang: &str,
    ) -> Result<Vec<WiktionaryCandidate>, HttpError> {
        let wikitext = self.wikitext(page)?;
        Ok(extract_translations(&wikitext, target_lang))
    }

    /// Like [`Self::translations`], but returns candidates grouped by
    /// sense block. Each block corresponds to one `{{trans-top|gloss}}`
    /// segment on English Wiktionary, or one `{{перев-блок|...}}` template
    /// on Russian Wiktionary. Polysemous words (e.g. "привет" — noun
    /// "regards" vs interjection "hello") therefore produce one block per
    /// sense, letting callers pick the right sense rather than mixing
    /// them.
    pub fn translation_blocks(
        &self,
        page: &str,
        target_lang: &str,
    ) -> Result<Vec<Vec<WiktionaryCandidate>>, HttpError> {
        let wikitext = self.wikitext(page)?;
        Ok(extract_translation_blocks(&wikitext, target_lang))
    }
}

/// Decode the JSON envelope returned by `action=parse&prop=wikitext`.
/// We do a string-search extraction rather than a full JSON parse to
/// keep the call surface tight (the response is single-key and we are
/// committed to `formatversion=2`).
fn extract_wikitext_from_parse_response(body: &str) -> Option<String> {
    let needle = "\"wikitext\":\"";
    let start = body.find(needle)? + needle.len();
    let mut out = String::with_capacity(body.len() / 2);
    let mut iter = body[start..].chars();
    while let Some(character) = iter.next() {
        if character == '\\' {
            match iter.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'u' => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        hex.push(iter.next()?);
                    }
                    let codepoint = u32::from_str_radix(&hex, 16).ok()?;
                    if (0xD800..=0xDBFF).contains(&codepoint) {
                        // High surrogate — expect a low surrogate next.
                        if iter.next() != Some('\\') || iter.next() != Some('u') {
                            return None;
                        }
                        let mut low_hex = String::with_capacity(4);
                        for _ in 0..4 {
                            low_hex.push(iter.next()?);
                        }
                        let low = u32::from_str_radix(&low_hex, 16).ok()?;
                        let combined = 0x1_0000 + ((codepoint - 0xD800) << 10) + (low - 0xDC00);
                        out.push(char::from_u32(combined)?);
                    } else if let Some(character) = char::from_u32(codepoint) {
                        out.push(character);
                    }
                }
                other => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else if character == '"' {
            return Some(out);
        } else {
            out.push(character);
        }
    }
    None
}

/// Extract translation candidates for `target_lang` from a wikitext blob.
///
/// The result is a deduplicated flat list. Use
/// [`extract_translation_blocks`] when callers need the sense-level
/// grouping (e.g. for picking between the noun and interjection senses
/// of "привет").
#[must_use]
pub fn extract_translations(wikitext: &str, target_lang: &str) -> Vec<WiktionaryCandidate> {
    let mut out: Vec<WiktionaryCandidate> = Vec::new();
    for block in extract_translation_blocks(wikitext, target_lang) {
        out.extend(block);
    }
    deduplicate(out)
}

/// Extract translation candidates grouped by sense block.
///
/// English Wiktionary delimits senses with `{{trans-top|gloss}}` …
/// `{{trans-bottom}}` template pairs (siblings, not nested). Russian
/// Wiktionary uses one `{{перев-блок|...}}` template per sense. We treat
/// each delimiter as one block so polysemous words don't blur their
/// senses together at the extraction layer.
#[must_use]
pub fn extract_translation_blocks(
    wikitext: &str,
    target_lang: &str,
) -> Vec<Vec<WiktionaryCandidate>> {
    let mut blocks: Vec<Vec<WiktionaryCandidate>> = Vec::new();
    let trans_segments = scan_trans_segments(wikitext);
    if trans_segments.is_empty() {
        // Pages without any `{{trans-top}}` markers (e.g. minimal entries)
        // still need extraction — treat the whole document as a single
        // block.
        let candidates = deduplicate(extract_t_templates(wikitext, target_lang));
        if !candidates.is_empty() {
            blocks.push(candidates);
        }
    } else {
        for segment in trans_segments {
            let candidates = deduplicate(extract_t_templates(segment, target_lang));
            if !candidates.is_empty() {
                blocks.push(candidates);
            }
        }
    }
    for block in scan_templates(wikitext, &["перев-блок", "перев"]) {
        let mut candidates: Vec<WiktionaryCandidate> = Vec::new();
        for line in block.split('|') {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if !lang_matches(key.trim(), target_lang) {
                continue;
            }
            for entry in split_inline_translations(value) {
                let surface = clean_surface(&entry);
                if !surface.is_empty() {
                    candidates.push(WiktionaryCandidate {
                        surface,
                        qualifier: None,
                    });
                }
            }
        }
        let candidates = deduplicate(candidates);
        if !candidates.is_empty() {
            blocks.push(candidates);
        }
    }
    // Chinese Wiktionary: `{{翻譯-頂}}` … `{{翻譯-底}}` blocks delimit
    // translation tables. Inside each block, every line takes the form
    // `*<lang_name>：[[surface]]; [[surface]] …` where `<lang_name>` is
    // the Chinese name of the language and `：` is the fullwidth colon.
    for segment in scan_named_segments(wikitext, "{{翻譯-頂", "{{翻譯-底") {
        let candidates = deduplicate(extract_chinese_translation_lines(segment, target_lang));
        if !candidates.is_empty() {
            blocks.push(candidates);
        }
    }
    blocks
}

/// Extract translation candidates from `{{t|...}}`-family templates within
/// `wikitext`. Used per-segment to keep sense blocks distinct.
fn extract_t_templates(wikitext: &str, target_lang: &str) -> Vec<WiktionaryCandidate> {
    let mut out: Vec<WiktionaryCandidate> = Vec::new();
    for template in scan_templates(wikitext, &["t", "t+", "t-", "tt", "tt+", "tt-"]) {
        let Some((_, args)) = template.split_once('|') else {
            continue;
        };
        let mut parts = args.split('|');
        let Some(language) = parts.next() else {
            continue;
        };
        if !lang_matches(language.trim(), target_lang) {
            continue;
        }
        let Some(surface_raw) = parts.next() else {
            continue;
        };
        let surface = clean_surface(surface_raw);
        if surface.is_empty() {
            continue;
        }
        let qualifier = parse_qualifier_args(parts);
        out.push(WiktionaryCandidate { surface, qualifier });
    }
    out
}

/// Extract translation candidates from a zh.wiktionary block, where
/// each line has the form `*<lang_name>：[[surface]]; [[surface]] …` and
/// `<lang_name>` is the Chinese name of the target language (e.g. 英语 for
/// English, 俄语 for Russian).
fn extract_chinese_translation_lines(block: &str, target_lang: &str) -> Vec<WiktionaryCandidate> {
    let mut out: Vec<WiktionaryCandidate> = Vec::new();
    let target_names = chinese_language_names_for(target_lang);
    if target_names.is_empty() {
        return out;
    }
    for line in block.lines() {
        // Lines we care about start with `*` (Wikitext bullet).
        let trimmed = line.trim_start_matches(|c: char| c == '*' || c.is_whitespace());
        if trimmed.is_empty() || trimmed == line {
            // Either empty or no bullet was stripped — not a bullet line.
            if !line.trim_start().starts_with('*') {
                continue;
            }
        }
        // Split on the fullwidth colon `：` or the ASCII `:` once.
        let Some((lang_name_raw, value)) = split_on_colon(trimmed) else {
            continue;
        };
        let lang_name = lang_name_raw.trim();
        if !target_names.contains(&lang_name) {
            continue;
        }
        for entry in split_inline_translations(value) {
            let surface = clean_surface(&entry);
            if !surface.is_empty() {
                out.push(WiktionaryCandidate {
                    surface,
                    qualifier: None,
                });
            }
        }
    }
    out
}

/// Split on the first occurrence of `：` (U+FF1A fullwidth colon) or `:`,
/// preferring the fullwidth variant when both appear.
fn split_on_colon(input: &str) -> Option<(&str, &str)> {
    let full = input.find('：');
    let half = input.find(':');
    let idx = match (full, half) {
        (Some(f), Some(h)) => f.min(h),
        (Some(f), None) => f,
        (None, Some(h)) => h,
        (None, None) => return None,
    };
    let (left, rest) = input.split_at(idx);
    // Skip the colon character (variable byte length in UTF-8).
    let colon_len = rest.chars().next()?.len_utf8();
    Some((left, &rest[colon_len..]))
}

/// Map a BCP-47 language tag to one or more Chinese names used by
/// zh.wiktionary to label translation rows. The list is intentionally
/// short — we only need the languages exercised by the integration tests
/// — and uses both Simplified (语) and Traditional (語) variants.
fn chinese_language_names_for(target_lang: &str) -> Vec<&'static str> {
    match target_lang.to_ascii_lowercase().as_str() {
        "en" => vec!["英语", "英語"],
        "ru" => vec!["俄语", "俄語"],
        "fr" => vec!["法语", "法語"],
        "de" => vec!["德语", "德語"],
        "es" => vec!["西班牙语", "西班牙語"],
        "it" => vec!["意大利语", "意大利語"],
        "pt" => vec!["葡萄牙语", "葡萄牙語"],
        "ja" => vec!["日语", "日語"],
        "ko" => vec!["韩语", "韓語"],
        "hi" => vec!["印地语", "印地語"],
        "ar" => vec!["阿拉伯语", "阿拉伯語"],
        "zh" | "cmn" => vec!["汉语", "漢語", "中文"],
        _ => Vec::new(),
    }
}

/// Scan `wikitext` for segments delimited by `open` … `close` template
/// markers (matching the *prefix* of the template name to allow optional
/// arguments). Each segment is the text between an `open` opening and the
/// next `close` (or the next `open`, whichever comes first).
fn scan_named_segments<'a>(wikitext: &'a str, open: &str, close: &str) -> Vec<&'a str> {
    let mut segments: Vec<&str> = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel_top) = wikitext[cursor..].find(open) {
        let top = cursor + rel_top;
        let after_top = top + open.len();
        let bottom = wikitext[after_top..].find(close).map(|i| after_top + i);
        let next_top = wikitext[after_top..].find(open).map(|i| after_top + i);
        let end = match (bottom, next_top) {
            (Some(b), Some(n)) => b.min(n),
            (Some(b), None) => b,
            (None, Some(n)) => n,
            (None, None) => wikitext.len(),
        };
        segments.push(&wikitext[top..end]);
        cursor = end;
    }
    segments
}

/// Split `wikitext` into segments delimited by `{{trans-top}}`/`{{trans-bottom}}`
/// markers. Each segment is the text between a `{{trans-top}}` and the
/// next `{{trans-bottom}}` (or the next `{{trans-top}}`, whichever comes
/// first). Text outside such pairs is dropped — it's never a translation
/// table.
fn scan_trans_segments(wikitext: &str) -> Vec<&str> {
    let mut segments: Vec<&str> = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel_top) = wikitext[cursor..].find("{{trans-top") {
        let top = cursor + rel_top;
        let after_top = top + "{{trans-top".len();
        let bottom = wikitext[after_top..]
            .find("{{trans-bottom")
            .map(|i| after_top + i);
        let next_top = wikitext[after_top..]
            .find("{{trans-top")
            .map(|i| after_top + i);
        let end = match (bottom, next_top) {
            (Some(b), Some(n)) => b.min(n),
            (Some(b), None) => b,
            (None, Some(n)) => n,
            (None, None) => wikitext.len(),
        };
        segments.push(&wikitext[top..end]);
        cursor = end;
    }
    segments
}

/// Iterate over `{{<name>|...}}` templates whose name appears in `names`.
/// Returns the inner body (including arguments after the name).
///
/// We must descend into templates we *don't* match, because English
/// Wiktionary nests its translation tables inside `{{multitrans|data=...}}`
/// wrappers: skipping the outer template body would discard every
/// `{{t+|ru|...}}` inside.
fn scan_templates(wikitext: &str, names: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let bytes = wikitext.as_bytes();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut depth: u32 = 1;
            let mut j = start;
            while j + 1 < bytes.len() && depth > 0 {
                if bytes[j] == b'{' && bytes[j + 1] == b'{' {
                    depth += 1;
                    j += 2;
                } else if bytes[j] == b'}' && bytes[j + 1] == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    j += 2;
                } else {
                    j += 1;
                }
            }
            if depth != 0 {
                // Unbalanced wikitext — advance past the opening braces
                // rather than abandoning the rest of the document.
                i += 2;
                continue;
            }
            let body = &wikitext[start..j];
            let name_end = body
                .find('|')
                .unwrap_or_else(|| body.find('}').unwrap_or(body.len()));
            let name = body[..name_end].trim();
            if names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
                out.push(body.to_owned());
                // Skip past this match. Matched templates (`{{t+|ru|…}}`,
                // `{{перев-блок|…}}`) don't nest within themselves, so
                // there's no need to descend further.
                i = j + 2;
            } else {
                // Descend into the unmatched template to find nested
                // candidates such as `{{t+|ru|…}}` inside
                // `{{multitrans|data=…}}`.
                i += 2;
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Strip `[[link]]`, `[[alt|display]]`, `{{nested|...}}`, `<ref>...</ref>`
/// and trailing punctuation from a raw template surface argument.
#[must_use]
pub fn clean_surface(raw: &str) -> String {
    // Drop `alt=`, `tr=`, `lit=` etc — they are key=value pairs and not
    // the surface form. The caller already split on `|`, so this
    // protects against malformed input where a `=` snuck through.
    let raw = if let Some((key, value)) = raw.split_once('=') {
        if !key.contains(' ')
            && !key.contains('[')
            && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            value
        } else {
            raw
        }
    } else {
        raw
    };
    let mut out = String::with_capacity(raw.len());
    let mut iter = raw.chars().peekable();
    while let Some(character) = iter.next() {
        match character {
            '[' if iter.peek() == Some(&'[') => {
                iter.next();
                // Read until "]]"
                let mut inner = String::new();
                while let Some(c) = iter.next() {
                    if c == ']' && iter.peek() == Some(&']') {
                        iter.next();
                        break;
                    }
                    inner.push(c);
                }
                // For `[[alt|display]]` keep the display form.
                let display = inner.rsplit_once('|').map_or(inner.as_str(), |s| s.1);
                out.push_str(display);
            }
            '{' if iter.peek() == Some(&'{') => {
                iter.next();
                let mut depth: u32 = 1;
                while let Some(c) = iter.next() {
                    if c == '{' && iter.peek() == Some(&'{') {
                        iter.next();
                        depth += 1;
                    } else if c == '}' && iter.peek() == Some(&'}') {
                        iter.next();
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
            }
            '<' => {
                // Skip <ref>...</ref> or stray tags.
                for c in iter.by_ref() {
                    if c == '>' {
                        break;
                    }
                }
            }
            _ => out.push(character),
        }
    }
    let trimmed = out.trim().trim_end_matches(',').trim();
    // Russian Wiktionary lists transliterations in parentheses after the
    // translation: `[[hello]] (хэлло)`. Strip the trailing parenthetical
    // so the surface stays a pure lemma. We only strip a *single* trailing
    // group of balanced parentheses so we do not accidentally remove
    // parts of e.g. `(?)` glosses inside the lemma.
    let trimmed = strip_trailing_parenthetical(trimmed);
    // Russian Wiktionary marks stressed vowels with U+0301 (combining
    // acute accent) inside translation templates: `{{t+|ru|приве́т}}`.
    // The actual orthographic form does not carry the accent — strip it
    // so callers see "привет" rather than "приве́т".
    strip_combining_accents(&trimmed)
}

/// Strip a single trailing `(...)` group from `input`, but only when the
/// parenthetical sits at the very end and contains balanced parentheses.
fn strip_trailing_parenthetical(input: &str) -> String {
    let bytes = input.as_bytes();
    if bytes.last() != Some(&b')') {
        return input.to_owned();
    }
    let mut depth: i32 = 0;
    let mut open_at: Option<usize> = None;
    for (idx, ch) in input.char_indices().rev() {
        match ch {
            ')' => depth += 1,
            '(' => {
                depth -= 1;
                if depth == 0 {
                    open_at = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(open) = open_at else {
        return input.to_owned();
    };
    let prefix = input[..open].trim_end();
    if prefix.is_empty() {
        input.to_owned()
    } else {
        prefix.to_owned()
    }
}

/// Remove combining acute / grave accents that Wiktionary uses to mark
/// stress in Cyrillic translation candidates. Other combining marks are
/// preserved (e.g. Hindi nukta, Vietnamese tone marks on combining forms).
fn strip_combining_accents(input: &str) -> String {
    input
        .chars()
        .filter(|c| !matches!(*c, '\u{0300}' | '\u{0301}' | '\u{0304}' | '\u{0306}'))
        .collect()
}

fn split_inline_translations(value: &str) -> Vec<String> {
    let mut depth: u32 = 0;
    let mut current = String::new();
    let mut parts: Vec<String> = Vec::new();
    for character in value.chars() {
        match character {
            '[' | '{' => {
                depth += 1;
                current.push(character);
            }
            ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            // Russian Wiktionary frequently uses terminal punctuation to
            // separate translation variants, e.g.
            // `|en=[[how do you do]]? [[hello]], [[good morning]]!`. Treat
            // any sentence-terminal punctuation at depth 0 as a delimiter
            // so each `[[...]]` lemma becomes its own candidate.
            ',' | ';' | '?' | '!' | '.' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(character),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts.into_iter().map(|p| p.trim().to_owned()).collect()
}

fn parse_qualifier_args<'a, I: Iterator<Item = &'a str>>(parts: I) -> Option<String> {
    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            if matches!(key.trim(), "q" | "qual" | "qualifier" | "n") {
                return Some(value.trim().to_owned());
            }
        }
    }
    None
}

fn lang_matches(template_lang: &str, requested: &str) -> bool {
    if template_lang.eq_ignore_ascii_case(requested) {
        return true;
    }
    // Macro-language fallback: Chinese (zh) ↔ Mandarin (cmn), Norwegian
    // (no) ↔ Bokmål (nb)/Nynorsk (nn). Wiktionary uses the more specific
    // tag for translation tables; the request usually uses the macro tag.
    matches!(
        (requested, template_lang),
        ("zh", "cmn" | "yue" | "wuu") | ("no", "nb" | "nn")
    )
}

fn deduplicate(input: Vec<WiktionaryCandidate>) -> Vec<WiktionaryCandidate> {
    let mut out: Vec<WiktionaryCandidate> = Vec::with_capacity(input.len());
    for candidate in input {
        if !out.iter().any(|c| c.surface == candidate.surface) {
            out.push(candidate);
        }
    }
    out
}

/// Percent-encode a UTF-8 string for safe inclusion in a Wikimedia URL.
///
/// Spaces become `_` (Wikimedia convention for page titles), letters and
/// digits pass through, everything else is `%XX`-encoded.
#[must_use]
pub fn url_encode(value: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('_'),
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

/// Percent-encode a UTF-8 string for use as a URL **query-string value**.
///
/// Unlike [`url_encode`], spaces become `%20` (the RFC 3986 query encoding)
/// because the Wikidata SPARQL endpoint rejects underscores inside the
/// `?query=` body — SPARQL keywords like `SELECT` and `WHERE` are
/// whitespace-separated and breaking them with `_` produces HTTP 400.
#[must_use]
pub fn query_encode(value: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

#[path = "../source_tests/translation/wiktionary/tests.rs"]
mod tests;
