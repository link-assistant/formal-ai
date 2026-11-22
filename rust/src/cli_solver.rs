use crate::cli_thinking::print_thinking_trace;
use crate::{DefinitionFusionMode, OutputFormat};
use formal_ai::agentic_coding::{corpus::fetch_cli_source, run_agentic_task_with_source_fetch};
use formal_ai::{
    ChatCompletionRequest, ChatMessage, DEFAULT_MODEL, ExecutionSurface, ResponsesRequest,
    SolverConfig, UniversalSolver, create_chat_completion_with_solver, create_response_with_solver,
};
use std::error::Error;

pub(super) fn run_agent(
    task: &str,
    transcript: bool,
    session_json: Option<&std::path::Path>,
) -> Result<(), Box<dyn Error>> {
    let outcome = run_agentic_task_with_source_fetch(task, fetch_cli_source)?;
    if transcript {
        print!("{}", outcome.transcript());
        println!();
    }
    if let Some(path) = session_json {
        let rendered = serde_json::to_string_pretty(&outcome.session_json())?;
        std::fs::write(path, format!("{rendered}\n"))?;
        formal_ai::diagnostic!("debug"; "wrote agent session to {}", path.display());
    }
    if outcome.hit_turn_cap {
        return Err(format!(
            "the agentic loop did not finish within its turn cap after {} tool call(s)",
            outcome.steps.len()
        )
        .into());
    }
    println!("{}", outcome.final_answer);
    Ok(())
}

fn solver_for_chat(
    definition_fusion: Option<DefinitionFusionMode>,
    compute_budget: Option<u32>,
    draft_count: Option<u8>,
) -> UniversalSolver {
    let mut config = SolverConfig::from_env();
    config.execution_surface = ExecutionSurface::Cli;
    if let Some(mode) = definition_fusion {
        config.definition_fusion_by_default = matches!(mode, DefinitionFusionMode::Auto);
    }
    if let Some(budget) = compute_budget {
        config.compute_budget = budget;
    }
    if let Some(count) = draft_count {
        config.draft_count = count.max(1);
    }
    UniversalSolver::new(config)
}

pub(super) fn run_chat(
    prompt: &str,
    format: OutputFormat,
    definition_fusion: Option<DefinitionFusionMode>,
    thinking: bool,
    compute_budget: Option<u32>,
    draft_count: Option<u8>,
) -> Result<(), Box<dyn Error>> {
    let solver = solver_for_chat(definition_fusion, compute_budget, draft_count);
    match format {
        OutputFormat::Text => {
            let response = solver.solve(prompt);
            if thinking {
                print_thinking_trace(&response);
            }
            println!("{}", response.answer);
        }
        OutputFormat::Chat => {
            let request = ChatCompletionRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                messages: vec![ChatMessage::user(prompt)],
                temperature: None,
                stream: false,
                tools: Vec::new(),
                tool_choice: None,
                functions: Vec::new(),
                function_call: None,
                output_standard: None,
                lexical_density: None,
                response_format: None,
                stream_options: None,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_chat_completion_with_solver(
                    &request, &solver
                ))?
            );
        }
        OutputFormat::Responses => {
            let request = ResponsesRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                input: serde_json::Value::String(String::from(prompt)),
                instructions: None,
                temperature: None,
                stream: false,
                ..ResponsesRequest::default()
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_response_with_solver(&request, &solver))?
            );
        }
    }

    Ok(())
}
