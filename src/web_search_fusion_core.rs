//! Seed-backed statement fusion for the browser's Rust→WASM search path.
//!
//! The browser sends already captured search excerpts through a compact,
//! percent-encoded row protocol. This module formalizes each sentence, merges
//! equivalent meanings, preserves contradictions, ranks the smallest useful
//! answer, and serializes the provenance model as JSON. Keeping this module
//! free of browser APIs lets the native crate and the `no_std` WASM worker test
//! the exact same deterministic transformation.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Write as _;

const WIKIDATA_MEANINGS: &str = include_str!("../data/seed/meanings-wikidata.lino");
const STATEMENT_MEANINGS: &str = include_str!("../data/seed/meanings-statement-merge.lino");
const ENTITY_ROLE: &str = "wikidata_entity_anchor";
const RELATION_ROLE: &str = "binary_relation_property";
const FUNCTION_WORD_ROLE: &str = "statement_function_word";
const NEGATION_ROLE: &str = "statement_negation_cue";
const MAX_MEANINGS: usize = 3;
const MAX_SOURCE_FRAGMENTS: usize = 8;

#[derive(Clone, Debug)]
struct Form {
    language: String,
    text: String,
    action: bool,
}

#[derive(Clone, Debug, Default)]
struct Meaning {
    grounded_in: String,
    role: String,
    forms: Vec<Form>,
}

#[derive(Clone, Debug)]
struct VocabularyForm {
    language: String,
    text: String,
}

#[derive(Clone, Debug, Default)]
struct Request {
    query: String,
    language: String,
    read_more: String,
    via: String,
    other_sources: String,
    sources: Vec<Source>,
}

#[derive(Clone, Debug)]
struct Source {
    url: String,
    title: String,
    excerpt: String,
    tier: String,
    language: String,
    providers: String,
    retrieval_rank: u32,
    alternate: bool,
}

#[derive(Clone, Debug)]
struct SourceCard {
    url: String,
    title: String,
    quote: String,
    tier: String,
    language: String,
    providers: String,
    retrieval_rank: u32,
    alternate: bool,
}

#[derive(Clone, Debug)]
struct Formalization {
    denied: bool,
    meaning: String,
    semantic: Vec<String>,
    rendered: String,
}

#[derive(Clone, Debug)]
struct Node {
    denied: bool,
    meaning: String,
    semantic: Vec<String>,
    text: String,
    sources: Vec<SourceCard>,
    support: u32,
    conflict: bool,
    posterior: f64,
    weight: u32,
    best_retrieval_rank: u32,
}

/// Fuse the row protocol used by the browser worker and return a JSON object
/// containing Markdown lines, trace evidence, and structured statements.
#[must_use]
pub fn fuse_statement_search_payload(payload: &str) -> String {
    let request = parse_request(payload);
    let anchors = parse_anchor_meanings();
    let vocabulary = parse_statement_vocabulary();
    let mut evidence = Vec::new();
    let mut nodes = build_nodes(&request, &anchors, &vocabulary, &mut evidence);
    rank_nodes(&mut nodes);
    let selected = smallest_sufficient(nodes);
    render_json(&request, &selected, &mut evidence)
}

fn parse_request(payload: &str) -> Request {
    let mut request = Request::default();
    for line in payload.lines() {
        let fields = line
            .split('\t')
            .map(decode_uri_component)
            .collect::<Vec<_>>();
        match fields.first().map(String::as_str) {
            Some("Q") => {
                request.query = field(&fields, 1);
                request.language = nonempty(field(&fields, 2), "en");
                request.read_more = field(&fields, 3);
                request.via = field(&fields, 4);
                request.other_sources = field(&fields, 5);
            }
            Some("S") if !field(&fields, 1).is_empty() => {
                let fallback_rank = u32::try_from(request.sources.len() + 1).unwrap_or(u32::MAX);
                let retrieval_rank = field(&fields, 7)
                    .parse::<u32>()
                    .ok()
                    .filter(|rank| *rank > 0)
                    .unwrap_or(fallback_rank);
                request.sources.push(Source {
                    url: field(&fields, 1),
                    title: field(&fields, 2),
                    excerpt: field(&fields, 3),
                    tier: canonical_tier(&field(&fields, 4)).to_string(),
                    language: field(&fields, 5),
                    providers: field(&fields, 6),
                    retrieval_rank,
                    alternate: field(&fields, 8) == "alternate",
                });
            }
            _ => {}
        }
    }
    request
}

