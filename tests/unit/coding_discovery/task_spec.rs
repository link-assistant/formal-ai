//! Issue #710: one structural coding-task specification for benchmark and
//! conversational prompt shapes.

use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Example, Parameter, recognise};

fn parameter(name: &str, annotation: Option<&str>) -> Parameter {
    Parameter {
        name: name.to_owned(),
        annotation: annotation.map(str::to_owned),
    }
}

fn example(arguments: &[&str], expected: &str) -> Example {
    Example {
        arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
        expected: expected.to_owned(),
    }
}

#[test]
fn humaneval_signature_docstring_and_examples_form_one_spec() {
    let prompt = r#"Complete this Python function. Reply with the full implementation in a code block.

from typing import List, Tuple

def sum_product(numbers: List[int]) -> Tuple[int, int]:
    """Return a tuple consisting of a sum and a product of all the integers in a list.
    >>> sum_product([])
    (0, 1)
    >>> sum_product([1, 2, 3, 4])
    (10, 24)
    """
"#;

    let spec = recognise(prompt).expect("a Python definition with a docstring is a coding task");
    assert_eq!(
        spec,
        CodingTaskSpec {
            language: "python".to_owned(),
            artifact_shape: ArtifactShape::Function,
            name: "sum_product".to_owned(),
            parameters: vec![parameter("numbers", Some("List[int]"))],
            return_annotation: Some("Tuple[int, int]".to_owned()),
            imports: vec!["from typing import List, Tuple".to_owned()],
            requirement_sentences: vec![
                "Return a tuple consisting of a sum and a product of all the integers in a list."
                    .to_owned(),
            ],
            examples: vec![
                example(&["[]"], "(0, 1)"),
                example(&["[1, 2, 3, 4]"], "(10, 24)"),
            ],
            expected_stdout: None,
            prose_language: "en".to_owned(),
        }
    );
    assert_eq!(
        spec.to_links_notation(),
        r#"coding_task_spec
  language "python"
  artifact_shape "function"
  name "sum_product"
  parameter "numbers"
    annotation "List[int]"
  return_annotation "Tuple[int, int]"
  import "from typing import List, Tuple"
  requirement "Return a tuple consisting of a sum and a product of all the integers in a list."
  example
    argument "[]"
    expected "(0, 1)"
  example
    argument "[1, 2, 3, 4]"
    expected "(10, 24)"
  prose_language "en""#
    );
}

#[test]
fn mbpp_assertions_supply_the_name_arity_and_examples_without_becoming_a_signature() {
    let prompt = r"Write a function to find the similar elements from the given two tuple lists.
Reply with the Python code in a code block. It must pass these tests:
assert similar_elements((3, 4, 5, 6), (5, 7, 4, 10)) == (4, 5)
assert similar_elements((1, 2), (3, 4)) == ()";

    let spec = recognise(prompt).expect("task prose followed by assertions is a coding task");
    assert_eq!(spec.language, "python");
    assert_eq!(spec.name, "similar_elements");
    assert_eq!(
        spec.parameters,
        vec![parameter("arg1", None), parameter("arg2", None)]
    );
    assert_eq!(spec.return_annotation, None);
    assert_eq!(spec.imports, Vec::<String>::new());
    assert_eq!(
        spec.requirement_sentences,
        vec!["Write a function to find the similar elements from the given two tuple lists."]
    );
    assert_eq!(
        spec.examples,
        vec![
            example(&["(3, 4, 5, 6)", "(5, 7, 4, 10)"], "(4, 5)"),
            example(&["(1, 2)", "(3, 4)"], "()"),
        ]
    );
}

#[test]
fn conversational_requests_in_five_languages_share_the_same_signature_shape() {
    let cases = [
        (
            "en",
            "Write a Python function `greatest_common_divisor(a, b)` that returns the greatest common divisor of two integers.",
        ),
        (
            "ru",
            "Напиши функцию Python `greatest_common_divisor(a, b)`, которая возвращает наибольший общий делитель двух целых чисел.",
        ),
        (
            "hi",
            "एक Python फ़ंक्शन `greatest_common_divisor(a, b)` लिखो जो दो पूर्णांकों का महत्तम समापवर्तक लौटाए।",
        ),
        (
            "zh",
            "写一个 Python 函数 `greatest_common_divisor(a, b)`，返回两个整数的最大公约数。",
        ),
        (
            "es",
            "Escribe una función Python `greatest_common_divisor(a, b)` que devuelva el máximo común divisor de dos enteros.",
        ),
    ];

    for (language, prompt) in cases {
        let spec =
            recognise(prompt).unwrap_or_else(|| panic!("{language} request was not recognised"));
        assert_eq!(spec.language, "python", "{language}");
        assert_eq!(spec.name, "greatest_common_divisor", "{language}");
        assert_eq!(
            spec.parameters,
            vec![parameter("a", None), parameter("b", None)],
            "{language}"
        );
        assert_eq!(spec.prose_language, language, "{language}");
        assert_eq!(spec.signature_identity(), "python(a,b)->_", "{language}");
    }
}

#[test]
fn conversational_signature_is_found_after_a_native_language_function_word() {
    let prompt = "Реализуй Python функцию count_letters(text: str) -> int. Верни количество букв.";
    let spec = recognise(prompt).expect("native-language request with inline signature");
    assert_eq!(spec.name, "count_letters");
    assert_eq!(spec.parameters, vec![parameter("text", Some("str"))]);
    assert_eq!(spec.prose_language, "ru");
}

#[test]
fn conversational_program_without_a_named_callable_uses_the_main_entry_point() {
    let spec = recognise("Write a Python program to count to 100 inclusive.")
        .expect("program request without a signature");
    assert_eq!(spec.artifact_shape, ArtifactShape::Program);
    assert_eq!(spec.name, "main");
    assert!(spec.parameters.is_empty());
}

#[test]
fn program_stdout_is_bound_through_multilingual_output_slots() {
    for (language, prompt) in [
        ("en", "Write a program in Python that prints Alpha, beta!"),
        (
            "ru",
            "напиши программу на Python, которая выводит Alpha, beta!",
        ),
        ("hi", "Python में एक प्रोग्राम लिखें जो Alpha, beta! प्रिंट करता है"),
        ("zh", "写一个打印 Alpha, beta! 的 Python 程序"),
    ] {
        let spec = recognise(prompt).unwrap_or_else(|| panic!("{language} program not recognized"));
        assert_eq!(spec.artifact_shape, ArtifactShape::Program, "{language}");
        assert_eq!(
            spec.expected_stdout.as_deref(),
            Some("Alpha, beta!"),
            "{language}"
        );
    }
}

#[test]
fn non_coding_questions_do_not_produce_a_task_spec() {
    assert_eq!(recognise("What is a function?"), None);
    assert_eq!(
        recognise("What is the value of the function f(x)=2x at 3?"),
        None
    );
    assert_eq!(
        recognise(
            "Convert this README installation guide into a sh script:\n\n```markdown\nRun the generated Python program.\n`python3 main.py`\n```"
        ),
        None
    );
}
