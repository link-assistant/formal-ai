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
//!
//! The handler only fires when three independent signals coincide — an operation
//! verb, a known programming language, and at least two numbers — so it never
//! steals plain prose, and it defers function-synthesis prompts to the dedicated
//! `program_synthesis` handler.

mod codegen;

use std::cmp::Ordering;
use std::fmt::Write as _;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};
use crate::seed::parser::{parse_lino, LinoNode};
use crate::seed::NUMERIC_LIST_OPERATIONS_LINO;

use super::finalize_simple;

/// Whether a list transformation orders the numbers up, down, or simply flips
/// their given order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    /// `sort` — ascending order.
    SortAscending,
    /// `reverse_sort` — descending order.
    SortDescending,
    /// `reverse` — the given order, reversed (no comparison).
    Reverse,
}

/// Which scalar a list reduction collapses the numbers into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reduce {
    Sum,
    Product,
    Minimum,
    Maximum,
}

/// One recognized numeric-list operation: either a transformation that yields
/// another list, or a reduction that yields a scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Transform(Transform),
    Reduce(Reduce),
}

impl Operation {
    /// The canonical operation token, matching both the
    /// `operation-vocabulary.lino` entry and the `numeric-list-operations.lino`
    /// ontology key.
    #[must_use]
    pub const fn canonical(self) -> &'static str {
        match self {
            Self::Transform(Transform::SortAscending) => "sort",
            Self::Transform(Transform::SortDescending) => "reverse_sort",
            Self::Transform(Transform::Reverse) => "reverse",
            Self::Reduce(Reduce::Sum) => "sum",
            Self::Reduce(Reduce::Product) => "product",
            Self::Reduce(Reduce::Minimum) => "minimum",
            Self::Reduce(Reduce::Maximum) => "maximum",
        }
    }
}

/// The value domain of a list item lifted from the prompt.
#[derive(Debug, Clone, Copy)]
pub enum ListValue {
    Number(f64),
    Text,
}

/// A single list item lifted from the prompt. The surface text is preserved for
/// echoing and code generation; the value domain drives sorting/reduction.
#[derive(Debug, Clone)]
pub struct ParsedListItem {
    pub text: String,
    pub value: ListValue,
}

impl ParsedListItem {
    const fn numeric_value(&self) -> Option<f64> {
        match self.value {
            ListValue::Number(value) => Some(value),
            ListValue::Text => None,
        }
    }

