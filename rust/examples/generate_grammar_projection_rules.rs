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
        // A typed fn names its return type: typescript keeps it after
        // the parameter list, javascript drops it with the rest of the
        // type grammar. A required `return_type` is what separates
        // this row from the plain one — an optional that does not bind
        // expands to nothing, so the `: ` cannot live in one template
        // (the return_expression:bare precedent, inverted: the row
        // with more structure claims first by seed order).
        name: "rust:function_item_typed",
        sexpression: "(function_item name: (identifier) @name type_parameters: (type_parameters)? @type_parameters parameters: (parameters) @parameters return_type: (_) @returns (where_clause)? @where body: (block) @body)",
        templates: &[
            (
                "javascript",
                "function {name}({parameters}) {{\n{body}\n}}{where}",
            ),
            (
                "typescript",
                "function {name}{type_parameters}({parameters}): {returns} {{\n{body}\n}}{where}",
            ),
        ],
    },
    RuleRow {
        // The untyped fn: generics still ride the typescript template
        // and drop toward javascript; a where clause erases toward js
        // (the rule row) and refuses toward ts (the no-form row), so
        // typed-looking output never loses a constraint silently.
        name: "rust:function_item",
        sexpression: "(function_item name: (identifier) @name type_parameters: (type_parameters)? @type_parameters parameters: (parameters) @parameters (where_clause)? @where body: (block) @body)",
        templates: &[
            (
                "javascript",
                "function {name}({parameters}) {{\n{body}\n}}{where}",
            ),
            (
                "typescript",
                "function {name}{type_parameters}({parameters}) {{\n{body}\n}}{where}",
            ),
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
        // The grammar's statement-level let is let_declaration; a `mut`
        // or type annotation rides unrendered children the template
        // omits.
        name: "rust:let_declaration",
        sexpression: "(let_declaration pattern: (_) @name value: (_) @value)",
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
    RuleRow {
        // The engine's rust grammar names the receiver field `value`, not
        // `base` — a wrong field name makes the row dead weight the
        // coverage table still counts, so the label is measured.
        name: "rust:field_expression",
        sexpression: "(field_expression value: (_) @value field: (field_identifier) @field)",
        templates: &[
            ("javascript", "{value}.{field}"),
            ("typescript", "{value}.{field}"),
        ],
    },
    RuleRow {
        // `path::name` crosses to property access — the mechanical
        // projection at token fidelity; a nested path recurses.
        name: "rust:scoped_identifier",
        sexpression: "(scoped_identifier path: (_) @path name: (_) @name)",
        templates: &[
            ("javascript", "{path}.{name}"),
            ("typescript", "{path}.{name}"),
        ],
    },
    RuleRow {
        // No target has borrows; the reference marker drops and the
        // referred type carries.
        name: "rust:reference_type",
        sexpression: "(reference_type type: (_) @type)",
        templates: &[("javascript", "{type}"), ("typescript", "{type}")],
    },
    RuleRow {
        name: "rust:reference_expression",
        sexpression: "(reference_expression value: (_) @value)",
        templates: &[("javascript", "{value}"), ("typescript", "{value}")],
    },
    RuleRow {
        // Where the target keeps type positions the annotation renders;
        // javascript drops it because the template below omits it.
        name: "rust:parameter",
        sexpression: "(parameter pattern: (_) @pattern type: (_) @type)",
        templates: &[
            ("javascript", "{pattern}"),
            ("typescript", "{pattern}: {type}"),
        ],
    },
    RuleRow {
        // No target has visibility; the marker renders empty where a
        // template reaches it at all.
        name: "rust:visibility_modifier",
        sexpression: "(visibility_modifier)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        // The generic call's argument list: typescript spells it (the
        // generic_function and generic_type rows already place it),
        // javascript drops it with the rest of the type grammar.
        name: "rust:type_arguments",
        sexpression: "(type_arguments (_)* @argument)",
        templates: &[("javascript", ""), ("typescript", "<{*argument|, }>")],
    },
    RuleRow {
        // `build::<u32>()` wraps the callee in generic_function — the
        // turbofish crosses to nothing in javascript and to typescript's
        // own angle-bracket generics.
        name: "rust:generic_function",
        sexpression: "(generic_function function: (_) @function type_arguments: (type_arguments) @arguments)",
        templates: &[
            ("javascript", "{function}"),
            ("typescript", "{function}{arguments}"),
        ],
    },
    RuleRow {
        name: "rust:generic_type",
        sexpression: "(generic_type type: (_) @type type_arguments: (type_arguments) @arguments)",
        templates: &[
            ("javascript", "{type}"),
            ("typescript", "{type}{arguments}"),
        ],
    },
    RuleRow {
        name: "rust:field_initializer",
        sexpression: "(field_initializer field: (_) @field value: (_) @value)",
        templates: &[
            ("javascript", "{field}: {value}"),
            ("typescript", "{field}: {value}"),
        ],
    },
    // Control flow and function shells. The specific shape of a two-shape
    // kind is listed before the bare one: the claim pass gives a link to
    // the first rule that matches it, so the valued/bare pairs only split
    // cleanly in that order.
    RuleRow {
        name: "rust:if_expression",
        sexpression: "(if_expression condition: (_) @condition consequence: (block) @consequence alternative: (else_clause) @alternative)",
        templates: &[
            (
                "javascript",
                "if ({condition}) {{\n{consequence}\n}} {alternative}",
            ),
            (
                "typescript",
                "if ({condition}) {{\n{consequence}\n}} {alternative}",
            ),
        ],
    },
    RuleRow {
        // `if` without an else — the general row's query demands the
        // alternative field, so this row claims only the else-less shape.
        name: "rust:if_expression:bare",
        sexpression: "(if_expression condition: (_) @condition consequence: (block) @consequence)",
        templates: &[
            ("javascript", "if ({condition}) {{\n{consequence}\n}}"),
            ("typescript", "if ({condition}) {{\n{consequence}\n}}"),
        ],
    },
    RuleRow {
        // `else if` chains: the clause wraps the nested if directly.
        name: "rust:else_clause:if",
        sexpression: "(else_clause (if_expression) @nested)",
        templates: &[
            ("javascript", "else {nested}"),
            ("typescript", "else {nested}"),
        ],
    },
    RuleRow {
        name: "rust:else_clause",
        sexpression: "(else_clause (block) @body)",
        templates: &[
            ("javascript", "else {{\n{body}\n}}"),
            ("typescript", "else {{\n{body}\n}}"),
        ],
    },
    RuleRow {
        name: "rust:while_expression",
        sexpression: "(while_expression condition: (_) @condition body: (block) @body)",
        templates: &[
            ("javascript", "while ({condition}) {{\n{body}\n}}"),
            ("typescript", "while ({condition}) {{\n{body}\n}}"),
        ],
    },
    RuleRow {
        // rust's unconditional loop crosses to the trivially-true
        // while. The label rides first in child order so it binds when
        // present — label is no-form toward both targets, so a labeled
        // loop refuses by name instead of erasing a target that
        // changes control flow.
        name: "rust:loop_expression",
        sexpression: "(loop_expression (label)? @label body: (block) @body)",
        templates: &[
            ("javascript", "{label}while (true) {{\n{body}\n}}"),
            ("typescript", "{label}while (true) {{\n{body}\n}}"),
        ],
    },
    RuleRow {
        // `break 'outer` must not render as plain `break` — that
        // retargets the break to the innermost loop. The label capture
        // refuses by name when bound; unbound it expands to nothing.
        name: "rust:break_expression",
        sexpression: "(break_expression (label)? @label)",
        templates: &[
            ("javascript", "{label}break"),
            ("typescript", "{label}break"),
        ],
    },
    RuleRow {
        name: "rust:continue_expression",
        sexpression: "(continue_expression (label)? @label)",
        templates: &[
            ("javascript", "{label}continue"),
            ("typescript", "{label}continue"),
        ],
    },
    RuleRow {
        name: "rust:return_expression",
        sexpression: "(return_expression (_) @argument)",
        templates: &[
            ("javascript", "return {argument}"),
            ("typescript", "return {argument}"),
        ],
    },
    RuleRow {
        // `return;` — listed after the valued row so a return with an
        // argument claims there first.
        name: "rust:return_expression:bare",
        sexpression: "(return_expression)",
        templates: &[("javascript", "return"), ("typescript", "return")],
    },
    RuleRow {
        // The method receiver crosses to the property receiver.
        name: "rust:self",
        sexpression: "(self)",
        templates: &[("javascript", "this"), ("typescript", "this")],
    },
    RuleRow {
        // A closure crosses to an arrow function: the move marker is an
        // anonymous leaf the template never reaches.
        name: "rust:closure_expression",
        sexpression: "(closure_expression parameters: (closure_parameters) @parameters body: (_) @body)",
        templates: &[
            ("javascript", "({parameters}) => {body}"),
            ("typescript", "({parameters}) => {body}"),
        ],
    },
    RuleRow {
        name: "rust:closure_parameters",
        sexpression: "(closure_parameters (_)* @param)",
        templates: &[("javascript", "{*param|, }"), ("typescript", "{*param|, }")],
    },
    RuleRow {
        // `name!(args)` drops the bang and keeps the call shape — the
        // mechanical projection at token fidelity, defined for every macro
        // at once.
        name: "rust:macro_invocation",
        sexpression: "(macro_invocation macro: (_) @macro (token_tree) @arguments)",
        templates: &[
            ("javascript", "{macro}{arguments}"),
            ("typescript", "{macro}{arguments}"),
        ],
    },
    RuleRow {
        // No target has attributes; the marker renders empty the way
        // visibility does — the row keeps the walk honest instead of
        // splicing `#[...]` into target source.
        name: "rust:attribute_item",
        sexpression: "(attribute_item)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        // `Some(x)` in a let binding crosses to object destructuring: a
        // call-shaped binding is not valid target syntax, so the bindings
        // survive as the pattern and the constructor drops.
        name: "rust:tuple_struct_pattern",
        sexpression: "(tuple_struct_pattern type: (_) @constructor (_)* @argument)",
        templates: &[
            ("javascript", "{{ {*argument|, } }}"),
            ("typescript", "{{ {*argument|, } }}"),
        ],
    },
    RuleRow {
        // `Point { x, y }` in a let binding crosses to object
        // destructuring the same way; the type name drops.
        name: "rust:struct_pattern",
        sexpression: "(struct_pattern type: (_) @constructor (_)* @field)",
        templates: &[
            ("javascript", "{{ {*field|, } }}"),
            ("typescript", "{{ {*field|, } }}"),
        ],
    },
    RuleRow {
        // The renamed field: `{ x: a }` is valid target destructuring.
        name: "rust:field_pattern",
        sexpression: "(field_pattern name: (_) @name pattern: (_) @pattern)",
        templates: &[
            ("javascript", "{name}: {pattern}"),
            ("typescript", "{name}: {pattern}"),
        ],
    },
    RuleRow {
        // The shorthand field: the name is the whole pattern.
        name: "rust:field_pattern:shorthand",
        sexpression: "(field_pattern name: (_) @name)",
        templates: &[("javascript", "{name}"), ("typescript", "{name}")],
    },
    RuleRow {
        // `!f` and `-1`: the operator and operand carry no field labels in
        // the rust grammar, and the glyph sequence is the target spelling
        // for the boolean/negation operators the corpus exercises.
        name: "rust:unary_expression",
        sexpression: "(unary_expression)",
        templates: &[("javascript", "{.:text}"), ("typescript", "{.:text}")],
    },
    RuleRow {
        // `Point { x: 1 }` lowers to a target object literal: the
        // constructor name has no callee in an expression position, the
        // field list is exactly an object body.
        name: "rust:struct_expression",
        sexpression: "(struct_expression name: (_) @constructor body: (field_initializer_list) @fields)",
        templates: &[("javascript", "{fields}"), ("typescript", "{fields}")],
    },
    RuleRow {
        name: "rust:field_initializer_list",
        sexpression: "(field_initializer_list (_)* @field)",
        templates: &[
            ("javascript", "{{ {*field|, } }}"),
            ("typescript", "{{ {*field|, } }}"),
        ],
    },
    RuleRow {
        // `..Default::default()` crosses to the spread inside the
        // struct-expression's object body. The variadic (named-only)
        // skips the anonymous `..` marker and binds the base
        // expression.
        name: "rust:base_field_initializer",
        sexpression: "(base_field_initializer (_)* @base)",
        templates: &[("javascript", "...{*base|}"), ("typescript", "...{*base|}")],
    },
    RuleRow {
        // `let (a, b) = t;` destructures as a target array pattern.
        name: "rust:tuple_pattern",
        sexpression: "(tuple_pattern (_)* @element)",
        templates: &[
            ("javascript", "[{*element|, }]"),
            ("typescript", "[{*element|, }]"),
        ],
    },
    RuleRow {
        // The receiver has no parameter slot in the targets: it renders
        // empty, and the body's `self` becomes `this` via the self row.
        name: "rust:self_parameter",
        sexpression: "(self_parameter)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        // Lifetimes have no target spelling; the `'a` marker drops the
        // way borrows do and the references stay.
        name: "rust:lifetime",
        sexpression: "(lifetime)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        // `'a` as a parameter drops whole: the type_parameters join
        // skips its empty piece.
        name: "rust:lifetime_parameter",
        sexpression: "(lifetime_parameter)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        // Generic parameters: typescript spells them, javascript drops
        // them with the rest of the type grammar — the type_arguments
        // precedent.
        name: "rust:type_parameters",
        sexpression: "(type_parameters (_)* @parameter)",
        templates: &[("javascript", ""), ("typescript", "<{*parameter|, }>")],
    },
    RuleRow {
        // `T: Clone + Send` — the bounds become the ts extends list;
        // lifetime parameters drop through their own row, so the join
        // inside skips them.
        name: "rust:type_parameter",
        sexpression: "(type_parameter name: (type_identifier) @name bounds: (trait_bounds)? @bounds)",
        templates: &[("javascript", ""), ("typescript", "{name}{bounds}")],
    },
    RuleRow {
        name: "rust:trait_bounds",
        sexpression: "(trait_bounds (_)* @bound)",
        templates: &[("javascript", ""), ("typescript", " extends {*bound| & }")],
    },
    RuleRow {
        // The unit type in type position is ts void; javascript drops
        // it with the other type positions.
        name: "rust:unit_type",
        sexpression: "(unit_type)",
        templates: &[("javascript", ""), ("typescript", "void")],
    },
    RuleRow {
        // The unit value is the undefined literal in both targets.
        name: "rust:unit_expression",
        sexpression: "(unit_expression)",
        templates: &[("javascript", "undefined"), ("typescript", "undefined")],
    },
    RuleRow {
        // A type alias is a ts type declaration; javascript has no
        // declaration to make.
        name: "rust:type_item",
        sexpression: "(type_item name: (type_identifier) @name type: (_) @type)",
        templates: &[("javascript", ""), ("typescript", "type {name} = {type};")],
    },
    RuleRow {
        // A where clause is type grammar: javascript erases it, and
        // typescript refuses it through the no-form row so output that
        // still looks typed never loses a constraint silently.
        name: "rust:where_clause",
        sexpression: "(where_clause)",
        templates: &[("javascript", "")],
    },
    RuleRow {
        // `std::fmt::Result` in type position crosses to dotted access
        // the same way scoped_identifier does.
        name: "rust:scoped_type_identifier",
        sexpression: "(scoped_type_identifier path: (_) @path name: (_) @name)",
        templates: &[
            ("javascript", "{path}.{name}"),
            ("typescript", "{path}.{name}"),
        ],
    },
    RuleRow {
        // `[u32; 3]` crosses to the element type with target brackets;
        // the fixed length has no target spelling.
        name: "rust:array_type",
        sexpression: "(array_type element: (_) @element)",
        templates: &[("javascript", "{element}"), ("typescript", "{element}[]")],
    },
    RuleRow {
        // A tuple type crosses to the target's bracketed type list —
        // typescript's tuple type; javascript never renders it.
        name: "rust:tuple_type",
        sexpression: "(tuple_type (_)* @element)",
        templates: &[
            ("javascript", "[{*element|, }]"),
            ("typescript", "[{*element|, }]"),
        ],
    },
    RuleRow {
        // A struct crosses to a class shell: the named fields become
        // class fields — untyped in javascript, typed in typescript.
        name: "rust:struct_item",
        sexpression: "(struct_item name: (_) @name body: (field_declaration_list) @body)",
        templates: &[
            ("javascript", "class {name} {body}"),
            ("typescript", "class {name} {body}"),
        ],
    },
    RuleRow {
        name: "rust:field_declaration_list",
        sexpression: "(field_declaration_list (_)* @field)",
        templates: &[
            ("javascript", "{{\n{*field|\n}\n}}"),
            ("typescript", "{{\n{*field|\n}\n}}"),
        ],
    },
    RuleRow {
        name: "rust:field_declaration",
        sexpression: "(field_declaration name: (_) @name type: (_) @type)",
        templates: &[("javascript", "{name};"), ("typescript", "{name}: {type};")],
    },
    RuleRow {
        // `const X: u32 = 5;` drops its type annotation the way
        // parameters do; the binding and value carry.
        name: "rust:const_item",
        sexpression: "(const_item name: (_) @name value: (_) @value)",
        templates: &[
            ("javascript", "const {name} = {value};"),
            ("typescript", "const {name} = {value};"),
        ],
    },
    // javascript / typescript → rust: the statement spine. A kind with
    // both a general and a specific row lists the specific row first —
    // the claim pass gives a link to the first rule that matches it.
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
        name: "es:function_declaration",
        sexpression: "(function_declaration name: (identifier) @name parameters: (formal_parameters) @parameters body: (statement_block) @body)",
        templates: &[("rust", "fn {name}({parameters}) {body}")],
    },
    RuleRow {
        name: "es:formal_parameters",
        sexpression: "(formal_parameters (_)* @parameter)",
        templates: &[("rust", "{*parameter|, }")],
    },
    RuleRow {
        // The block owns its braces; ruled parents like the function
        // template do not add their own.
        name: "es:statement_block",
        sexpression: "(statement_block (_)* @statement)",
        templates: &[("rust", "{{\n{*statement|\n}\n}}")],
    },
    RuleRow {
        name: "es:return_statement",
        sexpression: "(return_statement (_) @argument)",
        templates: &[("rust", "return {argument};")],
    },
    RuleRow {
        // `return;` — listed after the valued row so a return with an
        // argument claims there first.
        name: "es:return_statement:bare",
        sexpression: "(return_statement)",
        templates: &[("rust", "return;")],
    },
    RuleRow {
        name: "es:expression_statement",
        sexpression: "(expression_statement (_) @expression)",
        templates: &[("rust", "{expression};")],
    },
    RuleRow {
        name: "es:binary_expression",
        sexpression: "(binary_expression left: (_) @left operator: (_) @operator right: (_) @right)",
        templates: &[("rust", "{left} {operator:text} {right}")],
    },
    RuleRow {
        name: "es:call_expression",
        sexpression: "(call_expression function: (_) @function arguments: (arguments) @arguments)",
        templates: &[("rust", "{function}{arguments}")],
    },
    RuleRow {
        name: "es:arguments",
        sexpression: "(arguments (_)* @argument)",
        templates: &[("rust", "({*argument|, })")],
    },
    RuleRow {
        name: "es:member_expression",
        sexpression: "(member_expression object: (_) @object property: (property_identifier) @property)",
        templates: &[("rust", "{object}.{property}")],
    },
    RuleRow {
        // var, let and const are one node kind; all three cross to a
        // rust let, the mechanical projection at token fidelity.
        name: "es:variable_declaration",
        sexpression: "(variable_declaration (_)* @declarator)",
        templates: &[("rust", "let {*declarator|; let };")],
    },
    RuleRow {
        name: "es:variable_declarator",
        sexpression: "(variable_declarator name: (_) @name value: (_) @value)",
        templates: &[("rust", "{name} = {value}")],
    },
    RuleRow {
        // `var x;` — the valueless declarator, listed after the valued
        // row for the same reason as the bare return.
        name: "es:variable_declarator:bare",
        sexpression: "(variable_declarator name: (_) @name)",
        templates: &[("rust", "{name}")],
    },
    RuleRow {
        name: "es:parenthesized_expression",
        sexpression: "(parenthesized_expression (_) @expression)",
        templates: &[("rust", "({expression})")],
    },
    // Control flow and expression shells. Specific shapes list before
    // bare ones for the same first-match reason as the return pair.
    RuleRow {
        name: "es:if_statement",
        sexpression: "(if_statement condition: (_) @condition consequence: (_) @consequence alternative: (else_clause) @alternative)",
        templates: &[("rust", "if {condition} {consequence} {alternative}")],
    },
    RuleRow {
        // `if` without an else.
        name: "es:if_statement:bare",
        sexpression: "(if_statement condition: (_) @condition consequence: (_) @consequence)",
        templates: &[("rust", "if {condition} {consequence}")],
    },
    RuleRow {
        // `else if` chains: the clause wraps the nested statement.
        name: "es:else_clause:if",
        sexpression: "(else_clause (if_statement) @nested)",
        templates: &[("rust", "else {nested}")],
    },
    RuleRow {
        name: "es:else_clause",
        sexpression: "(else_clause (statement_block) @body)",
        templates: &[("rust", "else {body}")],
    },
    RuleRow {
        // The parenthesized condition stays parenthesized: rust accepts
        // parens in a condition position.
        name: "es:while_statement",
        sexpression: "(while_statement condition: (_) @condition body: (_) @body)",
        templates: &[("rust", "while {condition} {body}")],
    },
    RuleRow {
        // for-in and for-of are one kind; both cross to rust's for-in,
        // the kind/operator leaves dropping.
        name: "es:for_in_statement",
        sexpression: "(for_in_statement left: (_) @left right: (_) @right body: (_) @body)",
        templates: &[("rust", "for {left} in {right} {body}")],
    },
    RuleRow {
        name: "es:unary_expression",
        sexpression: "(unary_expression operator: (_) @operator argument: (_) @argument)",
        templates: &[("rust", "{operator:text}{argument}")],
    },
    RuleRow {
        // The absent literal crosses to rust's option-none spelling —
        // the one-to-one literal mapping, not a guess.
        name: "es:null",
        sexpression: "(null)",
        templates: &[("rust", "None")],
    },
    RuleRow {
        // The conditional expression crosses to rust's if expression,
        // which is an expression in exactly the positions a ternary is.
        name: "es:ternary_expression",
        sexpression: "(ternary_expression condition: (_) @condition consequence: (_) @consequence alternative: (_) @alternative)",
        templates: &[(
            "rust",
            "if {condition} {{ {consequence} }} else {{ {alternative} }}",
        )],
    },
    RuleRow {
        name: "es:subscript_expression",
        sexpression: "(subscript_expression object: (_) @object index: (_) @index)",
        templates: &[("rust", "{object}[{index}]")],
    },
    RuleRow {
        // The parenthesized parameter list crosses to rust's closure
        // parameter list — the formal_parameters row already renders the
        // comma-joined names without the wrapping parens.
        name: "es:arrow_function",
        sexpression: "(arrow_function parameters: (formal_parameters) @parameters body: (_) @body)",
        templates: &[("rust", "|{parameters}| {body}")],
    },
    RuleRow {
        // The single bare parameter — a different field, not a
        // single-element formal_parameters.
        name: "es:arrow_function:single",
        sexpression: "(arrow_function parameter: (_) @parameter body: (_) @body)",
        templates: &[("rust", "|{parameter}| {body}")],
    },
    RuleRow {
        name: "es:array",
        sexpression: "(array (_)* @element)",
        templates: &[("rust", "[{*element|, }]")],
    },
    RuleRow {
        // The comma sequence is flat however long: one variadic row
        // covers the two-element and the chained form.
        name: "es:sequence_expression",
        sexpression: "(sequence_expression (_)* @item)",
        templates: &[("rust", "{*item|, }")],
    },
    RuleRow {
        // let and const are one kind; both cross to a rust let the way
        // var does — the declared mutability drops at token fidelity.
        name: "es:lexical_declaration",
        sexpression: "(lexical_declaration (_)* @declarator)",
        templates: &[("rust", "let {*declarator|; let };")],
    },
    RuleRow {
        // switch crosses to match: the scrutinee keeps its parens, each
        // case one arm, default the wildcard arm.
        name: "es:switch_statement",
        sexpression: "(switch_statement value: (_) @value body: (switch_body) @body)",
        templates: &[("rust", "match {value} {body}")],
    },
    RuleRow {
        name: "es:switch_body",
        sexpression: "(switch_body (_)* @arm)",
        templates: &[("rust", "{{\n{*arm|\n}\n}}")],
    },
    RuleRow {
        name: "es:switch_case",
        sexpression: "(switch_case value: (_) @value (_)* @body)",
        templates: &[("rust", "{value} => {{\n{*body|\n}\n}},")],
    },
    RuleRow {
        name: "es:switch_default",
        sexpression: "(switch_default (_)* @body)",
        templates: &[("rust", "_ => {{\n{*body|\n}\n}},")],
    },
    RuleRow {
        name: "es:break_statement",
        sexpression: "(break_statement)",
        templates: &[("rust", "break;")],
    },
    RuleRow {
        name: "es:continue_statement",
        sexpression: "(continue_statement)",
        templates: &[("rust", "continue;")],
    },
    RuleRow {
        // The mirror of the rust self row: `this` crosses to `self` in
        // the position a member expression gives it.
        name: "es:this",
        sexpression: "(this)",
        templates: &[("rust", "self")],
    },
    RuleRow {
        // `new Foo(1)` — construction has no keyword in rust; the
        // constructor call is the whole projection.
        name: "es:new_expression",
        sexpression: "(new_expression constructor: (_) @constructor arguments: (arguments) @arguments)",
        templates: &[("rust", "{constructor}{arguments}")],
    },
    RuleRow {
        // An anonymous `function(a) { ... }` expression crosses to a
        // closure the way arrow functions do.
        name: "es:function_expression",
        sexpression: "(function_expression parameters: (formal_parameters) @parameters body: (_) @body)",
        templates: &[("rust", "|{parameters}| {body}")],
    },
    // typescript-only kinds: the parameter wrapper and the annotations it
    // carries. The annotation splices its own `: type` span — a type
    // position rust's parameter and let syntax accept as written.
    RuleRow {
        name: "ts:required_parameter",
        sexpression: "(required_parameter pattern: (_) @pattern type: (type_annotation) @type)",
        templates: &[("rust", "{pattern}{type}")],
    },
    RuleRow {
        // The unannotated parameter.
        name: "ts:required_parameter:bare",
        sexpression: "(required_parameter pattern: (_) @pattern)",
        templates: &[("rust", "{pattern}")],
    },
    RuleRow {
        name: "ts:type_annotation",
        sexpression: "(type_annotation)",
        templates: &[("rust", "{.:text}")],
    },
    RuleRow {
        name: "ts:predefined_type",
        sexpression: "(predefined_type)",
        templates: &[("rust", "{.:text}")],
    },
    // A static crosses as a const: the name and value keep their
    // spellings, the type child is walked for coverage and dropped.
    RuleRow {
        name: "rust:static_item",
        sexpression: "(static_item name: (_) @name value: (_) @value)",
        templates: &[
            ("javascript", "const {name} = {value};"),
            ("typescript", "const {name} = {value};"),
        ],
    },
    // The drop-empty class: modifiers and markers with no js/ts
    // spelling render nothing, the same documented glyph-drop class
    // as borrows and lifetimes.
    RuleRow {
        name: "rust:removed_trait_bound",
        sexpression: "(removed_trait_bound)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        name: "rust:inner_attribute_item",
        sexpression: "(inner_attribute_item)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        name: "rust:remaining_field_pattern",
        sexpression: "(remaining_field_pattern)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    RuleRow {
        name: "rust:function_modifiers",
        sexpression: "(function_modifiers)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    // A mut pattern drops its marker and passes the inner pattern
    // through — a drop-empty here would swallow the binding it wraps.
    // The capture is the named-only variadic over the children, and the
    // marker itself is a named `mutable_specifier` node, which is why
    // the marker carries its own drop rule below.
    RuleRow {
        name: "rust:mut_pattern",
        sexpression: "(mut_pattern (_)* @pattern)",
        templates: &[("javascript", "{*pattern|}"), ("typescript", "{*pattern|}")],
    },
    // The `mut` marker is a named node inside patterns and parameters.
    // Field-qualified parents never visit it, but a variadic enumeration
    // does: it must render nothing, not splice `mut` into a js pattern.
    RuleRow {
        name: "rust:mutable_specifier",
        sexpression: "(mutable_specifier)",
        templates: &[("javascript", ""), ("typescript", "")],
    },
    // A rust cast keeps its `as` spelling in typescript only; the js
    // leg is a declared no-form (javascript has no cast).
    RuleRow {
        name: "rust:type_cast_expression",
        sexpression: "(type_cast_expression)",
        templates: &[("typescript", "{.:text}")],
    },
    // A lone semicolon is an empty statement in rust too — the kinds
    // are the same shape on both sides.
    RuleRow {
        name: "es:empty_statement",
        sexpression: "(empty_statement)",
        templates: &[("rust", ";")],
    },
    // undefined joins null as the absent value: None is the
    // established spelling precedent.
    RuleRow {
        name: "es:undefined",
        sexpression: "(undefined)",
        templates: &[("rust", "None")],
    },
    // do/while inverts into loop + break: the body keeps its block,
    // the negated condition keeps its parens.
    RuleRow {
        name: "es:do_statement",
        sexpression: "(do_statement body: (_) @body condition: (_) @condition)",
        templates: &[(
            "rust",
            "loop {{\n{body}\nif !{condition} {{\nbreak;\n}}\n}}",
        )],
    },
    // An array pattern is a tuple pattern's mirror: brackets and
    // comma keep their spellings.
    RuleRow {
        name: "es:array_pattern",
        sexpression: "(array_pattern (_)* @element)",
        templates: &[("rust", "[{*element|, }]")],
    },
    // The ?. marker is a child of member_expression, which already
    // renders the plain access — the marker itself renders nothing.
    RuleRow {
        name: "es:optional_chain",
        sexpression: "(optional_chain)",
        templates: &[("rust", "")],
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
