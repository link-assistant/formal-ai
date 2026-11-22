//! Universal *numeric-list* coding algorithm (issue #395).
//!
//! Issue #395 reported one symptom — "sort these numbers in JavaScript, give me
//! the code and the result" answered `unknown`. Rather than bolt on a single
//! sorting handler, this module reasons about a whole *family* of tasks over a
//! list of given numbers and, because every member is a pure, decidable function,
//! computes the result deterministically while emitting runnable code in the
//! requested language.
//!
//! Nothing here is hardcoded per reported prompt:
//!
//! * **Recognition is seed data.** The operation verbs come from
//!   `data/seed/operation-vocabulary.lino` (`sort`, `reverse_sort`, `reverse`,
//!   `sum`, `product`, `minimum`, `maximum`) in every supported UI language, and
//!   the target language from the `program_language_<slug>` alias meanings.
//! * **Classification is seed data.** `data/seed/numeric-list-operations.lino` is a
//!   small type ontology that maps each operation to a *family*
//!   (`list_transformation` / `list_reduction`) and a *result kind*
//!   (`list` / `scalar`). The solver reads that ontology to decide whether the
//!   answer is a reordered list or a single value — it does not branch on the
//!   operation name in prose.
//! * **Computation is generic.** A single reducer/transformer folds the parsed
//!   values; adding an operation is seed data plus one arithmetic clause, not a
//!   new handler.
//! * **Code generation is seed data.** `data/seed/coding-idioms.lino` declares
//!   per-language scaffolds and idioms; `codegen` discovers the composition at
//!   execution time by walking the language's inheritance chain and recursively
//!   expanding idiom slots — there are no per-language renderer functions.
//!
//! The handler only fires when three independent signals coincide — an operation
//! verb, a known programming language, and at least two numbers — so it never
//! steals plain prose, and it defers function-synthesis prompts to the dedicated
//! `program_synthesis` handler.

mod stated_result;

use std::fmt::Write as _;

use crate::coding::ProgramLanguage;
use crate::coding::list_program::{
    self, compute, detect_operation, family_for, parse_list_items, result_kind_for,
    value_type_label,
};
pub use crate::coding::list_program::{
    Operation, ParsedListItem, Transform, names_list_items, parse_numbers,
};
use crate::language::{Language, detect as detect_language};
use crate::meta_algorithm_builder::{CodingSurface, MetaAlgorithmBuilder};
use crate::seed;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::{engine::SymbolicAnswer, event_log::EventLog};

use super::finalize_simple;

/// Coding context inherited from earlier conversation turns when the current
/// prompt is a bare numeric-list follow-up (issue #412).
///
/// Issue #412 reported `Отсортируй 4, 3, 1, 17, 8, 9, 15` answering `unknown`.
/// In isolation that turn names no programming language and does not ask for
/// code, so the numeric-list handler declined it. But the *previous* turn —
/// "…отсортируй их в JavaScript, дай мне код и результат" — established both: a
/// target language and a code+result request. A follow-up imperative over a new
/// list of numbers continues that coding context. We recover the missing
/// parameters from history rather than hardcoding the reported prompt.
#[derive(Clone, Default)]
struct InheritedCoding {
    /// The most recent programming language the user asked numeric-list code in.
    language: Option<&'static ProgramLanguage>,
    /// Whether an earlier numeric-list turn explicitly asked for code, so a bare
    /// reduction follow-up ("sum them") still counts as a code request.
    code_requested: bool,
    /// The list of numbers from the most recent numeric-list turn, inherited
    /// when a bare operation follow-up names an operation but no numbers of its
    /// own (issue #427: "Сделай инверсию сортировки." after a prior sort).
    items: Vec<ParsedListItem>,
}

