//! Constraint evidence for one rendered text: built-in checks plus callers' checkers.
//! JavaScript root: js/agentic/crate/cnl_output.mjs.
use super::grammar::{Check, CheckContext, Graph, builtin_check, graph_key};
use super::projection::{output_candidates, output_words, profile_of};
use super::{OutputConfig, OutputDensity, OutputPolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputReport {
    pub output_standard: String,
    pub lexical_density: String,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub unchecked: Vec<String>,
}

/// Caller-supplied evidence. `None` means "no checker for this id": the
/// built-in evidence applies. `Some(None)` leaves the check open; it never
/// supplies approval.
pub trait OutputCheckers {
    fn check(
        &self,
        _id: &str,
        _text: &str,
        _language: &str,
        _tier: Option<u64>,
    ) -> Option<Option<bool>> {
        None
    }
    /// Overrides `frequency-vocabulary` (a corpus checker may replace the tier table).
    fn frequency(&self, _text: &str, _language: &str, _tier: Option<u64>) -> Option<Option<bool>> {
        None
    }
}

impl OutputCheckers for () {}

/// Accepts `lexical-minimum` while the minimum itself is being measured.
struct MinimumPass;

impl OutputCheckers for MinimumPass {
    fn check(&self, check_name: &str, _: &str, _: &str, _: Option<u64>) -> Option<Option<bool>> {
        (check_name == "lexical-minimum").then_some(Some(true))
    }
}

/// The minimum word count over the domain's candidates compliant with everything else.
fn minimum_words(
    graph: &Graph,
    language: &str,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> Option<usize> {
    let grammar = policy.grammar(language)?;
    let key = [
        graph_key(graph),
        configuration.standard.slug().to_owned(),
        configuration.density.slug(),
    ]
    .join("|");
    if let Some(least) = grammar
        .minimum_cache
        .lock()
        .expect("minimum cache")
        .get(&key)
    {
        return *least;
    }
    let mut least: Option<usize> = None;
    for text in output_candidates(graph, language, configuration, policy) {
        if !validate_output(&text, language, configuration, policy, &MinimumPass).compliant {
            continue;
        }
        let count = output_words(&text, language).len();
        least = Some(least.map_or(count, |known| known.min(count)));
    }
    grammar
        .minimum_cache
        .lock()
        .expect("minimum cache")
        .insert(key, least);
    least
}

// Lexical-density evidence; every other violation belongs to the named profile.
const DENSITY_CHECKS: [&str; 6] = [
    "non-prime",
    "nsm-vocabulary",
    "nsm-syntax",
    "nsm-semantics",
    "frequency-vocabulary",
    "lexical-minimum",
];

/// Lexicographic cost: profile violations, then density violations, then unchecked evidence.
pub(crate) fn output_cost(report: &OutputReport) -> [usize; 3] {
    let profile = report
        .violations
        .iter()
        .filter(|check_name| !DENSITY_CHECKS.contains(&check_name.as_str()))
        .count();
    [
        profile,
        report.violations.len() - profile,
        report.unchecked.len(),
    ]
}

/// Constraint evidence, not a certificate. A standard is compliant only when
/// every required independent checker ran and passed. Missing checks stay open.
pub fn validate_output(
    text: &str,
    language: &str,
    configuration: OutputConfig,
    policy: &OutputPolicy,
    checkers: &impl OutputCheckers,
) -> OutputReport {
    let profile = profile_of(configuration, policy);
    let locale = policy.languages.iter().find(|item| item.id == language);
    let grammar = policy.grammar(language);
    let tier = match configuration.density {
        OutputDensity::Top(count) => Some(count),
        _ => None,
    };
    let count = |sentence: &str| output_words(sentence, language).len();
    let minimum = |graph: &Graph| minimum_words(graph, language, configuration, policy);
    let context = CheckContext {
        tier,
        count: &count,
        minimum: &minimum,
    };
    let builtin = |id: &str| {
        let check = policy.shared.checks.iter().find(|item| item.id == id)?;
        builtin_check(check, text, grammar?, &context)
    };
    // A caller-supplied checker overrides the built-in evidence for the same id.
    let evidence = |id: &str| {
        checkers
            .check(id, text, language, tier)
            .unwrap_or_else(|| builtin(id))
    };
    let mut violations: Vec<String> = Vec::new();
    let mut unchecked: Vec<String> = Vec::new();
    let mut record = |id: &str, result: Option<bool>| match result {
        Some(true) => {}
        Some(false) => violations.push(id.to_owned()),
        None => unchecked.push(id.to_owned()),
    };
    if locale.is_none() {
        record("language", None);
    }
    let tokens: Vec<String> = output_words(text, language)
        .iter()
        .map(|word| word.to_lowercase())
        .collect();
    let standard = configuration.density == OutputDensity::Standard;
    let maximum_words = profile
        .density_max_words
        .filter(|_| !standard)
        .or(profile.max_words);
    if let Some(maximum_words) = maximum_words {
        let long = text
            .split(['.', '!', '?', '。', '！', '？', '।'])
            .any(|sentence| count(sentence) > maximum_words);
        if long {
            record("sentence-length", Some(false));
        }
    }
    if profile.id == "e-prime" {
        // The profile lists the English forms of "to be"; every other language declares
        // its own copula in its grammar file. A language without one stays unchecked.
        let mut forbidden: HashSet<&str> = HashSet::new();
        if language == "en" {
            forbidden.extend(profile.forbidden.iter().map(String::as_str));
        }
        if let Some(grammar) = grammar {
            forbidden.extend(grammar.copula_words.iter().map(String::as_str));
        }
        if forbidden.is_empty() {
            record("e-prime-language", None);
        } else {
            let words: Vec<String> = tokens.iter().map(|word| word.replace('’', "'")).collect();
            if words.iter().any(|word| {
                forbidden.contains(word.as_str()) || word.ends_with("'m") || word.ends_with("'re")
            }) {
                record("forbidden-word", Some(false));
            }
            if words
                .iter()
                .any(|word| word.ends_with("'s") || word == "ain't")
            {
                record("e-prime-contraction", None);
            }
        }
    }
    if configuration.density == OutputDensity::Primes {
        match grammar.filter(|grammar| !grammar.prime_words.is_empty()) {
            Some(grammar) => {
                let primes = Check {
                    kind: "primes".to_owned(),
                    ..Check::default()
                };
                let pass = builtin_check(&primes, text, grammar, &context) == Some(true);
                if !pass && !tokens.is_empty() {
                    record("non-prime", Some(false));
                }
            }
            None => record("nsm-vocabulary", None),
        }
        for check_name in ["nsm-syntax", "nsm-semantics"] {
            record(check_name, evidence(check_name));
        }
    }
    if configuration.density == OutputDensity::MaxPrecision {
        record("lexical-minimum", evidence("lexical-minimum"));
    }
    // Built-in ranks are a declared, project-authored tier table; a corpus checker may replace them.
    if tier.is_some() {
        let check_name = "frequency-vocabulary";
        let result = checkers
            .frequency(text, language, tier)
            .or_else(|| checkers.check(check_name, text, language, tier))
            .unwrap_or_else(|| builtin(check_name));
        record(check_name, result);
    }
    let required = if !standard && !profile.density_required.is_empty() {
        &profile.density_required
    } else {
        &profile.required
    };
    for check_name in required {
        record(check_name, evidence(check_name));
    }
    OutputReport {
        output_standard: configuration.standard.slug().to_owned(),
        lexical_density: configuration.density.slug(),
        compliant: violations.is_empty() && unchecked.is_empty(),
        violations,
        unchecked,
    }
}
