//! Parametric "sort these numbers in <language>, give me the code and the
//! result" coding task (issue #395).
//!
//! Unlike the static [`crate::coding::catalog`] tasks (whose output is fixed),
//! this handler reads the *given* numbers straight from the prompt, reasons
//! about the requested order, generates idiomatic code in the requested
//! programming language, and — because sorting is a pure, decidable function —
//! computes the sorted result deterministically in the solver itself. No
//! external runtime is needed: the result the user sees is the verified output
//! of the same comparison the generated code performs.
//!
//! Recognition is seed-driven, not hardcoded per language: the sort verb comes
//! from the `sort` / `reverse_sort` operations in
//! `data/seed/operation-vocabulary.lino` (en/ru/hi/zh), and the target language
//! from the `program_language_<slug>` alias meanings (issue #386). The handler
//! only fires when all three signals are present — a sort verb, at least two
//! numbers, and a known programming language — so it never steals plain prose.

use std::cmp::Ordering;
use std::fmt::Write as _;

use crate::coding::catalog::ProgramLanguage;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};

use super::finalize_simple;

/// Ascending or descending, decided by whether the prompt also evidences the
/// `reverse_sort` operation (the descending phrasings already shipped for the
/// cancel-sort work in issue #386).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// A single number lifted from the prompt: the surface text the user typed
/// (preserved verbatim for echoing back and for code generation) plus the
/// numeric value used to order it.
#[derive(Debug, Clone)]
struct ParsedNumber {
    text: String,
    value: f64,
}

/// The fully-reasoned solution: the given numbers, the order, the resolved
/// language, the generated code, and the deterministically-computed result.
#[derive(Debug, Clone)]
pub struct SortNumbersSolution {
    pub language_slug: &'static str,
    pub language_name: &'static str,
    pub code_fence: &'static str,
    pub order: SortOrder,
    pub given: Vec<String>,
    pub sorted: Vec<String>,
    pub code: String,
}

/// Specialized-handler entry point. Returns `Some` only when the prompt is a
/// concrete "sort these numbers in <language>" request.
pub fn try_sort_numbers(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let solution = solve_sort_numbers(prompt)?;

    log.append("formalize", solution.formalization());
    log.append(
        "synthesis:spec",
        format!(
            "language={} task=sort_numbers order={}",
            solution.language_slug,
            solution.order.slug()
        ),
    );
    log.append("synthesis:given", solution.given.join(", "));
    log.append("composition:code_fragment", solution.code.clone());
    // The result is computed by the solver, not a sandbox: sorting is a pure,
    // total function over the parsed values, so the answer is verified by
    // construction rather than by running untrusted code.
    log.append("execution_status", "computed deterministically".to_owned());
    log.append(
        "execution_environment",
        "pure in-solver evaluation of a decidable comparison sort".to_owned(),
    );
    log.append("execution_result", solution.sorted.join(", "));

    let language = detect_language(prompt);
    let body = solution.render(language);
    Some(finalize_simple(
        prompt,
        log,
        "write_program",
        &format!(
            "response:write_program:sort_numbers:{}",
            solution.language_slug
        ),
        &body,
        1.0,
    ))
}

/// Pure core: parse the request and produce a [`SortNumbersSolution`], or
/// `None` when the prompt is not a sort-numbers coding task.
///
/// The prompt is re-normalized here with [`crate::web_engine_core::normalize_prompt`]
/// rather than trusting the dispatch's plain `to_lowercase()`: punctuation must
/// fold to whitespace so a language word glued to a comma (`JavaScript,`) still
/// resolves through the token-boundary alias matcher.
#[must_use]
pub fn solve_sort_numbers(prompt: &str) -> Option<SortNumbersSolution> {
    let normalized = crate::web_engine_core::normalize_prompt(prompt);
    let normalized = normalized.as_str();
    let vocabulary = crate::seed::operation_vocabulary();
    let wants_sort =
        vocabulary.matches("sort", normalized) || vocabulary.matches("reverse_sort", normalized);
    if !wants_sort {
        return None;
    }

    let language = crate::coding::program_language_by_alias(normalized)?;
    let numbers = parse_numbers(prompt);
    if numbers.len() < 2 {
        return None;
    }

    let order = if vocabulary.matches("reverse_sort", normalized) {
        SortOrder::Descending
    } else {
        SortOrder::Ascending
    };

    let mut ordered = numbers.clone();
    ordered.sort_by(|a, b| {
        let cmp = a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal);
        match order {
            SortOrder::Ascending => cmp,
            SortOrder::Descending => cmp.reverse(),
        }
    });

    let is_float = numbers.iter().any(|n| n.value.fract() != 0.0);
    let given: Vec<String> = numbers.iter().map(|n| n.text.clone()).collect();
    let sorted: Vec<String> = ordered.iter().map(|n| n.text.clone()).collect();
    let code = generate_code(language, &numbers, order, is_float);

    Some(SortNumbersSolution {
        language_slug: language.slug,
        language_name: language.name,
        code_fence: language.code_fence,
        order,
        given,
        sorted,
        code,
    })
}

