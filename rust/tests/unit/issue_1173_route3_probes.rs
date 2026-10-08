//! Issue #1173 R1173-3, the fifth routing-probe pass (ROUTE3), native root.
//!
//! "a program that prints <text>" names the catalog task whose output is the
//! request's operand (`print_text` in `data/seed/hello-world-programs.lino`):
//! the program is the documented `hello_world` print procedure with the text
//! bound into its output literal, never a stored program. And a handler the
//! precedence seed marks `before_promotion` is asked before the promoted
//! methods, as the browser asks it in its fixed early phase (issue #1175
//! p133). The browser twin is `rust/tests/web/issue-1173-route3-probes.test.mjs`.

use formal_ai::method_registry::{MethodRegistry, MethodSurface};
use formal_ai::{SolverConfig, UniversalSolver};

fn solved(prompt: &str) -> (String, String) {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve(prompt);
    (answer.intent, answer.answer)
}

#[test]
fn a_program_that_prints_a_greeting_is_the_print_text_task_composed_from_the_documented_procedure()
{
    let (intent, answer) = solved("Write a program in Rust that prints hello");
    assert_eq!(intent, "write_program", "{answer}");
    assert_eq!(
        answer,
        [
            "Here is a minimal Rust print text program:",
            "",
            "```rust",
            "fn main() {",
            "    println!(\"hello\");",
            "}",
            "```",
            "",
            "Execution status: not run; this program was rediscovered from \
             https://doc.rust-lang.org/book/ch01-02-hello-world.html and its output contract \
             was checked by decomposition, not by executing it.",
            "Check command: `rustc main.rs -o main`",
            "Run command: `./main`",
            "Expected output after verification:",
            "```text",
            "hello",
            "```",
            "",
            "How it works:",
            "The program performs the requested task and prints its result to standard output.",
            "",
            "How to test it yourself:",
            "1. Install the Rust toolchain from https://rustup.rs.",
            "2. Save the code above to a file named `main.rs`.",
            "3. Check that it compiles: `rustc main.rs -o main`.",
            "4. Run it: `./main`.",
            "5. Compare the output with the expected output shown above.",
        ]
        .join("\n")
    );
    let (intent, answer) = solved("Напиши программу на Python, которая печатает привет");
    assert_eq!(intent, "write_program", "{answer}");
    assert!(
        answer.contains("```python\nprint('привет')\n```"),
        "{answer}"
    );
    assert!(answer.contains("```text\nпривет\n```"), "{answer}");
}

#[test]
fn a_described_value_or_a_script_request_names_no_print_text_task() {
    assert_eq!(
        solved("Write a program in Rust that prints the date").0,
        "write_program_skill_gap"
    );
    assert_eq!(
        solved("Write a Python script that prints hello").0,
        "write_script_python"
    );
}

#[test]
fn a_before_promotion_handler_is_asked_ahead_of_the_promoted_methods() {
    assert_eq!(
        formal_ai::seed::before_promotion_handlers(),
        ["github_repository_traffic".to_owned()]
    );
    let registry = MethodRegistry::shared();
    let ordered = registry.dispatch_order(&["route:web_search".to_owned()]);
    let at = |name: &str| ordered.iter().position(|method| method == name);
    assert_eq!(
        at("github_repository_traffic"),
        Some(registry.count_on(MethodSurface::Prelude)),
        "{ordered:?}"
    );
    assert!(
        at("github_repository_traffic") < at("web_search"),
        "{ordered:?}"
    );
    assert_eq!(
        solved("Can I see who visited my GitHub repository?").0,
        "github_repository_traffic"
    );
}