fn field(fields: &[String], index: usize) -> String {
    fields.get(index).cloned().unwrap_or_default()
}

fn nonempty(value: String, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn parse_anchor_meanings() -> Vec<Meaning> {
    let mut meanings = Vec::new();
    let mut current: Option<Meaning> = None;
    let mut language = String::new();
    for raw in WIKIDATA_MEANINGS.lines() {
        let indent = raw.len() - raw.trim_start().len();
        let line = raw.trim();
        if indent == 2 && !line.is_empty() {
            if let Some(meaning) = current.take() {
                meanings.push(meaning);
            }
            current = Some(Meaning::default());
            language.clear();
        } else if indent == 4 && line.starts_with("grounded-in ") {
            if let Some(meaning) = current.as_mut() {
                meaning.grounded_in = line[12..].trim().to_string();
            }
        } else if indent == 4 && line.starts_with("role ") {
            if let Some(meaning) = current.as_mut() {
                meaning.role = line[5..].trim().to_string();
            }
        } else if indent == 4 && line.starts_with("lexeme ") {
            language = line[7..].trim().to_string();
        } else if indent == 8 && line.starts_with("text ") {
            if let Some(meaning) = current.as_mut() {
                meaning.forms.push(Form {
                    language: language.clone(),
                    text: unquote(line[5..].trim()).to_string(),
                    action: false,
                });
            }
        } else if indent == 8 && line.starts_with("action ") {
            if let Some(form) = current
                .as_mut()
                .and_then(|meaning| meaning.forms.last_mut())
            {
                form.action = true;
            }
        }
    }
    if let Some(meaning) = current {
        meanings.push(meaning);
    }
    meanings
}

fn parse_statement_vocabulary() -> Vec<(String, VocabularyForm)> {
    let mut forms = Vec::new();
    let mut role = String::new();
    let mut language = String::new();
    for raw in STATEMENT_MEANINGS.lines() {
        let indent = raw.len() - raw.trim_start().len();
        let line = raw.trim();
        if indent == 2 {
            role.clear();
            language.clear();
        } else if indent == 4 && line.starts_with("role ") {
            role = line[5..].trim().to_string();
        } else if indent == 4 && line.starts_with("lexeme ") {
            language = line[7..].trim().to_string();
        } else if indent == 8
            && line.starts_with("text ")
            && (role == FUNCTION_WORD_ROLE || role == NEGATION_ROLE)
        {
            forms.push((
                role.clone(),
                VocabularyForm {
                    language: language.clone(),
                    text: normalized(unquote(line[5..].trim())),
                },
            ));
        }
    }
    forms
}

fn build_nodes(
    request: &Request,
    anchors: &[Meaning],
    vocabulary: &[(String, VocabularyForm)],
    evidence: &mut Vec<String>,
) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut trusted_urls = Vec::new();
    for source in &request.sources {
        let tier_points = tier_points(&source.tier);
        let language = if source.language.is_empty() {
            request.language.as_str()
        } else {
            source.language.as_str()
        };
        let title = nonempty(source.title.clone(), &source.url);
        let mut excerpt = nonempty(source.excerpt.clone(), &title);
        let prefix = format!("{title} - ");
        if excerpt.starts_with(&prefix) {
            excerpt = excerpt[prefix.len()..].trim().to_string();
        }
        for fragment in sentences(&excerpt, MAX_SOURCE_FRAGMENTS) {
            let formal = formalize(&fragment, language, &request.language, anchors, vocabulary);
            evidence.push(format!(
                "search_fusion:formalization:{}:{}:{}",
                source.url,
                if formal.denied { "denied" } else { "asserted" },
                if formal.meaning.is_empty() {
                    "empty"
                } else {
                    &formal.meaning
                }
            ));
            if tier_points == 0 {
                evidence.push(format!("search_fusion:ignored:{}:unoriginal", source.url));
                continue;
            }
            if formal.meaning.is_empty() {
                continue;
            }
            push_unique(&mut trusted_urls, &source.url);
            let card = SourceCard {
                url: source.url.clone(),
                title: title.clone(),
                quote: fragment,
                tier: source.tier.clone(),
                language: language.to_string(),
                providers: source.providers.clone(),
                retrieval_rank: source.retrieval_rank,
                alternate: source.alternate,
            };
            if let Some(node) = nodes
                .iter_mut()
                .find(|node| node.denied == formal.denied && node.meaning == formal.meaning)
            {
                if !node.sources.iter().any(|existing| existing.url == card.url) {
                    evidence.push(format!(
                        "search_fusion:merge:{}:{}",
                        statement_key(node),
                        card.url
                    ));
                    node.sources.push(card);
                    node.support += tier_points;
                    node.best_retrieval_rank = node.best_retrieval_rank.min(source.retrieval_rank);
                }
            } else {
                nodes.push(Node {
                    denied: formal.denied,
                    meaning: formal.meaning,
                    semantic: formal.semantic,
                    text: formal.rendered,
                    sources: vec![card],
                    support: tier_points,
                    conflict: false,
                    posterior: 1.0,
                    weight: 0,
                    best_retrieval_rank: source.retrieval_rank,
                });
            }
        }
    }
    let trusted_count = u64::try_from(trusted_urls.len().max(1)).unwrap_or(u64::MAX);
    let snapshot = nodes.clone();
    for node in &mut nodes {
        let opposing = snapshot
            .iter()
            .find(|candidate| candidate.denied != node.denied && candidate.meaning == node.meaning);
        let oppose = opposing.map_or(0, |candidate| candidate.support);
        node.conflict = opposing.is_some();
        node.posterior = posterior(node.support, oppose);
        let support = u64::from(node.support);
        let total = u64::from((node.support + oppose).max(1));
        let denominator = 100 * trusted_count * total;
        let weighted = (60 * denominator + 200 * support * support) / (3 * denominator);
        node.weight = u32::try_from(weighted).unwrap_or(u32::MAX);
        node.sources.sort_by(|left, right| {
            tier_points(&right.tier)
                .cmp(&tier_points(&left.tier))
                .then_with(|| left.retrieval_rank.cmp(&right.retrieval_rank))
                .then_with(|| left.url.cmp(&right.url))
        });
    }
    nodes
}

