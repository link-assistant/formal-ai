//! PR #1188 dogfooding, T18: "Create hello.py that prints Hello, World! and
//! run it." names its output without quotes. The obligation graph binds that
//! utterance -- up to the seeded clause separator or the sentence mark that
//! ends it -- and never binds a description of a computation ("the sum of a
//! and b", "Fibonacci numbers up to 100", `FizzBuzz`) as text to print. The
//! JavaScript twin is pinned in rust/tests/web/pull-request-1188-dogfood.test.mjs.
//! The coding task specification (`coding/task_spec.rs`) binds a program's
//! stdout through the same reading.

use formal_ai::coding_task_spec::recognise;
use formal_ai::engine::ExecutionRecipe;
use formal_ai::event_log::EventLog;
use formal_ai::intent_formalization::bound_output_literals;
use formal_ai::program_contract;

const HELLO: &str = "# python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile hello.py\n\
# python3 hello.py\n# Emit the requested text followed by a newline.\nprint(\"Hello, World!\")\n";

fn recipe(prompt: &str) -> ExecutionRecipe {
    let mut log = EventLog::new();
    let answer = program_contract::answer(prompt, &mut log)
        .unwrap_or_else(|| panic!("{prompt} composed no program"));
    *answer
        .execution_recipe
        .unwrap_or_else(|| panic!("{prompt} carried no recipe"))
}

#[test]
fn an_unquoted_utterance_is_the_program_output() {
    for prompt in [
        "Create hello.py that prints Hello, World! and run it.",
        "Write a Python program hello.py that prints Hello, World! and run it.",
    ] {
        let recipe = recipe(prompt);
        assert_eq!(recipe.path, "hello.py", "{prompt}");
        assert_eq!(recipe.source, HELLO, "{prompt}");
    }
}

#[test]
fn the_utterance_ends_at_a_seeded_separator_or_its_sentence() {
    assert_eq!(
        bound_output_literals("write a program that prints Hello, World! and run it"),
        ["Hello, World!"]
    );
    assert_eq!(
        bound_output_literals(
            "Напиши программу на Python, которая выводит Привет, мир! и запусти её"
        ),
        ["Привет, мир!"]
    );
    assert_eq!(
        bound_output_literals("Write hello.py that prints Hi. Then print \"Bye\"."),
        ["Hi", "Bye"]
    );
}

#[test]
fn a_description_of_a_computation_is_never_text_to_print() {
    for prompt in [
        "Write a Python program that prints the sum of a and b and run it",
        "Write a program that prints Fibonacci numbers up to 100",
        "Write a program that prints FizzBuzz",
        "Write a program that prints FizzBuzz for 1 to 100",
        "Write a program that prints prime numbers below 50",
        "Print the greeting! Then \"Hello\"",
    ] {
        assert_eq!(bound_output_literals(prompt), [] as [String; 0], "{prompt}");
    }
}

/// Round 5, rung 5: the coding task specification binds a program's stdout
/// through the same T18 reading, so both bind the same output for one request.
#[test]
fn the_task_specification_binds_the_same_output() {
    for (prompt, expected) in [
        (
            "Write a Python program that prints Hello, World! and run it",
            "Hello, World!",
        ),
        (
            "Напиши программу на Python, которая выводит Привет, мир! и запусти её",
            "Привет, мир!",
        ),
        (
            "Write a Python program that prints Hi. Then print \"Bye\".",
            "Hi",
        ),
        (
            "Write a Python program that prints \"Hello, World!\" to stdout.",
            "Hello, World!",
        ),
    ] {
        let spec = recognise(prompt).unwrap_or_else(|| panic!("{prompt} not recognised"));
        assert_eq!(spec.expected_stdout.as_deref(), Some(expected), "{prompt}");
        assert_eq!(
            bound_output_literals(prompt).first().map(String::as_str),
            Some(expected),
            "{prompt}"
        );
    }
    for prompt in [
        "Write a Python program that prints the sum of a and b and run it",
        "Write a Python program that prints Fibonacci numbers up to 100",
        "Write a Python program that prints FizzBuzz",
        "Write a Python program that prints FizzBuzz for 1 to 100",
        "Write a Python program that prints prime numbers below 50",
        "Print the greeting! Then \"Hello\"",
    ] {
        assert_eq!(
            recognise(prompt).and_then(|spec| spec.expected_stdout),
            None,
            "{prompt}"
        );
    }
}
