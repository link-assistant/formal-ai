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

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::coding::catalog::ProgramLanguage;
use crate::seed::parser::{parse_lino, LinoNode};
use crate::seed::CODING_IDIOMS_LINO;

use super::{ListValue, Operation, ParsedListItem};

/// Safety cap on recursive `{slot}` expansion. The deepest legitimate chain is
/// scaffold → idiom → nested idiom; anything past this depth is a definition
/// cycle in the seed data and must fail composition instead of spinning.
const MAX_EXPANSION_DEPTH: usize = 8;

/// Safety cap on the `extends` inheritance chain length.
const MAX_INHERITANCE_DEPTH: usize = 4;

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
    while chain.len() < MAX_INHERITANCE_DEPTH {
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
        let family = super::family_for(self.operation.canonical());
        let scaffold = composer.scaffold(family)?;
        composer.expand(scaffold, 0)
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
            if let Some(names) = language.children.iter().find(|child| child.name == "names") {
                if let Some(entry) = names.children.iter().find(|child| child.name == key) {
                    return Some(entry.id.clone());
                }
            }
        }
        let name = default_name(key);
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    }

    /// The language's storage type for the program's value class, from the
    /// nearest `types` table in the inheritance chain that declares it.
    fn resolved_type(&self) -> Option<String> {
        let class = self.program.value_type.links_label();
        for language in &self.chain {
            if let Some(types) = language.children.iter().find(|child| child.name == "types") {
                if let Some(entry) = types.children.iter().find(|child| child.name == class) {
                    return Some(entry.id.clone());
                }
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
            if let Some(on) = on {
                if !on.id.split_whitespace().any(|token| token == class) {
                    continue;
                }
            }
            let score = u32::from(operation_exact) * 2 + u32::from(on.is_some());
            if best.map_or(true, |(_, current)| score > current) {
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
                direction: super::direction_for(canonical),
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
