//! PR #1188 dogfooding, round 5 rung 1: "write a program that prints Hello,
//! World! and run it" names no language, yet asks for a run the agent itself
//! performs. The program is written in the language the seed names for that
//! case (`unnamed_language` of data/meta/stdout-program-contracts.lino, with
//! its stated reason) instead of falling through to a web search. A request
//! that only asks for a program still asks which language (issue #906). The
//! JavaScript twin is pinned in rust/tests/web/pull-request-1188-dogfood.test.mjs.

use formal_ai::event_log::EventLog;
use formal_ai::intent_formalization::bound_output_literals;
use formal_ai::program_contract;

fn main_py(text: &str) -> String {
    format!(
        "# python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile main.py\n\
# python3 main.py\n# Emit the requested text followed by a newline.\nprint(\"{text}\")\n"
    )
}

#[test]
fn a_program_the_request_asks_to_run_is_written_in_the_seeded_language() {
    for (prompt, text) in [
        (
            "write a program that prints Hello, World! and run it",
            "Hello, World!",
        ),
        (
            "Напиши программу, которая выводит Привет, мир! и запусти её",
            "Привет, мир!",
        ),
        ("写一个打印 \"Ni hao\" 的程序并运行", "Ni hao"),
        (
            "एक प्रोग्राम लिखो जो \"Namaste\" प्रिंट करे और उसे चलाओ",
            "Namaste",
        ),
    ] {
        let mut log = EventLog::new();
        let answer = program_contract::answer(prompt, &mut log)
            .unwrap_or_else(|| panic!("{prompt} composed no program"));
        let recipe = answer
            .execution_recipe
            .unwrap_or_else(|| panic!("{prompt} carried no recipe"));
        assert_eq!(recipe.language, "python", "{prompt}");
        assert_eq!(recipe.path, "main.py", "{prompt}");
        assert_eq!(recipe.source, main_py(text), "{prompt}");
    }
}

#[test]
fn a_request_that_does_not_ask_to_run_the_program_gets_no_language_chosen() {
    for prompt in [
        "Write a program that prints Hello, World!",
        "Write a program that prints \"Hi\"",
    ] {
        let mut log = EventLog::new();
        assert!(
            program_contract::answer(prompt, &mut log).is_none(),
            "{prompt}"
        );
    }
}

/// Round 6: a verb-final language (the seeded `verb_final` flag of
/// data/seed/formal-targets.lino) states the output before its print verb.
#[test]
fn a_verb_final_language_binds_the_quoted_output_its_print_verb_follows() {
    assert_eq!(
        bound_output_literals("एक प्रोग्राम लिखो जो \"Namaste\" प्रिंट करे और उसे चलाओ"),
        ["Namaste"]
    );
    assert_eq!(
        bound_output_literals("Python में एक प्रोग्राम लिखें जो \"Alpha\" प्रिंट करता है"),
        ["Alpha"]
    );
    assert_eq!(
        bound_output_literals("\"notes.txt\" में \"x\" को \"y\" से बदलो"),
        [] as [String; 0]
    );
    assert_eq!(
        bound_output_literals("\"a.txt\" बनाओ और \"b\" प्रिंट करो"),
        ["b"]
    );
}
