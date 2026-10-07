//! Code examples as formal knowledge (issue #1164, E129).
//!
//! Code found on the internet is either rendered verbatim (the Rosetta
//! Code path, only for tasks in the seeded alias list) or not used. This
//! module parses a retrieved example into the meta language, decomposes it
//! into meaningful parts, generalizes those parts across languages by
//! deduplication, and recomposes them for a new requirement.
//!
//! Meanings come from exactly two inputs: the CST node kinds the
//! meta-language parse produced, and the per-language vocabulary recorded
//! in `data/seed/code-example-parts.lino` (R1164-3 -- no per-language
//! meaning table lives here). A slug with no registered grammar is
//! [`DecomposeError::UnknownGrammar`] naming the slug; the decomposer
//! never guesses (R1164-1). Pascal is held out that way today: the seed
//! carries no Pascal vocabulary and no program template, so a Pascal
//! program can only ever come from a decomposed Free Pascal example
//! (R1164-9).
//!
//! Recomposition builds the program from the aligned parts, never from a
//! stored template (R1164-6): each decomposed example keeps its own source
//! as a program body with the output literal cut out into a slot, the
//! generalized procedure carries one body per contributing language next to
//! the `output_call` parameter, and recomposing binds the requirement's
//! literal into that body after checking the body still makes the aligned
//! output call. A language no example contributed is a refusal.
//!
//! The schema of every record this module emits is
//! `data/seed/code-node-decomposition.lino` (R1164-10), and promotion of a
//! generalized procedure into an adopted one goes through the existing
//! execution-and-approval gate in `coding_research_learning.rs`;
//! [`GeneralizedProcedure::procedure_step_records`] produces the records that
//! gate consumes, so wiring cannot bypass its license and approval checks
//! (R1164-7).
#![cfg(feature = "meta-language")]

use std::collections::BTreeMap;

use meta_language::{LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};

use crate::procedure_text::ProcedureStepRecord;
use crate::seed::parser::{LinoNode, parse_lino};

/// The part vocabulary seed, mirrored into the embedded bundle.
const PARTS_TEXT: &str = include_str!("../embedded/data/seed/code-example-parts.lino");

/// A single named part extracted from a parsed code example.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodePart {
    /// What the part is.
    pub kind: CodePartKind,
    /// The part's source text (unquoted for literals).
    pub source_text: String,
    /// The CST node kind the meaning was derived from.
    pub cst_node_kind: String,
    /// Where the example came from.
    pub source_url: String,
}

/// The meaning a part carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CodePartKind {
    /// The language's entry point (`fn main`, `fun main`, ...).
    EntryPoint,
    /// An operation that writes output.
    OutputOperation,
    /// A literal string in the source.
    StringLiteral,
    /// An import or use declaration.
    Import,
    /// A build command supplied by surrounding prose.
    BuildCommand,
    /// A run command supplied by surrounding prose.
    RunCommand,
    /// A test assertion.
    TestAssertion,
}

impl CodePartKind {
    /// The schema's spelling of the kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EntryPoint => "entry_point",
            Self::OutputOperation => "output_operation",
            Self::StringLiteral => "string_literal",
            Self::Import => "import",
            Self::BuildCommand => "build_command",
            Self::RunCommand => "run_command",
            Self::TestAssertion => "test_assertion",
        }
    }
}

/// One decomposed example: a language slug and its meaningful parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecomposedCodeNode {
    /// The language the example is written in.
    pub language_slug: String,
    /// The parts found in it.
    pub parts: Vec<CodePart>,
    /// The example's own source with its output literal replaced by
    /// [`LITERAL_SLOT`] (quotes kept); empty when no literal was printed by
    /// the matched output call.
    pub program_body: String,
}

/// The slot a program body carries where the output literal was.
pub const LITERAL_SLOT: &str = concat!("{", "literal}");

/// One language-specific value of a generalized slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    /// The slot's name (`output_literal`, `output_call`).
    pub name: String,
    /// The value each contributing language carries.
    pub per_language: BTreeMap<String, String>,
}

/// A procedure shared across languages: the shared structure is the body,
/// the language-specific values are parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralizedProcedure {
    /// The procedure's identity.
    pub id: String,
    /// The part kinds every contributing example shares, in canonical order.
    pub shared_structure: Vec<CodePartKind>,
    /// The language-specific slots.
    pub parameters: Vec<Parameter>,
    /// Where every contributing part came from.
    pub source_urls: Vec<String>,
    /// Each contributing language's program body (its example's source with
    /// the output literal slotted), the source recomposition builds from.
    pub program_bodies: BTreeMap<String, String>,
}

