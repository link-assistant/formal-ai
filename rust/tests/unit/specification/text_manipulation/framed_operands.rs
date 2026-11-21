use super::text_solver;

pub(super) fn apostrophes_in_english_prose_do_not_swallow_the_quoted_operands() {
    let solver = text_solver();
    let prompts = [
        "It doesn't matter, replace \"cat\" with \"dog\" in this text: \"cat naps\"",
        "I can't decide, replace \"cat\" with \"dog\" in this text: \"cat naps\"",
        "That isn't what I meant: replace \"cat\" with \"dog\" in this text: \"cat naps\"",
        "The user's request is to replace \"cat\" with \"dog\" in this text: \"cat naps\"",
    ];

    for prompt in prompts {
        let response = solver.solve(prompt);
        assert_eq!(
            response.answer, "dog naps",
            "an apostrophe in the prose must not consume the operands of {prompt:?} \
             (got intent {}, answer {:?})",
            response.intent, response.answer
        );
    }
}

pub(super) fn apostrophe_delimited_operands_are_recognized_in_every_supported_language() {
    let solver = text_solver();
    for (language, prompt) in [
        ("en", "replace 'cat' with 'dog' in this text: 'cat naps'"),
        ("ru", "Замени 'cat' на 'dog': 'cat naps'"),
        ("hi", "'cat' को 'dog' से बदलें: 'cat naps'"),
        ("zh", "替换 'cat' 为 'dog': 'cat naps'"),
    ] {
        let response = solver.solve(prompt);
        assert_eq!(
            response.answer, "dog naps",
            "{language}: apostrophes delimiting the operands must still parse \
             (got intent {}, answer {:?})",
            response.intent, response.answer
        );
    }
}

pub(super) fn chained_operations_run_in_the_order_the_request_states_them() {
    // R1017: the chain follows the request, not the vocabulary's declaration
    // order; a count still ends the chain wherever it is named.
    let solver = text_solver();
    let sorted_first = solver.solve("Sort words and then reverse words: 'b a c'");
    assert_eq!(sorted_first.intent, "text_manipulation");
    assert_eq!(sorted_first.answer, "c b a");
    let reversed_first = solver.solve("Reverse words and then sort words: 'b a c'");
    assert_eq!(reversed_first.answer, "a b c");
    let reworded = solver.solve("Reverse the order of words: 'green CI release'");
    assert_eq!(reworded.answer, "release CI green");
}

pub(super) fn counting_questions_ask_the_seeded_count_of_the_framed_unit() {
    // R1017: a seeded counting cue plus a unit noun in the request's own
    // framing (outside its quoted payload) asks for that count.
    let solver = text_solver();
    for (prompt, expected) in [
        ("How many words are in 'the quick brown fox'?", "4"),
        ("How many lines are in this text: 'a\nb'", "2"),
        ("How many characters are in 'abc'?", "3"),
        ("How many unique words are in 'a b a c'?", "3"),
        ("Сколько слов в 'раз два три'?", "3"),
        ("How many characters are in 'two words'?", "9"),
    ] {
        let response = solver.solve(prompt);
        assert_eq!(response.intent, "text_manipulation", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
    let sorted = solver.solve("Sort these many words: pear apple");
    assert_eq!(sorted.answer, "apple pear");
}

pub(super) fn line_prefix_filter_reads_its_argument_and_composes_with_counts() {
    // R1017: the filter's argument follows its phrase (en, ru) or precedes it
    // (hi), bare or quoted, and a count named alongside ends the chain.
    let solver = text_solver();
    for (prompt, expected) in [
        (
            "Keep only the lines that start with x: 'x1\ny2\nx3'",
            "x1\nx3",
        ),
        (
            "How many lines start with x in this text: 'x1\ny2\nx3'",
            "2",
        ),
        ("Keep the lines starting with '#': '#a\nb\n#c'", "#a\n#c"),
        ("Сколько строк начинается с x в тексте: 'x1\ny2\nx3'", "2"),
        ("x से शुरू होने वाली पंक्तियाँ रखें: 'x1\ny2\nx3'", "x1\nx3"),
    ] {
        let response = solver.solve(prompt);
        assert_eq!(response.intent, "text_manipulation", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
}

pub(super) fn counting_a_seeded_character_class_counts_its_members() {
    // R1017: "vowels" declares its members per language in the operation
    // vocabulary; counting the class counts those characters, any case.
    let solver = text_solver();
    for (prompt, expected) in [
        ("Count the vowels in 'formal'", "2"),
        ("How many vowels are in 'Formal Area'?", "5"),
    ] {
        let response = solver.solve(prompt);
        assert_eq!(response.intent, "text_manipulation", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
}
