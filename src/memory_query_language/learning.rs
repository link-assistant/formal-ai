use std::collections::{BTreeMap, BTreeSet};

use super::{compile_exact_memory_query, CompiledMemoryQuery, MemoryQueryError, QueryDialect};
use crate::links_format::push_lino_node;
use crate::memory_program::MemoryProgramLimits;

#[derive(Debug, Clone)]
struct LearnedTemplate {
    natural_language: String,
    exact_query: String,
    exact_dialect: QueryDialect,
    candidate_id: String,
    reviewer: String,
    gate_suite: String,
}

/// One successful natural-language/exact-language pair observed by the learner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryQueryLearningObservation {
    pub natural_language: String,
    pub exact_query: String,
    pub exact_dialect: QueryDialect,
}

impl MemoryQueryLearningObservation {
    #[must_use]
    pub fn new(
        natural_language: impl Into<String>,
        exact_query: impl Into<String>,
        exact_dialect: QueryDialect,
    ) -> Self {
        Self {
            natural_language: natural_language.into(),
            exact_query: exact_query.into(),
            exact_dialect,
        }
    }
}

/// Automatically inferred template held outside the active compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryQueryLearningCandidate {
    pub id: String,
    pub natural_language_template: String,
    pub exact_query_template: String,
    pub exact_dialect: QueryDialect,
    pub evidence_count: usize,
}

/// Result of a regression/held-out suite run for one candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryQueryLearningGate {
    pub suite: String,
    pub passed: usize,
    pub failed: usize,
}

/// Explicit human decision required before a candidate can affect solving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryQueryLearningApproval {
    pub reviewer: String,
    pub granted: bool,
}

/// Associative learner for natural-language memory-query surfaces.
///
/// Learning stores a reviewed natural-language template beside an exact SQL or
/// GraphQL exemplar. Captures are injected only when they are safe identifier
/// or scalar fragments; the resulting exact query must still pass both
/// meta-language and the semantic parser before it can execute.
#[derive(Debug, Clone, Default)]
pub struct MemoryQueryCompiler {
    templates: Vec<LearnedTemplate>,
}

impl MemoryQueryCompiler {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// Import a template that was reviewed before it reached this API.
    ///
    /// Automatically observed examples must use [`Self::infer_candidate`] and
    /// [`Self::promote_candidate`] so that regression and human-review gates
    /// cannot be bypassed accidentally.
    pub fn learn_natural_language_template(
        &mut self,
        natural_language: &str,
        exact_query: &str,
        exact_dialect: QueryDialect,
    ) -> Result<(), MemoryQueryError> {
        if matches!(exact_dialect, QueryDialect::NaturalLanguage) {
            return Err(MemoryQueryError::new("learning_exact_exemplar_required"));
        }
        let natural_fields = placeholder_names(natural_language)?;
        let exact_fields = placeholder_names(exact_query)?;
        if natural_fields != exact_fields || natural_fields.is_empty() {
            return Err(MemoryQueryError::new(format!(
                "learning_placeholder_mismatch:{natural_fields:?}:{exact_fields:?}"
            )));
        }
        self.templates.push(LearnedTemplate {
            natural_language: natural_language.trim().to_owned(),
            exact_query: exact_query.trim().to_owned(),
            exact_dialect,
            candidate_id: {
                let mut identity = natural_language.to_owned();
                identity.push('\n');
                identity.push_str(exact_query);
                crate::engine::stable_id("memory_query_reviewed_import", &identity)
            },
            reviewer: String::from("pre_reviewed_configuration"),
            gate_suite: String::from("trusted_import"),
        });
        Ok(())
    }

    /// Infer a single-slot reusable template from at least two successful
    /// examples. The candidate is inert until [`Self::promote_candidate`].
    pub fn infer_candidate(
        observations: &[MemoryQueryLearningObservation],
        limits: MemoryProgramLimits,
    ) -> Result<MemoryQueryLearningCandidate, MemoryQueryError> {
        if observations.len() < 2 {
            return Err(MemoryQueryError::new("learning_observations_insufficient"));
        }
        let dialect = observations[0].exact_dialect;
        if matches!(dialect, QueryDialect::NaturalLanguage)
            || observations
                .iter()
                .any(|observation| observation.exact_dialect != dialect)
        {
            return Err(MemoryQueryError::new("learning_exact_dialect_mismatch"));
        }
        for observation in observations {
            compile_exact_memory_query(
                &observation.exact_query,
                observation.exact_dialect,
                limits,
                None,
            )?;
        }

        let natural_language_template = infer_single_span_template(
            &observations[0].natural_language,
            &observations[1].natural_language,
        )?;
        let exact_query_template =
            infer_single_span_template(&observations[0].exact_query, &observations[1].exact_query)?;
        validate_inferred_pair(
            &natural_language_template,
            &exact_query_template,
            observations,
            limits,
        )?;
        let identity = format!(
            "{}\n{}\n{}",
            dialect.as_str(),
            natural_language_template,
            exact_query_template
        );
        Ok(MemoryQueryLearningCandidate {
            id: crate::engine::stable_id("memory_query_learning_candidate", &identity),
            natural_language_template,
            exact_query_template,
            exact_dialect: dialect,
            evidence_count: observations.len(),
        })
    }