fn formalize(
    text: &str,
    source_language: &str,
    target_language: &str,
    anchors: &[Meaning],
    vocabulary: &[(String, VocabularyForm)],
) -> Formalization {
    let tokens = words(&normalized(text));
    let denied = tokens.iter().any(|token| {
        vocabulary
            .iter()
            .any(|(role, form)| role == NEGATION_ROLE && form.text == *token)
    });
    let without_negation = tokens
        .iter()
        .filter(|token| {
            !vocabulary
                .iter()
                .any(|(role, form)| role == NEGATION_ROLE && form.text == **token)
        })
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if let Some((relation, relation_surface, at)) =
        relation_match(&without_negation, source_language, anchors)
    {
        let subject_text = without_negation[..at].trim();
        let object_text = without_negation[at + relation_surface.len()..].trim();
        let subject = entity_match(subject_text, source_language, anchors);
        let object = entity_match(object_text, source_language, anchors);
        if let (Some(subject), Some(object)) = (subject, object) {
            let semantic = vec![
                format!("subject=wikidata:{}", subject.grounded_in),
                format!("predicate=wikidata:{}", relation.grounded_in),
                format!("object=wikidata:{}", object.grounded_in),
            ];
            let mut rendered = punctuated(text);
            if source_language != target_language {
                if let (Some(subject_word), Some(predicate_word), Some(object_word)) = (
                    target_form(subject, target_language, false),
                    target_form(relation, target_language, true),
                    target_form(object, target_language, false),
                ) {
                    let predicate = if denied {
                        denied_predicate(&predicate_word, target_language, vocabulary)
                    } else {
                        predicate_word
                    };
                    rendered = punctuated(&capitalize_first(
                        &[subject_word, predicate, object_word].join(" "),
                    ));
                }
            }
            return Formalization {
                denied,
                meaning: semantic.join("|"),
                semantic,
                rendered,
            };
        }
    }
    let mut terms = tokens
        .into_iter()
        .filter(|token| {
            !vocabulary.iter().any(|(role, form)| {
                (role == FUNCTION_WORD_ROLE || role == NEGATION_ROLE) && form.text == *token
            })
        })
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    Formalization {
        denied,
        meaning: terms.join(" "),
        semantic: Vec::new(),
        rendered: punctuated(text),
    }
}

