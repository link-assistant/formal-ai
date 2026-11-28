//! Structural code generation for numeric-list programs.
//!
//! Issue #395 explicitly asks the coding path to manipulate CST/AST-like
//! structures instead of memorizing final code snippets. This module therefore
//! lowers a resolved numeric-list meaning into a small, language-independent
//! [`NumericProgram`] tree first:
//!
//! * `literal_list` — the user-provided numbers, preserved in order.
//! * `sort_list` / `reverse_list` / `reduce_list` — the semantic operation.
//! * `print_joined` / `print_scalar` — the requested result projection.
//!
//! The projection from that tree into source code is *discovered at execution
//! time* from the `data/seed/coding-idioms.lino` knowledge base rather than
//! owned by per-language renderer functions. Each language section there
//! declares scaffolds (one per operation family) and idioms (named code
//! fragments whose cases are selected by the requested operation and the value
//! class of the list items). [`NumericProgram::render`] walks the language's
//! inheritance chain, picks the scaffold for the operation's ontology family,
//! and recursively expands every `{slot}` placeholder — so adding a language or
//! an operation idiom is seed data, not Rust code. The tree itself is logged in
//! Links Notation so the solver's reasoning can be inspected independently from
//! the printed code.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::coding::catalog::ProgramLanguage;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::seed::{self, CODING_IDIOMS_LINO, NUMERIC_LIST_OPERATIONS_LINO};

/// Safety cap on recursive `{slot}` expansion. The deepest legitimate chain is
/// scaffold → idiom → nested idiom; anything past this depth is a definition
/// cycle in the seed data and must fail composition instead of spinning.
const MAX_EXPANSION_DEPTH: usize = 8;

/// Parsed root of the coding-idioms knowledge base, loaded once per process.
fn idiom_catalog() -> Option<&'static LinoNode> {
    static TREE: OnceLock<LinoNode> = OnceLock::new();
    TREE.get_or_init(|| parse_lino(CODING_IDIOMS_LINO))
        .children
        .first()
}