    const fn is_text(&self) -> bool {
        matches!(self.value, ListValue::Text)
    }
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
pub fn try_numeric_list(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let solution = solve_numeric_list(prompt)?;

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
    let normalized = crate::web_engine_core::normalize_prompt(prompt);
    let normalized = normalized.as_str();
    let vocabulary = crate::seed::operation_vocabulary();

    // Defer genuine function-synthesis prompts ("write a function that returns
    // the sum of 3 and 5") to the dedicated program-synthesis handler: those
    // ask for a reusable function, not a one-shot computation over given values.
    if vocabulary.matches("function", normalized) || prompt.contains("def ") {
        return None;
    }

    let operation = detect_operation(&vocabulary, normalized)?;

    // Reductions (`sum`, `product`, `minimum`, `maximum`) phrase-overlap with
    // ordinary prose — "in total", "the largest building" — far more than the
    // imperative transform verbs do. They only fire when the prompt explicitly
    // asks for code (the `code_request` operation), which is exactly the issue
    // #395 contract: "give me the code and the result". Transformations keep
    // their unambiguous-verb behavior.
    if matches!(operation, Operation::Reduce(_)) && !vocabulary.matches("code_request", normalized)
    {
        return None;
    }

    let language = crate::coding::program_language_by_alias(normalized)?;
    let items = parse_list_items(prompt, operation);
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
    let program = codegen::build(language, &items, operation, is_float);
    let syntax_tree = program.links_notation();
    let code = program.render();
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

/// Recognize which numeric-list operation the prompt asks for, in priority
/// order. Sort phrasings are checked first because "sort in reverse order"
/// legitimately contains the bare `reverse` verb; the descending variant wins
/// whenever the `reverse_sort` phrasing is present.
fn detect_operation(
    vocabulary: &crate::seed::OperationVocabulary,
    normalized: &str,
) -> Option<Operation> {
    if vocabulary.matches("sort", normalized) || vocabulary.matches("reverse_sort", normalized) {
        let transform = if vocabulary.matches("reverse_sort", normalized) {
            Transform::SortDescending
        } else {
            Transform::SortAscending
        };
        return Some(Operation::Transform(transform));
    }
    if vocabulary.matches("reverse", normalized) {
        return Some(Operation::Transform(Transform::Reverse));
    }
    for (canonical, reduce) in [
        ("sum", Reduce::Sum),
        ("product", Reduce::Product),
        ("minimum", Reduce::Minimum),
        ("maximum", Reduce::Maximum),
    ] {
        if vocabulary.matches(canonical, normalized) {
            return Some(Operation::Reduce(reduce));
        }
    }
    None
}

/// Apply the operation to the parsed list and return the surface tokens to
/// display: the reordered list for a transformation, or a single computed scalar
/// for a numeric reduction.
fn compute(operation: Operation, items: &[ParsedListItem], is_float: bool) -> Vec<String> {
    match operation {
        Operation::Transform(transform) => {
            let mut ordered: Vec<ParsedListItem> = items.to_vec();
            match transform {
                Transform::SortAscending => ordered.sort_by(compare_items),
                Transform::SortDescending => ordered.sort_by(|a, b| compare_items(b, a)),
                Transform::Reverse => ordered.reverse(),
            }
            ordered.into_iter().map(|n| n.text).collect()
        }
        Operation::Reduce(reduce) => {
            let value = match reduce {
                Reduce::Sum => items
                    .iter()
                    .filter_map(ParsedListItem::numeric_value)
                    .sum::<f64>(),
                Reduce::Product => items
                    .iter()
                    .filter_map(ParsedListItem::numeric_value)
                    .product::<f64>(),
                Reduce::Minimum => items
                    .iter()
                    .filter_map(ParsedListItem::numeric_value)
                    .fold(f64::INFINITY, f64::min),
                Reduce::Maximum => items
                    .iter()
                    .filter_map(ParsedListItem::numeric_value)
                    .fold(f64::NEG_INFINITY, f64::max),
            };
            vec![format_scalar(value, is_float)]
        }
    }
}

fn compare_items(left: &ParsedListItem, right: &ParsedListItem) -> Ordering {
    match (left.numeric_value(), right.numeric_value()) {
        (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
        _ => left.text.cmp(&right.text),
    }
}

fn value_type_label(items: &[ParsedListItem], is_float: bool) -> &'static str {
    if items.iter().any(ParsedListItem::is_text) {
        "string"
    } else if is_float {
        "float"
    } else {
        "integer"
    }
}

/// Format a computed scalar so its textual form matches the runnable code's
/// stdout: an integer with no decimal point when every input was an integer.
fn format_scalar(value: f64, is_float: bool) -> String {
    if is_float {
        format!("{value}")
    } else {
        // Integer inputs fold to an integer-valued `f64`; rendering it without a
        // decimal point keeps the textual form identical to the generated code's
        // integer stdout. The value is exact (sum/product/min/max of i64-range
        // inputs), so the cast cannot truncate meaningfully.
        #[allow(clippy::cast_possible_truncation)]
        let rounded = value.round() as i64;
        format!("{rounded}")
    }
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
        let parts = Localization::for_language(language);
        let intro = parts.intro(self.operation, self.language_name, &given, self.value_type);
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
            intro, self.code_fence, self.code, parts.result_label, shown
        );
        body
    }
}

/// Localized phrasing for the four supported UI languages. The numbers, code,
/// and result are language-independent; only the surrounding prose differs.
///
/// The `sort` / `reverse_sort` sentences are kept byte-identical to the original
/// issue #395 handler so existing golden assertions stay green.
struct Localization {
    result_label: &'static str,
    language: Language,
}

impl Localization {
    const fn for_language(language: Language) -> Self {
        let result_label = match language {
            Language::Russian => "Результат:",
            Language::Hindi => "परिणाम:",
            Language::Chinese => "结果:",
            _ => "Result:",
        };
        Self {
            result_label,
            language,
        }
    }

    fn intro(&self, operation: Operation, lang: &str, given: &str, value_type: &str) -> String {
        match self.language {
            Language::Russian => Self::intro_ru(operation, lang, given),
            Language::Hindi => Self::intro_hi(operation, lang, given),
            Language::Chinese => Self::intro_zh(operation, lang, given),
            _ => Self::intro_en(operation, lang, given, value_type),
        }
    }

