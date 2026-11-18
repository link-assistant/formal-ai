//! Lossless output documents and their projection to one configuration.
//! JavaScript root: js/agentic/crate/cnl_output.mjs.
use super::grammar::{
    CompiledGrammar, Graph, PieceKind, all_pairs, graph_key, graph_logic, js_trim,
    output_sentences, realizations, utterance_graphs,
};
use super::validation::{OutputReport, output_cost, validate_output};
use super::{OutputCheckers, OutputConfig, OutputDensity, OutputPolicy, Profile};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum OutputPart {
    Opaque {
        text: String,
    },
    Meaning {
        id: String,
        surface: String,
    },
    Graph {
        graph: Graph,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        surface: Option<String>,
    },
    Space {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDocument {
    pub language: String,
    pub source: String,
    pub parts: Vec<OutputPart>,
}

/// Lossless representation. Only declared whole utterances acquire a meaning.
/// Substrings cannot change a quote, identifier, negation or quantifier scope.
/// All other bytes remain opaque. Retention does not count as understanding.
pub fn formalize_output(text: &str, language: &str, policy: &OutputPolicy) -> OutputDocument {
    let trimmed = js_trim(text);
    let matches: Vec<_> = policy
        .lexemes
        .iter()
        .filter(|item| {
            item.language == language
                && item
                    .forms
                    .iter()
                    .any(|form| form.text.to_lowercase() == trimmed.to_lowercase())
        })
        .collect();
    let mut parts = Vec::new();
    if matches.len() == 1 {
        let start = text.find(trimmed).unwrap_or(0);
        if start > 0 {
            parts.push(OutputPart::Opaque {
                text: text[..start].to_owned(),
            });
        }
        parts.push(OutputPart::Meaning {
            id: matches[0].id.clone(),
            surface: trimmed.to_owned(),
        });
        if start + trimmed.len() < text.len() {
            parts.push(OutputPart::Opaque {
                text: text[start + trimmed.len()..].to_owned(),
            });
        }
    } else if !text.is_empty() {
        parts = graph_parts(text, policy.grammar(language).map(AsRef::as_ref));
    }
    OutputDocument {
        language: language.to_owned(),
        source: text.to_owned(),
        parts,
    }
}

/// Sentences with exactly one reading become graphs; everything else stays opaque.
fn graph_parts(text: &str, grammar: Option<&CompiledGrammar>) -> Vec<OutputPart> {
    let opaque = || {
        vec![OutputPart::Opaque {
            text: text.to_owned(),
        }]
    };
    let Some(grammar) = grammar else {
        return opaque();
    };
    let pieces: Vec<OutputPart> = output_sentences(text)
        .into_iter()
        .map(|piece| match piece.kind {
            PieceKind::Space => OutputPart::Space { text: piece.text },
            PieceKind::Opaque => OutputPart::Opaque { text: piece.text },
            PieceKind::Sentence => {
                let mut graphs = utterance_graphs(&piece.text, grammar);
                if graphs.len() == 1 {
                    OutputPart::Graph {
                        graph: graphs.remove(0),
                        surface: Some(piece.text),
                    }
                } else {
                    OutputPart::Opaque { text: piece.text }
                }
            }
        })
        .collect();
    if !pieces
        .iter()
        .any(|piece| matches!(piece, OutputPart::Graph { .. }))
    {
        return opaque();
    }
    let mut parts: Vec<OutputPart> = Vec::new();
    for piece in pieces {
        if let (OutputPart::Opaque { text }, Some(OutputPart::Opaque { text: last })) =
            (&piece, parts.last_mut())
        {
            last.push_str(text);
        } else {
            parts.push(piece);
        }
    }
    parts
}

fn part_meaning(part: &OutputPart) -> Option<(&'static str, String)> {
    match part {
        OutputPart::Space { .. } => None,
        OutputPart::Meaning { id, .. } => Some(("meaning", id.clone())),
        OutputPart::Graph { graph, .. } => Some(("graph", graph_key(graph))),
        OutputPart::Opaque { text } => Some(("opaque", text.clone())),
    }
}

/// The canonical lexical identity includes every unresolved byte and its order.
#[must_use]
pub fn output_meaning(document: &OutputDocument) -> Vec<(&'static str, String)> {
    document.parts.iter().filter_map(part_meaning).collect()
}

/// Whitespace-delimited languages use Unicode words; CJK has no whitespace promise.
pub fn output_words(text: &str, language: &str) -> Vec<String> {
    static HAN: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"[\p{Script=Han}]|[\p{L}\p{M}\p{N}]+").expect("fixed Han word grammar")
    });
    static WORD: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"[\p{L}\p{M}\p{N}]+(?:[-’'][\p{L}\p{M}\p{N}]+)*")
            .expect("fixed output word grammar")
    });
    let pattern = if language == "zh" { &HAN } else { &WORD };
    pattern
        .find_iter(text)
        .map(|item| item.as_str().to_owned())
        .collect()
}