/// The fully-reasoned solution: the given numbers, the operation, the resolved
/// language, the generated code, and the deterministically-computed result.
#[derive(Debug, Clone)]
pub struct NumericListSolution {
    pub language_slug: &'static str,
    pub language_name: &'static str,
    pub code_fence: &'static str,
    pub operation: Operation,
    /// `"list"` or `"scalar"`, read from the seed ontology.
    pub result_kind: String,
    pub value_type: &'static str,
    pub given: Vec<String>,
    /// For a transformation: the reordered surface tokens. For a reduction: a
    /// single-element vector holding the computed scalar.
    pub result: Vec<String>,
    /// Structural program tree used to render `code`.
    pub syntax_tree: String,
    /// CST/AST parsed from the rendered source by the meta-language links
    /// network — the sole CST/AST engine.
    pub cst_tree: String,
    /// Which engine validated the source (always `meta_language`).
    pub cst_engine: String,
    pub code: String,
}

/// Specialized-handler entry point. Returns `Some` only when the prompt is a
/// concrete "<operation> these numbers in <language>" request.
///
/// This context-free variant keeps the uniform specialized-handler signature.
/// The dispatch calls [`try_numeric_list_with_history`] so a bare follow-up can
/// inherit the language and code request from earlier turns (issue #412).
pub fn try_numeric_list(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_numeric_list_with_history(prompt, normalized, log, &[])
}

/// Conversation-aware entry point. When the current prompt omits the target
/// language (and, for reductions, the code request), recover them from the most
/// recent numeric-list coding turn so a bare "<operation> these numbers"
/// follow-up continues the established coding context (issue #412).
pub fn try_numeric_list_with_history(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
    history: &[ConversationTurn],
) -> Option<SymbolicAnswer> {
    let vocabulary = seed::operation_vocabulary();
    let normalized = crate::web_engine_core::normalize_prompt(prompt);
    if vocabulary.matches("algorithm_request", &normalized)
        && matches!(
            detect_operation(&vocabulary, &normalized),
            Some(Operation::Transform(
                Transform::SortAscending | Transform::SortDescending
            ))
        )
    {
        return crate::rule_interpreter::run_handler(
            "algorithm",
            prompt,
            &prompt.to_lowercase(),
            log,
        );
    }
    let inherited = numeric_list_history_context(history);
    let has_inheritance =
        inherited.language.is_some() || inherited.code_requested || !inherited.items.is_empty();
    let solution = if has_inheritance {
        solve_numeric_list_with_context(prompt, &inherited)
    } else {
        solve_numeric_list(prompt)
    };
    // R1017: with no language and no code request, "sort the numbers 5, 2,
    // 9, 1" asks for the sorted list itself.
    let Some(solution) = solution else {
        return stated_result::try_stated_result(prompt, log);
    };
    MetaAlgorithmBuilder::for_surface(CodingSurface::NumericList).record(log);
    if has_inheritance {
        log.append(
            "numeric_list_coreference",
            format!(
                "inherited_language={} inherited_code_request={}",
                inherited.language.map_or("none", |language| language.slug),
                inherited.code_requested,
            ),
        );
    }

    log.append("formalize", solution.formalization());
    log.append(
        "synthesis:spec",
        format!(
            "language={} task=numeric_list operation={} family={} result_kind={} value_type={}",
            solution.language_slug,
            solution.operation.canonical(),
            solution.family(),
            solution.result_kind,
            solution.value_type,
        ),
    );
    log.append("synthesis:given", solution.given.join(", "));
    log.append("synthesis:syntax_tree", solution.syntax_tree.clone());
    log.append("composition:code_fragment", solution.code.clone());
    log.append("synthesis:cst_engine", solution.cst_engine.clone());
    log.append("synthesis:cst_tree", solution.cst_tree.clone());
    // The result is computed by the solver, not a sandbox: every operation is a
    // pure, total function over the parsed values, so the answer is verified by
    // construction rather than by running untrusted code.
    log.append("execution_status", "computed deterministically".to_owned());
    log.append(
        "execution_environment",
        "pure in-solver evaluation of a decidable numeric-list operation".to_owned(),
    );
    log.append("execution_result", solution.result.join(", "));

    let language = detect_language(prompt);
    let body = solution.render(language);
    Some(finalize_simple(
        prompt,
        log,
        "write_program",
        &format!(
            "response:write_program:numeric_list:{}:{}",
            solution.operation.canonical(),
            solution.language_slug
        ),
        &body,
        1.0,
    ))
}