    fn intro_en(operation: Operation, lang: &str, given: &str, value_type: &str) -> String {
        let noun = if value_type == "string" {
            "strings"
        } else {
            "numbers"
        };
        match operation {
            Operation::Transform(Transform::SortAscending) => {
                format!("Here is {lang} code that sorts the {noun} {given} in ascending order:")
            }
            Operation::Transform(Transform::SortDescending) => {
                format!("Here is {lang} code that sorts the {noun} {given} in descending order:")
            }
            Operation::Transform(Transform::Reverse) => {
                format!("Here is {lang} code that reverses the {noun} {given}:")
            }
            Operation::Reduce(Reduce::Sum) => {
                format!("Here is {lang} code that sums the {noun} {given}:")
            }
            Operation::Reduce(Reduce::Product) => {
                format!("Here is {lang} code that multiplies the {noun} {given}:")
            }
            Operation::Reduce(Reduce::Minimum) => {
                format!("Here is {lang} code that finds the smallest of the {noun} {given}:")
            }
            Operation::Reduce(Reduce::Maximum) => {
                format!("Here is {lang} code that finds the largest of the {noun} {given}:")
            }
        }
    }

    fn intro_ru(operation: Operation, lang: &str, given: &str) -> String {
        match operation {
            Operation::Transform(Transform::SortAscending) => {
                format!("Вот код на {lang}, который сортирует числа {given} по возрастанию:")
            }
            Operation::Transform(Transform::SortDescending) => {
                format!("Вот код на {lang}, который сортирует числа {given} по убыванию:")
            }
            Operation::Transform(Transform::Reverse) => {
                format!("Вот код на {lang}, который переворачивает числа {given}:")
            }
            Operation::Reduce(Reduce::Sum) => {
                format!("Вот код на {lang}, который суммирует числа {given}:")
            }
            Operation::Reduce(Reduce::Product) => {
                format!("Вот код на {lang}, который перемножает числа {given}:")
            }
            Operation::Reduce(Reduce::Minimum) => {
                format!("Вот код на {lang}, который находит наименьшее из чисел {given}:")
            }
            Operation::Reduce(Reduce::Maximum) => {
                format!("Вот код на {lang}, который находит наибольшее из чисел {given}:")
            }
        }
    }

    fn intro_hi(operation: Operation, lang: &str, given: &str) -> String {
        match operation {
            Operation::Transform(Transform::SortAscending) => {
                format!("यह {lang} कोड है जो संख्याओं {given} को आरोही क्रम में क्रमबद्ध करता है:")
            }
            Operation::Transform(Transform::SortDescending) => {
                format!("यह {lang} कोड है जो संख्याओं {given} को अवरोही क्रम में क्रमबद्ध करता है:")
            }
            Operation::Transform(Transform::Reverse) => {
                format!("यह {lang} कोड है जो संख्याओं {given} को उलट देता है:")
            }
            Operation::Reduce(Reduce::Sum) => {
                format!("यह {lang} कोड है जो संख्याओं {given} का योग करता है:")
            }
            Operation::Reduce(Reduce::Product) => {
                format!("यह {lang} कोड है जो संख्याओं {given} का गुणनफल निकालता है:")
            }
            Operation::Reduce(Reduce::Minimum) => {
                format!("यह {lang} कोड है जो संख्याओं {given} में से सबसे छोटी ढूँढता है:")
            }
            Operation::Reduce(Reduce::Maximum) => {
                format!("यह {lang} कोड है जो संख्याओं {given} में से सबसे बड़ी ढूँढता है:")
            }
        }
    }

    fn intro_zh(operation: Operation, lang: &str, given: &str) -> String {
        match operation {
            Operation::Transform(Transform::SortAscending) => {
                format!("这是用 {lang} 编写的将数字 {given} 按升序排序的代码:")
            }
            Operation::Transform(Transform::SortDescending) => {
                format!("这是用 {lang} 编写的将数字 {given} 按降序排序的代码:")
            }
            Operation::Transform(Transform::Reverse) => {
                format!("这是用 {lang} 编写的将数字 {given} 反转的代码:")
            }
            Operation::Reduce(Reduce::Sum) => {
                format!("这是用 {lang} 编写的对数字 {given} 求和的代码:")
            }
            Operation::Reduce(Reduce::Product) => {
                format!("这是用 {lang} 编写的计算数字 {given} 乘积的代码:")
            }
            Operation::Reduce(Reduce::Minimum) => {
                format!("这是用 {lang} 编写的求数字 {given} 最小值的代码:")
            }
            Operation::Reduce(Reduce::Maximum) => {
                format!("这是用 {lang} 编写的求数字 {given} 最大值的代码:")
            }
        }
    }
}

