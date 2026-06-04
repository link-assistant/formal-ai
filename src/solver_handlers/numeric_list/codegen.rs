//! Per-language code generation for the numeric-list operations.
//!
//! Two output shapes cover the whole family:
//!
//! * **List transformations** (`sort` ascending/descending, `reverse`) build the
//!   array, transform it, and print the elements comma-separated.
//! * **Scalar reductions** (`sum`, `product`, `minimum`, `maximum`) build the
//!   array, fold it, and print the single value.
//!
//! The generated programs print exactly what the solver computes, so the
//! `examples/numeric_list_execution.rs` harness can compile and run each
//! `(operation, language)` pair and assert the runtime stdout equals the
//! solver's deterministic result. The `sort` / `reverse_sort` snippets are kept
//! byte-identical to the original issue #395 handler.

use crate::coding::catalog::ProgramLanguage;

use super::{Operation, ParsedNumber, Reduce, Transform};

/// Generate runnable code for `operation` over `numbers` in `language`.
pub fn generate(
    language: &ProgramLanguage,
    numbers: &[ParsedNumber],
    operation: Operation,
    is_float: bool,
) -> String {
    match operation {
        Operation::Transform(transform) => transform_code(language, numbers, transform, is_float),
        Operation::Reduce(reduce) => reduce_code(language, numbers, reduce, is_float),
    }
}