/// Requirement-supplied values for the generalized slots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParameterBindings(pub BTreeMap<String, String>);

/// A recomposed program: concrete source for one language, carrying the
/// source URL of every part that contributed to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeRecomposition {
    /// The language recomposed into.
    pub language_slug: String,
    /// The recomposed source.
    pub source: String,
    /// The source URL of every contributing `CodePart`.
    pub part_source_urls: Vec<String>,
}

/// A prose link from the page around an example: "compile with", "run with".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProseLink {
    /// The relation, spelled as in the seed (`compile_with`, `run_with`).
    pub relation: String,
    /// The command or text the prose carries.
    pub text: String,
    /// Where the prose came from.
    pub source_url: String,
}

/// Why a decomposition or recomposition could not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecomposeError {
    /// The language slug has no registered grammar; the slug is named.
    UnknownGrammar(String),
    /// The source parsed to nothing recognizable.
    ParseFailed(String),
}

/// The seed vocabulary the decomposer interprets.
#[derive(Debug, Clone, Default)]
struct PartVocabulary {
    output_calls: BTreeMap<String, Vec<String>>,
    entry_names: BTreeMap<String, String>,
    entry_hint_kinds: Vec<String>,
    call_hint_kinds: Vec<String>,
    string_hint_kinds: Vec<String>,
    import_hint_kinds: Vec<String>,
    assert_hint_kinds: Vec<String>,
    prose_relations: Vec<(String, CodePartKind)>,
    /// Prose relations whose text names the page's output call.
    ///
    /// The seed's `print` relation (kind `print_stdout`): a page teaches the
    /// call, so a held-out language needs no stored `output_call` row
    /// (R1164-9).
    output_call_relations: Vec<String>,
}

