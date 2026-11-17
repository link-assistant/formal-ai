//! Shared output selection runs before the agentic planner on rewrite turns.
use super::*;
use crate::cnl_output::{
    cnl_policy, formalize_output, output_configuration, output_source, output_turn, render_output,
};

#[must_use]
pub fn create_chat_completion_with_solver_and_memory(
    request: &ChatCompletionRequest,
    solver: &UniversalSolver,
    memory_events: &[MemoryEvent],
) -> ChatCompletion {
    let mut configuration = solver.config.output;
    if let Some(named) = &request.output_standard {
        if let Ok(selected) = output_configuration(Some(named), None, cnl_policy()) {
            if selected.density == crate::cnl_output::OutputDensity::Standard {
                configuration.standard = selected.standard;
            } else {
                configuration.density = selected.density;
            }
        }
    }
    if let Some(density) = &request.lexical_density {
        if let Ok(selected) = output_configuration(None, Some(density), cnl_policy()) {
            configuration.density = selected.density;
        }
    }
    let (prompt, history) = chat_prompt_and_history(&request.messages);
    let turn = output_turn(&prompt, &history, configuration, cnl_policy());
    let mut solver_configuration = solver.config;
    solver_configuration.output = turn.config;
    let selected = UniversalSolver::new(solver_configuration);
    let mut completion = if turn.rewrite.is_some() {
        chat_completion_from_symbolic(
            request,
            &prompt,
            selected.solve_with_history(&prompt, &history),
        )
    } else {
        let mut planning_request = request.clone();
        if turn.prompt != prompt {
            if let Some(message) = planning_request
                .messages
                .iter_mut()
                .rev()
                .find(|message| message.role.eq_ignore_ascii_case("user"))
            {
                if let MessageContent::Text(content) = &mut message.content {
                    if let Some(start) = content.find(&prompt) {
                        content.replace_range(start..start + prompt.len(), &turn.prompt);
                    }
                }
            }
        }
        create_unprojected_chat_completion(&planning_request, &selected, memory_events)
    };
    if let Some(choice) = completion.choices.first_mut() {
        let text = choice.message.content.plain_text();
        let language = selected
            .config
            .forced_response_language
            .unwrap_or_else(|| crate::language::detect(&text).slug());
        let source = output_source(&text, language, cnl_policy());
        let rendered = render_output(
            &formalize_output(&source, language, cnl_policy()),
            turn.config,
            cnl_policy(),
            &(),
        );
        choice.message.content = MessageContent::Text(rendered.content.clone());
        completion.usage.completion_tokens = estimate_tokens(&rendered.content);
        completion.usage.total_tokens = completion
            .usage
            .prompt_tokens
            .saturating_add(completion.usage.completion_tokens);
        completion.output_projection =
            Some(serde_json::to_value(rendered).expect("output projection"));
    }
    completion
}
