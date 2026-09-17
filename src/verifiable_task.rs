//! A task whose answer can be checked, in any domain (#1138 B8).
//!
//! Plan 08 owns this module. Recognition is seed data in five languages, never a
//! suite id and never an English literal in Rust; `identity()` deliberately
//! excludes literal values so a renumbered paraphrase recalls the same
//! *procedure* and is recomputed rather than replayed.
//!
//! The benchmark grader must stay invisible here: nothing under
//! `src/verifiable_task*` may import `external_benchmarks`.
//!
//! Wave T lands the shapes only; wave I8 leaves 08-L3 through 08-L6 fill the
//! bodies in.

pub mod ledger;
pub mod quantities;

use std::collections::BTreeSet;

use crate::coding::task_spec::Example;
use crate::links_format::push_lino_node;
use crate::seed::Slot;
use quantities::{Entity, Quantity};

/// What kind of observation would settle this task — the shape the **answer**
/// must take.
///
/// This is *not* plan 05's `ObligationExpectation`, which declares what must be
/// **observed** before an obligation node may be called satisfied. The two are
/// bridged once, by [`TaskExpectation::to_obligation_expectation`], so there is
/// exactly one satisfaction rule in the tree (plan 00 §9 R15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExpectation {
    /// A single number is the answer. `unit` is `Some` when the prompt names one.
    Numeric {
        /// The unit the prompt names, when it names one.
        unit: Option<String>,
    },
    /// A cardinality over a described collection.
    Count {
        /// The category the prompt counts over.
        subject: String,
    },
    /// A transformed version of a supplied text.
    EditedText {
        /// The text the prompt supplied.
        source: String,
    },
    /// A value for a named unknown, as an equation states it.
    Unknown {
        /// The unknown's name.
        name: String,
    },
    /// A named callable checked by examples — today's `CodingTaskSpec`.
    Callable {
        /// The examples the prompt states.
        examples: Vec<Example>,
    },
    /// A process whose stdout is the observation.
    Stdout {
        /// The stdout the prompt states, when it states one.
        expected: Option<String>,
    },
    /// A yes/no claim.
    Boolean,
}

impl TaskExpectation {
    /// Bridge to plan 05's obligation vocabulary.
    ///
    /// `check_id` is `"<VerifiedAnswer::derivation_id>:<check slug>"`.
    #[must_use]
    pub fn to_obligation_expectation(
        &self,
        check_id: &str,
    ) -> crate::obligation_ledger::ObligationExpectation {
        crate::obligation_ledger::ObligationExpectation::SymbolicCheck {
            check_id: check_id.to_owned(),
        }
    }

    /// Stable slug used in the trace and the ledger.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Numeric { .. } => "numeric",
            Self::Count { .. } => "count",
            Self::EditedText { .. } => "edited_text",
            Self::Unknown { .. } => "unknown",
            Self::Callable { .. } => "callable",
            Self::Stdout { .. } => "stdout",
            Self::Boolean => "boolean",
        }
    }
}

/// How an answer of this expectation must be presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerShape {
    /// The number stands alone at the end of the answer.
    TrailingNumber,
    /// The value is delimited the way the prompt's own convention requires.
    DelimitedValue,
    /// The edited text is the whole answer body.
    WholeBody,
    /// A fenced program.
    CodeBlock,
}

/// A task whose answer can be checked, in any domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiableTask {
    /// The prompt as received, unmodified.
    pub prompt: String,
    /// Sentences the requirement decomposes into.
    pub requirement_sentences: Vec<String>,
    /// What kind of observation would settle it.
    pub expectation: TaskExpectation,
    /// How the answer must be presented.
    pub shape: AnswerShape,
    /// Named quantities the prompt states, in order of appearance.
    pub quantities: Vec<Quantity>,
    /// Entities the prompt lists, for `Count` tasks.
    pub entities: Vec<Entity>,
    /// Detected prose language slug (`en`, `ru`, `hi`, `zh`, `es`).
    pub prose_language: String,
}

