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
use formal_ai::rust_projection::{ANY_TARGET, REFUSAL_LANGUAGE, REFUSAL_NOFORM_LANGUAGE};
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
    // Assignment is glyph-identical in all three grammars — the operator
    // token has no field label to capture, and every compound form
    // (`+=`, `-=`, ...) splices verbatim, so the whole expression is
    // splice-class rather than ruled.
    "assignment_expression",
    // The augmented forms splice for the same reason: `a += b` keeps its
    // spelling in every target.
    "augmented_assignment_expression",
    // Rust's compound assignment is the same expression: `a += b` is
    // glyph-identical in javascript and typescript.
    "compound_assignment_expr",
    // Literals that keep their spelling.
    "integer_literal",
    "float_literal",
    "string_literal",
    "number",
    "string",
    "true",
    "false",
    // The interior of a refused literal or comment: the parent's splice
    // carries it whole, and the coverage table answers for named kinds.
    "string_content",
    "string_fragment",
    // Comments share the // and /* */ syntax; rust doc comments are //
    // comments in every target.
    "comment",
    "line_comment",
    "doc_comment",
    "outer_doc_comment_marker",
    // The //! marker rides the line_comment splice whole.
    "inner_doc_comment_marker",
    // A character keeps its quotes and escapes in every target: 'a' and
    // '\n' are valid string spellings in javascript and typescript too.
    "char_literal",
    "escape_sequence",
    // An attribute's inner text `derive(Debug)` is a valid expression in
    // every target; the wrapping attribute_item row decides whether the
    // walk reaches it at all.
    "attribute",
    // A bare type name splices: where a template renders it the span is
    // the answer, and inside ruled parents the template decides.
    "primitive_type",
    // `crate` is a plain identifier spelling in the other grammars; it
    // rides the scoped paths that rule it.
    "crate",
    // The wildcard pattern `_` is a valid identifier in both targets,
    // so it splices: `let _ = x` keeps its binding shape.
    "_",
    // `super` rides the scoped paths the same way crate does.
    "super",
    // A tuple expression `(a, b)` keeps its parens and comma in the
    // targets, where it parses as a parenthesized sequence.
    "tuple_expression",
    // A parenthesized expression keeps its parens in every target.
    "parenthesized_expression",
    // `Point { x, y }` — the shorthand field splices inside the
    // struct-expression row's braces.
    "shorthand_field_initializer",
    // The shorthand name of a field pattern splices the same way.
    "shorthand_field_identifier",
    // `true` / `false` are the same literals in every target.
    "boolean_literal",
    // `x[i]` keeps its brackets and index verbatim in every target.
    "index_expression",
    // An array expression `[a, b]` is the same literal in every target.
    "array_expression",
    // A macro token tree is balanced token soup, valid in every target.
    "token_tree",
];

