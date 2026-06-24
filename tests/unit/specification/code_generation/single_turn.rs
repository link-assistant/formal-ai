//! Single-turn program-generation tests: hello-world seeds, language
//! aliases and transliterations, the popular-language sweep, execution
//! evidence, and the parameterized `write_program` templates (issue #386 split).

use super::{answer, assert_write_program_parameters, POPULAR_LANGUAGES};

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
fn rust_hello_world_accepts_inline_output_replacement() {
    let response = answer(
        "Напиши мне Hello World программу на Rust, но выведи вместо Hello World \"Bye World\"",
    );

    assert_write_program_parameters(&response, "rust", "hello_world");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("println!(\"Bye World\");"));
    assert!(response.answer.contains("```text\nBye World\n```"));
    assert!(!response.answer.contains("Hello, world!"));
}

#[test]
fn hello_world_inline_replacement_accepts_diverse_prompt_surfaces() {
    struct Case {
        language: &'static str,
        slug: &'static str,
        prompt: &'static str,
        code_fragment: &'static str,
        output_fragment: &'static str,
    }

    let cases = [
        Case {
            language: "Python",
            slug: "python",
            prompt: "Write hello world in Python, but print “Bye Python” instead",
            code_fragment: "print(\"Bye Python\")",
            output_fragment: "```text\nBye Python\n```",
        },
        Case {
            language: "JavaScript",
            slug: "javascript",
            prompt: "Write hello world in JavaScript and replace `Hello World` with `Bye JS`",
            code_fragment: "console.log(\"Bye JS\");",
            output_fragment: "```text\nBye JS\n```",
        },
        Case {
            language: "Go",
            slug: "go",
            prompt: "Write hello world in Go and 替换「Hello World」为「Bye Go」",
            code_fragment: "fmt.Println(\"Bye Go\")",
            output_fragment: "```text\nBye Go\n```",
        },
    ];

    for case in cases {
        let response = answer(case.prompt);
        assert_write_program_parameters(&response, case.slug, "hello_world");
        assert!(
            response.answer.contains(case.code_fragment),
            "{} prompt should replace the generated program literal, got: {}",
            case.language,
            response.answer
        );
        assert!(
            response.answer.contains(case.output_fragment),
            "{} prompt should replace the expected output block, got: {}",
            case.language,
            response.answer
        );
        assert!(
            !response.answer.contains("Hello, world!"),
            "{} prompt should not leave the old hello-world literal behind",
            case.language
        );
    }
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
fn python_list_files_reverse_sort_sample_output_matches_saved_file_name() {
    let response = answer(
        "Write me a Python program that lists files in the current directory in reverse-sorted order",
    );

    assert_write_program_parameters(&response, "python", "list_files_reverse_sort");
    assert!(response.answer.contains("```python"));
    assert!(
        response
            .answer
            .contains("```text\nmain.py\ndata.txt\nREADME.md\n```"),
        "Python reverse file-list output should match the documented sample directory, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Cargo.toml"),
        "Python sample output should not mention Rust project files, got: {}",
        response.answer
    );
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

#[test]
fn unknown_language_after_the_language_noun_is_extracted_in_both_head_initial_languages() {
    // Issue #386: the unknown-implementation-language fallback skips the head
    // noun "language" ("in language <name>") / "языке" ("на языке <name>") to
    // read the language name after it. Both the target preposition and the noun
    // are seed data (roles implementation_language_preposition /
    // implementation_language_noun, sourced via words_for_role_in_languages),
    // not literals baked into the parser. This exercises the noun-skip path in
    // both head-initial languages so the seed-driven extractor stays covered.
    let english = answer("hello world in language elvish");
    assert_eq!(
        english.intent, "write_program_unsupported",
        "English noun-skip request should be an unsupported write_program, got: {}",
        english.intent
    );
    assert!(
        english.answer.contains("`elvish`"),
        "English noun-skip extraction should name the elvish parameter, got: {}",
        english.answer
    );

    let russian = answer("хелло ворлд на языке elvish");
    assert_eq!(
        russian.intent, "write_program_unsupported",
        "Russian noun-skip request should be an unsupported write_program, got: {}",
        russian.intent
    );
    assert!(
        russian.answer.contains("`elvish`"),
        "Russian noun-skip extraction should name the elvish parameter, got: {}",
        russian.answer
    );
}