impl VerifiableTask {
    /// Inspectable material hashed by [`Self::identity`].
    ///
    /// Keeping this projection separate makes the anti-memorization property
    /// testable without searching an opaque hexadecimal digest for a one-digit
    /// quantity that can occur there by chance.
    #[must_use]
    pub fn identity_payload(&self) -> String {
        let requirements = self
            .requirement_sentences
            .iter()
            .map(|sentence| quantities::normalized_shape(sentence, &self.prose_language))
            .collect::<Vec<_>>()
            .join("\u{0}");
        let quantity_shape = self
            .quantities
            .iter()
            .map(|quantity| {
                [
                    quantity.unit.as_deref().unwrap_or("_"),
                    quantity.label.as_deref().unwrap_or("_"),
                ]
                .join(":")
            })
            .collect::<Vec<_>>()
            .join("\u{0}");
        [
            self.expectation.slug(),
            answer_shape_slug(self.shape),
            &self.prose_language,
            &requirements,
            &quantity_shape,
        ]
        .join("\u{1}")
    }

    /// Stable identity for ledger recall: normalized sentences + expectation
    /// kind + quantity shape. Deliberately excludes literal values.
    #[must_use]
    pub fn identity(&self) -> String {
        crate::engine::stable_id("verifiable_task", &self.identity_payload())
    }

    /// Links Notation projection of the task.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "verifiable_task", None);
        push_lino_node(&mut out, 2, "id", Some(&self.identity()));
        push_lino_node(&mut out, 2, "prompt", Some(&self.prompt));
        push_lino_node(&mut out, 2, "expectation", Some(self.expectation.slug()));
        push_lino_node(
            &mut out,
            2,
            "answer_shape",
            Some(answer_shape_slug(self.shape)),
        );
        push_lino_node(&mut out, 2, "prose_language", Some(&self.prose_language));
        for requirement in &self.requirement_sentences {
            push_lino_node(&mut out, 2, "requirement", Some(requirement));
        }
        for quantity in &self.quantities {
            push_lino_node(&mut out, 2, "quantity", Some(&quantity.value));
            push_lino_node(&mut out, 4, "offset", Some(&quantity.offset.to_string()));
            if let Some(unit) = &quantity.unit {
                push_lino_node(&mut out, 4, "unit", Some(unit));
            }
            if let Some(label) = &quantity.label {
                push_lino_node(&mut out, 4, "label", Some(label));
            }
        }
        for entity in &self.entities {
            push_lino_node(&mut out, 2, "entity", Some(&entity.surface));
            push_lino_node(&mut out, 4, "multiplicity", Some(&entity.multiplicity));
            push_lino_node(&mut out, 4, "offset", Some(&entity.offset.to_string()));
        }
        out.trim_end().to_owned()
    }

    /// Present `value` the way `shape` requires, in `prose_language`.
    #[must_use]
    pub fn render(&self, value: &str, reasoning: &str) -> String {
        if self.shape == AnswerShape::WholeBody {
            return value.to_owned();
        }
        let subject = match &self.expectation {
            TaskExpectation::Unknown { name } => name.as_str(),
            TaskExpectation::Count { subject } => subject.as_str(),
            _ => "_",
        };
        let intent = match self.shape {
            AnswerShape::TrailingNumber => "verifiable_task_trailing_number",
            AnswerShape::DelimitedValue => "verifiable_task_delimited_value",
            AnswerShape::WholeBody => unreachable!(),
            AnswerShape::CodeBlock => "verifiable_task_code_block",
        };
        crate::seed::render_response(
            intent,
            &self.prose_language,
            &[
                ("value", value),
                ("reasoning", reasoning),
                ("subject", subject),
            ],
        )
        .unwrap_or_else(|| match self.shape {
            AnswerShape::TrailingNumber | AnswerShape::DelimitedValue | AnswerShape::WholeBody => {
                value.to_owned()
            }
            AnswerShape::CodeBlock => format!("```\n{value}\n```"),
        })
        .trim()
        .to_owned()
    }
}