/// The `language "<slug>"` node followed by its transitive `extends` parents,
/// nearest first. Empty when the catalog does not know the slug.
fn language_chain(catalog: &'static LinoNode, slug: &str) -> Vec<&'static LinoNode> {
    let mut chain = Vec::new();
    let mut current = slug.to_owned();
    let mut seen = BTreeSet::new();
    while seen.insert(current.clone()) {
        let Some(node) = catalog
            .children
            .iter()
            .find(|child| child.name == "language" && child.id == current)
        else {
            break;
        };
        chain.push(node);
        let parent = node.find_child_value("extends");
        if parent.is_empty() {
            break;
        }
        parent.clone_into(&mut current);
    }
    chain
}

/// The canonical semantic-tree variable name for `key` (`list` / `transformed`
/// / `reduced`), read from the catalog's `defaults` node.
fn default_name(key: &str) -> String {
    idiom_catalog()
        .and_then(|catalog| {
            catalog
                .children
                .iter()
                .find(|child| child.name == "defaults")
        })
        .and_then(|defaults| {
            defaults
                .children
                .iter()
                .find(|child| child.name == key)
                .map(|child| child.id.clone())
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueType {
    Integer,
    Float,
    Text,
}

impl ValueType {
    fn from_items(items: &[ParsedListItem], is_float: bool) -> Self {
        if items
            .iter()
            .any(|item| matches!(item.value, ListValue::Text))
        {
            Self::Text
        } else if is_float {
            Self::Float
        } else {
            Self::Integer
        }
    }

    /// The value-class token used both in the Links Notation trace and as the
    /// `on` selector in `coding-idioms.lino` cases.
    const fn links_label(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Text => "string",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProgramStatement {
    LiteralList {
        name: String,
        mutable: bool,
    },
    TransformList {
        semantic_node: &'static str,
        source: String,
        target: String,
        direction: String,
    },
    ReduceList {
        source: String,
        target: String,
        reducer: &'static str,
    },
    PrintJoined {
        source: String,
        separator: &'static str,
    },
    PrintScalar {
        source: String,
    },
}

impl ProgramStatement {
    fn links_line(&self, out: &mut String) {
        match self {
            Self::LiteralList { name, mutable } => {
                let _ = writeln!(
                    out,
                    "  semantic_node literal_list name={name} mutable={mutable}"
                );
            }
            Self::TransformList {
                semantic_node,
                source,
                target,
                direction,
            } => {
                let _ = writeln!(
                    out,
                    "  semantic_node {semantic_node} source={source} target={target} direction={direction}"
                );
            }
            Self::ReduceList {
                source,
                target,
                reducer,
            } => {
                let _ = writeln!(
                    out,
                    "  semantic_node reduce_list source={source} target={target} reducer={reducer}"
                );
            }
            Self::PrintJoined { source, separator } => {
                let _ = writeln!(
                    out,
                    "  semantic_node print_joined source={source} separator={separator:?}"
                );
            }
            Self::PrintScalar { source } => {
                let _ = writeln!(out, "  semantic_node print_scalar source={source}");
            }
        }
    }
}

/// Language-independent program tree for one numeric-list task.
#[derive(Clone)]
pub struct NumericProgram {
    language: &'static ProgramLanguage,
    value_type: ValueType,
    literals: Vec<String>,
    display_values: Vec<String>,
    operation: Operation,
    statements: Vec<ProgramStatement>,
}

impl NumericProgram {
    /// Render the program tree into the requested target language by composing
    /// the scaffold and idioms discovered in `coding-idioms.lino`. Returns
    /// `None` when the knowledge base has no language section, no scaffold for
    /// the operation's family, or no idiom case matching the operation and
    /// value class — composition failures are explicit, never silent fallbacks.
    #[must_use]
    pub fn render(&self) -> Option<String> {
        let catalog = idiom_catalog()?;
        let chain = language_chain(catalog, self.language.slug);
        if chain.is_empty() {
            return None;
        }
        let composer = Composer::new(self, chain);
        let family = family_for(self.operation.canonical());
        let scaffold = composer.scaffold(family)?;
        composer.expand(scaffold, 0)
    }

    /// Use the same operation idioms as the script, without binding the input
    /// list to a literal in the callable. Tests bind inputs and expected values.
    fn render_callable(&self, expected: Option<&[String]>) -> Option<String> {
        let chain = language_chain(idiom_catalog()?, self.language.slug);
        let mut composer = Composer::new(self, chain);
        composer.bindings.insert("list", "values".to_owned());
        let scaffold = if let Some(values) = expected {
            let literals = values.iter().map(|value| {
                if self.value_type == ValueType::Float && !value.contains('.') {
                    format!("{value}.0")
                } else {
                    value.clone()
                }
            });
            composer
                .bindings
                .insert("expected", literals.collect::<Vec<_>>().join(", "));
            "list_function_test"
        } else {
            "list_function"
        };
        composer.expand(composer.scaffold(scaffold)?, 0)
    }

    /// Trace-friendly Links Notation view of the program tree.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("program_syntax_tree\n");
        let _ = writeln!(out, "  language {}", self.language.slug);
        let _ = writeln!(out, "  value_type {}", self.value_type.links_label());
        let _ = writeln!(out, "  operation {}", self.operation.canonical());
        let _ = writeln!(out, "  literal_values {}", self.display_values.join("|"));
        for statement in &self.statements {
            statement.links_line(&mut out);
        }
        out.trim_end().to_owned()
    }

    fn literal(&self) -> String {
        self.literals.join(", ")
    }
}

/// One rendering pass: the resolved language chain plus the computed slot
/// bindings for the program being rendered.
struct Composer<'p> {
    program: &'p NumericProgram,
    chain: Vec<&'static LinoNode>,
    bindings: BTreeMap<&'static str, String>,
}

impl<'p> Composer<'p> {
    fn new(program: &'p NumericProgram, chain: Vec<&'static LinoNode>) -> Self {
        let mut composer = Self {
            program,
            chain,
            bindings: BTreeMap::new(),
        };
        for key in ["list", "transformed", "reduced"] {
            if let Some(name) = composer.resolved_name(key) {
                composer.bindings.insert(key, name);
            }
        }
        if let Some(type_name) = composer.resolved_type() {
            composer.bindings.insert("type", type_name);
        }
        composer.bindings.insert("literal", program.literal());
        composer
            .bindings
            .insert("count", program.literals.len().to_string());
        // Links Notation values cannot encode a raw tab, so templates spell it
        // as a computed slot.
        composer.bindings.insert("tab", "\t".to_owned());
        composer
    }

    /// Per-language variable name for `key`: the nearest `names` override in
    /// the inheritance chain, else the catalog-wide `defaults` entry.
    fn resolved_name(&self, key: &str) -> Option<String> {
        for language in &self.chain {
            if let Some(names) = language.children.iter().find(|child| child.name == "names")
                && let Some(entry) = names.children.iter().find(|child| child.name == key)
            {
                return Some(entry.id.clone());
            }
        }
        let name = default_name(key);
        if name.is_empty() { None } else { Some(name) }
    }

    /// The language's storage type for the program's value class, from the
    /// nearest `types` table in the inheritance chain that declares it.
    fn resolved_type(&self) -> Option<String> {
        let class = self.program.value_type.links_label();
        for language in &self.chain {
            if let Some(types) = language.children.iter().find(|child| child.name == "types")
                && let Some(entry) = types.children.iter().find(|child| child.name == class)
            {
                return Some(entry.id.clone());
            }
        }
        None
    }

    /// The scaffold template for the operation family, from the nearest
    /// language in the chain that declares one.
    fn scaffold(&self, family: &str) -> Option<&'static str> {
        for language in &self.chain {
            if let Some(scaffold) = language
                .children
                .iter()
                .find(|child| child.name == "scaffold" && child.id == family)
            {
                return Some(scaffold.find_child_value("code"));
            }
        }
        None
    }

    /// The idiom definition for `slot`, from the nearest language in the chain
    /// that declares it. Idioms are not merged across the chain: the nearest
    /// definition fully shadows inherited ones.
    fn idiom(&self, slot: &str) -> Option<&'static LinoNode> {
        for language in &self.chain {
            if let Some(idiom) = language
                .children
                .iter()
                .find(|child| child.name == "idiom" && child.id == slot)
            {
                return Some(idiom);
            }
        }
        None
    }

    /// Pick the idiom case that best matches the requested operation and value
    /// class. A case applies when its `for` tokens contain the operation (or
    /// `any`) and its `on` tokens, when present, contain the value class.
    /// Specific matches outrank generic ones: an exact operation token scores
    /// over `any`, and a value-class constraint scores over none. The first
    /// case with the highest score wins, so declaration order breaks ties.
    fn select_case(&self, idiom: &'static LinoNode) -> Option<&'static str> {
        let operation = self.program.operation.canonical();
        let class = self.program.value_type.links_label();
        let mut best: Option<(&'static str, u32)> = None;
        for case in idiom.children.iter().filter(|child| child.name == "case") {
            let mut for_tokens = case.find_child_value("for").split_whitespace();
            let operation_exact = for_tokens.clone().any(|token| token == operation);
            if !operation_exact && !for_tokens.any(|token| token == "any") {
                continue;
            }
            let on = case.children.iter().find(|child| child.name == "on");
            if let Some(on) = on
                && !on.id.split_whitespace().any(|token| token == class)
            {
                continue;
            }
            let score = u32::from(operation_exact) * 2 + u32::from(on.is_some());
            if best.is_none_or(|(_, current)| score > current) {
                best = Some((case.find_child_value("code"), score));
            }
        }
        best.map(|(code, _)| code)
    }

    /// Recursively expand `{slot}` placeholders. Computed bindings are inserted
    /// verbatim (never rescanned, so user-provided literals cannot inject
    /// further slots); idiom slots expand their selected case recursively; any
    /// other brace sequence — `{}`, `{ return`, `{{literal}}` — is ordinary
    /// target-language syntax and passes through unchanged.
    fn expand(&self, template: &str, depth: usize) -> Option<String> {
        if depth > MAX_EXPANSION_DEPTH {
            return None;
        }
        let chars: Vec<char> = template.chars().collect();
        let mut out = String::new();
        let mut index = 0;
        while index < chars.len() {
            if chars[index] != '{' {
                out.push(chars[index]);
                index += 1;
                continue;
            }
            let mut end = index + 1;
            while end < chars.len() && is_slot_char(chars[end]) {
                end += 1;
            }
            if end >= chars.len() || chars[end] != '}' || end == index + 1 {
                out.push('{');
                index += 1;
                continue;
            }
            let name: String = chars[index + 1..end].iter().collect();
            if let Some(value) = self.bindings.get(name.as_str()) {
                out.push_str(value);
            } else if let Some(idiom) = self.idiom(&name) {
                let code = self.select_case(idiom)?;
                out.push_str(&self.expand(code, depth + 1)?);
            } else {
                out.push('{');
                out.push_str(&name);
                out.push('}');
            }
            index = end + 1;
        }
        Some(out)
    }
}

const fn is_slot_char(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'
}

/// Whether the language declares (in `coding-idioms.lino`) that its list
/// transformations mutate the literal list in place rather than building a new
/// collection.
fn mutates_list_in_place(slug: &str) -> bool {
    let Some(catalog) = idiom_catalog() else {
        return false;
    };
    for language in language_chain(catalog, slug) {
        if let Some(node) = language
            .children
            .iter()
            .find(|child| child.name == "mutable_list")
        {
            return node.id == "true";
        }
    }
    false
}

/// Build a semantic program tree for `operation` over `numbers` in `language`.
#[must_use]
pub fn build(
    language: &'static ProgramLanguage,
    items: &[ParsedListItem],
    operation: Operation,
    is_float: bool,
) -> NumericProgram {
    let value_type = ValueType::from_items(items, is_float);
    let literals = item_literals(items, value_type);
    let display_values = items.iter().map(|item| item.text.clone()).collect();
    let canonical = operation.canonical();
    let list = default_name("list");
    let mut statements = vec![ProgramStatement::LiteralList {
        name: list.clone(),
        mutable: mutates_list_in_place(language.slug),
    }];

    match operation {
        Operation::Transform(_) => {
            let target = default_name("transformed");
            statements.push(ProgramStatement::TransformList {
                semantic_node: if canonical == "reverse" {
                    "reverse_list"
                } else {
                    "sort_list"
                },
                source: list,
                target: target.clone(),
                direction: direction_for(canonical),
            });
            statements.push(ProgramStatement::PrintJoined {
                source: target,
                separator: ", ",
            });
        }
        Operation::Reduce(_) => {
            let target = default_name("reduced");
            statements.push(ProgramStatement::ReduceList {
                source: list,
                target: target.clone(),
                reducer: canonical,
            });
            statements.push(ProgramStatement::PrintScalar { source: target });
        }
    }

    NumericProgram {
        language,
        value_type,
        literals,
        display_values,
        operation,
        statements,
    }
}

/// Render the list literal. Numeric surfaces are preserved, with a `.0` suffix
/// when needed for homogeneous static float containers; text values become string
/// literals and are escaped once before each language renderer joins them.
fn item_literals(items: &[ParsedListItem], value_type: ValueType) -> Vec<String> {
    items
        .iter()
        .map(|item| match item.value {
            ListValue::Text => quoted_string_literal(&item.text),
            ListValue::Number(_) => {
                if value_type == ValueType::Float && !item.text.contains('.') {
                    format!("{}.0", item.text)
                } else {
                    item.text.clone()
                }
            }
        })
        .collect()
}

fn quoted_string_literal(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

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
    pub(crate) const fn numeric_value(&self) -> Option<f64> {
        match self.value {
            ListValue::Number(value) => Some(value),
            ListValue::Text => None,
        }
    }

    pub(crate) const fn is_text(&self) -> bool {
        matches!(self.value, ListValue::Text)
    }
}

/// Recognize which numeric-list operation the prompt asks for, in priority
/// order. Sort phrasings are checked first because "sort in reverse order"
/// legitimately contains the bare `reverse` verb; the descending variant wins
/// whenever the `reverse_sort` phrasing is present.
pub(crate) fn detect_operation(
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

/// Compose a callable list transformation or a standalone test of that callable.
pub(crate) fn render_list_callable(
    slug: &str,
    prompt: &str,
    normalized: &str,
    test: bool,
) -> Option<String> {
    let language = crate::coding::program_language_by_slug(slug)?;
    let operation = detect_operation(&seed::operation_vocabulary(), normalized)?;
    let Operation::Transform(transform) = operation else {
        return None;
    };
    let mut items = parse_numbers(prompt);
    for item in &mut items {
        let unsigned = item.text.trim_start_matches(['+', '-']);
        let significant = unsigned.trim_start_matches('0');
        let sign = if item.text.starts_with('-') { "-" } else { "" };
        item.text = if significant.is_empty() || significant.starts_with('.') {
            format!("{sign}0{significant}")
        } else {
            format!("{sign}{significant}")
        };
    }
    if test && items.is_empty() {
        // Use a sequence ordered opposite to the requested result so an identity
        // implementation fails even when the caller supplied no examples.
        let mut values = [0, 1, 2];
        if transform != Transform::SortDescending {
            values.reverse();
        }
        items.extend(values.into_iter().map(|value| ParsedListItem {
            text: value.to_string(),
            value: ListValue::Number(f64::from(value)),
        }));
    }
    let is_float = items.iter().any(|item| item.text.contains('.'));
    let program = build(language, &items, operation, is_float);
    let expected = test.then(|| compute(operation, &items, is_float));
    program.render_callable(expected.as_deref())
}

/// Apply the operation to the parsed list and return the surface tokens to
/// display: the reordered list for a transformation, or a single computed scalar
/// for a numeric reduction.
pub(crate) fn compute(
    operation: Operation,
    items: &[ParsedListItem],
    is_float: bool,
) -> Vec<String> {
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

pub(crate) fn value_type_label(items: &[ParsedListItem], is_float: bool) -> &'static str {
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

/// Parsed root of the numeric-list type ontology, loaded once per lookup.
fn ontology() -> LinoNode {
    parse_lino(NUMERIC_LIST_OPERATIONS_LINO)
}

/// Look up an operation's declared `result_kind` (`"list"` / `"scalar"`) from
/// the seed ontology, falling back to `"list"` for an unknown token.
pub(crate) fn result_kind_for(canonical: &str) -> &'static str {
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
pub(crate) fn family_for(canonical: &str) -> &'static str {
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

/// Look up a transformation's declared `direction` from the seed ontology.
/// The token feeds the program trace so the semantic tree stays inspectable.
fn direction_for(canonical: &str) -> String {
    let tree = ontology();
    if let Some(root) = tree.children.first() {
        for op in root.children.iter().filter(|c| c.name == "operation") {
            if op.id == canonical {
                return op.find_child_value("direction").to_owned();
            }
        }
    }
    String::new()
}

mod literals;
pub(crate) use literals::parse_list_items;
pub use literals::{names_list_items, parse_numbers};
