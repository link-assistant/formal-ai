use super::RuleRow;

// Projection placeholders are not format strings.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) const RULE_ROWS: &[RuleRow] = &[
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
