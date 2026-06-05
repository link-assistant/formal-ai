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
//! Language renderers then project that tree into source code. The renderers
//! still emit textual tokens because source code is text, but they no longer own
//! final-program templates; the tree is logged in Links Notation so the solver's
//! reasoning can be inspected independently from the printed code.

use std::fmt::Write as _;

use crate::coding::catalog::ProgramLanguage;

use super::{ListValue, Operation, ParsedListItem, Reduce, Transform};

const NUMBERS: &str = "numbers";
const SORTED: &str = "sorted";
const SORTED_NUMBERS: &str = "sorted_numbers";
const RESULT: &str = "result";

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

    const fn rust(self) -> &'static str {
        match self {
            Self::Integer => "i64",
            Self::Float => "f64",
            Self::Text => "&str",
        }
    }

    const fn go(self) -> &'static str {
        match self {
            Self::Integer => "int",
            Self::Float => "float64",
            Self::Text => "string",
        }
    }

    const fn c_family(self) -> &'static str {
        match self {
            Self::Integer => "int",
            Self::Float => "double",
            Self::Text => "string",
        }
    }

    const fn java_boxed(self) -> &'static str {
        match self {
            Self::Integer => "Integer",
            Self::Float => "Double",
            Self::Text => "String",
        }
    }

    const fn c_printf(self) -> &'static str {
        match self {
            Self::Integer => "%d",
            Self::Float => "%g",
            Self::Text => "%s",
        }
    }

    const fn scalar_zero(self) -> &'static str {
        match self {
            Self::Integer => "0",
            Self::Float => "0.0",
            Self::Text => "",
        }
    }

    const fn scalar_one(self) -> &'static str {
        match self {
            Self::Integer => "1",
            Self::Float => "1.0",
            Self::Text => "",
        }
    }

    const fn links_label(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Text => "string",
        }
    }

    const fn typed_array(self) -> &'static str {
        match self {
            Self::Text => "string",
            Self::Integer | Self::Float => "number",
        }
    }

    const fn cpp(self) -> &'static str {
        match self {
            Self::Text => "std::string",
            Self::Integer | Self::Float => self.c_family(),
        }
    }

    const fn c_storage(self) -> &'static str {
        match self {
            Self::Text => "char *",
            Self::Integer | Self::Float => self.c_family(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProgramStatement {
    LiteralList {
        name: &'static str,
        mutable: bool,
    },
    TransformList {
        semantic_node: &'static str,
        source: &'static str,
        target: &'static str,
        transform: Transform,
    },
    ReduceList {
        source: &'static str,
        target: &'static str,
        reduce: Reduce,
    },
    PrintJoined {
        source: &'static str,
        separator: &'static str,
    },
    PrintScalar {
        source: &'static str,
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
                transform,
            } => {
                let _ = writeln!(
                    out,
                    "  semantic_node {semantic_node} source={source} target={target} direction={}",
                    transform.direction_label()
                );
            }
            Self::ReduceList {
                source,
                target,
                reduce,
            } => {
                let _ = writeln!(
                    out,
                    "  semantic_node reduce_list source={source} target={target} reducer={}",
                    reduce.label()
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

impl Transform {
    const fn direction_label(self) -> &'static str {
        match self {
            Self::SortAscending => "ascending",
            Self::SortDescending => "descending",
            Self::Reverse => "given_order_reversed",
        }
    }

    const fn semantic_node(self) -> &'static str {
        match self {
            Self::SortAscending | Self::SortDescending => "sort_list",
            Self::Reverse => "reverse_list",
        }
    }
}

impl Reduce {
    const fn label(self) -> &'static str {
        match self {
            Self::Sum => "sum",
            Self::Product => "product",
            Self::Minimum => "minimum",
            Self::Maximum => "maximum",
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
    /// Render the program tree into the requested target language.
    #[must_use]
    pub fn render(&self) -> String {
        match self.language.slug {
            "javascript" => render_js(self, false),
            "typescript" => render_js(self, true),
            "rust" => render_rust(self),
            "go" => render_go(self),
            "ruby" => render_ruby(self),
            "java" => render_java(self),
            "csharp" => render_csharp(self),
            "c" => render_c(self),
            "cpp" => render_cpp(self),
            _ => render_python(self),
        }
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

    fn transform(&self) -> Option<Transform> {
        self.statements
            .iter()
            .find_map(|statement| match statement {
                ProgramStatement::TransformList { transform, .. } => Some(*transform),
                _ => None,
            })
    }

    fn reduce(&self) -> Option<Reduce> {
        self.statements
            .iter()
            .find_map(|statement| match statement {
                ProgramStatement::ReduceList { reduce, .. } => Some(*reduce),
                _ => None,
            })
    }
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
    let mut statements = vec![ProgramStatement::LiteralList {
        name: NUMBERS,
        mutable: matches!(language.slug, "rust" | "go" | "java" | "c" | "cpp"),
    }];

    match operation {
        Operation::Transform(transform) => {
            statements.push(ProgramStatement::TransformList {
                semantic_node: transform.semantic_node(),
                source: NUMBERS,
                target: SORTED,
                transform,
            });
            statements.push(ProgramStatement::PrintJoined {
                source: SORTED,
                separator: ", ",
            });
        }
        Operation::Reduce(reduce) => {
            statements.push(ProgramStatement::ReduceList {
                source: NUMBERS,
                target: RESULT,
                reduce,
            });
            statements.push(ProgramStatement::PrintScalar { source: RESULT });
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

fn render_js(program: &NumericProgram, typed: bool) -> String {
    let literal = program.literal();
    let decl = if typed {
        format!(
            "const {NUMBERS}: {}[] = [{literal}];",
            program.value_type.typed_array()
        )
    } else {
        format!("const {NUMBERS} = [{literal}];")
    };
    if let Some(transform) = program.transform() {
        let expr = match (transform, program.value_type) {
            (Transform::SortAscending, ValueType::Text) => "[...numbers].sort()",
            (Transform::SortDescending, ValueType::Text) => "[...numbers].sort().reverse()",
            (Transform::SortAscending, _) => "[...numbers].sort((a, b) => a - b)",
            (Transform::SortDescending, _) => "[...numbers].sort((a, b) => b - a)",
            (Transform::Reverse, _) => "[...numbers].reverse()",
        };
        return format!("{decl}\nconst {SORTED} = {expr};\nconsole.log({SORTED}.join(\", \"));");
    }
    let reduce = program.reduce().expect("reduction program has reducer");
    let expr = match reduce {
        Reduce::Sum => "numbers.reduce((a, b) => a + b, 0)",
        Reduce::Product => "numbers.reduce((a, b) => a * b, 1)",
        Reduce::Minimum => "Math.min(...numbers)",
        Reduce::Maximum => "Math.max(...numbers)",
    };
    format!("{decl}\nconst {RESULT} = {expr};\nconsole.log({RESULT});")
}

fn render_python(program: &NumericProgram) -> String {
    let literal = program.literal();
    if let Some(transform) = program.transform() {
        let action = match transform {
            Transform::SortAscending => "sorted(numbers)",
            Transform::SortDescending => "sorted(numbers, reverse=True)",
            Transform::Reverse => "list(reversed(numbers))",
        };
        return format!(
            "{NUMBERS} = [{literal}]\n{SORTED_NUMBERS} = {action}\nprint(\", \".join(str(n) for n in {SORTED_NUMBERS}))"
        );
    }
    match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => {
            format!("{NUMBERS} = [{literal}]\n{RESULT} = sum({NUMBERS})\nprint({RESULT})")
        }
        Reduce::Product => format!(
            "import math\n\n{NUMBERS} = [{literal}]\n{RESULT} = math.prod({NUMBERS})\nprint({RESULT})"
        ),
        Reduce::Minimum => {
            format!("{NUMBERS} = [{literal}]\n{RESULT} = min({NUMBERS})\nprint({RESULT})")
        }
        Reduce::Maximum => {
            format!("{NUMBERS} = [{literal}]\n{RESULT} = max({NUMBERS})\nprint({RESULT})")
        }
    }
}

fn render_rust(program: &NumericProgram) -> String {
    let literal = program.literal();
    let ty = program.value_type.rust();
    if let Some(transform) = program.transform() {
        let action = match (transform, program.value_type) {
            (Transform::SortAscending, ValueType::Integer | ValueType::Text) => {
                "numbers.sort();".to_owned()
            }
            (Transform::SortDescending, ValueType::Integer | ValueType::Text) => {
                "numbers.sort_by(|a, b| b.cmp(a));".to_owned()
            }
            (Transform::SortAscending, ValueType::Float) => {
                "numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());".to_owned()
            }
            (Transform::SortDescending, ValueType::Float) => {
                "numbers.sort_by(|a, b| b.partial_cmp(a).unwrap());".to_owned()
            }
            (Transform::Reverse, _) => "numbers.reverse();".to_owned(),
        };
        return format!(
            "fn main() {{\n    let mut {NUMBERS}: Vec<{ty}> = vec![{literal}];\n    {action}\n    let rendered: Vec<String> = {NUMBERS}.iter().map(|n| n.to_string()).collect();\n    println!(\"{{}}\", rendered.join(\", \"));\n}}"
        );
    }
    let compute = match (
        program.reduce().expect("reduction program has reducer"),
        program.value_type,
    ) {
        (Reduce::Sum, _) => format!("{NUMBERS}.iter().copied().sum::<{ty}>()"),
        (Reduce::Product, _) => format!("{NUMBERS}.iter().copied().product::<{ty}>()"),
        (Reduce::Minimum, ValueType::Integer) => format!("*{NUMBERS}.iter().min().unwrap()"),
        (Reduce::Maximum, ValueType::Integer) => format!("*{NUMBERS}.iter().max().unwrap()"),
        (Reduce::Minimum, ValueType::Float) => {
            format!("{NUMBERS}.iter().copied().fold(f64::INFINITY, f64::min)")
        }
        (Reduce::Maximum, ValueType::Float) => {
            format!("{NUMBERS}.iter().copied().fold(f64::NEG_INFINITY, f64::max)")
        }
        (_, ValueType::Text) => unreachable!("text lists do not support reductions"),
    };
    format!(
        "fn main() {{\n    let {NUMBERS}: Vec<{ty}> = vec![{literal}];\n    let {RESULT} = {compute};\n    println!(\"{{}}\", {RESULT});\n}}"
    )
}

fn render_go(program: &NumericProgram) -> String {
    let literal = program.literal();
    let ty = program.value_type.go();
    if let Some(transform) = program.transform() {
        let format_item = match program.value_type {
            ValueType::Integer => "strconv.Itoa(n)",
            ValueType::Float => "strconv.FormatFloat(n, 'g', -1, 64)",
            ValueType::Text => "n",
        };
        let (imports, action) = match (transform, program.value_type) {
            (Transform::SortAscending, ValueType::Text) => (
                "\t\"fmt\"\n\t\"sort\"\n\t\"strings\"",
                format!("sort.Strings({NUMBERS})"),
            ),
            (Transform::SortDescending, ValueType::Text) => (
                "\t\"fmt\"\n\t\"sort\"\n\t\"strings\"",
                format!("sort.Sort(sort.Reverse(sort.StringSlice({NUMBERS})))"),
            ),
            (Transform::SortAscending, _) => (
                "\t\"fmt\"\n\t\"sort\"\n\t\"strconv\"\n\t\"strings\"",
                format!("sort.Slice({NUMBERS}, func(i, j int) bool {{ return {NUMBERS}[i] < {NUMBERS}[j] }})"),
            ),
            (Transform::SortDescending, _) => (
                "\t\"fmt\"\n\t\"sort\"\n\t\"strconv\"\n\t\"strings\"",
                format!("sort.Slice({NUMBERS}, func(i, j int) bool {{ return {NUMBERS}[i] > {NUMBERS}[j] }})"),
            ),
            (Transform::Reverse, ValueType::Text) => (
                "\t\"fmt\"\n\t\"strings\"",
                format!("for i, j := 0, len({NUMBERS})-1; i < j; i, j = i+1, j-1 {{\n\t\t{NUMBERS}[i], {NUMBERS}[j] = {NUMBERS}[j], {NUMBERS}[i]\n\t}}"),
            ),
            (Transform::Reverse, _) => (
                "\t\"fmt\"\n\t\"strconv\"\n\t\"strings\"",
                format!("for i, j := 0, len({NUMBERS})-1; i < j; i, j = i+1, j-1 {{\n\t\t{NUMBERS}[i], {NUMBERS}[j] = {NUMBERS}[j], {NUMBERS}[i]\n\t}}"),
            ),
        };
        return format!(
            "package main\n\nimport (\n{imports}\n)\n\nfunc main() {{\n\t{NUMBERS} := []{ty}{{{literal}}}\n\t{action}\n\tparts := make([]string, len({NUMBERS}))\n\tfor i, n := range {NUMBERS} {{\n\t\tparts[i] = {format_item}\n\t}}\n\tfmt.Println(strings.Join(parts, \", \"))\n}}"
        );
    }
    let body = match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => {
            format!(
                "var {RESULT} {ty} = 0\n\tfor _, n := range {NUMBERS} {{\n\t\t{RESULT} += n\n\t}}"
            )
        }
        Reduce::Product => {
            format!(
                "var {RESULT} {ty} = 1\n\tfor _, n := range {NUMBERS} {{\n\t\t{RESULT} *= n\n\t}}"
            )
        }
        Reduce::Minimum => {
            format!("{RESULT} := {NUMBERS}[0]\n\tfor _, n := range {NUMBERS}[1:] {{\n\t\tif n < {RESULT} {{\n\t\t\t{RESULT} = n\n\t\t}}\n\t}}")
        }
        Reduce::Maximum => {
            format!("{RESULT} := {NUMBERS}[0]\n\tfor _, n := range {NUMBERS}[1:] {{\n\t\tif n > {RESULT} {{\n\t\t\t{RESULT} = n\n\t\t}}\n\t}}")
        }
    };
    format!(
        "package main\n\nimport \"fmt\"\n\nfunc main() {{\n\t{NUMBERS} := []{ty}{{{literal}}}\n\t{body}\n\tfmt.Println({RESULT})\n}}"
    )
}

fn render_ruby(program: &NumericProgram) -> String {
    let literal = program.literal();
    if let Some(transform) = program.transform() {
        let action = match transform {
            Transform::SortAscending => "numbers.sort",
            Transform::SortDescending => "numbers.sort.reverse",
            Transform::Reverse => "numbers.reverse",
        };
        return format!("{NUMBERS} = [{literal}]\n{SORTED} = {action}\nputs {SORTED}.join(\", \")");
    }
    let expr = match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => "numbers.sum",
        Reduce::Product => "numbers.inject(1, :*)",
        Reduce::Minimum => "numbers.min",
        Reduce::Maximum => "numbers.max",
    };
    format!("{NUMBERS} = [{literal}]\n{RESULT} = {expr}\nputs {RESULT}")
}

fn render_java(program: &NumericProgram) -> String {
    let literal = program.literal();
    let ty = match program.value_type {
        ValueType::Text => "String",
        _ => program.value_type.c_family(),
    };
    if let Some(transform) = program.transform() {
        let boxed = program.value_type.java_boxed();
        let action = match transform {
            Transform::SortAscending => "Arrays.sort(numbers);".to_owned(),
            Transform::SortDescending if program.value_type == ValueType::Text => {
                "Arrays.sort(numbers, Collections.reverseOrder());".to_owned()
            }
            Transform::SortDescending => {
                format!(
                    "{boxed}[] boxed = Arrays.stream({NUMBERS}).boxed().toArray({boxed}[]::new);\n        Arrays.sort(boxed, Collections.reverseOrder());\n        for (int i = 0; i < {NUMBERS}.length; i++) {NUMBERS}[i] = boxed[i];"
                )
            }
            Transform::Reverse => format!(
                "for (int i = 0, j = {NUMBERS}.length - 1; i < j; i++, j--) {{\n            {ty} tmp = {NUMBERS}[i];\n            {NUMBERS}[i] = {NUMBERS}[j];\n            {NUMBERS}[j] = tmp;\n        }}"
            ),
        };
        return format!(
            "import java.util.Arrays;\nimport java.util.Collections;\nimport java.util.StringJoiner;\n\npublic class Main {{\n    public static void main(String[] args) {{\n        {ty}[] {NUMBERS} = {{{literal}}};\n        {action}\n        StringJoiner joiner = new StringJoiner(\", \");\n        for ({ty} n : {NUMBERS}) joiner.add(String.valueOf(n));\n        System.out.println(joiner.toString());\n    }}\n}}"
        );
    }
    let body = match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => {
            format!("{ty} {RESULT} = 0;\n        for ({ty} n : {NUMBERS}) {RESULT} += n;")
        }
        Reduce::Product => {
            format!("{ty} {RESULT} = 1;\n        for ({ty} n : {NUMBERS}) {RESULT} *= n;")
        }
        Reduce::Minimum => {
            format!("{ty} {RESULT} = {NUMBERS}[0];\n        for ({ty} n : {NUMBERS}) {RESULT} = Math.min({RESULT}, n);")
        }
        Reduce::Maximum => {
            format!("{ty} {RESULT} = {NUMBERS}[0];\n        for ({ty} n : {NUMBERS}) {RESULT} = Math.max({RESULT}, n);")
        }
    };
    format!(
        "public class Main {{\n    public static void main(String[] args) {{\n        {ty}[] {NUMBERS} = {{{literal}}};\n        {body}\n        System.out.println({RESULT});\n    }}\n}}"
    )
}

fn render_csharp(program: &NumericProgram) -> String {
    let literal = program.literal();
    let ty = program.value_type.c_family();
    if let Some(transform) = program.transform() {
        let action = match transform {
            Transform::SortAscending => "numbers.OrderBy(n => n)",
            Transform::SortDescending => "numbers.OrderByDescending(n => n)",
            Transform::Reverse => "numbers.Reverse()",
        };
        return format!(
            "using System;\nusing System.Linq;\n\nclass Program {{\n    static void Main() {{\n        {ty}[] {NUMBERS} = {{{literal}}};\n        var {SORTED} = {action};\n        Console.WriteLine(string.Join(\", \", {SORTED}));\n    }}\n}}"
        );
    }
    let expr = match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => "numbers.Sum()".to_owned(),
        Reduce::Product => format!("{NUMBERS}.Aggregate(({ty})1, (a, b) => a * b)"),
        Reduce::Minimum => "numbers.Min()".to_owned(),
        Reduce::Maximum => "numbers.Max()".to_owned(),
    };
    format!(
        "using System;\nusing System.Linq;\n\nclass Program {{\n    static void Main() {{\n        {ty}[] {NUMBERS} = {{{literal}}};\n        var {RESULT} = {expr};\n        Console.WriteLine({RESULT});\n    }}\n}}"
    )
}

fn render_cpp(program: &NumericProgram) -> String {
    let literal = program.literal();
    let ty = program.value_type.cpp();
    let string_include = if program.value_type == ValueType::Text {
        "#include <string>\n"
    } else {
        ""
    };
    if let Some(transform) = program.transform() {
        let action = match transform {
            Transform::SortAscending => "std::sort(numbers.begin(), numbers.end());",
            Transform::SortDescending => {
                "std::sort(numbers.begin(), numbers.end(), std::greater<>());"
            }
            Transform::Reverse => "std::reverse(numbers.begin(), numbers.end());",
        };
        return format!(
            "#include <algorithm>\n#include <iostream>\n{string_include}#include <vector>\n\nint main() {{\n    std::vector<{ty}> {NUMBERS} = {{{literal}}};\n    {action}\n    for (size_t i = 0; i < {NUMBERS}.size(); ++i) {{\n        if (i) std::cout << \", \";\n        std::cout << {NUMBERS}[i];\n    }}\n    std::cout << std::endl;\n    return 0;\n}}"
        );
    }
    let expr = match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => format!("std::accumulate({NUMBERS}.begin(), {NUMBERS}.end(), ({ty})0)"),
        Reduce::Product => format!(
            "std::accumulate({NUMBERS}.begin(), {NUMBERS}.end(), ({ty})1, std::multiplies<{ty}>())"
        ),
        Reduce::Minimum => format!("*std::min_element({NUMBERS}.begin(), {NUMBERS}.end())"),
        Reduce::Maximum => format!("*std::max_element({NUMBERS}.begin(), {NUMBERS}.end())"),
    };
    format!(
        "#include <algorithm>\n#include <iostream>\n#include <numeric>\n{string_include}#include <vector>\n\nint main() {{\n    std::vector<{ty}> {NUMBERS} = {{{literal}}};\n    {ty} {RESULT} = {expr};\n    std::cout << {RESULT} << std::endl;\n    return 0;\n}}"
    )
}

fn render_c(program: &NumericProgram) -> String {
    let literal = program.literal();
    let count = program.literals.len();
    let ty = program.value_type.c_storage();
    let element_size_ty = if program.value_type == ValueType::Text {
        "char *"
    } else {
        ty
    };
    let fmt = program.value_type.c_printf();
    if let Some(transform) = program.transform() {
        let body = match transform {
            Transform::Reverse => format!(
                "    {ty} {NUMBERS}[] = {{{literal}}};\n    size_t count = {count};\n    for (size_t i = 0, j = count - 1; i < j; ++i, --j) {{\n        {ty} tmp = {NUMBERS}[i];\n        {NUMBERS}[i] = {NUMBERS}[j];\n        {NUMBERS}[j] = tmp;\n    }}"
            ),
            Transform::SortAscending | Transform::SortDescending => format!(
                "    {ty} {NUMBERS}[] = {{{literal}}};\n    size_t count = {count};\n    qsort({NUMBERS}, count, sizeof({element_size_ty}), compare);"
            ),
        };
        let comparator = c_comparator(transform, program.value_type);
        let string_include = if program.value_type == ValueType::Text {
            "#include <string.h>\n"
        } else {
            ""
        };
        return format!(
            "#include <stdio.h>\n#include <stdlib.h>\n{string_include}\n{comparator}int main(void) {{\n{body}\n    for (size_t i = 0; i < count; ++i) {{\n        if (i) printf(\", \");\n        printf(\"{fmt}\", {NUMBERS}[i]);\n    }}\n    printf(\"\\n\");\n    return 0;\n}}"
        );
    }
    let init = match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => program.value_type.scalar_zero().to_owned(),
        Reduce::Product => program.value_type.scalar_one().to_owned(),
        Reduce::Minimum | Reduce::Maximum => format!("{NUMBERS}[0]"),
    };
    let step = match program.reduce().expect("reduction program has reducer") {
        Reduce::Sum => format!("        {RESULT} += {NUMBERS}[i];"),
        Reduce::Product => format!("        {RESULT} *= {NUMBERS}[i];"),
        Reduce::Minimum => format!("        if ({NUMBERS}[i] < {RESULT}) {RESULT} = {NUMBERS}[i];"),
        Reduce::Maximum => format!("        if ({NUMBERS}[i] > {RESULT}) {RESULT} = {NUMBERS}[i];"),
    };
    format!(
        "#include <stdio.h>\n\nint main(void) {{\n    {ty} {NUMBERS}[] = {{{literal}}};\n    size_t count = {count};\n    {ty} {RESULT} = {init};\n    for (size_t i = 0; i < count; ++i) {{\n{step}\n    }}\n    printf(\"{fmt}\\n\", {RESULT});\n    return 0;\n}}"
    )
}

fn c_comparator(transform: Transform, value_type: ValueType) -> String {
    match transform {
        Transform::Reverse => String::new(),
        Transform::SortAscending | Transform::SortDescending => {
            let cmp_body = match (transform, value_type) {
                (Transform::SortDescending, ValueType::Float) => {
                    "    double diff = *(const double *)b - *(const double *)a;\n    return (diff > 0) - (diff < 0);"
                }
                (_, ValueType::Float) => {
                    "    double diff = *(const double *)a - *(const double *)b;\n    return (diff > 0) - (diff < 0);"
                }
                (Transform::SortDescending, ValueType::Integer) => {
                    "    return (*(const int *)b - *(const int *)a);"
                }
                (_, ValueType::Integer) => "    return (*(const int *)a - *(const int *)b);",
                (Transform::SortDescending, ValueType::Text) => {
                    "    const char *left = *(const char * const *)a;\n    const char *right = *(const char * const *)b;\n    return strcmp(right, left);"
                }
                (_, ValueType::Text) => {
                    "    const char *left = *(const char * const *)a;\n    const char *right = *(const char * const *)b;\n    return strcmp(left, right);"
                }
            };
            format!("static int compare(const void *a, const void *b) {{\n{cmp_body}\n}}\n\n")
        }
    }
}