/// Render the array literal: each number's surface text, comma-separated. When
/// the list mixes integers and decimals (`is_float`), integer surfaces gain a
/// `.0` suffix so statically-typed targets keep a single element type.
fn number_literals(numbers: &[ParsedNumber], is_float: bool) -> String {
    numbers
        .iter()
        .map(|n| {
            if is_float && !n.text.contains('.') {
                format!("{}.0", n.text)
            } else {
                n.text.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// List transformations
// ---------------------------------------------------------------------------

fn transform_code(
    language: &ProgramLanguage,
    numbers: &[ParsedNumber],
    transform: Transform,
    is_float: bool,
) -> String {
    let literal = number_literals(numbers, is_float);
    match language.slug {
        "javascript" => js_transform(&literal, transform, false),
        "typescript" => js_transform(&literal, transform, true),
        "rust" => rust_transform(&literal, transform, is_float),
        "go" => go_transform(&literal, transform, is_float),
        "ruby" => ruby_transform(&literal, transform),
        "java" => java_transform(&literal, transform, is_float),
        "csharp" => csharp_transform(&literal, transform, is_float),
        "c" => c_transform(numbers, transform, is_float),
        "cpp" => cpp_transform(&literal, transform, is_float),
        _ => python_transform(&literal, transform),
    }
}

fn js_transform(literal: &str, transform: Transform, typed: bool) -> String {
    let expr = match transform {
        Transform::SortAscending => "[...numbers].sort((a, b) => a - b)",
        Transform::SortDescending => "[...numbers].sort((a, b) => b - a)",
        Transform::Reverse => "[...numbers].reverse()",
    };
    let decl = if typed {
        format!("const numbers: number[] = [{literal}];")
    } else {
        format!("const numbers = [{literal}];")
    };
    format!("{decl}\nconst sorted = {expr};\nconsole.log(sorted.join(\", \"));")
}

fn python_transform(literal: &str, transform: Transform) -> String {
    let action = match transform {
        Transform::SortAscending => "sorted(numbers)",
        Transform::SortDescending => "sorted(numbers, reverse=True)",
        Transform::Reverse => "list(reversed(numbers))",
    };
    format!(
        "numbers = [{literal}]\nsorted_numbers = {action}\nprint(\", \".join(str(n) for n in sorted_numbers))"
    )
}

fn rust_transform(literal: &str, transform: Transform, is_float: bool) -> String {
    let action = match (transform, is_float) {
        (Transform::SortAscending, false) => "numbers.sort();".to_owned(),
        (Transform::SortDescending, false) => "numbers.sort_by(|a, b| b.cmp(a));".to_owned(),
        (Transform::SortAscending, true) => {
            "numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());".to_owned()
        }
        (Transform::SortDescending, true) => {
            "numbers.sort_by(|a, b| b.partial_cmp(a).unwrap());".to_owned()
        }
        (Transform::Reverse, _) => "numbers.reverse();".to_owned(),
    };
    let ty = if is_float { "f64" } else { "i64" };
    format!(
        "fn main() {{\n    let mut numbers: Vec<{ty}> = vec![{literal}];\n    {action}\n    let rendered: Vec<String> = numbers.iter().map(|n| n.to_string()).collect();\n    println!(\"{{}}\", rendered.join(\", \"));\n}}"
    )
}

fn go_transform(literal: &str, transform: Transform, is_float: bool) -> String {
    let (ty, format_item) = if is_float {
        ("float64", "strconv.FormatFloat(n, 'g', -1, 64)")
    } else {
        ("int", "strconv.Itoa(n)")
    };
    let action = match transform {
        Transform::SortAscending => {
            "sort.Slice(numbers, func(i, j int) bool { return numbers[i] < numbers[j] })".to_owned()
        }
        Transform::SortDescending => {
            "sort.Slice(numbers, func(i, j int) bool { return numbers[i] > numbers[j] })".to_owned()
        }
        Transform::Reverse => {
            "for i, j := 0, len(numbers)-1; i < j; i, j = i+1, j-1 {\n\t\tnumbers[i], numbers[j] = numbers[j], numbers[i]\n\t}".to_owned()
        }
    };
    format!(
        "package main\n\nimport (\n\t\"fmt\"\n\t\"sort\"\n\t\"strconv\"\n\t\"strings\"\n)\n\nfunc main() {{\n\tnumbers := []{ty}{{{literal}}}\n\t{action}\n\tparts := make([]string, len(numbers))\n\tfor i, n := range numbers {{\n\t\tparts[i] = {format_item}\n\t}}\n\tfmt.Println(strings.Join(parts, \", \"))\n}}"
    )
}

fn ruby_transform(literal: &str, transform: Transform) -> String {
    let action = match transform {
        Transform::SortAscending => "numbers.sort",
        Transform::SortDescending => "numbers.sort.reverse",
        Transform::Reverse => "numbers.reverse",
    };
    format!("numbers = [{literal}]\nsorted = {action}\nputs sorted.join(\", \")")
}

fn java_transform(literal: &str, transform: Transform, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let boxed = if is_float { "Double" } else { "Integer" };
    let action = match transform {
        Transform::SortAscending => "Arrays.sort(numbers);".to_owned(),
        Transform::SortDescending => format!(
            "{boxed}[] boxed = Arrays.stream(numbers).boxed().toArray({boxed}[]::new);\n        Arrays.sort(boxed, Collections.reverseOrder());\n        for (int i = 0; i < numbers.length; i++) numbers[i] = boxed[i];"
        ),
        Transform::Reverse => {
            format!("for (int i = 0, j = numbers.length - 1; i < j; i++, j--) {{\n            {ty} tmp = numbers[i];\n            numbers[i] = numbers[j];\n            numbers[j] = tmp;\n        }}")
        }
    };
    format!(
        "import java.util.Arrays;\nimport java.util.Collections;\nimport java.util.StringJoiner;\n\npublic class Main {{\n    public static void main(String[] args) {{\n        {ty}[] numbers = {{{literal}}};\n        {action}\n        StringJoiner joiner = new StringJoiner(\", \");\n        for ({ty} n : numbers) joiner.add(String.valueOf(n));\n        System.out.println(joiner.toString());\n    }}\n}}"
    )
}

fn csharp_transform(literal: &str, transform: Transform, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let action = match transform {
        Transform::SortAscending => "numbers.OrderBy(n => n)",
        Transform::SortDescending => "numbers.OrderByDescending(n => n)",
        Transform::Reverse => "numbers.Reverse()",
    };
    format!(
        "using System;\nusing System.Linq;\n\nclass Program {{\n    static void Main() {{\n        {ty}[] numbers = {{{literal}}};\n        var sorted = {action};\n        Console.WriteLine(string.Join(\", \", sorted));\n    }}\n}}"
    )
}

fn cpp_transform(literal: &str, transform: Transform, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let action = match transform {
        Transform::SortAscending => "std::sort(numbers.begin(), numbers.end());",
        Transform::SortDescending => "std::sort(numbers.begin(), numbers.end(), std::greater<>());",
        Transform::Reverse => "std::reverse(numbers.begin(), numbers.end());",
    };
    format!(
        "#include <algorithm>\n#include <iostream>\n#include <vector>\n\nint main() {{\n    std::vector<{ty}> numbers = {{{literal}}};\n    {action}\n    for (size_t i = 0; i < numbers.size(); ++i) {{\n        if (i) std::cout << \", \";\n        std::cout << numbers[i];\n    }}\n    std::cout << std::endl;\n    return 0;\n}}"
    )
}

fn c_transform(numbers: &[ParsedNumber], transform: Transform, is_float: bool) -> String {
    let literal = number_literals(numbers, is_float);
    let count = numbers.len();
    let (ty, fmt) = if is_float {
        ("double", "%g")
    } else {
        ("int", "%d")
    };
    // `reverse` swaps in place; the sort variants drive `qsort` with a
    // comparator. Generate whichever body the operation needs.
    let body = match transform {
        Transform::Reverse => format!(
            "    {ty} numbers[] = {{{literal}}};\n    size_t count = {count};\n    for (size_t i = 0, j = count - 1; i < j; ++i, --j) {{\n        {ty} tmp = numbers[i];\n        numbers[i] = numbers[j];\n        numbers[j] = tmp;\n    }}"
        ),
        Transform::SortAscending | Transform::SortDescending => format!(
            "    {ty} numbers[] = {{{literal}}};\n    size_t count = {count};\n    qsort(numbers, count, sizeof({ty}), compare);"
        ),
    };
    let comparator = match transform {
        Transform::Reverse => String::new(),
        Transform::SortAscending | Transform::SortDescending => {
            let cmp_body = if is_float {
                match transform {
                    Transform::SortDescending => "    double diff = *(const double *)b - *(const double *)a;\n    return (diff > 0) - (diff < 0);",
                    _ => "    double diff = *(const double *)a - *(const double *)b;\n    return (diff > 0) - (diff < 0);",
                }
            } else {
                match transform {
                    Transform::SortDescending => "    return (*(const int *)b - *(const int *)a);",
                    _ => "    return (*(const int *)a - *(const int *)b);",
                }
            };
            format!("static int compare(const void *a, const void *b) {{\n{cmp_body}\n}}\n\n")
        }
    };
    format!(
        "#include <stdio.h>\n#include <stdlib.h>\n\n{comparator}int main(void) {{\n{body}\n    for (size_t i = 0; i < count; ++i) {{\n        if (i) printf(\", \");\n        printf(\"{fmt}\", numbers[i]);\n    }}\n    printf(\"\\n\");\n    return 0;\n}}"
    )
}

// ---------------------------------------------------------------------------
// Scalar reductions
// ---------------------------------------------------------------------------

fn reduce_code(
    language: &ProgramLanguage,
    numbers: &[ParsedNumber],
    reduce: Reduce,
    is_float: bool,
) -> String {
    let literal = number_literals(numbers, is_float);
    match language.slug {
        "javascript" => js_reduce(&literal, reduce, false),
        "typescript" => js_reduce(&literal, reduce, true),
        "rust" => rust_reduce(&literal, reduce, is_float),
        "go" => go_reduce(&literal, reduce, is_float),
        "ruby" => ruby_reduce(&literal, reduce),
        "java" => java_reduce(&literal, reduce, is_float),
        "csharp" => csharp_reduce(&literal, reduce, is_float),
        "c" => c_reduce(numbers, reduce, is_float),
        "cpp" => cpp_reduce(&literal, reduce, is_float),
        _ => python_reduce(&literal, reduce),
    }
}

fn js_reduce(literal: &str, reduce: Reduce, typed: bool) -> String {
    let expr = match reduce {
        Reduce::Sum => "numbers.reduce((a, b) => a + b, 0)",
        Reduce::Product => "numbers.reduce((a, b) => a * b, 1)",
        Reduce::Minimum => "Math.min(...numbers)",
        Reduce::Maximum => "Math.max(...numbers)",
    };
    let decl = if typed {
        format!("const numbers: number[] = [{literal}];")
    } else {
        format!("const numbers = [{literal}];")
    };
    format!("{decl}\nconst result = {expr};\nconsole.log(result);")
}

fn python_reduce(literal: &str, reduce: Reduce) -> String {
    match reduce {
        Reduce::Sum => {
            format!("numbers = [{literal}]\nresult = sum(numbers)\nprint(result)")
        }
        Reduce::Product => format!(
            "import math\n\nnumbers = [{literal}]\nresult = math.prod(numbers)\nprint(result)"
        ),
        Reduce::Minimum => {
            format!("numbers = [{literal}]\nresult = min(numbers)\nprint(result)")
        }
        Reduce::Maximum => {
            format!("numbers = [{literal}]\nresult = max(numbers)\nprint(result)")
        }
    }
}

fn rust_reduce(literal: &str, reduce: Reduce, is_float: bool) -> String {
    let ty = if is_float { "f64" } else { "i64" };
    let compute = match (reduce, is_float) {
        (Reduce::Sum, _) => format!("numbers.iter().copied().sum::<{ty}>()"),
        (Reduce::Product, _) => format!("numbers.iter().copied().product::<{ty}>()"),
        (Reduce::Minimum, false) => "*numbers.iter().min().unwrap()".to_owned(),
        (Reduce::Maximum, false) => "*numbers.iter().max().unwrap()".to_owned(),
        (Reduce::Minimum, true) => {
            "numbers.iter().copied().fold(f64::INFINITY, f64::min)".to_owned()
        }
        (Reduce::Maximum, true) => {
            "numbers.iter().copied().fold(f64::NEG_INFINITY, f64::max)".to_owned()
        }
    };
    format!(
        "fn main() {{\n    let numbers: Vec<{ty}> = vec![{literal}];\n    let result = {compute};\n    println!(\"{{}}\", result);\n}}"
    )
}

fn go_reduce(literal: &str, reduce: Reduce, is_float: bool) -> String {
    let ty = if is_float { "float64" } else { "int" };
    let body = match reduce {
        Reduce::Sum => format!(
            "var result {ty} = 0\n\tfor _, n := range numbers {{\n\t\tresult += n\n\t}}"
        ),
        Reduce::Product => format!(
            "var result {ty} = 1\n\tfor _, n := range numbers {{\n\t\tresult *= n\n\t}}"
        ),
        Reduce::Minimum => {
            "result := numbers[0]\n\tfor _, n := range numbers[1:] {\n\t\tif n < result {\n\t\t\tresult = n\n\t\t}\n\t}".to_owned()
        }
        Reduce::Maximum => {
            "result := numbers[0]\n\tfor _, n := range numbers[1:] {\n\t\tif n > result {\n\t\t\tresult = n\n\t\t}\n\t}".to_owned()
        }
    };
    format!(
        "package main\n\nimport \"fmt\"\n\nfunc main() {{\n\tnumbers := []{ty}{{{literal}}}\n\t{body}\n\tfmt.Println(result)\n}}"
    )
}

fn ruby_reduce(literal: &str, reduce: Reduce) -> String {
    let expr = match reduce {
        Reduce::Sum => "numbers.sum",
        Reduce::Product => "numbers.inject(1, :*)",
        Reduce::Minimum => "numbers.min",
        Reduce::Maximum => "numbers.max",
    };
    format!("numbers = [{literal}]\nresult = {expr}\nputs result")
}

fn java_reduce(literal: &str, reduce: Reduce, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let body = match reduce {
        Reduce::Sum => format!(
            "{ty} result = 0;\n        for ({ty} n : numbers) result += n;"
        ),
        Reduce::Product => format!(
            "{ty} result = 1;\n        for ({ty} n : numbers) result *= n;"
        ),
        Reduce::Minimum => format!(
            "{ty} result = numbers[0];\n        for ({ty} n : numbers) result = Math.min(result, n);"
        ),
        Reduce::Maximum => format!(
            "{ty} result = numbers[0];\n        for ({ty} n : numbers) result = Math.max(result, n);"
        ),
    };
    format!(
        "public class Main {{\n    public static void main(String[] args) {{\n        {ty}[] numbers = {{{literal}}};\n        {body}\n        System.out.println(result);\n    }}\n}}"
    )
}

fn csharp_reduce(literal: &str, reduce: Reduce, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let expr = match reduce {
        Reduce::Sum => "numbers.Sum()".to_owned(),
        Reduce::Product => format!("numbers.Aggregate(({ty})1, (a, b) => a * b)"),
        Reduce::Minimum => "numbers.Min()".to_owned(),
        Reduce::Maximum => "numbers.Max()".to_owned(),
    };
    format!(
        "using System;\nusing System.Linq;\n\nclass Program {{\n    static void Main() {{\n        {ty}[] numbers = {{{literal}}};\n        var result = {expr};\n        Console.WriteLine(result);\n    }}\n}}"
    )
}

fn cpp_reduce(literal: &str, reduce: Reduce, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let expr = match reduce {
        Reduce::Sum => format!("std::accumulate(numbers.begin(), numbers.end(), ({ty})0)"),
        Reduce::Product => format!(
            "std::accumulate(numbers.begin(), numbers.end(), ({ty})1, std::multiplies<{ty}>())"
        ),
        Reduce::Minimum => "*std::min_element(numbers.begin(), numbers.end())".to_owned(),
        Reduce::Maximum => "*std::max_element(numbers.begin(), numbers.end())".to_owned(),
    };
    format!(
        "#include <algorithm>\n#include <iostream>\n#include <numeric>\n#include <vector>\n\nint main() {{\n    std::vector<{ty}> numbers = {{{literal}}};\n    {ty} result = {expr};\n    std::cout << result << std::endl;\n    return 0;\n}}"
    )
}

fn c_reduce(numbers: &[ParsedNumber], reduce: Reduce, is_float: bool) -> String {
    let literal = number_literals(numbers, is_float);
    let count = numbers.len();
    let (ty, fmt) = if is_float {
        ("double", "%g")
    } else {
        ("int", "%d")
    };
    let init = match reduce {
        Reduce::Sum => "0".to_owned(),
        Reduce::Product => "1".to_owned(),
        Reduce::Minimum | Reduce::Maximum => "numbers[0]".to_owned(),
    };
    let step = match reduce {
        Reduce::Sum => "        result += numbers[i];".to_owned(),
        Reduce::Product => "        result *= numbers[i];".to_owned(),
        Reduce::Minimum => "        if (numbers[i] < result) result = numbers[i];".to_owned(),
        Reduce::Maximum => "        if (numbers[i] > result) result = numbers[i];".to_owned(),
    };
    format!(
        "#include <stdio.h>\n\nint main(void) {{\n    {ty} numbers[] = {{{literal}}};\n    size_t count = {count};\n    {ty} result = {init};\n    for (size_t i = 0; i < count; ++i) {{\n{step}\n    }}\n    printf(\"{fmt}\\n\", result);\n    return 0;\n}}"
    )
}
