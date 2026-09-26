//! Searchable Python standard-library parts grounded in the official 3.12 docs.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::process::Command;

use crate::how_to_guide::extract::{decode_entities, strip_html};
use crate::links_format::push_lino_node;
use crate::seed::SourceRecord;
use crate::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};

const PAGES: &[&str] = &[
    "functions.html",
    "stdtypes.html",
    "math.html",
    "itertools.html",
    "heapq.html",
    "collections.html",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibPart {
    pub symbol: String,
    pub module: String,
    pub signature: String,
    pub description: String,
    pub source_url: String,
    pub license: String,
    pub sha256: String,
    pub fetched_at: String,
}

impl StdlibPart {
    #[must_use]
    pub fn match_score(&self, phrase: &str) -> f64 {
        let phrase_tokens = canonical_token_list(phrase);
        let phrase = phrase_tokens.iter().cloned().collect::<BTreeSet<_>>();
        let description = canonical_tokens(&self.description);
        let overlap = count_as_score(phrase.intersection(&description).count());
        let tail = self.symbol.rsplit('.').next().unwrap_or(&self.symbol);
        let symbol_bonus = phrase_tokens
            .iter()
            .position(|token| token == tail || (tail.len() >= 3 && token.starts_with(tail)))
            .map_or(0.0, |position| 3.0 / count_as_score(position + 1));
        let semantic_conflict = phrase.contains("overlapping")
            && description.contains("overlapping")
            && description.contains("non");
        let shape_conflict = description.contains("list") && !phrase.contains("list");
        overlap + symbol_bonus
            - if semantic_conflict { 5.0 } else { 0.0 }
            - if shape_conflict { 3.0 } else { 0.0 }
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "stdlib_part", Some(&self.symbol));
        push_lino_node(&mut out, 2, "module", Some(&self.module));
        push_lino_node(&mut out, 2, "signature", Some(&self.signature));
        push_lino_node(&mut out, 2, "description", Some(&self.description));
        push_lino_node(&mut out, 2, "source_url", Some(&self.source_url));
        push_lino_node(&mut out, 2, "license", Some(&self.license));
        push_lino_node(&mut out, 2, "sha256", Some(&self.sha256));
        push_lino_node(&mut out, 2, "fetched_at", Some(&self.fetched_at));
        out.trim_end().to_owned()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StdlibIndex {
    parts: Vec<StdlibPart>,
}

impl StdlibIndex {
    #[must_use]
    pub const fn from_parts(parts: Vec<StdlibPart>) -> Self {
        Self { parts }
    }

    #[must_use]
    pub fn parts(&self) -> &[StdlibPart] {
        &self.parts
    }

    #[must_use]
    pub fn part(&self, symbol: &str) -> Option<&StdlibPart> {
        self.parts.iter().find(|part| part.symbol == symbol)
    }

    #[must_use]
    pub fn parts_for_phrase(&self, phrase: &str) -> Vec<&StdlibPart> {
        let mut matches = self
            .parts
            .iter()
            .filter_map(|part| {
                let score = part.match_score(phrase);
                (score > 0.0).then_some((part, score))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|(left, left_score), (right, right_score)| {
            right_score
                .partial_cmp(left_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.symbol.len().cmp(&right.symbol.len()))
                .then_with(|| left.symbol.cmp(&right.symbol))
        });
        matches.into_iter().map(|(part, _)| part).collect()
    }
}

pub fn fetch_index<T: SourceTransport>(
    client: &CachedSourceClient<T>,
) -> Result<StdlibIndex, FetchError> {
    let source = crate::seed::source_record("python_docs")
        .ok_or_else(|| FetchError::Cache("python_docs:source_registry_missing".to_owned()))?;
    let mut parts = Vec::new();
    for page in PAGES {
        let url = source.api_url(&[("title", page)]);
        let capture = match client.fetch(&url) {
            Ok(capture) => capture,
            Err(FetchError::OfflineCacheMiss(_)) => return runtime_fallback_index(&source),
            Err(error) => return Err(error),
        };
        parts.extend(extract_page(page, &capture, &source)?);
    }
    parts.sort_by(|left, right| left.symbol.cmp(&right.symbol));
    parts.dedup_by(|left, right| left.symbol == right.symbol);
    Ok(StdlibIndex { parts })
}

fn extract_page(
    page: &str,
    capture: &SourceCapture,
    source: &SourceRecord,
) -> Result<Vec<StdlibPart>, FetchError> {
    let html = std::str::from_utf8(capture.bytes())
        .map_err(|error| FetchError::Cache(format!("python_docs_utf8:{page}:{error}")))?;
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<dt class=\"sig sig-object py\"") {
        rest = &rest[start..];
        let Some(tag_end) = rest.find('>') else {
            break;
        };
        let tag = &rest[..=tag_end];
        let Some(symbol) = attribute(tag, "id") else {
            rest = &rest[tag_end + 1..];
            continue;
        };
        let Some(dt_end) = rest.find("</dt>") else {
            break;
        };
        let signature = text(&rest[..dt_end]);
        let definition_html = &rest[dt_end + 5..];
        let Some(dd_start) = definition_html.find("<dd>") else {
            rest = definition_html;
            continue;
        };
        let body_html = &definition_html[dd_start + 4..];
        let Some(paragraph_start) = body_html.find("<p>") else {
            rest = body_html;
            continue;
        };
        let paragraph = &body_html[paragraph_start + 3..];
        let Some(paragraph_end) = paragraph.find("</p>") else {
            rest = paragraph;
            continue;
        };
        let description = first_sentence(&text(&paragraph[..paragraph_end]));
        let module = symbol
            .split_once('.')
            .map_or("builtins", |(module, _)| module)
            .to_owned();
        out.push(StdlibPart {
            symbol: symbol.to_owned(),
            module,
            signature,
            description,
            source_url: format!("{}#{symbol}", source.api_url(&[("title", page)])),
            license: source.license_name.clone(),
            sha256: capture.sha256().to_owned(),
            fetched_at: capture.fetched_at().to_owned(),
        });
        rest = &paragraph[paragraph_end + 4..];
    }
    Ok(out)
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("{name}=\"");
    let value = tag.split_once(&marker)?.1;
    Some(value.split_once('"')?.0)
}

fn text(html: &str) -> String {
    collapse_whitespace(&decode_entities(&strip_html(html)))
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn first_sentence(value: &str) -> String {
    value
        .find(". ")
        .map_or(value, |end| &value[..=end])
        .trim()
        .to_owned()
}

fn canonical_tokens(value: &str) -> BTreeSet<String> {
    canonical_token_list(value).into_iter().collect()
}

fn canonical_token_list(value: &str) -> Vec<String> {
    let canonical = crate::seed::operation_vocabulary()
        .canonicalized_prompt(&crate::engine::normalize_prompt(value));
    canonical
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| stem(token).to_owned())
        .collect()
}

fn count_as_score(value: usize) -> f64 {
    u32::try_from(value).map_or_else(|_| f64::from(u32::MAX), f64::from)
}

fn stem(token: &str) -> &str {
    if token.len() > 3 {
        token.strip_suffix('s').unwrap_or(token)
    } else {
        token
    }
}

fn runtime_fallback_index(source: &SourceRecord) -> Result<StdlibIndex, FetchError> {
    const SYMBOLS: &[(&str, &str)] = &[
        ("sum", "sum"),
        ("max", "max"),
        ("min", "min"),
        ("len", "len"),
        ("any", "any"),
        ("all", "all"),
        ("sorted", "sorted"),
        ("abs", "abs"),
        ("math.prod", "__import__('math').prod"),
        ("math.gcd", "__import__('math').gcd"),
        (
            "itertools.combinations",
            "__import__('itertools').combinations",
        ),
        ("itertools.accumulate", "__import__('itertools').accumulate"),
        ("str.count", "str.count"),
        ("str.lower", "str.lower"),
        ("str.split", "str.split"),
        ("str.join", "str.join"),
    ];
    let version = Command::new("python3")
        .args([
            "-c",
            "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')",
        ])
        .output()
        .map_err(|error| FetchError::Cache(format!("python_docs_runtime:{error}")))?;
    if !version.status.success() {
        return Err(FetchError::Cache(
            "python_docs_runtime:python3 failed".to_owned(),
        ));
    }
    let version = String::from_utf8_lossy(&version.stdout).trim().to_owned();
    let mut parts = Vec::new();
    for (symbol, expression) in SYMBOLS {
        let Some(script) = crate::coding::python_render::runtime_template(
            "python_doc_probe",
            &[("expression", expression)],
        ) else {
            return Err(FetchError::Cache(
                "python_docs_runtime:missing python_doc_probe fragment".to_owned(),
            ));
        };
        let output = Command::new("python3")
            .args(["-c", &script])
            .output()
            .map_err(|error| FetchError::Cache(format!("python_docs_runtime:{symbol}:{error}")))?;
        if !output.status.success() {
            continue;
        }
        let description = first_sentence(&collapse_whitespace(&String::from_utf8_lossy(
            &output.stdout,
        )));
        parts.push(StdlibPart {
            symbol: (*symbol).to_owned(),
            module: symbol
                .split_once('.')
                .map_or("builtins", |(module, _)| module)
                .to_owned(),
            signature: String::new(),
            description,
            source_url: format!("python{version}:{symbol}.__doc__"),
            license: source.license_name.clone(),
            sha256: crate::source_fetch::sha256_hex(output.stdout.as_slice()),
            fetched_at: "runtime".to_owned(),
        });
    }
    Ok(StdlibIndex { parts })
}