/// Recognize any task carrying a checkable expectation.
///
/// Returns `None` for an open-ended request. Every cue is a seed meaning from
/// `data/seed/meanings-verifiable-task.lino`; no suite id, no benchmark name and
/// no hard-coded English phrase appears here.
#[must_use]
pub fn recognise_verifiable(prompt: &str) -> Option<VerifiableTask> {
    if let Some(spec) = crate::coding::task_spec::recognise(prompt) {
        let (expectation, shape) = match spec.artifact_shape {
            crate::coding::task_spec::ArtifactShape::Function => (
                TaskExpectation::Callable {
                    examples: spec.examples.clone(),
                },
                AnswerShape::CodeBlock,
            ),
            crate::coding::task_spec::ArtifactShape::Program => (
                TaskExpectation::Stdout {
                    expected: spec.expected_stdout.clone(),
                },
                AnswerShape::CodeBlock,
            ),
        };
        let quantities = quantities::extract_quantities(prompt, &spec.prose_language);
        return Some(VerifiableTask {
            prompt: prompt.to_owned(),
            requirement_sentences: spec.requirement_sentences,
            expectation,
            shape,
            quantities,
            entities: Vec::new(),
            prose_language: spec.prose_language,
        });
    }

    let detected_language = crate::language::detect(prompt).slug();
    let mut language_priority = vec![detected_language];
    language_priority.extend(
        crate::language::registered_languages()
            .into_iter()
            .map(crate::language::Language::slug)
            .filter(|language| *language != detected_language),
    );
    let matched = [
        ("verifiable_expectation_count", AnswerShape::TrailingNumber),
        ("verifiable_expectation_edited_text", AnswerShape::WholeBody),
        (
            "verifiable_expectation_unknown",
            AnswerShape::DelimitedValue,
        ),
        (
            "verifiable_expectation_numeric",
            AnswerShape::TrailingNumber,
        ),
    ]
    .into_iter()
    .find_map(|(role, shape)| {
        language_priority.iter().find_map(|language| {
            slot_capture_for_role(prompt, language, role)
                .map(|capture| (role, shape, capture, (*language).to_owned()))
        })
    })?;
    let prose_language = matched.3;
    let quantities = quantities::extract_quantities(prompt, &prose_language);
    if matched.0 == "verifiable_expectation_unknown"
        && (!prompt.contains('=') || quantities.len() < 2)
    {
        // A request to find an integer under inequalities has a checkable
        // answer, but it is already owned by the interval-reasoning method.
        // The generic unknown shape is specifically an equation declaration;
        // without an equality there is no substitution round trip to verify.
        return None;
    }
    let entities = if matched.0 == "verifiable_expectation_count" {
        quantities::extract_entities(prompt, &prose_language)
    } else {
        Vec::new()
    };
    let expectation = match matched.0 {
        "verifiable_expectation_count" => TaskExpectation::Count {
            subject: clean_capture(&matched.2),
        },
        "verifiable_expectation_edited_text" => TaskExpectation::EditedText {
            source: clean_capture(&matched.2),
        },
        "verifiable_expectation_unknown" => TaskExpectation::Unknown {
            name: clean_capture(&matched.2),
        },
        "verifiable_expectation_numeric" => TaskExpectation::Numeric {
            unit: quantities
                .iter()
                .rev()
                .find_map(|quantity| quantity.unit.clone()),
        },
        _ => return None,
    };
    Some(VerifiableTask {
        prompt: prompt.to_owned(),
        requirement_sentences: split_requirements(prompt),
        expectation,
        shape: matched.1,
        quantities,
        entities,
        prose_language,
    })
}

const fn answer_shape_slug(shape: AnswerShape) -> &'static str {
    match shape {
        AnswerShape::TrailingNumber => "trailing_number",
        AnswerShape::DelimitedValue => "delimited_value",
        AnswerShape::WholeBody => "whole_body",
        AnswerShape::CodeBlock => "code_block",
    }
}

