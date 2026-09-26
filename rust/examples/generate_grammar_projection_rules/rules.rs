//! The authored rule table for `generate_grammar_projection_rules` —
//! split out of the example root when the table outgrew the file-size
//! gate. Pure data: the root file owns how it serializes.

use super::RuleRow;

// The template literals below are projection placeholders, not format
// strings.
#[allow(clippy::literal_string_with_formatting_args)]
pub const RULE_ROWS: &[RuleRow] = &[
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
