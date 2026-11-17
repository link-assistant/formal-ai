use super::{OutputConfig, OutputDensity, OutputPolicy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum OutputPart {
    Opaque { text: String },
    Meaning { id: String, surface: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDocument {
    pub language: String,
    pub source: String,
    pub parts: Vec<OutputPart>,
}

pub fn formalize_output(text: &str, language: &str, policy: &OutputPolicy) -> OutputDocument {
    let trimmed = text.trim();
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
        parts.push(OutputPart::Opaque {
            text: text.to_owned(),
        });
    }
    OutputDocument {
        language: language.to_owned(),
        source: text.to_owned(),
        parts,
    }
}

pub fn output_meaning(document: &OutputDocument) -> Vec<(&str, &str)> {
    document
        .parts
        .iter()
        .map(|part| match part {
            OutputPart::Opaque { text } => ("opaque", text.as_str()),
            OutputPart::Meaning { id, .. } => ("meaning", id.as_str()),
        })
        .collect()
}

pub fn output_words(text: &str, language: &str) -> Vec<String> {
    // Match the Unicode word grammar of the JS root, including combining marks.
    let pattern = if language == "zh" {
        r"[\p{Han}]|[\p{L}\p{M}\p{N}]+"
    } else {
        r"[\p{L}\p{M}\p{N}]+(?:[-’'][\p{L}\p{M}\p{N}]+)*"
    };
    regex::Regex::new(pattern)
        .expect("fixed output word grammar")
        .find_iter(text)
        .map(|item| item.as_str().to_owned())
        .collect()
}

pub fn project_output(
    document: &OutputDocument,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> String {
    let desired = if configuration.density == OutputDensity::Standard {
        configuration.standard.slug().to_owned()
    } else {
        configuration.density.slug()
    };
    document
        .parts
        .iter()
        .map(|part| {
            let OutputPart::Meaning { id, surface } = part else {
                let OutputPart::Opaque { text } = part else {
                    unreachable!()
                };
                return text.clone();
            };
            let Some(lexeme) = policy
                .lexemes
                .iter()
                .find(|item| item.id == *id && item.language == document.language)
            else {
                return surface.clone();
            };
            let mut forms: Vec<_> = lexeme
                .forms
                .iter()
                .filter(|form| form.mode.split('|').any(|mode| mode == desired))
                .collect();
            if configuration.density == OutputDensity::MaxPrecision {
                forms.sort_by(|left, right| {
                    output_words(&left.text, &document.language)
                        .len()
                        .cmp(&output_words(&right.text, &document.language).len())
                        .then_with(|| left.text.chars().count().cmp(&right.text.chars().count()))
                        .then_with(|| left.text.encode_utf16().cmp(right.text.encode_utf16()))
                });
            }
            let Some(form) = forms.first() else {
                return surface.clone();
            };
            let Some(first) = surface.chars().next() else {
                return form.text.clone();
            };
            if first.to_lowercase().to_string() == first.to_string() {
                return form.text.clone();
            }
            let mut chars = form.text.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().to_string() + chars.as_str()
            })
        })
        .collect()
}

pub fn output_links_notation(document: &OutputDocument) -> String {
    let mut text = String::from("output-meaning");
    text.push('\n');
    text.push_str("  language ");
    text.push_str(&serde_json::to_string(&document.language).expect("string"));
    text.push('\n');
    for (kind, value) in output_meaning(document) {
        text.push_str(&format!(
            "  {kind} {}\n",
            serde_json::to_string(value).expect("string")
        ));
    }
    text
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputReport {
    pub output_standard: String,
    pub lexical_density: String,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub unchecked: Vec<String>,
}

/// Absent independent evidence leaves a check open; it never supplies approval.
pub trait OutputCheckers {
    fn check(&self, _name: &str, _text: &str, _language: &str) -> Option<bool> {
        None
    }
    fn frequency(&self, _text: &str, _language: &str, _tier: u64) -> Option<bool> {
        None
    }
}
impl OutputCheckers for () {}

fn evidence(
    name: &str,
    result: Option<bool>,
    violations: &mut Vec<String>,
    unchecked: &mut Vec<String>,
) {
    match result {
        Some(true) => {}
        Some(false) => violations.push(name.to_owned()),
        None => unchecked.push(name.to_owned()),
    }
}

pub fn validate_output(
    text: &str,
    language: &str,
    configuration: OutputConfig,
    policy: &OutputPolicy,
    checkers: &impl OutputCheckers,
) -> OutputReport {
    let mut violations = Vec::new();
    let mut unchecked = Vec::new();
    let profile = policy
        .profiles
        .iter()
        .find(|item| item.id == configuration.standard.slug());
    let locale = policy.languages.iter().find(|item| item.id == language);
    if locale.is_none() {
        unchecked.push("language".to_owned());
    }
    let tokens: Vec<_> = output_words(text, language)
        .into_iter()
        .map(|word| word.to_lowercase())
        .collect();
    if let Some(profile) = profile {
        if let Some(max) = profile.max_words {
            if text
                .split(['.', '!', '?', '。', '！', '？'])
                .any(|sentence| output_words(sentence, language).len() > max)
            {
                violations.push("sentence_length".to_owned());
            }
        }
        if profile.id == "e-prime" {
            if language != "en" {
                unchecked.push("e_prime_language".to_owned());
            } else {
                let words: Vec<_> = tokens.iter().map(|word| word.replace('’', "'")).collect();
                if words.iter().any(|word| {
                    profile.forbidden.contains(word)
                        || word.ends_with("'m")
                        || word.ends_with("'re")
                }) {
                    violations.push("forbidden_word".to_owned());
                }
                if words
                    .iter()
                    .any(|word| word.ends_with("'s") || word == "ain't")
                {
                    unchecked.push("e_prime_contraction".to_owned());
                }
            }
        }
    } else {
        unchecked.push("profile".to_owned());
    }
    if configuration.density == OutputDensity::Primes {
        if let Some(locale) = locale.filter(|item| !item.primes.is_empty()) {
            let allowed: Vec<_> = locale
                .primes
                .iter()
                .chain(&locale.grammar)
                .flat_map(|word| output_words(word, language))
                .map(|word| word.to_lowercase())
                .collect();
            if tokens.iter().any(|word| !allowed.contains(word)) {
                violations.push("non_prime".to_owned());
            }
        } else {
            unchecked.push("nsm_vocabulary".to_owned());
        }
        for name in ["nsm_syntax", "nsm_semantics"] {
            evidence(
                name,
                checkers.check(name, text, language),
                &mut violations,
                &mut unchecked,
            );
        }
    }
    if configuration.density == OutputDensity::MaxPrecision {
        evidence(
            "lexical_minimum",
            checkers.check("lexical_minimum", text, language),
            &mut violations,
            &mut unchecked,
        );
    }
    if let OutputDensity::Top(tier) = configuration.density {
        evidence(
            "frequency_vocabulary",
            checkers.frequency(text, language, tier),
            &mut violations,
            &mut unchecked,
        );
    }
    if let Some(profile) = profile {
        for name in &profile.required {
            if name == "e_prime_english" && language == "en" {
                continue;
            }
            evidence(
                name,
                checkers.check(name, text, language),
                &mut violations,
                &mut unchecked,
            );
        }
    }
    OutputReport {
        output_standard: configuration.standard.slug().to_owned(),
        lexical_density: configuration.density.slug(),
        compliant: violations.is_empty() && unchecked.is_empty(),
        violations,
        unchecked,
    }
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
        links_notation: output_links_notation(document),
        report,
    }
}
