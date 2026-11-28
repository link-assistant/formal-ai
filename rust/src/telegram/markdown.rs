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

        if in_code_block {
            rendered.push_str(&html_escape(line));
            rendered.push('\n');
            continue;
        }

        if let Some(quote) = trimmed.strip_prefix('>') {
            rendered.push_str("<blockquote>");
            rendered.push_str(&telegram_inline_html(quote.trim_start()));
            rendered.push_str("</blockquote>\n");
        } else {
            rendered.push_str(&telegram_inline_html(line));
            rendered.push('\n');
        }
    }

    if in_code_block {
        rendered.push_str("</code></pre>\n");
    }

    rendered.trim_end().to_owned()
}

/// Render Telegram's supported inline Markdown subset while escaping every
/// character that is not part of a recognized construct. Search source cards
/// use links, bold titles, and machine-readable code spans.
fn telegram_inline_html(markdown: &str) -> String {
    let mut rendered = String::new();
    let mut rest = markdown;
    while !rest.is_empty() {
        if let Some(inner) = rest.strip_prefix("**")
            && let Some(end) = inner.find("**")
        {
            rendered.push_str("<b>");
            rendered.push_str(&telegram_inline_html(&inner[..end]));
            rendered.push_str("</b>");
            rest = &inner[end + 2..];
            continue;
        }
        if let Some(inner) = rest.strip_prefix('`')
            && let Some(end) = inner.find('`')
        {
            rendered.push_str("<code>");
            rendered.push_str(&html_escape(&inner[..end]));
            rendered.push_str("</code>");
            rest = &inner[end + 1..];
            continue;
        }
        if let Some(label) = rest.strip_prefix('[')
            && let Some(label_end) = label.find("](")
        {
            let after_label = &label[label_end + 2..];
            if let Some(url_end) = after_label.find(')') {
                let url = &after_label[..url_end];
                if url.starts_with("https://") || url.starts_with("http://") {
                    rendered.push_str("<a href=\"");
                    rendered.push_str(&html_escape(url));
                    rendered.push_str("\">");
                    rendered.push_str(&html_escape(&label[..label_end]));
                    rendered.push_str("</a>");
                    rest = &after_label[url_end + 1..];
                    continue;
                }
            }
        }
        let character = rest.chars().next().expect("non-empty inline text");
        rendered.push_str(&html_escape(&character.to_string()));
        rest = &rest[character.len_utf8()..];
    }
    rendered
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

pub(super) fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
