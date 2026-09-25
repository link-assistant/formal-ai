//! Generate `data/seed/grammar-projection-rules.lino` (issue #1138, plan
//! 16 L8).
//!
//! ```bash
//! cargo run --example generate_grammar_projection_rules
//! ```
//!
//! The authored table lives here — one builder row per ruled kind and
//! direction, one declared refusal per splice-class kind — and the seed is
//! the composition, never hand-written: the rule set serializes through
//! its own `to_lino`, the refusal rows ride the same document as extra
//! root children of the network round-trip (the rule set's `to_lino`
//! drops unknown links, so the rows must travel through the network).
//! The generator also prints the per-direction coverage the corpus
//! ratchet pins, and flags any authored kind no committed corpus parses
//! into, so the table stays measured.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use formal_ai::grammar_kinds::{CORPORA, corpus_inventory, corpus_sources, named_corpus_inventory};
use formal_ai::rust_projection::{ANY_TARGET, REFUSAL_LANGUAGE};
use meta_language::{
    LinkMetadata, LinkNetwork, LinkQuery, LinkType, TranslationRule, TranslationRuleSet,
};

const RULE_SET_NAME: &str = "grammar-projection";
const RULE_SET_TERM: &str = "translation-rule-set";

/// One authored rule row: the query, keyed by target label templates.
struct RuleRow {
    /// Rule name, unique across the seed.
    name: &'static str,
    /// The s-expression the engine matches (root kind is its head).
    sexpression: &'static str,
    /// (target grammar label, template source) pairs.
    templates: &'static [(&'static str, &'static str)],
}

/// Kinds whose source text is already valid in every target — declared
/// refused, which the walk honors by splicing the span verbatim.
const SPLICE_CLASS_KINDS: &[&str] = &[
    // Names splice unchanged between the three grammars.
    "identifier",
    "field_identifier",
    "property_identifier",
    "shorthand_property_identifier",
    "statement_identifier",
    "type_identifier",
    // Punctuation and operators the grammars share glyph for glyph.
    "(",
    ")",
    "{",
    "}",
    "[",
    "]",
    ",",
    ";",
    ".",
    "=",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "+",
    "-",
    "*",
    "/",
    "%",
    "&&",
    "||",
    "!",
    // Literals that keep their spelling.
    "integer_literal",
    "float_literal",
    "string_literal",
    "number",
    "string",
    "true",
    "false",
    // Comments share the // and /* */ syntax.
    "comment",
    "line_comment",
];

/// The four L8 legs: (from grammar, target grammar).
const DIRECTIONS: &[(&str, &str)] = &[
    ("rust", "javascript"),
    ("rust", "typescript"),
    ("javascript", "rust"),
    ("typescript", "rust"),
];