fn relation_match<'a>(
    text: &str,
    language: &str,
    anchors: &'a [Meaning],
) -> Option<(&'a Meaning, String, usize)> {
    let padded = format!(" {text} ");
    let mut best: Option<(&Meaning, String, usize)> = None;
    for meaning in anchors
        .iter()
        .filter(|meaning| meaning.role == RELATION_ROLE)
    {
        for form in meaning
            .forms
            .iter()
            .filter(|form| form.language == language)
        {
            let surface = normalized(&form.text);
            let needle = format!(" {surface} ");
            if let Some(index) = padded.find(&needle) {
                let at = index;
                if best
                    .as_ref()
                    .is_none_or(|(_, current, _)| surface.len() > current.len())
                {
                    best = Some((meaning, surface, at));
                }
            }
        }
    }
    best
}

fn entity_match<'a>(text: &str, language: &str, anchors: &'a [Meaning]) -> Option<&'a Meaning> {
    anchors.iter().find(|meaning| {
        meaning.role == ENTITY_ROLE
            && !meaning.grounded_in.is_empty()
            && meaning
                .forms
                .iter()
                .any(|form| form.language == language && normalized(&form.text) == text)
    })
}

fn target_form(meaning: &Meaning, language: &str, prefer_action: bool) -> Option<String> {
    let forms = meaning
        .forms
        .iter()
        .filter(|form| form.language == language);
    if prefer_action {
        if let Some(form) = forms.clone().find(|form| form.action) {
            return Some(form.text.clone());
        }
    }
    forms.into_iter().next().map(|form| form.text.clone())
}

fn denied_predicate(
    predicate: &str,
    language: &str,
    vocabulary: &[(String, VocabularyForm)],
) -> String {
    let cue = vocabulary
        .iter()
        .find(|(role, form)| role == NEGATION_ROLE && form.language == language)
        .map(|(_, form)| form.text.as_str())
        .unwrap_or_default();
    if cue.is_empty() {
        return predicate.to_string();
    }
    let mut parts = predicate.splitn(2, ' ');
    let first = parts.next().unwrap_or_default();
    let mut rendered = String::from(first);
    rendered.push(' ');
    rendered.push_str(cue);
    if let Some(rest) = parts.next() {
        rendered.push(' ');
        rendered.push_str(rest);
    }
    rendered
}

fn sentences(text: &str, limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut buffer = String::new();
    for character in text.chars() {
        buffer.push(character);
        if matches!(
            character,
            '.' | '!' | '?' | '。' | '！' | '？' | '।' | '॥' | '\n'
        ) {
            if !buffer.trim().is_empty() {
                out.push(buffer.trim().to_string());
                if out.len() >= limit {
                    return out;
                }
            }
            buffer.clear();
        }
    }
    if !buffer.trim().is_empty() {
        out.push(buffer.trim().to_string());
    }
    if out.is_empty() && !text.trim().is_empty() {
        out.push(text.trim().to_string());
    }
    out
}

fn normalized(text: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for character in text.to_lowercase().chars() {
        if character.is_alphanumeric() || character == '\'' {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            out.push(character);
            pending_space = false;
        } else {
            pending_space = true;
        }
    }
    out
}

fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(ToString::to_string).collect()
}

fn punctuated(text: &str) -> String {
    let mut rendered = text.trim().to_string();
    if !rendered.ends_with(['.', '!', '?', '。', '！', '？', '।', '॥']) {
        rendered.push('.');
    }
    rendered
}

fn capitalize_first(text: &str) -> String {
    let mut characters = text.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_uppercase().chain(characters).collect()
}

fn canonical_tier(tier: &str) -> &'static str {
    match tier {
        "original_first_party" => "original_first_party",
        "original_journalism" => "original_journalism",
        "unoriginal" => "unoriginal",
        _ => "independent_corroboration",
    }
}

fn tier_points(tier: &str) -> u32 {
    match tier {
        "original_first_party" => 100,
        "original_journalism" => 85,
        "independent_corroboration" => 50,
        _ => 0,
    }
}

fn posterior(support: u32, oppose: u32) -> f64 {
    if oppose == 0 {
        f64::from(6_000 + 40 * support.min(100)) / 10_000.0
    } else {
        f64::from(support) / f64::from(support + oppose)
    }
}

fn rank_nodes(nodes: &mut [Node]) {
    nodes.sort_by(|left, right| {
        right
            .weight
            .cmp(&left.weight)
            .then_with(|| left.best_retrieval_rank.cmp(&right.best_retrieval_rank))
            .then_with(|| right.sources.len().cmp(&left.sources.len()))
            .then_with(|| statement_key(left).cmp(&statement_key(right)))
    });
}