impl SortOrder {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }
}

impl SortNumbersSolution {
    fn formalization(&self) -> String {
        format!(
            "(@USER OP:sort numbers:[{}] language:{} order:{} request:[code result])",
            self.given.join(" "),
            self.language_slug,
            self.order.slug()
        )
    }

    /// Render the localized answer: a sentence naming the language, the given
    /// numbers, and the order; the generated code; and the computed result.
    #[must_use]
    pub fn render(&self, language: Language) -> String {
        let given = self.given.join(", ");
        let sorted = self.sorted.join(", ");
        let parts = Localization::for_language(language);
        let order_word = parts.order_word(self.order);
        let mut body = String::new();
        let _ = write!(
            body,
            "{}\n\n```{}\n{}\n```\n\n{} {}",
            parts.intro(self.language_name, &given, order_word),
            self.code_fence,
            self.code,
            parts.result_label,
            sorted
        );
        body
    }
}

/// Localized phrasing for the four supported UI languages. The numbers, code,
/// and result are language-independent; only the surrounding prose differs.
struct Localization {
    result_label: &'static str,
    ascending: &'static str,
    descending: &'static str,
    intro: fn(&str, &str, &str) -> String,
}

impl Localization {
    fn for_language(language: Language) -> Self {
        match language {
            Language::Russian => Self {
                result_label: "Результат:",
                ascending: "по возрастанию",
                descending: "по убыванию",
                intro: |lang, given, order| {
                    format!("Вот код на {lang}, который сортирует числа {given} {order}:")
                },
            },
            Language::Hindi => Self {
                result_label: "परिणाम:",
                ascending: "आरोही क्रम में",
                descending: "अवरोही क्रम में",
                intro: |lang, given, order| {
                    format!("यह {lang} कोड है जो संख्याओं {given} को {order} क्रमबद्ध करता है:")
                },
            },
            Language::Chinese => Self {
                result_label: "结果:",
                ascending: "升序",
                descending: "降序",
                intro: |lang, given, order| {
                    format!("这是用 {lang} 编写的将数字 {given} 按{order}排序的代码:")
                },
            },
            _ => Self {
                result_label: "Result:",
                ascending: "in ascending order",
                descending: "in descending order",
                intro: |lang, given, order| {
                    format!("Here is {lang} code that sorts the numbers {given} {order}:")
                },
            },
        }
    }

    const fn order_word(&self, order: SortOrder) -> &'static str {
        match order {
            SortOrder::Ascending => self.ascending,
            SortOrder::Descending => self.descending,
        }
    }

    fn intro(&self, language_name: &str, given: &str, order_word: &str) -> String {
        (self.intro)(language_name, given, order_word)
    }
}

/// Extract every number token from the raw prompt, in order of appearance.
///
/// Recognizes optionally-signed integers and decimals (e.g. `-3`, `5`, `7.5`).
/// The surface text is preserved so the echo and the generated code use exactly
/// what the user typed; the parsed `f64` value drives the ordering.
fn parse_numbers(prompt: &str) -> Vec<ParsedNumber> {
    let chars: Vec<char> = prompt.chars().collect();
    let mut numbers = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        let sign = if (ch == '-' || ch == '+')
            && chars.get(index + 1).is_some_and(char::is_ascii_digit)
            && !preceded_by_digit_or_word(&chars, index)
        {
            let s = ch;
            index += 1;
            Some(s)
        } else if ch.is_ascii_digit() {
            None
        } else {
            index += 1;
            continue;
        };

        let start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }
        // Optional single decimal point followed by digits.
        if index < chars.len()
            && chars[index] == '.'
            && chars.get(index + 1).is_some_and(char::is_ascii_digit)
        {
            index += 1;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
        }

        if start == index {
            continue;
        }
        let mut text = String::new();
        if let Some(s) = sign {
            text.push(s);
        }
        text.extend(&chars[start..index]);
        if let Ok(value) = text.parse::<f64>() {
            numbers.push(ParsedNumber { text, value });
        }
    }
    numbers
}

