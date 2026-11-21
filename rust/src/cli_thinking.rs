use formal_ai::{
    SymbolicAnswer, naturalize_thinking_step_in, thinking_answer_language, thinking_trace_heading,
};

/// Render the solver's concrete thinking trace for the `--thinking` flag.
///
/// Each step is shown by its naturalized, human-readable `summary` (issue #488)
/// — the same meta-language description the web UI and API surfaces expose —
/// rather than its internal `step` slug. Composite steps are nested under their
/// parent with a `↳` marker so the recursively composite (fractal) structure of
/// the reasoning is visible on the CLI too.
///
/// The sentences are written in the language the answer itself is written in
/// (issue #889): a Russian answer gets a Russian trace, not an English
/// description of a Russian answer.
pub(crate) fn print_thinking_trace(answer: &SymbolicAnswer) {
    if answer.thinking_steps.is_empty() {
        return;
    }
    let language = thinking_answer_language(&answer.thinking_steps);
    let heading = thinking_trace_heading(&language);
    if !heading.is_empty() {
        println!("{heading}:");
    }
    for step in &answer.thinking_steps {
        let sentence = naturalize_thinking_step_in(&language, &step.step, &step.detail);
        if step.parent_id.is_some() {
            println!("    ↳ {sentence}");
        } else {
            println!("  {sentence}");
        }
    }
    println!();
}