/// Pure core: parse the request and produce a [`NumericListSolution`], or `None`
/// when the prompt is not a numeric-list coding task.
///
/// The prompt is re-normalized here with [`crate::web_engine_core::normalize_prompt`]
/// rather than trusting the dispatch's plain `to_lowercase()`: punctuation must
/// fold to whitespace so a language word glued to a comma (`JavaScript,`) still
/// resolves through the token-boundary alias matcher.
#[must_use]
pub fn solve_numeric_list(prompt: &str) -> Option<NumericListSolution> {
    solve_numeric_list_with_context(prompt, &InheritedCoding::default())
}

/// Core solver with optional conversation-inherited coding context (issue #412).
/// When `inherited.language` is present the current prompt need not name a
/// language itself; when `inherited.code_requested` is set a reduction
/// follow-up counts as a code request even without the verb in this turn.
#[must_use]
fn solve_numeric_list_with_context(
    prompt: &str,
    inherited: &InheritedCoding,
) -> Option<NumericListSolution> {
    let normalized = crate::web_engine_core::normalize_prompt(prompt);
    let normalized = normalized.as_str();
    let vocabulary = crate::seed::operation_vocabulary();

    // Defer genuine function-synthesis prompts ("write a function that returns
    // the sum of 3 and 5") to the dedicated program-synthesis handler: those
    // ask for a reusable function, not a one-shot computation over given values.
    if vocabulary.matches("function", normalized) || prompt.contains("def ") {
        return None;
    }
    // A named format conversion reads its payload as data, not as a list to
    // reduce ("Convert this JSON to YAML: {\"sort\": \"words\"}").
    if super::text_manipulation::names_format_conversion(prompt) {
        return None;
    }

    let operation = detect_operation(&vocabulary, normalized)?;

    // Reductions (`sum`, `product`, `minimum`, `maximum`) phrase-overlap with
    // ordinary prose — "in total", "the largest building" — far more than the
    // imperative transform verbs do. They only fire when the prompt explicitly
    // asks for code (the `code_request` operation), which is exactly the issue
    // #395 contract: "give me the code and the result". A reduction follow-up
    // inherits the code request from an earlier numeric-list turn (issue #412).
    // Transformations keep their unambiguous-verb behavior.
    if matches!(operation, Operation::Reduce(_))
        && !vocabulary.matches("code_request", normalized)
        && !inherited.code_requested
    {
        return None;
    }

    // The target language may come from this turn or, for a bare follow-up, from
    // the most recent numeric-list coding turn in the conversation.
    let language = crate::coding::composition_language(normalized, inherited.language)?;
    let mut items = parse_list_items(prompt, operation);
    // Issue #427: a bare operation follow-up ("Сделай инверсию сортировки.")
    // names no numbers of its own — it refers to the list from the previous
    // numeric-list turn. Inherit that list so the operation has data to act on.
    if items.is_empty() && !inherited.items.is_empty() {
        items.clone_from(&inherited.items);
    }
    if items.len() < 2 {
        return None;
    }
    if matches!(operation, Operation::Reduce(_)) && items.iter().any(ParsedListItem::is_text) {
        return None;
    }

    let is_float = items
        .iter()
        .filter_map(ParsedListItem::numeric_value)
        .any(|value| value.fract() != 0.0);
    let value_type = value_type_label(&items, is_float);
    let given: Vec<String> = items.iter().map(|n| n.text.clone()).collect();
    let result = compute(operation, &items, is_float);
    let program = list_program::build(language, &items, operation, is_float);
    let syntax_tree = program.links_notation();
    let code = program.render()?;
    let cst = crate::coding::validated_program_cst(language.slug, &code)?;
    let cst_engine = cst.engine().to_owned();
    let cst_tree = cst.links_notation();
    let result_kind = result_kind_for(operation.canonical()).to_owned();

    Some(NumericListSolution {
        language_slug: language.slug,
        language_name: language.name,
        code_fence: language.code_fence,
        operation,
        result_kind,
        value_type,
        given,
        result,
        syntax_tree,
        cst_tree,
        cst_engine,
        code,
    })
}