    /// Promote an inferred candidate only after a green regression gate and a
    /// named human approval. Until promotion, later queries cannot match it.
    pub fn promote_candidate(
        &mut self,
        candidate: MemoryQueryLearningCandidate,
        gate: &MemoryQueryLearningGate,
        approval: &MemoryQueryLearningApproval,
    ) -> Result<(), MemoryQueryError> {
        if gate.suite.trim().is_empty() || gate.passed == 0 || gate.failed != 0 {
            return Err(MemoryQueryError::new("learning_green_gate_required"));
        }
        if !approval.granted || approval.reviewer.trim().is_empty() {
            return Err(MemoryQueryError::new("learning_human_approval_required"));
        }
        if candidate.evidence_count < 2 {
            return Err(MemoryQueryError::new(
                "learning_candidate_evidence_insufficient",
            ));
        }
        let natural_fields = placeholder_names(&candidate.natural_language_template)?;
        let exact_fields = placeholder_names(&candidate.exact_query_template)?;
        if natural_fields != exact_fields || natural_fields.is_empty() {
            return Err(MemoryQueryError::new("learning_binding_schema_mismatch"));
        }
        if self
            .templates
            .iter()
            .any(|template| template.candidate_id == candidate.id)
        {
            return Err(MemoryQueryError::new("learning_candidate_already_promoted"));
        }
        self.templates.push(LearnedTemplate {
            natural_language: candidate.natural_language_template,
            exact_query: candidate.exact_query_template,
            exact_dialect: candidate.exact_dialect,
            candidate_id: candidate.id,
            reviewer: approval.reviewer.trim().to_owned(),
            gate_suite: gate.suite.trim().to_owned(),
        });
        Ok(())
    }

    pub fn compile(
        &self,
        source: &str,
        dialect: QueryDialect,
        limits: MemoryProgramLimits,
    ) -> Result<CompiledMemoryQuery, MemoryQueryError> {
        if dialect != QueryDialect::NaturalLanguage {
            return compile_exact_memory_query(source, dialect, limits, None);
        }
        for learned in &self.templates {
            let Some(bindings) = match_template(&learned.natural_language, source) else {
                continue;
            };
            let exact = instantiate_exact_query(&learned.exact_query, &bindings)?;
            return compile_exact_memory_query(
                &exact,
                learned.exact_dialect,
                limits,
                Some(learned.natural_language.clone()),
            );
        }
        Err(MemoryQueryError::new(
            "program_gap:no_learned_natural_language_memory_query",
        ))
    }

    #[must_use]
    pub fn learning_links_notation(&self) -> String {
        let mut out = String::from("memory_query_learning");
        push_lino_node(&mut out, 2, "promotion_policy", Some("human_gated"));
        for template in &self.templates {
            push_lino_node(&mut out, 2, "natural_language_template", None);
            push_lino_node(&mut out, 4, "candidate", Some(&template.candidate_id));
            push_lino_node(&mut out, 4, "surface", Some(&template.natural_language));
            push_lino_node(
                &mut out,
                4,
                "exact_dialect",
                Some(template.exact_dialect.as_str()),
            );
            push_lino_node(&mut out, 4, "exact_query", Some(&template.exact_query));
            push_lino_node(&mut out, 4, "gate_suite", Some(&template.gate_suite));
            push_lino_node(&mut out, 4, "reviewer", Some(&template.reviewer));
        }
        out
    }
}

fn infer_single_span_template(first: &str, second: &str) -> Result<String, MemoryQueryError> {
    let prefix = first
        .char_indices()
        .zip(second.chars())
        .take_while(|((_, left), right)| left.eq_ignore_ascii_case(right))
        .map(|((index, character), _)| index + character.len_utf8())
        .last()
        .unwrap_or(0);
    let remaining = first
        .len()
        .saturating_sub(prefix)
        .min(second.len().saturating_sub(prefix));
    let suffix = first[prefix..]
        .chars()
        .rev()
        .zip(second[prefix..].chars().rev())
        .take_while(|(left, right)| left.eq_ignore_ascii_case(right))
        .map(|(character, _)| character.len_utf8())
        .scan(0usize, |total, width| {
            *total += width;
            Some(*total)
        })
        .take_while(|total| *total <= remaining)
        .last()
        .unwrap_or(0);
    let first_capture = first[prefix..first.len() - suffix].trim();
    let second_capture = second[prefix..second.len() - suffix].trim();
    if first_capture.is_empty()
        || second_capture.is_empty()
        || first_capture.eq_ignore_ascii_case(second_capture)
    {
        return Err(MemoryQueryError::new("learning_reusable_span_missing"));
    }
    Ok(format!(
        "{}{{value}}{}",
        &first[..prefix],
        &first[first.len() - suffix..]
    ))
}