/// Capitalize like the original surface. JavaScript compares UTF-16 units, so
/// an astral first character never changes case.
fn capitalized_like(text: &str, original: &str) -> String {
    let (Some(first), Some(head)) = (text.chars().next(), original.chars().next()) else {
        return text.to_owned();
    };
    let astral = |character: char| u32::from(character) > 0xFFFF;
    if astral(head) || astral(first) || head.to_lowercase().eq(std::iter::once(head)) {
        return text.to_owned();
    }
    first.to_uppercase().collect::<String>() + &text[first.len_utf8()..]
}

pub(crate) fn profile_of<'a>(configuration: OutputConfig, policy: &'a OutputPolicy) -> &'a Profile {
    policy
        .profiles
        .iter()
        .find(|item| item.id == configuration.standard.slug())
        .expect("validated output configurations name a declared profile")
}

/// Select a realization without ever deleting an opaque span or a proposition.
pub fn project_output(
    document: &OutputDocument,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> String {
    let language = document.language.as_str();
    let standard = configuration.standard.slug();
    let desired = if configuration.density == OutputDensity::Standard {
        standard.to_owned()
    } else {
        configuration.density.slug()
    };
    let words = |text: &str| output_words(text, language).len();
    let project_meaning = |id: &str, surface: &str| {
        let Some(lexeme) = policy
            .lexemes
            .iter()
            .find(|item| item.id == id && item.language == language)
        else {
            return surface.to_owned();
        };
        let declares =
            |form: &&super::OutputForm, mode: &str| form.mode.split('|').any(|item| item == mode);
        let mut forms: Vec<_> = lexeme
            .forms
            .iter()
            .filter(|form| declares(form, &desired))
            .collect();
        // Minimum over the declared equivalent realizations, with a stable tie break.
        if configuration.density == OutputDensity::MaxPrecision {
            forms.sort_by(|left, right| {
                words(&left.text)
                    .cmp(&words(&right.text))
                    .then_with(|| left.text.chars().count().cmp(&right.text.chars().count()))
                    .then_with(|| left.text.encode_utf16().cmp(right.text.encode_utf16()))
            });
        }
        if desired != standard && standard != "cnl-default" {
            forms.extend(lexeme.forms.iter().filter(|form| declares(form, standard)));
        }
        forms
            .into_iter()
            .find(|form| {
                validate_output(&form.text, language, configuration, policy, &())
                    .violations
                    .is_empty()
            })
            .map_or_else(
                || surface.to_owned(),
                |form| capitalized_like(&form.text, surface),
            )
    };
    document
        .parts
        .iter()
        .map(|part| match part {
            OutputPart::Graph { graph, surface } => {
                project_graph(graph, surface.as_deref(), language, configuration, policy)
            }
            OutputPart::Meaning { id, surface } => project_meaning(id, surface),
            OutputPart::Opaque { text } | OutputPart::Space { text } => text.clone(),
        })
        .collect()
}

fn pairs(styles: &[String], registers: &[String]) -> Vec<(String, String)> {
    styles
        .iter()
        .flat_map(|style| {
            registers
                .iter()
                .map(move |register| (style.clone(), register.clone()))
        })
        .collect()
}

fn precision_registers(policy: &OutputPolicy) -> &[String] {
    if policy.shared.precision_registers.is_empty() {
        &policy.shared.registers
    } else {
        &policy.shared.precision_registers
    }
}

/// Max-precision searches every output style with the declared precise registers.
fn precision_pairs(grammar: &CompiledGrammar, policy: &OutputPolicy) -> Vec<(String, String)> {
    let order = &policy.shared.precision_styles;
    let position = |style: &String| {
        order
            .iter()
            .position(|item| item == style)
            .unwrap_or(order.len())
    };
    let mut styles = grammar.output_styles.clone();
    styles.sort_by_key(position);
    pairs(&styles, precision_registers(policy))
}

/// Style/register pairs to try, most preferred first, for one configuration.
fn candidate_pairs(
    configuration: OutputConfig,
    grammar: &CompiledGrammar,
    policy: &OutputPolicy,
) -> Vec<(String, String)> {
    let profile = profile_of(configuration, policy);
    let primes = ["primes".to_owned()];
    if configuration.density == OutputDensity::MaxPrecision {
        // The precise registers first, then the profile's own registers: a closed-vocabulary
        // profile (Basic, Special) may need its own exact explication of a word it lacks.
        let mut registers: Vec<String> = Vec::new();
        for register in precision_registers(policy).iter().chain(&profile.registers) {
            if !registers.contains(register) {
                registers.push(register.clone());
            }
        }
        let mut found = pairs(&profile.styles, &registers);
        found.extend(precision_pairs(grammar, policy));
        return found;
    }
    let preferred = pairs(&profile.styles, &profile.registers);
    let all = all_pairs(grammar);
    let mut found = Vec::new();
    match configuration.density {
        OutputDensity::Primes => {
            found.push(("primes".to_owned(), "primes".to_owned()));
            found.extend(pairs(&profile.styles, &primes));
        }
        OutputDensity::Top(_) => {
            found.extend(preferred);
            found.extend(pairs(&profile.styles, &primes));
            found.push(("primes".to_owned(), "primes".to_owned()));
        }
        _ => found.extend(preferred),
    }
    found.extend(all);
    found
}

/// The search domain of one configuration: every realization (all variants)
/// that reads back as exactly this graph, most preferred first. Max-precision
/// orders it by word count; its domain is the declared precise registers plus the profile's own.
pub fn output_candidates(
    graph: &Graph,
    language: &str,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> Vec<String> {
    let Some(grammar) = policy.grammar(language) else {
        return Vec::new();
    };
    let key = graph_key(graph);
    let mut candidates: Vec<String> = realizations(
        graph,
        &candidate_pairs(configuration, grammar, policy),
        grammar,
    )
    .into_iter()
    .filter(|text| {
        let graphs = utterance_graphs(text, grammar);
        graphs.len() == 1 && graph_key(&graphs[0]) == key
    })
    .collect();
    if configuration.density == OutputDensity::MaxPrecision {
        // A stable sort keeps the preference order among equal word counts.
        candidates.sort_by_key(|text| output_words(text, language).len());
    }
    candidates
}

/// Realize one meaning graph. A compliant surface stays as written; otherwise
/// the first compliant candidate wins. When the profile and the density cannot
/// both hold, the named profile wins over the density and the report says so.
pub(crate) fn project_graph(
    graph: &Graph,
    surface: Option<&str>,
    language: &str,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> String {
    if policy.grammar(language).is_none() {
        return surface.unwrap_or_default().to_owned();
    }
    let mut candidates = output_candidates(graph, language, configuration, policy);
    if let Some(surface) = surface {
        candidates.insert(0, surface.to_owned());
    }
    let mut unique: Vec<String> = Vec::new();
    for text in candidates {
        if !unique.contains(&text) {
            unique.push(text);
        }
    }
    let mut best: Option<(String, [usize; 3])> = None;
    for text in unique {
        let report = validate_output(&text, language, configuration, policy, &());
        if report.compliant {
            return text;
        }
        let cost = output_cost(&report);
        if best.as_ref().is_none_or(|(_, least)| cost < *least) {
            best = Some((text, cost));
        }
    }
    best.map(|(text, _)| text).unwrap_or_default()
}

/// Canonical Links Notation; the original surface remains in the document.
#[must_use]
pub fn output_links_notation(document: &OutputDocument, policy: &OutputPolicy) -> String {
    let quote = |text: &str| serde_json::to_string(text).expect("string");
    let mut rows = vec![
        "output-meaning".to_owned(),
        format!("  language {}", quote(&document.language)),
    ];
    for part in &document.parts {
        let OutputPart::Graph { graph, .. } = part else {
            if let Some((kind, value)) = part_meaning(part) {
                rows.push(format!("  {kind} {}", quote(&value)));
            }
            continue;
        };
        rows.push(format!("  graph {}", quote(&graph.frame)));
        for (slot, value) in graph.slots.iter() {
            rows.push(format!("    {slot} {}", quote(value)));
        }
        rows.push(format!("    polarity {}", quote(&graph.polarity)));
        rows.push(format!("    mood {}", quote(&graph.mood)));
        let logic = graph_logic(graph, &policy.shared);
        if !logic.is_empty() {
            rows.push(format!("    logic {}", quote(&logic)));
        }
    }
    rows.join("\n") + "\n"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedOutput {
    pub text: String,
    pub notice: String,
    pub content: String,
    pub document: OutputDocument,
    pub links_notation: String,
    pub report: OutputReport,
}

/// Render from the retained document each time; notices never enter its meaning.
pub fn render_output(
    document: &OutputDocument,
    configuration: OutputConfig,
    policy: &OutputPolicy,
    checkers: &impl OutputCheckers,
) -> RenderedOutput {
    let text = project_output(document, configuration, policy);
    let report = validate_output(&text, &document.language, configuration, policy, checkers);
    let notice = if configuration.density == OutputDensity::MaxPrecision {
        policy
            .languages
            .iter()
            .find(|item| item.id == document.language)
            .map(|item| item.notice.clone())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let content = if notice.is_empty() {
        text.clone()
    } else {
        format!("{notice}\n\n{text}")
    };
    RenderedOutput {
        text,
        notice,
        content,
        document: document.clone(),
        links_notation: output_links_notation(document, policy),
        report,
    }
}