/// Recover the coding context (target language and whether code was requested)
/// from the most recent prior turn that was itself a numeric-list coding
/// request. A turn qualifies only when it names a numeric-list operation, a
/// supported programming language, and at least two numbers — the same three
/// signals the live handler requires — so unrelated chatter never leaks a
/// language into a later follow-up (issue #412).
fn numeric_list_history_context(history: &[ConversationTurn]) -> InheritedCoding {
    let vocabulary = crate::seed::operation_vocabulary();
    let mut inherited = InheritedCoding::default();
    for turn in history.iter().rev() {
        if turn.role != ConversationRole::User {
            continue;
        }
        let normalized = crate::web_engine_core::normalize_prompt(&turn.content);
        let normalized = normalized.as_str();
        if detect_operation(&vocabulary, normalized).is_none() {
            continue;
        }
        let numbers = parse_numbers(&turn.content);
        if numbers.len() < 2 {
            continue;
        }
        // The most recent numeric-list turn with a concrete list seeds the list
        // a bare operation follow-up inherits (issue #427). This turn may name
        // no language of its own — the language is recovered separately below.
        if inherited.items.is_empty() {
            inherited.items = numbers;
        }
        // The language (and code request) come from the most recent turn that
        // named a programming language alongside its list (issue #412).
        if inherited.language.is_none()
            && let Some(language) = crate::coding::program_language_by_alias(normalized)
        {
            inherited.language = Some(language);
            inherited.code_requested = vocabulary.matches("code_request", normalized);
        }
        if !inherited.items.is_empty() && inherited.language.is_some() {
            break;
        }
    }
    inherited
}

/// Whether an earlier turn established the list, language or code request.
///
/// The `prior_numeric_list` claim evidence of issue #1175 R3.
#[must_use]
pub fn continues_numeric_list(history: &[ConversationTurn]) -> bool {
    let inherited = numeric_list_history_context(history);
    inherited.language.is_some() || inherited.code_requested || !inherited.items.is_empty()
}

impl NumericListSolution {
    /// The seed ontology family this operation belongs to.
    #[must_use]
    pub fn family(&self) -> &'static str {
        family_for(self.operation.canonical())
    }

    fn formalization(&self) -> String {
        format!(
            "(@USER OP:{} values:[{}] value_type:{} language:{} result_kind:{} request:[code result])",
            self.operation.canonical(),
            self.given.join(" "),
            self.value_type,
            self.language_slug,
            self.result_kind,
        )
    }

    /// Render the localized answer: a sentence naming the language, the given
    /// numbers, and the operation; the generated code; and the computed result.
    #[must_use]
    pub fn render(&self, language: Language) -> String {
        let given = self.given.join(", ");
        // Issue #918: the intro, the value noun and the result label are the
        // seeded numeric_list_* responses the browser twin renders too.
        let noun_intent = if self.value_type == "string" {
            "numeric_list_noun_strings"
        } else {
            "numeric_list_noun_numbers"
        };
        let noun = seed::localized_response(noun_intent, "en").unwrap_or_default();
        let intro = seed::render_localized_once(
            &format!("numeric_list_intro_{}", self.operation.canonical()),
            language.slug(),
            &[
                ("lang", self.language_name),
                ("given", &given),
                ("noun", &noun),
            ],
        );
        let result_label = seed::localized_response("numeric_list_result_label", language.slug())
            .unwrap_or_default();
        // The seed ontology decides how the result reads: a transformation lists
        // the reordered numbers, a reduction shows a single value.
        let shown = if self.result_kind == "scalar" {
            self.result.first().cloned().unwrap_or_default()
        } else {
            self.result.join(", ")
        };
        let mut body = String::new();
        let _ = write!(
            body,
            "{}\n\n```{}\n{}\n```\n\n{} {}",
            intro, self.code_fence, self.code, result_label, shown
        );
        body
    }
}