fn validate_inferred_pair(
    natural_template: &str,
    exact_template: &str,
    observations: &[MemoryQueryLearningObservation],
    limits: MemoryProgramLimits,
) -> Result<(), MemoryQueryError> {
    for observation in observations {
        let natural_bindings = match_template(natural_template, &observation.natural_language)
            .ok_or_else(|| MemoryQueryError::new("learning_natural_exemplar_lost"))?;
        let exact_bindings = match_template(exact_template, &observation.exact_query)
            .ok_or_else(|| MemoryQueryError::new("learning_exact_exemplar_lost"))?;
        if !natural_bindings.iter().all(|(name, value)| {
            exact_bindings
                .get(name)
                .is_some_and(|exact| exact.eq_ignore_ascii_case(value))
        }) {
            return Err(MemoryQueryError::new("learning_inferred_binding_mismatch"));
        }
        let instantiated = instantiate_exact_query(exact_template, &natural_bindings)?;
        let inferred =
            compile_exact_memory_query(&instantiated, observation.exact_dialect, limits, None)?;
        let observed = compile_exact_memory_query(
            &observation.exact_query,
            observation.exact_dialect,
            limits,
            None,
        )?;
        if inferred.canonical_semantics() != observed.canonical_semantics() {
            return Err(MemoryQueryError::new("learning_semantic_drift"));
        }
    }
    Ok(())
}

fn placeholder_names(template: &str) -> Result<BTreeSet<String>, MemoryQueryError> {
    let mut names = BTreeSet::new();
    let mut cursor = 0;
    while let Some(relative_open) = template[cursor..].find('{') {
        let open = cursor + relative_open;
        let close = open
            + template[open..]
                .find('}')
                .ok_or_else(|| MemoryQueryError::new("learning_placeholder_unterminated"))?;
        let name = &template[open + 1..close];
        if name.is_empty()
            || !name
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_')
        {
            return Err(MemoryQueryError::new(format!(
                "learning_placeholder_invalid:{name}"
            )));
        }
        names.insert(name.to_owned());
        cursor = close + 1;
    }
    Ok(names)
}

fn match_template(template: &str, source: &str) -> Option<BTreeMap<String, String>> {
    let mut bindings: BTreeMap<String, String> = BTreeMap::new();
    let mut template_cursor = 0;
    let mut source_cursor = 0;
    while let Some(relative_open) = template[template_cursor..].find('{') {
        let open = template_cursor + relative_open;
        let close = open + template[open..].find('}')?;
        let literal = &template[template_cursor..open];
        source_cursor += case_insensitive_prefix_len(&source[source_cursor..], literal)?;
        let name = &template[open + 1..close];
        template_cursor = close + 1;
        let next_open = template[template_cursor..]
            .find('{')
            .map_or(template.len(), |position| template_cursor + position);
        let next_literal = &template[template_cursor..next_open];
        let capture_end = if next_literal.is_empty() {
            source.len()
        } else {
            find_case_insensitive(&source[source_cursor..], next_literal)? + source_cursor
        };
        let captured = source[source_cursor..capture_end].trim();
        if captured.is_empty() {
            return None;
        }
        if let Some(previous) = bindings.get(name) {
            if !previous.eq_ignore_ascii_case(captured) {
                return None;
            }
        } else {
            bindings.insert(name.to_owned(), captured.to_owned());
        }
        source_cursor = capture_end;
    }
    let suffix = &template[template_cursor..];
    case_insensitive_prefix_len(&source[source_cursor..], suffix)
        .is_some_and(|length| source_cursor + length == source.len())
        .then_some(bindings)
}

fn instantiate_exact_query(
    template: &str,
    bindings: &BTreeMap<String, String>,
) -> Result<String, MemoryQueryError> {
    let mut query = template.to_owned();
    for (name, value) in bindings {
        if value
            .chars()
            .any(|character| matches!(character, '\'' | '"' | '`' | ';' | '{' | '}' | '(' | ')'))
        {
            return Err(MemoryQueryError::new(format!(
                "learning_capture_unsafe:{name}"
            )));
        }
        query = query.replace(&format!("{{{name}}}"), value);
    }
    Ok(query)
}

fn case_insensitive_prefix_len(text: &str, prefix: &str) -> Option<usize> {
    text.get(..prefix.len())
        .filter(|candidate| candidate.eq_ignore_ascii_case(prefix))
        .map(str::len)
}

fn find_case_insensitive(text: &str, needle: &str) -> Option<usize> {
    let lower_text = text.to_lowercase();
    let lower_needle = needle.to_lowercase();
    lower_text.find(&lower_needle)
}
