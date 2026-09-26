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
    // Assignment is glyph-identical in all three grammars — the operator
    // token has no field label to capture, and every compound form
    // (`+=`, `-=`, ...) splices verbatim, so the whole expression is
    // splice-class rather than ruled.
    "assignment_expression",
    // The augmented forms splice for the same reason: `a += b` keeps its
    // spelling in every target.
    "augmented_assignment_expression",
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
    // A tuple expression `(a, b)` keeps its parens and comma in the
    // targets, where it parses as a parenthesized sequence.
    "tuple_expression",
    // `Point { x, y }` — the shorthand field splices inside the
    // struct-expression row's braces.
    "shorthand_field_initializer",
    // `true` / `false` are the same literals in every target.
    "boolean_literal",
    // `x[i]` keeps its brackets and index verbatim in every target.
    "index_expression",
    // An array expression `[a, b]` is the same literal in every target.
    "array_expression",
    // Rust borrow markers ride their ruled parents' templates; a
    // `mut` inside a let never renders because the let template omits it.
    "mutable_specifier",
    // A macro token tree is balanced token soup, valid in every target.
    "token_tree",
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
        name: "rust:type_arguments",
        sexpression: "(type_arguments)",
        templates: &[("javascript", ""), ("typescript", "")],
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
        // rust's unconditional loop crosses to the trivially-true while.
        name: "rust:loop_expression",
        sexpression: "(loop_expression body: (block) @body)",
        templates: &[
            ("javascript", "while (true) {{\n{body}\n}}"),
            ("typescript", "while (true) {{\n{body}\n}}"),
        ],
    },
    RuleRow {
        name: "rust:break_expression",
        sexpression: "(break_expression)",
        templates: &[("javascript", "break"), ("typescript", "break")],
    },
    RuleRow {
        name: "rust:continue_expression",
        sexpression: "(continue_expression)",
        templates: &[("javascript", "continue"), ("typescript", "continue")],
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

    println!(
        "seed: {} rules, {} refusal rows",
        projection.rule_count(),
        projection.refusal_count()
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
        let head: Vec<String> = work_list
            .iter()
            .take(15)
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