fn smallest_sufficient(nodes: Vec<Node>) -> Vec<Node> {
    let mut meanings = Vec::new();
    nodes
        .into_iter()
        .filter(|node| {
            let seen = meanings.iter().any(|meaning| meaning == &node.meaning);
            if !seen && meanings.len() >= MAX_MEANINGS {
                return false;
            }
            if !seen {
                meanings.push(node.meaning.clone());
            }
            true
        })
        .collect()
}

fn statement_key(node: &Node) -> String {
    format!(
        "{}:{}",
        if node.denied { "denied" } else { "asserted" },
        node.meaning
    )
}

fn render_json(request: &Request, nodes: &[Node], evidence: &mut Vec<String>) -> String {
    let mut lines = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        evidence.push(format!(
            "search_fusion:rank:{}:{}:{}:retrieval_rank={}",
            index + 1,
            statement_key(node),
            node.weight,
            node.best_retrieval_rank
        ));
        if node.conflict {
            evidence.push(format!(
                "conflict:source_disagreement:{}",
                statement_key(node)
            ));
        }
        lines.push(format!("{}. {}", index + 1, node.text));
        let tiers = node
            .sources
            .iter()
            .map(|source| source.tier.as_str())
            .collect::<Vec<_>>()
            .join("|");
        let mut metadata = String::from("   `posterior=");
        metadata.push_str(&format_args!("{:.6}", node.posterior).to_string());
        metadata.push(' ');
        metadata.push_str("source_count=");
        metadata.push_str(&node.sources.len().to_string());
        metadata.push(' ');
        metadata.push_str("source_tier=");
        metadata.push_str(&tiers);
        if node.conflict {
            metadata.push(' ');
            metadata.push_str("conflict=source_disagreement");
        }
        metadata.push('`');
        lines.push(metadata);
        for source in &node.sources {
            let domain = url_domain(&source.url);
            lines.push(format!(
                "   - {}**[{}]({})**{}",
                if source.alternate && !request.other_sources.is_empty() {
                    format!("_{}:_ ", request.other_sources)
                } else {
                    String::new()
                },
                source.title,
                source.url,
                if domain.is_empty() {
                    String::new()
                } else {
                    format!("  `{domain}`")
                }
            ));
            lines.push(format!("     > {}", source.quote));
            let via = if source.providers.is_empty() {
                String::new()
            } else {
                format!(" — _{} {}_", request.via, source.providers)
            };
            lines.push(format!(
                "     [{}]({}){}",
                request.read_more, source.url, via
            ));
        }
        lines.push(String::new());
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }

    let mut output = String::from("{\"lines\":[");
    push_json_array(&mut output, &lines);
    output.push_str("],\"evidence\":[");
    push_json_array(&mut output, evidence);
    output.push_str("],\"statements\":[");
    for (index, node) in nodes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str("{\"id\":");
        push_json_string(&mut output, &statement_key(node));
        output.push_str(",\"text\":");
        push_json_string(&mut output, &node.text);
        output.push_str(",\"semanticLinks\":[");
        push_json_array(&mut output, &node.semantic);
        let _ = write!(
            output,
            "],\"posterior\":{:.6},\"weight\":{},\"conflict\":{},\"sources\":[",
            node.posterior, node.weight, node.conflict
        );
        for (source_index, source) in node.sources.iter().enumerate() {
            if source_index > 0 {
                output.push(',');
            }
            output.push_str("{\"url\":");
            push_json_string(&mut output, &source.url);
            output.push_str(",\"title\":");
            push_json_string(&mut output, &source.title);
            output.push_str(",\"quote\":");
            push_json_string(&mut output, &source.quote);
            output.push_str(",\"readMore\":");
            push_json_string(&mut output, &source.url);
            output.push_str(",\"language\":");
            push_json_string(&mut output, &source.language);
            output.push_str(",\"tier\":");
            push_json_string(&mut output, &source.tier);
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push_str("]}");
    output
}

fn push_json_array(output: &mut String, values: &[String]) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(output, value);
    }
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{1f}' => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let code = character as usize;
                output.push_str("\\u00");
                output.push(HEX[code >> 4] as char);
                output.push(HEX[code & 0x0f] as char);
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

fn url_domain(url: &str) -> String {
    url.split_once("://")
        .map_or(url, |(_, rest)| rest)
        .split('/')
        .next()
        .unwrap_or_default()
        .to_string()
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn decode_uri_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex(bytes[index + 1]), hex(bytes[index + 2])) {
                decoded.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(decoded).unwrap_or_default()
}

const fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
