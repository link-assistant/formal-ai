//! Code-generation tests covering the top programming languages and the
//! execution-evidence requirements from issue #8.

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: parameterized write_program templates.
// ---------------------------------------------------------------------------

fn assert_write_program_parameters(response: &SymbolicAnswer, language: &str, task: &str) {
    assert_eq!(response.intent, "write_program");
    assert!(
        response
            .links_notation
            .contains(&format!("program_parameter:language {language}")),
        "Links Notation trace should include language={language}, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains(&format!("program_parameter:task {task}")),
        "Links Notation trace should include task={task}, got: {}",
        response.links_notation
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| { link == &format!("response:write_program:{task}:{language}") }),
        "evidence links should include the parameterized response link, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn rust_hello_world_seed_compiles_and_runs() {
    let response = answer("Write me hello world program in Rust");
    assert_write_program_parameters(&response, "rust", "hello_world");
    assert!(response
        .links_notation
        .contains("legacy_intent hello_world_rust"));
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("fn main()"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
    assert!(response.answer.contains("Check command: `rustc"));
    assert!(response.answer.contains("Run command: `./main`"));
    assert!(response.answer.contains("Hello, world!"));
}

#[test]
fn python_hello_world_seed_runs() {
    let response = answer("Write hello world in Python");
    assert_write_program_parameters(&response, "python", "hello_world");
    assert!(response.answer.contains("```python"));
    assert!(response.answer.contains("print(\"Hello, world!\")"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
    assert!(response.answer.contains("python3"));
}

#[test]
fn javascript_hello_world_seed_runs() {
    let response = answer("Show me hello world in JavaScript");
    assert_write_program_parameters(&response, "javascript", "hello_world");
    assert!(response.answer.contains("```javascript"));
    assert!(response.answer.contains("console.log(\"Hello, world!\");"));
    assert!(response.answer.contains("node"));
}

#[test]
fn go_hello_world_seed_runs() {
    let response = answer("hello world in Go");
    assert_write_program_parameters(&response, "go", "hello_world");
    assert!(response.answer.contains("```go"));
    assert!(response.answer.contains("fmt.Println"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
}

#[test]
fn c_hello_world_seed_compiles_and_runs() {
    let response = answer("hello world in C");
    assert_write_program_parameters(&response, "c", "hello_world");
    assert!(response.answer.contains("```c"));
    assert!(response.answer.contains("#include <stdio.h>"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
}

#[test]
fn typescript_hello_world_seed_reports_unavailable_execution() {
    let response = answer("hello world in TypeScript");
    assert_write_program_parameters(&response, "typescript", "hello_world");
    assert!(response.answer.contains("```typescript"));
    assert!(response
        .answer
        .contains("Execution status: not compiled or run"));
    assert!(response
        .answer
        .contains("Expected output after verification"));
}

#[test]
fn rust_alias_rs_is_supported() {
    let response = answer("hello world in rs");
    assert_write_program_parameters(&response, "rust", "hello_world");
}

#[test]
fn javascript_alias_node_is_supported() {
    let response = answer("hello world in node");
    assert_write_program_parameters(&response, "javascript", "hello_world");
}

#[test]
fn python_alias_py_is_supported() {
    let response = answer("hello world in py");
    assert_write_program_parameters(&response, "python", "hello_world");
}

#[test]
fn go_alias_golang_is_supported() {
    let response = answer("hello world in golang");
    assert_write_program_parameters(&response, "go", "hello_world");
}

#[test]
fn hello_world_without_recognized_language_returns_unsupported_parameters() {
    let response = answer("hello world in elvish");
    assert_eq!(response.intent, "write_program_unsupported");
    assert!(response.answer.contains("language `elvish`"));
    assert!(response.answer.contains("task `hello_world`"));
    assert!(response.answer.contains("Supported languages:"));
}

// ---------------------------------------------------------------------------
// Issue #53: Russian transliteration of "hello world" must be recognized.
// The reporter sent "Напиши хелло ворлд на питоне" and received "unknown".
// ---------------------------------------------------------------------------

#[test]
fn russian_transliteration_хелло_ворлд_питоне_returns_python() {
    // The exact prompt from the reported issue.
    let response = answer("Напиши хелло ворлд на питоне");
    assert_write_program_parameters(&response, "python", "hello_world");
    assert_eq!(
        response.intent, "write_program",
        "Russian-transliterated hello world in Python should resolve, got: {}",
        response.intent
    );
    assert!(
        response.answer.contains("```python"),
        "answer should include a Python code block, got: {}",
        response.answer
    );
}

#[test]
fn russian_transliteration_хелло_ворлд_на_джаваскрипт_returns_javascript() {
    let response = answer("хелло ворлд на джаваскрипт");
    assert_write_program_parameters(&response, "javascript", "hello_world");
    assert_eq!(
        response.intent, "write_program",
        "Russian-transliterated hello world in JavaScript should resolve, got: {}",
        response.intent
    );
}

#[test]
fn russian_transliteration_хелло_ворлд_на_расте_returns_rust() {
    let response = answer("хелло ворлд на расте");
    assert_write_program_parameters(&response, "rust", "hello_world");
    assert_eq!(
        response.intent, "write_program",
        "Russian-transliterated hello world in Rust should resolve, got: {}",
        response.intent
    );
}

// ---------------------------------------------------------------------------
// Issue #252 acceptance: top programming languages and richer code generation.
// ---------------------------------------------------------------------------

const POPULAR_LANGUAGES: &[(&str, &str, &str)] = &[
    ("Rust", "rust", "```rust"),
    ("Python", "python", "```python"),
    ("JavaScript", "javascript", "```javascript"),
    ("TypeScript", "typescript", "```typescript"),
    ("Go", "go", "```go"),
    ("C", "c", "```c"),
    ("C++", "cpp", "```cpp"),
    ("Java", "java", "```java"),
    ("C#", "csharp", "```csharp"),
    ("Ruby", "ruby", "```ruby"),
];

#[test]
fn top_ten_popular_languages_each_use_the_write_program_intent() {
    for (language, slug, fence) in POPULAR_LANGUAGES {
        let response = answer(&format!("Write me hello world in {language}"));
        assert_write_program_parameters(&response, slug, "hello_world");
        assert!(
            response.answer.contains(fence),
            "missing hello-world template for {language}: {}",
            response.answer
        );
    }
}

#[test]
fn code_answers_include_execution_links_in_notation() {
    let response = answer("Write me hello world program in Rust");
    assert!(
        response
            .links_notation
            .contains("execution_status compiled and ran"),
        "Links Notation trace should describe execution status, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("execution_environment"),
        "Links Notation trace should describe execution environment"
    );
}

#[test]
fn code_answers_declare_isolation_level() {
    let response = answer("Write me hello world program in Rust");
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("docker")
            || lower.contains("sandbox")
            || lower.contains("webvm")
            || lower.contains("isolated"),
        "execution evidence should declare an isolation boundary, got: {}",
        response.answer
    );
}

#[test]
fn sorting_algorithm_request_returns_code_and_tests() {
    let response = answer("Write me a sorting algorithm in Python with tests");
    assert!(response.intent.starts_with("algorithm_"));
    assert!(response.answer.contains("```python"));
    assert!(response.answer.contains("def test_") || response.answer.contains("assert "));
}

#[test]
fn translating_a_program_between_languages_keeps_semantics() {
    let rust = answer("Translate `fn add(a: i32, b: i32) -> i32 { a + b }` to Python");
    assert!(rust.intent.starts_with("translate_"));
    assert!(rust.answer.contains("def add"));
    assert!(rust.answer.contains("return a + b"));
}

#[test]
fn execution_failures_are_reported_with_full_trace() {
    let response = answer("Write a Python script that calls undefined_function()");
    assert!(
        response.answer.contains("Execution status: failed"),
        "failures must be honest, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:execution_failure")),
        "failed runs must expose a trace link"
    );
}

#[test]
fn parametric_write_program_handles_new_task_for_supported_language() {
    let response = answer("Write a Python program that counts to three");
    assert_write_program_parameters(&response, "python", "count_to_three");
    assert!(response.answer.contains("```python"));
    assert!(response.answer.contains("range(1, 4)"));
    assert!(
        !response.answer.contains("Hello, world!"),
        "new write-program tasks must not be routed through the legacy hello-world shortcut: {}",
        response.answer
    );
}

#[test]
fn supported_language_with_missing_template_returns_unsupported_parameters() {
    let response = answer("Write a Ruby program that counts to three");
    assert_eq!(response.intent, "write_program_unsupported");
    assert!(response.answer.contains("language `ruby`"));
    assert!(response.answer.contains("task `count_to_three`"));
    assert!(response.answer.contains("Supported tasks:"));
}

// ---------------------------------------------------------------------------
// Issue #312: "Напиши мне программу на Rust, которая выдаёт список файлов в
// текущей директории" returned "unknown" (the WASM worker) or a Rust concept
// definition (the CLI). The class of code-generation prompts that ask, in any
// supported language, for a program that lists files in the current directory
// must resolve through the parameterized write_program intent.
// ---------------------------------------------------------------------------

#[test]
fn russian_list_files_in_rust_returns_program() {
    // The exact prompt from the reported issue.
    let response =
        answer("Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории");
    assert_write_program_parameters(&response, "rust", "list_files");
    assert_eq!(
        response.intent, "write_program",
        "Russian list-files request in Rust should resolve to write_program, got: {}",
        response.intent
    );
    assert!(
        response.answer.contains("```rust"),
        "answer should include a Rust code block, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("read_dir"),
        "Rust list-files template should read the directory, got: {}",
        response.answer
    );
    // Issue #324: a Russian request must be answered in Russian. The natural
    // language framing (intro line and execution status) is localized while the
    // code stays canonical. The English status text still appears in the
    // language-independent Links Notation trace below.
    assert!(
        response
            .answer
            .contains("Статус выполнения: скомпилировано и запущено"),
        "Russian request should yield a Russian execution status line, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("Вот минимальная программа на языке"),
        "Russian request should yield a Russian intro line, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("execution_status compiled and ran"),
        "Links Notation trace stays language-independent, got: {}",
        response.links_notation
    );
}

#[test]
fn english_list_files_in_python_returns_program() {
    let response = answer("Write me a Python program that lists files in the current directory");
    assert_write_program_parameters(&response, "python", "list_files");
    assert!(response.answer.contains("```python"));
    assert!(response.answer.contains("listdir"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
}

#[test]
fn list_files_supported_for_every_popular_language() {
    for (language, slug, fence) in POPULAR_LANGUAGES {
        let response = answer(&format!(
            "Write me a program in {language} that lists files in the current directory"
        ));
        assert_write_program_parameters(&response, slug, "list_files");
        assert!(
            response.answer.contains(fence),
            "missing list-files template for {language}: {}",
            response.answer
        );
    }
}

#[test]
fn hindi_list_files_in_rust_returns_program() {
    // Hindi: "Write a program in Rust that shows the list of files".
    let response = answer("Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो");
    assert_write_program_parameters(&response, "rust", "list_files");
    assert_eq!(
        response.intent, "write_program",
        "Hindi list-files request in Rust should resolve to write_program, got: {}",
        response.intent
    );
    assert!(
        response.answer.contains("```rust"),
        "answer should include a Rust code block, got: {}",
        response.answer
    );
    assert!(response.answer.contains("read_dir"));
}

#[test]
fn chinese_list_files_in_rust_returns_program() {
    // Chinese: "Write a program in Rust that lists files in the current directory".
    let response = answer("用 Rust 编写一个列出当前目录中文件的程序");
    assert_write_program_parameters(&response, "rust", "list_files");
    assert_eq!(
        response.intent, "write_program",
        "Chinese list-files request in Rust should resolve to write_program, got: {}",
        response.intent
    );
    assert!(
        response.answer.contains("```rust"),
        "answer should include a Rust code block, got: {}",
        response.answer
    );
    assert!(response.answer.contains("read_dir"));
}

#[test]
fn russian_program_request_with_unknown_task_is_not_unknown() {
    // The whole class: a Russian "write a program in <lang>" request whose task
    // is not in the catalog must still be recognized as a (currently
    // unsupported) write_program request instead of falling through to unknown.
    let response = answer("Напиши программу на Python, которая вычисляет факториал числа");
    assert_eq!(
        response.intent, "write_program_unsupported",
        "Russian program request should be recognized as write_program, got: {}",
        response.intent
    );
    // Issue #324: the unsupported message is localized to Russian, so assert on
    // the backtick-quoted parameter (stable across languages) rather than the
    // English word "language".
    assert!(
        response.answer.contains("`python`"),
        "unsupported answer should name the extracted python parameter, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("write_program(language, task)"),
        "unsupported answer should reference the write_program route, got: {}",
        response.answer
    );
}

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
fn explicit_list_files_with_path_argument_is_supported() {
    // The path-argument variant is also reachable directly in a single turn.
    let response = answer("Write me a Rust program that lists files with a path argument");
    assert_write_program_parameters(&response, "rust", "list_files_arg");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("env::args"));
}
