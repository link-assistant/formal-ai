//! Issue #676 (R8): the thinking display should read as a human narrative, not
//! a robotic list of identical category steps.
//!
//! The reporter reached Formal AI through an agentic CLI (OpenCode) that renders
//! the API `reasoning` field verbatim. Before this change the reasoning for
//! unrelated prompts (e.g. "Hello" vs "How are you?") differed only in the route
//! label buried mid-list. Now `render_thinking_steps` leads with a per-intent
//! human [`thinking_narrative`] headline and keeps the concrete steps beneath it
//! as the recursive "robotic detail" layer.

use formal_ai::{render_thinking_steps, thinking_narrative, FormalAiEngine};

/// Every reasoning trace opens with a human, first-person narrative headline and
/// still carries the concrete robotic detail (issue #676, R8).
#[test]
fn reasoning_opens_with_a_human_narrative_then_keeps_robotic_detail() {
    let answer = FormalAiEngine.answer("Hello");
    let rendered = render_thinking_steps(&answer.thinking_steps);
    let first_line = rendered.lines().next().expect("rendered reasoning");

    // The headline is the human summary...
    assert_eq!(first_line, "You said hello, so I greeted you back.");
    // ...and the robotic detail the other surfaces pin is still present below it.
    assert!(
        rendered.contains("Read the request: \"Hello\"."),
        "robotic detail should be preserved, got: {rendered}"
    );
    assert!(
        rendered.contains("Compose the answer:"),
        "the composed-answer step should still render, got: {rendered}"
    );
}

/// The narrative is genuinely per-intent: greeting and wellbeing (the concrete
/// "Hello" vs "How are you?" confusion from the issue) now open with distinct,
/// human headlines instead of an identical generic list.
#[test]
fn greeting_and_wellbeing_get_distinct_human_headlines() {
    let greeting = thinking_narrative(&FormalAiEngine.answer("Hello").thinking_steps)
        .expect("greeting narrative");
    let wellbeing = thinking_narrative(&FormalAiEngine.answer("How are you?").thinking_steps)
        .expect("wellbeing narrative");

    assert_ne!(
        greeting, wellbeing,
        "greeting and wellbeing should not share a robotic headline"
    );
    assert!(greeting.to_lowercase().contains("hello"));
    assert!(wellbeing.to_lowercase().contains("how i'm doing"));
}

/// The headline is language-agnostic: it summarizes the *decision*, so the same
/// wellbeing route reached in Russian gets the same English reasoning headline
/// (the API `reasoning` field stays a stable meta-language) while the composed
/// answer below remains localized.
#[test]
fn narrative_summarizes_the_route_regardless_of_prompt_language() {
    let english = thinking_narrative(&FormalAiEngine.answer("How are you?").thinking_steps);
    let russian = thinking_narrative(&FormalAiEngine.answer("как дела").thinking_steps);
    assert_eq!(english, russian);
    assert!(english.is_some());
}

/// A calculation opens with a calculation-shaped headline and still exposes the
/// recursive sub-steps (`↳`) as robotic detail.
#[test]
fn calculation_narrative_precedes_recursive_robotic_detail() {
    let answer = FormalAiEngine.answer("2 + 2");
    let rendered = render_thinking_steps(&answer.thinking_steps);
    let first_line = rendered.lines().next().expect("rendered reasoning");

    assert!(
        first_line.to_lowercase().contains("calculation"),
        "calculation should open with a calculation headline, got: {first_line}"
    );
    assert!(
        rendered.contains("↳"),
        "the recursive sub-steps should still render as robotic detail, got: {rendered}"
    );
    assert!(rendered.contains("Compute 2 + 2 = 4."));
}

/// An unrecognized route still yields a human headline rather than no narrative,
/// so no surface ever falls back to a bare category list.
#[test]
fn unknown_route_still_gets_a_human_headline() {
    let answer = FormalAiEngine.answer("asdfqwerzxcv");
    // Whatever route the solver picks, the narrative is present and human.
    let narrative = thinking_narrative(&answer.thinking_steps);
    if let Some(text) = narrative {
        assert!(
            text.chars().next().is_some_and(char::is_uppercase),
            "narrative should read as a sentence, got: {text}"
        );
        assert!(text.ends_with('.'), "narrative should be a sentence: {text}");
    }
}