/// Kinds with no spelling in a target grammar — declared no-form. The
/// walk honors these by refusing the construct by name instead of
/// splicing source text the target grammar would reject.
///
/// Each row is (kind, target labels): a `rust` target answers the
/// js/ts→rust legs, `javascript`/`typescript` targets answer the
/// rust→js/ts legs. A kind never rides both the splice list and this
/// one — a kind is authored into exactly one disposition.
const NO_FORM_KINDS: &[(&str, &[&str])] = &[
    // The object cluster: object literals and destructuring have no
    // rust form (rust has no object literals, and a rust struct
    // pattern is grammatically incomplete without its constructor
    // path, which the js construct does not carry).
    ("object", &["rust"]),
    ("pair", &["rust"]),
    ("object_pattern", &["rust"]),
    ("pair_pattern", &["rust"]),
    ("assignment_pattern", &["rust"]),
    ("shorthand_property_identifier_pattern", &["rust"]),
    ("computed_property_name", &["rust"]),
    ("rest_pattern", &["rust"]),
    ("spread_element", &["rust"]),
    // Classes and their parts: the struct+impl split has no single
    // target form, and a method needs the impl context a query cannot
    // see from inside the body.
    ("class", &["rust"]),
    ("class_body", &["rust"]),
    ("class_declaration", &["rust"]),
    ("class_heritage", &["rust"]),
    ("extends_clause", &["rust"]),
    ("method_definition", &["rust"]),
    ("field_definition", &["rust"]),
    ("public_field_definition", &["rust"]),
    // ES modules: rust's use is intra-crate, not a module system.
    ("import", &["rust"]),
    ("import_statement", &["rust"]),
    ("import_clause", &["rust"]),
    ("import_specifier", &["rust"]),
    ("named_imports", &["rust"]),
    ("export_statement", &["rust"]),
    ("export_clause", &["rust"]),
    ("export_specifier", &["rust"]),
    ("meta_property", &["rust"]),
    // The dynamic family: template strings are format macros, regexes
    // are crates, C-style for is while+index, try/catch and throw are
    // Result and panic!, await needs an async runtime, ++/-- need an
    // operator-literal query the engine does not carry, and a js label
    // can name any statement while a rust label names loops and
    // blocks only.
    ("template_string", &["rust"]),
    ("template_substitution", &["rust"]),
    ("regex", &["rust"]),
    ("regex_pattern", &["rust"]),
    ("regex_flags", &["rust"]),
    ("for_statement", &["rust"]),
    ("throw_statement", &["rust"]),
    ("try_statement", &["rust"]),
    ("catch_clause", &["rust"]),
    ("finally_clause", &["rust"]),
    ("update_expression", &["rust"]),
    ("labeled_statement", &["rust"]),
    // Parse-recovery artifacts are not constructs, and a literal type
    // has no rust spelling.
    ("ERROR", &["rust"]),
    ("literal_type", &["rust"]),
    // The match family has no js/ts form: switch arms are statements,
    // not expressions, and inventing a ternary chain would be
    // semantics, not syntax.
    ("match_arm", &["javascript", "typescript"]),
    ("match_pattern", &["javascript", "typescript"]),
    ("match_block", &["javascript", "typescript"]),
    ("match_expression", &["javascript", "typescript"]),
    // The use family, extern crate and modules: js/ts have no
    // path-namespace imports (the ES module family above is the
    // mirror refusal).
    ("use_declaration", &["javascript", "typescript"]),
    ("use_list", &["javascript", "typescript"]),
    ("scoped_use_list", &["javascript", "typescript"]),
    ("use_as_clause", &["javascript", "typescript"]),
    ("use_wildcard", &["javascript", "typescript"]),
    ("extern_crate_declaration", &["javascript", "typescript"]),
    ("mod_item", &["javascript", "typescript"]),
    // Range and loop forms with no glyph-level target: for-range is
    // the iterator protocol, try is Result, a range has no js/ts
    // spelling, and slice patterns have no destructuring form.
    ("for_expression", &["javascript", "typescript"]),
    ("try_expression", &["javascript", "typescript"]),
    ("range_expression", &["javascript", "typescript"]),
    ("range_pattern", &["javascript", "typescript"]),
    ("slice_pattern", &["javascript", "typescript"]),
    // Let conditions and chains have no js/ts form at statement
    // level, and an or-pattern is not a js pattern.
    ("let_condition", &["javascript", "typescript"]),
    ("let_chain", &["javascript", "typescript"]),
    ("or_pattern", &["javascript", "typescript"]),
    // A tuple-struct's ordered body has no class spelling, a fn
    // signature without a body is not a js function, and an async
    // block is a runtime construct.
    (
        "ordered_field_declaration_list",
        &["javascript", "typescript"],
    ),
    ("function_signature_item", &["javascript", "typescript"]),
    ("async_block", &["javascript", "typescript"]),
    // Raw strings carry delimiter hashes no js/ts lexer accepts.
    ("raw_string_literal", &["javascript", "typescript"]),
    // await is postfix `x.await` in rust and prefix `await x` in js/ts;
    // neither leg can reorder into an async context a rule cannot
    // check — the js/ts legs refuse by name, and the rust leg refuses
    // the construct outright (rust await needs `.await` spelling the
    // js source does not carry).
    ("await_expression", &["rust"]),
    // A rust cast is ts `as` syntax (the type_cast rule row carries
    // the ts template) but js has no cast: the js leg refuses by name.
    ("type_cast_expression", &["javascript"]),
    // The enum cluster: payload variants have no target spelling, and
    // a query cannot condition on "variant without a body", so
    // unit-only enums refuse with the rest — the same
    // negative-condition unlock family as update_expression.
    ("enum_item", &["javascript", "typescript"]),
    ("enum_variant_list", &["javascript", "typescript"]),
    ("enum_variant", &["javascript", "typescript"]),
    // impl and trait items with their parts: the struct+impl split has
    // no single js form, a trait method is not an interface method
    // without the impl context a query cannot see, and a declaration
    // list is an impl body.
    ("impl_item", &["javascript", "typescript"]),
    ("declaration_list", &["javascript", "typescript"]),
    ("trait_item", &["javascript", "typescript"]),
    ("associated_type", &["javascript", "typescript"]),
    // The type family with no target spelling: `impl Trait`, `dyn`,
    // `T as Tr`, associated-type bindings, `for<'a>`, and a fn type
    // whose params carry meaning the ts arrow type cannot name.
    ("abstract_type", &["javascript", "typescript"]),
    ("dynamic_type", &["javascript", "typescript"]),
    ("qualified_type", &["javascript", "typescript"]),
    ("bounded_type", &["javascript", "typescript"]),
    ("type_binding", &["javascript", "typescript"]),
    ("higher_ranked_trait_bound", &["javascript", "typescript"]),
    ("bracketed_type", &["javascript", "typescript"]),
    ("function_type", &["javascript", "typescript"]),
    // A where clause is ts-refused — erasing a constraint from output
    // that still looks typed lies — and its predicates follow it (the
    // javascript leg erases the whole clause through the rule row, so
    // the predicate row is dormant there).
    ("where_clause", &["typescript"]),
    ("where_predicate", &["javascript", "typescript"]),
    // Patterns with no target form: `x @ 1`, `&y` and `ref x` are
    // destructuring spellings the targets do not carry.
    ("captured_pattern", &["javascript", "typescript"]),
    ("reference_pattern", &["javascript", "typescript"]),
    ("ref_pattern", &["javascript", "typescript"]),
    // A label cannot lose its quote — no text transformation exists —
    // so labeled control flow refuses by name instead of silently
    // retargetting `break 'outer` to the innermost loop.
    ("label", &["javascript", "typescript"]),
    // `x.await` toward js/ts: the await spelling exists, but its async
    // context rides anonymous modifier tokens the named-children
    // filters drop, so output would not reparse. The unlock
    // (anonymous-token capture) is upstream-shaped and joins the
    // update_expression queue.
    ("await_expression", &["javascript", "typescript"]),
];

