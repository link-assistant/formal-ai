//! Which task class does the router assign to a greeting in each registered language?
//!
//! Issue #1073's multilingual invariant compares one audit per language, and the
//! instruction gate reports the *task class*, so a language whose greeting routes
//! elsewhere makes the comparison about routing rather than about the standard.
//! This prints the class per candidate prompt so the choice is measured, not guessed.
//!
//! Run it by copying it into `examples/` -- it links against the crate:
//!
//! ```sh
//! cp experiments/issue-1073-greeting-task-class.rs examples/issue_1073_greeting_task_class.rs
//! cargo run --example issue_1073_greeting_task_class
//! rm examples/issue_1073_greeting_task_class.rs
//! ```
//!
//! Measured on this branch: en/ru/hi/zh greetings route to `courtesy`, and every
//! Spanish greeting tried routes to `statement` or `question`, because
//! `data/seed/prompt-patterns.lino` has no `es` greeting keyword.

fn main() {
    let prompts: &[(&str, &str)] = &[
        ("en", "Hello"),
        ("en", "hi"),
        ("ru", "привет"),
        ("hi", "नमस्ते"),
        ("zh", "你好"),
        ("es", "hola"),
        ("es", "hola, ¿cómo estás?"),
        ("es", "Hola"),
        ("es", "buenos días"),
    ];
    for (language, prompt) in prompts {
        let candidate = formal_ai::translation::formalize_prompt(prompt, language);
        let formalization =
            formal_ai::intent_formalization::formalize_intent(prompt, language, Some(&candidate));
        println!("{language:>3}  {prompt:<24} -> {}", formalization.kind.slug());
    }
}