impl PartVocabulary {
    fn load() -> Self {
        let tree = parse_lino(PARTS_TEXT);
        let mut vocabulary = Self::default();
        let mut current_language = String::new();
        // The rows nest under `code_example_parts` and their `part_language`
        // headers, so the records are read in document order at every
        // depth; a header sets the language its children belong to.
        let mut records: Vec<&LinoNode> = Vec::new();
        let mut pending: Vec<&LinoNode> = tree.children.iter().rev().collect();
        while let Some(node) = pending.pop() {
            records.push(node);
            pending.extend(node.children.iter().rev());
        }
        for record in records {
            match record.name.as_str() {
                "part_language" => {
                    current_language.clone_from(&record.id);
                }
                "output_call" => {
                    if !current_language.is_empty() && !record.id.is_empty() {
                        vocabulary
                            .output_calls
                            .entry(current_language.clone())
                            .or_default()
                            .push(record.id.clone());
                    }
                }
                "entry_name" => {
                    if !current_language.is_empty() {
                        vocabulary
                            .entry_names
                            .insert(current_language.clone(), record.id.clone());
                    }
                }
                "entry_hint_kind" => vocabulary.entry_hint_kinds.push(record.id.clone()),
                "call_hint_kind" => vocabulary.call_hint_kinds.push(record.id.clone()),
                "string_kind_hint" => vocabulary.string_hint_kinds.push(record.id.clone()),
                "import_kind_hint" => vocabulary.import_hint_kinds.push(record.id.clone()),
                "assert_kind_hint" => vocabulary.assert_hint_kinds.push(record.id.clone()),
                "prose_relation" => {
                    let relation = record.find_child_value("relation").to_owned();
                    let kind = record.find_child_value("kind").to_owned();
                    if relation.is_empty() {
                        continue;
                    }
                    match kind.as_str() {
                        "build_command" => vocabulary
                            .prose_relations
                            .push((relation, CodePartKind::BuildCommand)),
                        "run_command" => vocabulary
                            .prose_relations
                            .push((relation, CodePartKind::RunCommand)),
                        "print_stdout" => vocabulary.output_call_relations.push(relation),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        vocabulary
    }
}

/// The canonical part order shared structure is reported in.
const KIND_ORDER: [CodePartKind; 7] = [
    CodePartKind::Import,
    CodePartKind::EntryPoint,
    CodePartKind::OutputOperation,
    CodePartKind::StringLiteral,
    CodePartKind::TestAssertion,
    CodePartKind::BuildCommand,
    CodePartKind::RunCommand,
];

/// Decompose one retrieved example into meaningful parts (R1164-1/2/3/4).
///
/// The language must have a registered grammar in
/// `data/seed/program-cst-grammars.lino` (checked through the `cst`
/// bridge); otherwise [`DecomposeError::UnknownGrammar`] names the slug.
/// Meanings derive from the CST node kinds of the meta-language parse and
/// the seed vocabulary -- never from a table in Rust.
pub fn decompose_code_node(
    source: &str,
    language_slug: &str,
    prose_links: &[ProseLink],
) -> Result<DecomposedCodeNode, DecomposeError> {
    decompose_code_node_from(source, language_slug, prose_links, "")
}

/// [`decompose_code_node`] for an example read from a known page.
///
/// Every part derived from the code carries `source_url`, so a recomposition
/// can cite the page each contributing part came from (R1164-6). Prose parts
/// keep the prose's own URL.
pub fn decompose_code_node_from(
    source: &str,
    language_slug: &str,
    prose_links: &[ProseLink],
    source_url: &str,
) -> Result<DecomposedCodeNode, DecomposeError> {
    let grammar = crate::coding::cst::grammar_metadata(language_slug)
        .ok_or_else(|| DecomposeError::UnknownGrammar(language_slug.to_owned()))?;
    let vocabulary = PartVocabulary::load();
    let network = LinkNetwork::parse(
        source,
        &grammar.meta_language_label,
        ParseConfiguration::default(),
    );
    let mut kinds: Vec<String> = Vec::new();
    let mut tokens: Vec<String> = Vec::new();
    for link in network.projected_links(NetworkProjection::ConcreteSyntax) {
        let Some(term) = link.metadata().term() else {
            continue;
        };
        match link.metadata().link_type() {
            Some(LinkType::Grammar | LinkType::Syntax) => kinds.push(term.to_owned()),
            Some(LinkType::Token) => tokens.push(term.to_owned()),
            _ => {}
        }
    }
    if kinds.is_empty() && tokens.is_empty() {
        return Err(DecomposeError::ParseFailed(crate::seed::report_text(
            "code_example_parse_recognized_nothing",
            &[("language", language_slug)],
        )));
    }
    let mut parts = Vec::new();
    // Imports (R1164-2): any import-shaped node kind.
    for kind in &kinds {
        if vocabulary
            .import_hint_kinds
            .iter()
            .any(|hint| kind.contains(hint.as_str()))
        {
            parts.push(CodePart {
                kind: CodePartKind::Import,
                source_text: String::new(),
                cst_node_kind: kind.clone(),
                source_url: source_url.to_owned(),
            });
            break;
        }
    }
    // Entry point (R1164-2): the seed's entry name for the language, when
    // the language has one and the parse shows a function-shaped kind.
    let entry_name = vocabulary.entry_names.get(language_slug).cloned();
    let function_kind = kinds
        .iter()
        .find(|kind| {
            vocabulary
                .entry_hint_kinds
                .iter()
                .any(|hint| kind.contains(hint.as_str()))
        })
        .cloned();
    if let Some(entry) = entry_name
        && entry != "none"
    {
        let named = tokens.iter().any(|token| token == &entry)
            || source
                .split_whitespace()
                .any(|word| word.trim_matches(|c: char| c == '(' || c == '!') == entry);
        if named {
            parts.push(CodePart {
                kind: CodePartKind::EntryPoint,
                source_text: entry,
                cst_node_kind: function_kind.unwrap_or_else(|| "function_declaration".to_owned()),
                source_url: source_url.to_owned(),
            });
        }
    }
    // Output operation: the seed's output calls for the language.
    let call_kind = kinds
        .iter()
        .find(|kind| {
            vocabulary
                .call_hint_kinds
                .iter()
                .any(|hint| kind.contains(hint.as_str()))
        })
        .cloned()
        .unwrap_or_else(|| "call_expression".to_owned());
    // The seed's calls come first; a prose link the seed marks as naming
    // the output call (kind `print_stdout`) adds the page's own call, keeping the
    // prose's URL as the part's source (R1164-9: a held-out language learns
    // its output call from its documentation).
    let seed_calls = vocabulary
        .output_calls
        .get(language_slug)
        .into_iter()
        .flatten()
        .map(|call| (call.clone(), source_url.to_owned()));
    let prose_calls = prose_links
        .iter()
        .filter(|prose| {
            vocabulary
                .output_call_relations
                .contains(&normalize_relation(&prose.relation))
        })
        .map(|prose| (prose.text.trim().to_owned(), prose.source_url.clone()))
        .filter(|(call, _)| !call.is_empty());
    let mut matched_call: Option<String> = None;
    for (call, call_url) in seed_calls.chain(prose_calls) {
        let present = tokens.contains(&call) || source.contains(call.as_str());
        if present {
            parts.push(CodePart {
                kind: CodePartKind::OutputOperation,
                source_text: call.clone(),
                cst_node_kind: call_kind,
                source_url: call_url,
            });
            matched_call = Some(call);
            break;
        }
    }
    // String literals: located in the source, with the CST's string-shaped
    // node kind recorded when the parse produced one.
    let string_kind = kinds
        .iter()
        .find(|kind| {
            vocabulary
                .string_hint_kinds
                .iter()
                .any(|hint| kind.contains(hint.as_str()))
        })
        .cloned()
        .unwrap_or_else(|| "string_literal".to_owned());
    // The output literal is what the output call prints, so literals are
    // harvested from the matched call's line first; only a call-less
    // example falls back to every literal in the source (an import's
    // "fmt" is a literal, but it is not the output).
    let literal_scope = matched_call
        .as_deref()
        .and_then(|call| {
            let lines: Vec<&str> = source.lines().filter(|line| line.contains(call)).collect();
            (!lines.is_empty()).then(|| lines.join("\n"))
        })
        .unwrap_or_else(|| source.to_owned());
    let literals = quoted_literals(&literal_scope);
    let program_body = matched_call
        .as_deref()
        .zip(literals.first())
        .map(|(call, literal)| slot_output_literal(source, call, literal))
        .unwrap_or_default();
    let literals = if literals.is_empty() {
        quoted_literals(source)
    } else {
        literals
    };
    for literal in literals {
        parts.push(CodePart {
            kind: CodePartKind::StringLiteral,
            source_text: literal,
            cst_node_kind: string_kind.clone(),
            source_url: source_url.to_owned(),
        });
    }
    // Test assertions: any assert-shaped node kind.
    for kind in &kinds {
        if vocabulary
            .assert_hint_kinds
            .iter()
            .any(|hint| kind.contains(hint.as_str()))
        {
            parts.push(CodePart {
                kind: CodePartKind::TestAssertion,
                source_text: String::new(),
                cst_node_kind: kind.clone(),
                source_url: source_url.to_owned(),
            });
            break;
        }
    }
    // Prose links (R1164-4): build and run commands from the page around
    // the example, with the prose's source URL retained.
    for prose in prose_links {
        let relation = normalize_relation(&prose.relation);
        let Some(kind) = vocabulary
            .prose_relations
            .iter()
            .find(|(seed_relation, _)| *seed_relation == relation)
            .map(|(_, kind)| *kind)
        else {
            continue;
        };
        parts.push(CodePart {
            kind,
            source_text: prose.text.clone(),
            cst_node_kind: "prose_link".to_owned(),
            source_url: prose.source_url.clone(),
        });
    }
    Ok(DecomposedCodeNode {
        language_slug: language_slug.to_owned(),
        parts,
        program_body,
    })
}

/// The quote characters a string literal may open with.
const LITERAL_QUOTES: [char; 2] = ['"', '\''];

/// The source with the first quoted `literal` on a line that makes `call`
/// replaced by [`LITERAL_SLOT`] (the quotes stay); empty when no such line
/// carries the literal.
fn slot_output_literal(source: &str, call: &str, literal: &str) -> String {
    let mut offset = 0usize;
    for line in source.split_inclusive('\n') {
        if line.contains(call) {
            for quote in LITERAL_QUOTES {
                let quoted = format!("{quote}{literal}{quote}");
                if let Some(at) = line.find(&quoted) {
                    let start = offset + at + quote.len_utf8();
                    let end = start + literal.len();
                    return format!("{}{LITERAL_SLOT}{}", &source[..start], &source[end..]);
                }
            }
        }
        offset += line.len();
    }
    String::new()
}

/// Escape a bound literal for the quote that opens its slot: the escape
/// character and that quote are the only characters a literal cannot carry
/// verbatim.
fn escape_for_quote(literal: &str, quote: Option<char>) -> String {
    let mut out = String::with_capacity(literal.len());
    for character in literal.chars() {
        if character == '\\' || Some(character) == quote {
            out.push('\\');
        }
        out.push(character);
    }
    out
}

/// Lowercase and underscore a prose relation so "compile with" and
/// `compile_with` are one relation.
fn normalize_relation(relation: &str) -> String {
    relation
        .to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_whitespace() {
                '_'
            } else {
                character
            }
        })
        .collect()
}

/// Every string literal in the source, unquoted.
fn quoted_literals(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let characters: Vec<char> = source.chars().collect();
    let mut index = 0usize;
    while index < characters.len() {
        let quote = characters[index];
        if quote != '"' && quote != '\'' {
            index += 1;
            continue;
        }
        let mut literal = String::new();
        let mut cursor = index + 1;
        let mut closed = false;
        while cursor < characters.len() {
            let character = characters[cursor];
            if character == '\\' && cursor + 1 < characters.len() {
                literal.push(character);
                literal.push(characters[cursor + 1]);
                cursor += 2;
                continue;
            }
            if character == quote {
                closed = true;
                cursor += 1;
                break;
            }
            literal.push(character);
            cursor += 1;
        }
        if closed && !literal.is_empty() {
            out.push(literal);
        }
        index = if closed { cursor } else { index + 1 };
    }
    out
}

/// Align two or more decomposed examples for the same need (R1164-5): the
/// shared structure is the procedure body, the language-specific values
/// become `Parameter` entries.
#[must_use]
pub fn generalize_examples(examples: &[DecomposedCodeNode]) -> GeneralizedProcedure {
    let mut languages: Vec<&str> = examples
        .iter()
        .map(|example| example.language_slug.as_str())
        .collect();
    languages.sort_unstable();
    languages.dedup();
    let mut shared_structure = Vec::new();
    for kind in KIND_ORDER {
        let everywhere = examples
            .iter()
            .all(|example| example.parts.iter().any(|part| part.kind == kind));
        if everywhere {
            shared_structure.push(kind);
        }
    }
    let mut parameters = Vec::new();
    if !examples.is_empty() {
        let mut output_literal = Parameter {
            name: "output_literal".to_owned(),
            per_language: BTreeMap::new(),
        };
        let mut output_call = Parameter {
            name: "output_call".to_owned(),
            per_language: BTreeMap::new(),
        };
        for example in examples {
            for part in &example.parts {
                match part.kind {
                    CodePartKind::StringLiteral => {
                        output_literal
                            .per_language
                            .entry(example.language_slug.clone())
                            .or_insert_with(|| part.source_text.clone());
                    }
                    CodePartKind::OutputOperation => {
                        output_call
                            .per_language
                            .entry(example.language_slug.clone())
                            .or_insert_with(|| part.source_text.clone());
                    }
                    _ => {}
                }
            }
        }
        parameters.push(output_literal);
        parameters.push(output_call);
    }
    let mut source_urls = Vec::new();
    let mut program_bodies = BTreeMap::new();
    for example in examples {
        for part in &example.parts {
            if !part.source_url.is_empty() && !source_urls.contains(&part.source_url) {
                source_urls.push(part.source_url.clone());
            }
        }
        if !example.program_body.is_empty() {
            program_bodies
                .entry(example.language_slug.clone())
                .or_insert_with(|| example.program_body.clone());
        }
    }
    GeneralizedProcedure {
        id: format!("generalized:{}", languages.join("+")),
        shared_structure,
        parameters,
        source_urls,
        program_bodies,
    }
}

/// Bind requirement-supplied values into a generalized procedure and
/// recompose for one language (R1164-6).
///
/// The program is built from the aligned parts, not from a stored
/// template: the target language's program body (its own example's source
/// with the output literal slotted) must still make the `output_call` the
/// alignment recorded for that language, and the bound literal fills the
/// slot, escaped for the quote that opens it. A language no example
/// contributed a body for is [`DecomposeError::ParseFailed`] naming it,
/// never a guessed program. The recomposition carries the source URL of
/// every contributing `CodePart`.
pub fn recompose_for_requirement(
    template: &GeneralizedProcedure,
    bindings: &ParameterBindings,
    target_language: &str,
) -> Result<CodeRecomposition, DecomposeError> {
    let no_body = || {
        DecomposeError::ParseFailed(crate::seed::report_text(
            "code_example_no_program_shape",
            &[("language", target_language)],
        ))
    };
    let per_language_value = |name: &str| {
        template
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .and_then(|parameter| parameter.per_language.get(target_language).cloned())
    };
    let body = template
        .program_bodies
        .get(target_language)
        .ok_or_else(no_body)?;
    let call = bindings
        .0
        .get("output_call")
        .cloned()
        .or_else(|| per_language_value("output_call"))
        .ok_or_else(no_body)?;
    let slot_at = body.find(LITERAL_SLOT).ok_or_else(no_body)?;
    if !body.contains(call.as_str()) {
        return Err(no_body());
    }
    let literal = bindings
        .0
        .get("output_literal")
        .cloned()
        .or_else(|| per_language_value("output_literal"))
        // The literal is bound from the requirement (or the examples'
        // per-language binding), never invented: an unbound literal is a
        // refusal naming the language, not a guessed greeting (R1164-6).
        .ok_or_else(|| {
            DecomposeError::ParseFailed(crate::seed::report_text(
                "code_example_unbound_output_literal",
                &[("language", target_language)],
            ))
        })?;
    let quote = body[..slot_at].chars().next_back();
    let source = format!(
        "{}{}{}",
        &body[..slot_at],
        escape_for_quote(&literal, quote),
        &body[slot_at + LITERAL_SLOT.len()..]
    );
    Ok(CodeRecomposition {
        language_slug: target_language.to_owned(),
        source,
        part_source_urls: template.source_urls.clone(),
    })
}

impl GeneralizedProcedure {
    /// The `ProcedureStepRecord`s the adoption gate in
    /// `coding_research_learning.rs` consumes (R1164-7): one step per
    /// shared part kind. The license and provenance fields are left empty
    /// here on purpose -- they are filled by the gate that approves the
    /// procedure, never by the decomposition that proposes it.
    #[must_use]
    pub fn procedure_step_records(&self) -> Vec<ProcedureStepRecord> {
        self.shared_structure
            .iter()
            .enumerate()
            .map(|(index, kind)| ProcedureStepRecord {
                // The gate's procedures count steps from one.
                ordinal: index + 1,
                text: crate::seed::report_text(
                    "code_example_procedure_step",
                    &[("part", kind.as_str())],
                ),
                source_id: self.id.clone(),
                source_url: String::new(),
                sha256: String::new(),
                fetched_at: String::new(),
                license_name: String::new(),
                license_url: String::new(),
                depth: 0,
            })
            .collect()
    }
}

/// A `DecomposedCodeNode` as a Links Notation document (R1164-10).
#[must_use]
pub fn decomposed_links_notation(node: &DecomposedCodeNode) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("decomposed_code_node\n");
    let _ = writeln!(out, "  language_slug {}", node.language_slug);
    for part in &node.parts {
        let _ = writeln!(out, "  part");
        let _ = writeln!(out, "    kind {}", part.kind.as_str());
        if !part.source_text.is_empty() {
            let _ = writeln!(out, "    source_text \"{}\"", part.source_text);
        }
        let _ = writeln!(out, "    cst_node_kind {}", part.cst_node_kind);
        if !part.source_url.is_empty() {
            let _ = writeln!(out, "    source_url \"{}\"", part.source_url);
        }
    }
    out
}

/// A `GeneralizedProcedure` as a Links Notation document (R1164-10).
#[must_use]
pub fn generalized_links_notation(procedure: &GeneralizedProcedure) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("generalized_procedure\n");
    let _ = writeln!(out, "  id \"{}\"", procedure.id);
    for kind in &procedure.shared_structure {
        let _ = writeln!(out, "  shared_structure {}", kind.as_str());
    }
    for parameter in &procedure.parameters {
        let _ = writeln!(out, "  parameter");
        let _ = writeln!(out, "    name {}", parameter.name);
        for (language, value) in &parameter.per_language {
            let _ = writeln!(out, "    {language} \"{value}\"");
        }
    }
    for url in &procedure.source_urls {
        let _ = writeln!(out, "  source_url \"{url}\"");
    }
    for (language, body) in &procedure.program_bodies {
        let _ = writeln!(out, "  program_body");
        let _ = writeln!(out, "    language {language}");
        let _ = writeln!(out, "    source \"{}\"", escape_notation(body));
    }
    out
}

/// A multi-line source as one Links Notation string value.
fn escape_notation(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// A `CodeRecomposition` as a Links Notation document (R1164-10).
#[must_use]
pub fn recomposition_links_notation(recomposition: &CodeRecomposition) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("code_recomposition\n");
    let _ = writeln!(out, "  language_slug {}", recomposition.language_slug);
    let _ = writeln!(
        out,
        "  source \"{}\"",
        recomposition.source.replace('\n', "\\n")
    );
    for url in &recomposition.part_source_urls {
        let _ = writeln!(out, "  part_source_url \"{url}\"");
    }
    out
}
