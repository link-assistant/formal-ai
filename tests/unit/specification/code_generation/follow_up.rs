//! Conversation follow-up tests: context-reusing modification requests and
//! the explain-and-instruct code answers, including first-turn versus
//! follow-up instruction trimming (issue #386 split).

use super::{answer, assert_write_program_parameters};
use formal_ai::{ConversationTurn, UniversalSolver};

// ---------------------------------------------------------------------------
// Issue #324: a follow-up modification request must reuse the conversation
// context. The reporter first asked (in Russian) for a Rust program that lists
// files, then asked "Сделай так, чтобы программа принимала путь как аргумент"
// (make the program accept a path as an argument). That follow-up routes to
// write_program but names neither a task nor a language, so before the fix it
// failed with "I do not have a template for language `missing` and task
// `missing`". It must now recover the task (list_files -> list_files_arg) and
// language (rust) from the prior turns and answer in Russian.
// ---------------------------------------------------------------------------

#[test]
fn russian_follow_up_path_argument_modification_reuses_context() {
    let solver = UniversalSolver::default();
    let first = "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
    let plan = solver.solve(first);
    assert_eq!(plan.intent, "write_program");

    let history = [
        ConversationTurn::user(first),
        ConversationTurn::assistant(plan.answer),
    ];
    let response = solver.solve_with_history(
        "Сделай так, чтобы программа принимала путь как аргумент",
        &history,
    );

    assert_eq!(
        response.intent, "write_program",
        "follow-up modification should recover the write_program intent, got: {}",
        response.intent
    );
    // The task is upgraded to the path-argument variant in the recovered Rust
    // language.
    assert!(
        response
            .links_notation
            .contains("program_parameter:task list_files_arg"),
        "follow-up should resolve to the list_files_arg task, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains("program_parameter:language rust"),
        "follow-up should reuse the Rust language from context, got: {}",
        response.links_notation
    );
    assert!(
        response.answer.contains("```rust"),
        "follow-up answer should include a Rust code block, got: {}",
        response.answer
    );
    // The generated program reads the path from the command-line arguments.
    assert!(
        response.answer.contains("env::args"),
        "Rust path-argument template should read argv, got: {}",
        response.answer
    );
    // The conversation is in Russian, so the framing must be Russian and the
    // "missing template" error must be gone.
    assert!(
        response
            .answer
            .contains("Вот минимальная программа на языке"),
        "follow-up answer should be framed in Russian, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("missing"),
        "follow-up must not surface the missing-template error, got: {}",
        response.answer
    );
}

#[test]
fn hindi_follow_up_path_argument_modification_reuses_context() {
    let solver = UniversalSolver::default();
    let first = "Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो";
    let plan = solver.solve(first);
    assert_eq!(plan.intent, "write_program");

    let history = [
        ConversationTurn::user(first),
        ConversationTurn::assistant(plan.answer),
    ];
    // Hindi: "make it so the program accepts a path as an argument".
    let response = solver.solve_with_history(
        "इसे ऐसा बनाओ कि प्रोग्राम पथ को तर्क के रूप में स्वीकार करे",
        &history,
    );

    assert_eq!(response.intent, "write_program");
    assert!(
        response
            .links_notation
            .contains("program_parameter:task list_files_arg"),
        "Hindi follow-up should resolve to list_files_arg, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains("program_parameter:language rust"),
        "Hindi follow-up should reuse Rust from context, got: {}",
        response.links_notation
    );
    assert!(response.answer.contains("env::args"));
    // The conversation is in Hindi, so the framing must be Hindi.
    assert!(
        response.answer.contains("में एक न्यूनतम प्रोग्राम है"),
        "Hindi follow-up answer should be framed in Hindi, got: {}",
        response.answer
    );
    assert!(!response.answer.contains("missing"));
}