/// The four L8 legs: (from grammar, target grammar).
const DIRECTIONS: &[(&str, &str)] = &[
    ("rust", "javascript"),
    ("rust", "typescript"),
    ("javascript", "rust"),
    ("typescript", "rust"),
];

/// The authored rule table — one row per ruled kind and direction, in
/// claim-resolution order (the first query that matches wins, which is
/// why `rust:function_item_typed` precedes `rust:function_item`).
mod rules;

use rules::RULE_ROWS;

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
    for (kind, targets) in NO_FORM_KINDS {
        for target in *targets {
            network.insert_link(
                [seed_root],
                LinkMetadata::new()
                    .with_link_type(LinkType::Semantic)
                    .with_named(true)
                    .with_term(*kind)
                    .with_language(REFUSAL_NOFORM_LANGUAGE)
                    .with_definition(*target),
            );
        }
    }
    let lino = network.to_lino();

    // The seed must load through the same public loader the walk uses.
    let projection = formal_ai::rust_projection::projection_from(&lino)
        .expect("the generated seed loads through projection_from");

    let corpus_kinds: Vec<(
        &'static str,
        BTreeSet<String>,
        std::collections::BTreeMap<String, usize>,
    )> = CORPORA
        .iter()
        .map(|corpus| {
            let sources = corpus_sources(root, *corpus);
            let inventory = named_corpus_inventory(corpus.label, &sources);
            let kinds = inventory.keys().cloned().collect();
            (corpus.label, kinds, inventory)
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
    for (kind, _) in NO_FORM_KINDS {
        if !all_kinds.contains(*kind) {
            fantasy.push(*kind);
        }
    }

    println!(
        "seed: {} rules, {} refusal rows, {} no-form rows",
        projection.rule_count(),
        projection.refusal_count(),
        projection.noform_count()
    );
    for (from, target) in DIRECTIONS {
        let (kinds, inventory) = corpus_kinds
            .iter()
            .find(|(label, _, _)| label == from)
            .map(|(_, kinds, inventory)| (kinds, inventory))
            .expect("the direction's corpus is committed");
        let remaining = projection.coverage_kinds(kinds, target);
        // The authoring work list: what a rule row clears next, most
        // frequent first.
        let mut work_list: Vec<(usize, String)> = remaining
            .iter()
            .map(|refusal| {
                (
                    inventory
                        .get(refusal.construct.as_str())
                        .copied()
                        .unwrap_or(0),
                    refusal.construct.clone(),
                )
            })
            .collect();
        work_list.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        // The full remaining list when the count is asked for on the
        // command line (the default stays the readable head).
        let listed = std::env::args().any(|arg| arg == "--full-work-list");
        let head: Vec<String> = work_list
            .iter()
            .take(if listed { usize::MAX } else { 15 })
            .map(|(count, kind)| format!("{kind}:{count}"))
            .collect();
        println!(
            "{from} -> {target}: {} kinds, {} remaining; next: {}",
            kinds.len(),
            remaining.len(),
            head.join(" ")
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