/// Parsed root of the numeric-list type ontology, loaded once per lookup.
fn ontology() -> LinoNode {
    parse_lino(NUMERIC_LIST_OPERATIONS_LINO)
}

/// Look up an operation's declared `result_kind` (`"list"` / `"scalar"`) from
/// the seed ontology, falling back to `"list"` for an unknown token.
fn result_kind_for(canonical: &str) -> &'static str {
    let tree = ontology();
    if let Some(root) = tree.children.first() {
        for op in root.children.iter().filter(|c| c.name == "operation") {
            if op.id == canonical {
                return match op.find_child_value("result_kind") {
                    "scalar" => "scalar",
                    _ => "list",
                };
            }
        }
    }
    "list"
}

/// Look up an operation's declared `family` from the seed ontology.
fn family_for(canonical: &str) -> &'static str {
    let tree = ontology();
    if let Some(root) = tree.children.first() {
        for op in root.children.iter().filter(|c| c.name == "operation") {
            if op.id == canonical {
                return match op.find_child_value("family") {
                    "list_reduction" => "list_reduction",
                    _ => "list_transformation",
                };
            }
        }
    }
    "list_transformation"
}

fn parse_list_items(prompt: &str, operation: Operation) -> Vec<ParsedListItem> {
    let quoted = parse_quoted_strings(prompt);
    if matches!(operation, Operation::Transform(_)) && quoted.len() >= 2 {
        return quoted;
    }
    parse_numbers(prompt)
}

/// Extract every quoted string literal from the raw prompt, in order. Quoted
/// strings let the same list algorithm handle non-numeric values without trying
/// to infer arbitrary unquoted prose as data.
fn parse_quoted_strings(prompt: &str) -> Vec<ParsedListItem> {
    let chars: Vec<char> = prompt.chars().collect();
    let mut items = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let quote = chars[index];
        if quote != '"' && quote != '\'' {
            index += 1;
            continue;
        }
        index += 1;
        let mut text = String::new();
        while index < chars.len() {
            let ch = chars[index];
            if ch == '\\' {
                if let Some(next) = chars.get(index + 1) {
                    text.push(*next);
                    index += 2;
                    continue;
                }
            }
            if ch == quote {
                break;
            }
            text.push(ch);
            index += 1;
        }
        if index < chars.len() && chars[index] == quote {
            index += 1;
            if !text.is_empty() {
                items.push(ParsedListItem {
                    text,
                    value: ListValue::Text,
                });
            }
        }
    }
    items
}

/// Extract every number token from the raw prompt, in order of appearance.
///
/// Recognizes optionally-signed integers and decimals (e.g. `-3`, `5`, `7.5`).
/// The surface text is preserved so the echo and the generated code use exactly
/// what the user typed; the parsed `f64` value drives the computation.
pub fn parse_numbers(prompt: &str) -> Vec<ParsedListItem> {
    let chars: Vec<char> = prompt.chars().collect();
    let mut numbers = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        let sign = if (ch == '-' || ch == '+')
            && chars.get(index + 1).is_some_and(char::is_ascii_digit)
            && !preceded_by_digit_or_word(&chars, index)
        {
            let s = ch;
            index += 1;
            Some(s)
        } else if ch.is_ascii_digit() {
            None
        } else {
            index += 1;
            continue;
        };

        let start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }
        // Optional single decimal point followed by digits.
        if index < chars.len()
            && chars[index] == '.'
            && chars.get(index + 1).is_some_and(char::is_ascii_digit)
        {
            index += 1;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
        }

        if start == index {
            continue;
        }
        let mut text = String::new();
        if let Some(s) = sign {
            text.push(s);
        }
        text.extend(&chars[start..index]);
        if let Ok(value) = text.parse::<f64>() {
            numbers.push(ParsedListItem {
                text,
                value: ListValue::Number(value),
            });
        }
    }
    numbers
}

/// A leading sign only introduces a negative/positive literal when it is not
/// glued to a preceding digit or letter (so `main2` or `a-b` do not spawn a
/// bogus number, but a standalone `-3` does).
fn preceded_by_digit_or_word(chars: &[char], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|prev| chars.get(prev))
        .is_some_and(|prev| prev.is_alphanumeric())
}