// The template literals below are projection placeholders, not format
// strings.
#[allow(clippy::literal_string_with_formatting_args)]
const RULE_ROWS: &[RuleRow] = &[
    // rust → javascript / typescript: the module-level spine.
    RuleRow {
        name: "rust:source_file",
        sexpression: "(source_file (_)* @item)",
        templates: &[
            ("javascript", "{*item|\n\n}"),
            ("typescript", "{*item|\n\n}"),
        ],
    },
    RuleRow {
        name: "rust:identifier",
        sexpression: "(identifier)",
        templates: &[("javascript", "{.:text}"), ("typescript", "{.:text}")],
    },
    RuleRow {
        name: "rust:function_item",
        sexpression: "(function_item name: (identifier) @name parameters: (parameters) @parameters body: (block) @body)",
        templates: &[
            ("javascript", "function {name}({parameters}) {{\n{body}\n}}"),
            ("typescript", "function {name}({parameters}) {{\n{body}\n}}"),
        ],
    },
    RuleRow {
        name: "rust:parameters",
        sexpression: "(parameters (_)* @param)",
        templates: &[("javascript", "{*param|, }"), ("typescript", "{*param|, }")],
    },
    RuleRow {
        // A parameter without a type annotation is a bare type_identifier
        // child of parameters — there is no parameter node at all — so the
        // name class is what the variadic enumeration renders.
        name: "rust:type_identifier",
        sexpression: "(type_identifier)",
        templates: &[("javascript", "{.:text}"), ("typescript", "{.:text}")],
    },
    RuleRow {
        name: "rust:block",
        sexpression: "(block (_)* @statement)",
        templates: &[
            ("javascript", "{*statement|\n}"),
            ("typescript", "{*statement|\n}"),
        ],
    },
    RuleRow {
        name: "rust:let_statement",
        sexpression: "(let_statement pattern: (identifier) @name value: (_) @value)",
        templates: &[
            ("javascript", "let {name} = {value};"),
            ("typescript", "let {name} = {value};"),
        ],
    },
    RuleRow {
        name: "rust:expression_statement",
        sexpression: "(expression_statement (_) @expression)",
        templates: &[
            ("javascript", "{expression};"),
            ("typescript", "{expression};"),
        ],
    },
    RuleRow {
        name: "rust:binary_expression",
        sexpression: "(binary_expression left: (_) @left operator: (_) @operator right: (_) @right)",
        templates: &[
            ("javascript", "{left} {operator:text} {right}"),
            ("typescript", "{left} {operator:text} {right}"),
        ],
    },
    RuleRow {
        name: "rust:call_expression",
        sexpression: "(call_expression function: (_) @function arguments: (arguments) @arguments)",
        templates: &[
            ("javascript", "{function}{arguments}"),
            ("typescript", "{function}{arguments}"),
        ],
    },
    RuleRow {
        name: "rust:arguments",
        sexpression: "(arguments (_)* @argument)",
        templates: &[
            ("javascript", "({*argument|, })"),
            ("typescript", "({*argument|, })"),
        ],
    },
    // javascript / typescript → rust: the statement spine.
    RuleRow {
        name: "es:program",
        sexpression: "(program (_)* @statement)",
        templates: &[("rust", "{*statement|\n\n}")],
    },
    RuleRow {
        name: "es:identifier",
        sexpression: "(identifier)",
        templates: &[("rust", "{.:text}")],
    },
    RuleRow {
        name: "es:number",
        sexpression: "(number)",
        templates: &[("rust", "{.:text}")],
    },
    RuleRow {
        name: "es:expression_statement:identifier",
        sexpression: "(expression_statement (identifier) @expression)",
        templates: &[("rust", "{expression};")],
    },
    RuleRow {
        name: "es:expression_statement:number",
        sexpression: "(expression_statement (number) @expression)",
        templates: &[("rust", "{expression};")],
    },
];

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");

    let mut rule_set = TranslationRuleSet::new(RULE_SET_NAME);
    for row in RULE_ROWS {
        let query = LinkQuery::from_sexpression(row.sexpression)
            .unwrap_or_else(|error| panic!("rule {}: {error}", row.name))
            .with_link_type(LinkType::Syntax);
        let mut rule = TranslationRule::new(row.name, query);
        for &(target, template) in row.templates {
            rule = rule.with_template(target, template);
        }
        rule_set = rule_set.with_rule(rule);
    }

    // The refusal rows ride the network: the rule set's own to_lino drops
    // links it does not model, so the seed is the network round-trip.
    let mut network =
        LinkNetwork::from_lino(&rule_set.to_lino()).expect("the generated rule set serializes");
    let seed_root = network
        .links()
        .find(|link| {
            link.metadata().link_type() == Some(LinkType::Semantic)
                && link.metadata().term() == Some(RULE_SET_TERM)
        })
        .expect("the serialized rule set carries its root");
    let seed_root = seed_root.id();
    for kind in SPLICE_CLASS_KINDS {
        network.insert_link(
            [seed_root],
            LinkMetadata::new()
                .with_link_type(LinkType::Semantic)
                .with_named(true)
                .with_term(*kind)
                .with_language(REFUSAL_LANGUAGE)
                .with_definition(ANY_TARGET),
        );
    }
    let lino = network.to_lino();

    // The seed must load through the same public loader the walk uses.
    let projection = formal_ai::rust_projection::projection_from(&lino)
        .expect("the generated seed loads through projection_from");

    let corpus_kinds: Vec<(&'static str, BTreeSet<String>)> = CORPORA
        .iter()
        .map(|corpus| {
            let sources = corpus_sources(root, *corpus);
            (
                corpus.label,
                named_corpus_inventory(corpus.label, &sources)
                    .into_keys()
                    .collect(),
            )
        })
        .collect();
    // The fantasy check answers over the full inventory — anonymous
    // punctuation kinds are real parse kinds even though the coverage
    // table only accounts for named ones.
    let all_kinds: BTreeSet<String> = CORPORA
        .iter()
        .flat_map(|corpus| {
            let sources = corpus_sources(root, *corpus);
            corpus_inventory(corpus.label, &sources).into_keys()
        })
        .collect();

    let mut fantasy = Vec::new();
    for kind in SPLICE_CLASS_KINDS {
        if !all_kinds.contains(*kind) {
            fantasy.push(*kind);
        }
    }

    println!(
        "seed: {} rules, {} refusal rows",
        projection.rule_count(),
        projection.refusal_count()
    );
    for (from, target) in DIRECTIONS {
        let kinds = corpus_kinds
            .iter()
            .find(|(label, _)| label == from)
            .map(|(_, kinds)| kinds)
            .expect("the direction's corpus is committed");
        let remaining = projection.coverage_kinds(kinds, target);
        println!(
            "{from} -> {target}: {} kinds, {} remaining",
            kinds.len(),
            remaining.len()
        );
    }
    if !fantasy.is_empty() {
        println!("fantasy refusal rows (no corpus parses into them): {fantasy:?}");
    }

    let seed_path = root
        .join("data")
        .join("seed")
        .join("grammar-projection-rules.lino");
    fs::write(&seed_path, lino).expect("the seed path is writable");
    println!("wrote {}", seed_path.display());
}