#[test]
fn chinese_follow_up_path_argument_modification_reuses_context() {
    let solver = UniversalSolver::default();
    let first = "用 Rust 编写一个列出当前目录中文件的程序";
    let plan = solver.solve(first);
    assert_eq!(plan.intent, "write_program");

    let history = [
        ConversationTurn::user(first),
        ConversationTurn::assistant(plan.answer),
    ];
    // Chinese: "make the program accept a path as an argument".
    let response = solver.solve_with_history("制作程序，使其接受路径作为参数", &history);

    assert_eq!(response.intent, "write_program");
    assert!(
        response
            .links_notation
            .contains("program_parameter:task list_files_arg"),
        "Chinese follow-up should resolve to list_files_arg, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains("program_parameter:language rust"),
        "Chinese follow-up should reuse Rust from context, got: {}",
        response.links_notation
    );
    assert!(response.answer.contains("env::args"));
    // The conversation is in Chinese, so the framing must be Chinese.
    assert!(
        response.answer.contains("这是一个最小的"),
        "Chinese follow-up answer should be framed in Chinese, got: {}",
        response.answer
    );
    assert!(!response.answer.contains("missing"));
}

#[test]
fn explicit_list_files_with_path_argument_is_supported() {
    // The path-argument variant is also reachable directly in a single turn.
    let response = answer("Write me a Rust program that lists files with a path argument");
    assert_write_program_parameters(&response, "rust", "list_files_arg");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("env::args"));
}

#[test]
fn english_follow_up_modification_emits_substitution_plan_trace() {
    // Issue #324 R4/R6: the follow-up modification is lowered through the Links
    // Notation substitution pipeline, and the plan is surfaced as an inspectable
    // `write_program_plan` evidence link so the reasoning is transparent.
    let solver = UniversalSolver::default();
    let first = "Write me a Rust program that lists the files in the current directory";
    let plan = solver.solve(first);
    assert_eq!(plan.intent, "write_program");

    let history = [
        ConversationTurn::user(first),
        ConversationTurn::assistant(plan.answer),
    ];
    let response =
        solver.solve_with_history("Make the program accept a path as an argument", &history);

    assert_eq!(response.intent, "write_program");
    assert!(
        response
            .links_notation
            .contains("program_parameter:task list_files_arg"),
        "English follow-up should resolve to list_files_arg, got: {}",
        response.links_notation
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("write_program_plan:")),
        "the substitution plan should be surfaced as an evidence link, got: {:?}",
        response.evidence_links
    );
}

// ---------------------------------------------------------------------------
// Issue #330: a code answer must teach a novice — every generated program is
// accompanied by a plain-language "how it works" explanation and step-by-step
// testing instructions, localized for every supported response language. When
// the dialog already walked the user through running code, a follow-up edit
// omits the verbose setup steps and shows a concise "test it the same way"
// note instead.
// ---------------------------------------------------------------------------

#[test]
fn english_code_answer_explains_and_instructs() {
    let response = answer("Write me a Rust program that lists files in the current directory");
    assert_write_program_parameters(&response, "rust", "list_files");
    let text = &response.answer;
    assert!(
        text.contains("How it works:"),
        "English code answer should explain how it works, got: {text}"
    );
    assert!(
        text.contains("sorts the list alphabetically"),
        "English explanation should describe the algorithm, got: {text}"
    );
    assert!(
        text.contains("How to test it yourself:"),
        "English code answer should give test instructions, got: {text}"
    );
    assert!(
        text.contains("Install the Rust toolchain from https://rustup.rs"),
        "English instructions should include the toolchain setup, got: {text}"
    );
    assert!(
        text.contains("Save the code above to a file named `main.rs`"),
        "English instructions should tell the user where to save the code, got: {text}"
    );
    assert!(
        text.contains("Run it: `./main`"),
        "English instructions should include the run command, got: {text}"
    );
}

#[test]
fn russian_code_answer_explains_and_instructs() {
    let response =
        answer("Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории");
    assert_write_program_parameters(&response, "rust", "list_files");
    let text = &response.answer;
    assert!(
        text.contains("Как это работает:"),
        "Russian code answer should explain how it works, got: {text}"
    );
    assert!(
        text.contains("Как проверить это самостоятельно:"),
        "Russian code answer should give test instructions, got: {text}"
    );
    assert!(
        text.contains("Сохраните приведённый выше код в файл `main.rs`"),
        "Russian instructions should tell the user where to save the code, got: {text}"
    );
    assert!(
        text.contains("Запустите программу: `./main`"),
        "Russian instructions should include the run command, got: {text}"
    );
}

