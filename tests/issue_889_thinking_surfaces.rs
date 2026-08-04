//! Issue #889 (parent #710): the CLI, the OpenAI-compatible APIs and the
//! Anthropic API must narrate their thinking in the language of the answer.
//!
//! The browser thinking panel was already localized through the web i18n
//! catalog, but every other surface rendered the English literals that used to
//! live in `src/thinking.rs`: a Russian answer arrived with an English
//! explanation of how it was produced. These tests drive each non-UI surface
//! once per registered language and assert the rendered trace is the seed prose
//! of that language — and, for the non-English ones, that the English prose is
//! absent — while the machine-readable `step`/`detail` keys stay untouched.

use std::process::Command;

use formal_ai::language::registered_languages;
use formal_ai::thinking_prose::thinking_prose;
use formal_ai::{
    create_anthropic_message_with_solver, create_chat_completion_with_solver,
    AnthropicContentBlock, AnthropicMessagesRequest, ChatCompletionRequest, FormalAiEngine,
    UniversalSolver,
};

/// One prompt per registered language, each unambiguous enough that the solver
/// resolves the answer language from it.
const PROMPTS: &[(&str, &str)] = &[
    ("en", "Hello"),
    ("ru", "привет"),
    ("hi", "नमस्ते"),
    ("zh", "你好"),
    ("es", "hola, ¿cómo estás?"),
];

fn prompt_for(slug: &str) -> &'static str {
    match PROMPTS.iter().find(|(language, _)| *language == slug) {
        Some((_, prompt)) => prompt,
        None => panic!("issue #889: add a prompt for the newly registered {slug}"),
    }
}

/// The sentence the `impulse` step must render for `prompt` in `slug`.
fn expected_impulse(slug: &str, prompt: &str) -> String {
    thinking_prose("thinking_step_impulse", slug, &[("prompt", prompt)])
        .unwrap_or_else(|| panic!("missing impulse prose for {slug}"))
}

/// Assert `rendered` narrates in `slug` rather than in English.
fn assert_localized(surface: &str, slug: &str, prompt: &str, rendered: &str) {
    let expected = expected_impulse(slug, prompt);
    assert!(
        rendered.contains(&expected),
        "{surface} should narrate in {slug} ({expected}), got: {rendered}"
    );
    if slug != "en" {
        let english = expected_impulse("en", prompt);
        assert!(
            !rendered.contains(&english),
            "{surface} leaked the English trace into a {slug} answer, got: {rendered}"
        );
    }
}

/// Every registered language has a prompt here, so registering a language
/// without extending this matrix fails rather than silently going untested.
#[test]
fn the_language_matrix_covers_every_registered_language() {
    for language in registered_languages() {
        let prompt = prompt_for(language.slug());
        assert!(!prompt.is_empty());
    }
    assert_eq!(
        PROMPTS.len(),
        registered_languages().len(),
        "the prompt matrix should track the language registry exactly"
    );
}

/// Surface 1 — the CLI `--thinking` trace, including the heading it prints
/// above the steps.
#[test]
fn cli_thinking_trace_is_written_in_the_answer_language() {
    for language in registered_languages() {
        let slug = language.slug();
        let prompt = prompt_for(slug);
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["chat", "--prompt", prompt, "--thinking", "--silent"])
            .output()
            .expect("run the chat command");
        assert!(output.status.success(), "{output:?}");
        let rendered = String::from_utf8(output.stdout).expect("utf-8 stdout");

        let heading = thinking_prose("thinking_trace_heading", slug, &[])
            .unwrap_or_else(|| panic!("missing trace heading for {slug}"));
        assert!(
            rendered.contains(&format!("{heading}:")),
            "the CLI should label the trace in {slug} ({heading}), got: {rendered}"
        );
        assert_localized("the CLI trace", slug, prompt, &rendered);
    }
}

/// Surface 2 — the OpenAI-compatible Chat Completions `reasoning` /
/// `reasoning_content` fields, which agentic CLIs render verbatim.
#[test]
fn chat_completion_reasoning_is_written_in_the_answer_language() {
    for language in registered_languages() {
        let slug = language.slug();
        let prompt = prompt_for(slug);
        let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": prompt}]
        }))
        .unwrap();
        let completion = create_chat_completion_with_solver(&request, &UniversalSolver::default());
        let message = &completion.choices[0].message;

        assert_localized(
            "the chat-completions reasoning",
            slug,
            prompt,
            &message.reasoning,
        );
        assert_eq!(
            message.reasoning, message.reasoning_content,
            "both reasoning fields should carry the same localized trace"
        );

        // The machine-readable trace keys stay language-neutral so downstream
        // consumers never have to parse prose.
        assert!(
            message
                .thinking_steps
                .iter()
                .any(|step| step.step == "impulse"),
            "the `step` keys must stay language-neutral, got: {:?}",
            message.thinking_steps
        );
    }
}

/// Surface 3 — the Anthropic Messages extended-thinking block.
#[test]
fn anthropic_thinking_block_is_written_in_the_answer_language() {
    for language in registered_languages() {
        let slug = language.slug();
        let prompt = prompt_for(slug);
        let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
            "model": "claude-sonnet-4-5",
            "max_tokens": 1024,
            "thinking": {"type": "enabled", "budget_tokens": 1024},
            "messages": [{"role": "user", "content": prompt}]
        }))
        .unwrap();
        let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
        let thinking = message
            .content
            .iter()
            .find_map(|block| match block {
                AnthropicContentBlock::Thinking { thinking, .. } => Some(thinking.clone()),
                _ => None,
            })
            .expect("extended thinking should produce a thinking block");

        assert_localized("the Anthropic thinking block", slug, prompt, &thinking);
    }
}

/// The serialized `summary` of each step — the field the browser and the JSON
/// surfaces read — is localized at trace assembly, not only at render time.
#[test]
fn step_summaries_are_localized_while_trace_keys_stay_neutral() {
    for language in registered_languages() {
        let slug = language.slug();
        let prompt = prompt_for(slug);
        let answer = FormalAiEngine.answer(prompt);
        let impulse = answer
            .thinking_steps
            .iter()
            .find(|step| step.step == "impulse")
            .expect("every trace opens with an impulse step");

        assert_eq!(impulse.detail, prompt, "the detail stays the raw prompt");
        assert_eq!(
            impulse.summary,
            expected_impulse(slug, prompt),
            "the stored summary should already be in {slug}"
        );
    }
}