fn slot_capture_for_role(prompt: &str, language: &str, role: &str) -> Option<String> {
    let lowered = prompt.to_lowercase();
    crate::seed::lexicon()
        .meanings_with_role(role)
        .flat_map(|meaning| &meaning.lexemes)
        .filter(|lexeme| lexeme.language == language)
        .flat_map(|lexeme| &lexeme.words)
        .filter_map(|form| capture_slot(prompt, &lowered, &form.text.to_lowercase(), form.slot()))
        .max_by_key(String::len)
}

fn capture_slot(original: &str, prompt: &str, form: &str, slot: Slot) -> Option<String> {
    let before = form.split_once('…').map_or(form, |(before, _)| before);
    let after = form.split_once('…').map_or("", |(_, after)| after);
    match slot {
        Slot::Bare => prompt.contains(form).then(String::new),
        Slot::Prefix => {
            let begin = prompt.find(before)? + before.len();
            Some(original[begin..].to_owned())
        }
        Slot::Suffix => {
            let end = prompt.find(after)?;
            Some(original[..end].to_owned())
        }
        Slot::Circumfix => {
            let begin = prompt.find(before)? + before.len();
            let end = prompt[begin..].find(after)? + begin;
            Some(original[begin..end].to_owned())
        }
    }
}

fn clean_capture(capture: &str) -> String {
    capture
        .trim_matches(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    ':' | '：' | ',' | '.' | '?' | '¿' | '。' | '？' | '।'
                )
        })
        .to_owned()
}

fn split_requirements(prompt: &str) -> Vec<String> {
    prompt
        .split(['.', '?', '!', '。', '？', '！', '।'])
        .map(str::trim)
        .filter(|sentence| !sentence.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Trusted source anchors proving that `entity` belongs to `category` in the
/// seed taxonomy. The solver handler supplies observations; the task model owns
/// this domain-neutral relationship walk and its cycle guard.
pub(crate) fn seeded_membership_sources(
    entity: &str,
    category: &str,
    language: &str,
) -> Option<Vec<String>> {
    let entity = seeded_meaning_for_surface(entity, language)?;
    let category = seeded_meaning_for_surface(category, language)?;
    if !seeded_is_a(&entity.slug, &category.slug, &mut BTreeSet::new()) {
        return None;
    }
    let mut sources = [entity, category]
        .into_iter()
        .filter_map(|meaning| grounding_url(&meaning.wikidata))
        .collect::<Vec<_>>();
    sources.sort();
    sources.dedup();
    (!sources.is_empty()).then_some(sources)
}

pub(crate) fn normalized_category(value: &str) -> String {
    value
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.strip_suffix('s').unwrap_or(token))
        .collect::<Vec<_>>()
        .join(" ")
}

fn seeded_meaning_for_surface(
    surface: &str,
    language: &str,
) -> Option<&'static crate::seed::Meaning> {
    let normalized = normalized_category(surface);
    crate::seed::lexicon()
        .meanings
        .iter()
        .filter_map(|meaning| {
            meaning
                .lexemes
                .iter()
                .filter(|lexeme| lexeme.language == language)
                .flat_map(|lexeme| &lexeme.words)
                .map(|word| normalized_category(&word.text))
                .filter(|word| !word.is_empty() && normalized.contains(word.as_str()))
                .map(|word| (word.len(), meaning))
                .max_by_key(|(length, _)| *length)
        })
        .max_by_key(|(length, _)| *length)
        .map(|(_, meaning)| meaning)
}

fn seeded_is_a(slug: &str, category: &str, visited: &mut BTreeSet<String>) -> bool {
    if slug == category {
        return true;
    }
    if !visited.insert(slug.to_owned()) {
        return false;
    }
    crate::seed::lexicon().meaning(slug).is_some_and(|meaning| {
        meaning
            .defined_by
            .iter()
            .any(|parent| seeded_is_a(parent, category, visited))
    })
}

fn grounding_url(grounding: &str) -> Option<String> {
    if matches!(grounding.as_bytes().first(), Some(b'Q' | b'P'))
        && grounding[1..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        Some(format!("https://www.wikidata.org/wiki/{grounding}"))
    } else {
        grounding
            .strip_prefix("https://")
            .map(|_| grounding.to_owned())
    }
}