#[test]
fn hindi_code_answer_explains_and_instructs() {
    let response = answer("Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो");
    assert_write_program_parameters(&response, "rust", "list_files");
    let text = &response.answer;
    assert!(
        text.contains("यह कैसे काम करता है:"),
        "Hindi code answer should explain how it works, got: {text}"
    );
    assert!(
        text.contains("इसे स्वयं कैसे जाँचें:"),
        "Hindi code answer should give test instructions, got: {text}"
    );
    assert!(
        text.contains("`main.rs`"),
        "Hindi instructions should reference the file name, got: {text}"
    );
}

#[test]
fn chinese_code_answer_explains_and_instructs() {
    let response = answer("用 Rust 编写一个列出当前目录中文件的程序");
    assert_write_program_parameters(&response, "rust", "list_files");
    let text = &response.answer;
    assert!(
        text.contains("工作原理："),
        "Chinese code answer should explain how it works, got: {text}"
    );
    assert!(
        text.contains("如何自行测试："),
        "Chinese code answer should give test instructions, got: {text}"
    );
    assert!(
        text.contains("`main.rs`"),
        "Chinese instructions should reference the file name, got: {text}"
    );
}

#[test]
fn unavailable_language_explains_and_instructs() {
    // Ruby's toolchain is "Unavailable" in this runtime, but a novice still
    // needs the explanation and the setup/run steps.
    let response = answer("Write me a Ruby program that lists files in the current directory");
    assert_write_program_parameters(&response, "ruby", "list_files");
    let text = &response.answer;
    assert!(
        text.contains("How it works:"),
        "Ruby code answer should explain how it works, got: {text}"
    );
    assert!(
        text.contains("Install Ruby from https://www.ruby-lang.org/en/downloads/"),
        "Ruby instructions should include the toolchain setup, got: {text}"
    );
    assert!(
        text.contains("Save the code above to a file named `main.rb`"),
        "Ruby instructions should tell the user where to save the code, got: {text}"
    );
    assert!(
        text.contains("Run it: `ruby main.rb`"),
        "Ruby instructions should include the run command, got: {text}"
    );
}

#[test]
fn follow_up_code_edit_omits_setup_steps() {
    let solver = UniversalSolver::default();
    let first = "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
    let plan = solver.solve(first);
    let history = [
        ConversationTurn::user(first),
        ConversationTurn::assistant(plan.answer),
    ];
    let response = solver.solve_with_history(
        "Сделай так, чтобы программа принимала путь как аргумент",
        &history,
    );
    let text = &response.answer;
    // The verbose setup walkthrough is replaced by a concise reminder.
    assert!(
        !text.contains("Как проверить это самостоятельно:"),
        "follow-up edit should omit the verbose setup steps, got: {text}"
    );
    assert!(
        text.contains("Проверьте обновлённую программу так же, как и раньше:"),
        "follow-up edit should keep a concise test reminder, got: {text}"
    );
    assert!(
        text.contains("`./main`"),
        "follow-up reminder should still include the run command, got: {text}"
    );
    // The explanation of the (now changed) program is still present.
    assert!(
        text.contains("Как это работает:"),
        "follow-up edit should still explain how the updated program works, got: {text}"
    );
}

#[test]
fn first_turn_without_prior_code_keeps_full_instructions() {
    // A first turn that happens to carry unrelated history (no prior code
    // block) must still receive the full novice instructions.
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("Hello"),
        ConversationTurn::assistant("Hi! How can I help?".to_owned()),
    ];
    let response = solver.solve_with_history(
        "Write me a Rust program that lists files in the current directory",
        &history,
    );
    assert!(
        response.answer.contains("How to test it yourself:"),
        "a first code answer should include full instructions, got: {}",
        response.answer
    );
}
