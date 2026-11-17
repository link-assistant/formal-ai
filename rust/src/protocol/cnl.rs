//! Shared output selection runs before the agentic planner on rewrite turns.
use super::*;
use crate::cnl_output::{
    OutputDensity, cnl_policy, formalize_output, output_configuration, output_source, output_turn,
    render_output, validate_output,
};

pub(super) fn strip_notice_for_envelope(
    context: &request_context::RequestContext,
    solver: &UniversalSolver,
    answer: &mut SymbolicAnswer,
) {
    if context.preserve_answer_body()
        && solver.config.output.density == OutputDensity::MaxPrecision
        && let Some(locale) = cnl_policy()
            .languages
            .iter()
            .find(|locale| !locale.notice.is_empty() && answer.answer.starts_with(&locale.notice))
    {
        answer.answer = output_source(&answer.answer, &locale.id, cnl_policy());
    }
}

#[must_use]
pub fn create_chat_completion_with_solver_and_memory(
    request: &ChatCompletionRequest,
    solver: &UniversalSolver,
    memory_events: &[MemoryEvent],
) -> ChatCompletion {
    let mut configuration = solver.config.output;
    if let Some(named) = &request.output_standard {
        if let Ok(selected) = output_configuration(Some(named), None, cnl_policy()) {
            if selected.density == OutputDensity::Standard {
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
    let context = request_context::RequestContext::new(&request.messages);
    let mut solver_configuration = solver.config;
    solver_configuration.output = turn.config;
    let selected = context.solver(&UniversalSolver::new(solver_configuration));
    let mut completion = if turn.rewrite.is_some() {
        let mut answer = selected.solve_with_history(&prompt, &history);
        strip_notice_for_envelope(&context, &selected, &mut answer);
        chat_completion_from_symbolic(request, &prompt, answer)
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
    if let Some(choice) = completion
        .choices
        .first_mut()
        .filter(|choice| choice.finish_reason == "stop")
    {
        let body = choice.message.content.plain_text();
        let text = context.answer_text(&body);
        let language = selected
            .config
            .forced_response_language
            .unwrap_or_else(|| crate::language::detect(&text).slug());
        let preserve_body = context.preserve_answer_body();
        let source = if preserve_body {
            text
        } else {
            output_source(&text, language, cnl_policy())
        };
        let mut rendered = render_output(
            &formalize_output(&source, language, cnl_policy()),
            turn.config,
            cnl_policy(),
            &(),
        );
        if preserve_body {
            rendered.content = body;
            if rendered.text != source {
                rendered.report =
                    validate_output(&source, language, turn.config, cnl_policy(), &());
                rendered.text = source;
            }
        }
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