/// A leading sign only introduces a negative/positive literal when it is not
/// glued to a preceding digit or letter (so `main2` or `a-b` do not spawn a
/// bogus number, but a standalone `-3` does).
fn preceded_by_digit_or_word(chars: &[char], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|prev| chars.get(prev))
        .is_some_and(|prev| prev.is_alphanumeric())
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

/// Generate runnable code that sorts the given numbers and prints the result as
/// a comma-separated line — matching the deterministic result the solver shows.
fn generate_code(
    language: &ProgramLanguage,
    numbers: &[ParsedNumber],
    order: SortOrder,
    is_float: bool,
) -> String {
    let literal = number_literals(numbers, is_float);
    match language.slug {
        "javascript" => js_code(&literal, order, false),
        "typescript" => js_code(&literal, order, true),
        "rust" => rust_code(&literal, order, is_float),
        "go" => go_code(&literal, order, is_float),
        "ruby" => ruby_code(&literal, order),
        "java" => java_code(&literal, order, is_float),
        "csharp" => csharp_code(&literal, order, is_float),
        "c" => c_code(numbers, order, is_float),
        "cpp" => cpp_code(&literal, order, is_float),
        // "python" and any future language fall back to the Python rendering.
        _ => python_code(&literal, order),
    }
}

fn js_code(literal: &str, order: SortOrder, typed: bool) -> String {
    let cmp = match order {
        SortOrder::Ascending => "a - b",
        SortOrder::Descending => "b - a",
    };
    let decl = if typed {
        format!("const numbers: number[] = [{literal}];")
    } else {
        format!("const numbers = [{literal}];")
    };
    format!(
        "{decl}\nconst sorted = [...numbers].sort((a, b) => {cmp});\nconsole.log(sorted.join(\", \"));"
    )
}

fn python_code(literal: &str, order: SortOrder) -> String {
    let reverse = match order {
        SortOrder::Ascending => "",
        SortOrder::Descending => ", reverse=True",
    };
    format!(
        "numbers = [{literal}]\nsorted_numbers = sorted(numbers{reverse})\nprint(\", \".join(str(n) for n in sorted_numbers))"
    )
}

fn rust_code(literal: &str, order: SortOrder, is_float: bool) -> String {
    let sort_call = match (order, is_float) {
        (SortOrder::Ascending, false) => "numbers.sort();".to_owned(),
        (SortOrder::Descending, false) => "numbers.sort_by(|a, b| b.cmp(a));".to_owned(),
        (SortOrder::Ascending, true) => {
            "numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());".to_owned()
        }
        (SortOrder::Descending, true) => {
            "numbers.sort_by(|a, b| b.partial_cmp(a).unwrap());".to_owned()
        }
    };
    let ty = if is_float { "f64" } else { "i64" };
    format!(
        "fn main() {{\n    let mut numbers: Vec<{ty}> = vec![{literal}];\n    {sort_call}\n    let rendered: Vec<String> = numbers.iter().map(|n| n.to_string()).collect();\n    println!(\"{{}}\", rendered.join(\", \"));\n}}"
    )
}

fn go_code(literal: &str, order: SortOrder, is_float: bool) -> String {
    let cmp = match order {
        SortOrder::Ascending => "numbers[i] < numbers[j]",
        SortOrder::Descending => "numbers[i] > numbers[j]",
    };
    let (ty, format_item) = if is_float {
        ("float64", "strconv.FormatFloat(n, 'g', -1, 64)")
    } else {
        ("int", "strconv.Itoa(n)")
    };
    format!(
        "package main\n\nimport (\n\t\"fmt\"\n\t\"sort\"\n\t\"strconv\"\n\t\"strings\"\n)\n\nfunc main() {{\n\tnumbers := []{ty}{{{literal}}}\n\tsort.Slice(numbers, func(i, j int) bool {{ return {cmp} }})\n\tparts := make([]string, len(numbers))\n\tfor i, n := range numbers {{\n\t\tparts[i] = {format_item}\n\t}}\n\tfmt.Println(strings.Join(parts, \", \"))\n}}"
    )
}

fn ruby_code(literal: &str, order: SortOrder) -> String {
    let sort_call = match order {
        SortOrder::Ascending => "numbers.sort",
        SortOrder::Descending => "numbers.sort.reverse",
    };
    format!("numbers = [{literal}]\nsorted = {sort_call}\nputs sorted.join(\", \")")
}

fn java_code(literal: &str, order: SortOrder, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let boxed = if is_float { "Double" } else { "Integer" };
    let sort_line = match order {
        SortOrder::Ascending => "Arrays.sort(numbers);".to_owned(),
        SortOrder::Descending => format!(
            "{boxed}[] boxed = Arrays.stream(numbers).boxed().toArray({boxed}[]::new);\n        Arrays.sort(boxed, Collections.reverseOrder());\n        for (int i = 0; i < numbers.length; i++) numbers[i] = boxed[i];"
        ),
    };
    format!(
        "import java.util.Arrays;\nimport java.util.Collections;\nimport java.util.StringJoiner;\n\npublic class Main {{\n    public static void main(String[] args) {{\n        {ty}[] numbers = {{{literal}}};\n        {sort_line}\n        StringJoiner joiner = new StringJoiner(\", \");\n        for ({ty} n : numbers) joiner.add(String.valueOf(n));\n        System.out.println(joiner.toString());\n    }}\n}}"
    )
}

fn csharp_code(literal: &str, order: SortOrder, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let order_call = match order {
        SortOrder::Ascending => "OrderBy(n => n)",
        SortOrder::Descending => "OrderByDescending(n => n)",
    };
    format!(
        "using System;\nusing System.Linq;\n\nclass Program {{\n    static void Main() {{\n        {ty}[] numbers = {{{literal}}};\n        var sorted = numbers.{order_call};\n        Console.WriteLine(string.Join(\", \", sorted));\n    }}\n}}"
    )
}

fn cpp_code(literal: &str, order: SortOrder, is_float: bool) -> String {
    let ty = if is_float { "double" } else { "int" };
    let comparator = match order {
        SortOrder::Ascending => "std::sort(numbers.begin(), numbers.end());",
        SortOrder::Descending => "std::sort(numbers.begin(), numbers.end(), std::greater<>());",
    };
    format!(
        "#include <algorithm>\n#include <iostream>\n#include <vector>\n\nint main() {{\n    std::vector<{ty}> numbers = {{{literal}}};\n    {comparator}\n    for (size_t i = 0; i < numbers.size(); ++i) {{\n        if (i) std::cout << \", \";\n        std::cout << numbers[i];\n    }}\n    std::cout << std::endl;\n    return 0;\n}}"
    )
}

fn c_code(numbers: &[ParsedNumber], order: SortOrder, is_float: bool) -> String {
    let literal = number_literals(numbers, is_float);
    let count = numbers.len();
    let (ty, fmt, cmp_body) = if is_float {
        (
            "double",
            "%g",
            match order {
                SortOrder::Ascending => "    double diff = *(const double *)a - *(const double *)b;\n    return (diff > 0) - (diff < 0);",
                SortOrder::Descending => "    double diff = *(const double *)b - *(const double *)a;\n    return (diff > 0) - (diff < 0);",
            },
        )
    } else {
        (
            "int",
            "%d",
            match order {
                SortOrder::Ascending => "    return (*(const int *)a - *(const int *)b);",
                SortOrder::Descending => "    return (*(const int *)b - *(const int *)a);",
            },
        )
    };
    format!(
        "#include <stdio.h>\n#include <stdlib.h>\n\nstatic int compare(const void *a, const void *b) {{\n{cmp_body}\n}}\n\nint main(void) {{\n    {ty} numbers[] = {{{literal}}};\n    size_t count = {count};\n    qsort(numbers, count, sizeof({ty}), compare);\n    for (size_t i = 0; i < count; ++i) {{\n        if (i) printf(\", \");\n        printf(\"{fmt}\", numbers[i]);\n    }}\n    printf(\"\\n\");\n    return 0;\n}}"
    )
}
