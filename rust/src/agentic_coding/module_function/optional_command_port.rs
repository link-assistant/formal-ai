//! Read-only optional-check provenance owned by the module-function planner.
use crate::agentic_coding::planner::{AgenticPlan, Capability};
use crate::engine::{ExecutionRecipe, SymbolicAnswer};
use crate::protocol::ChatMessage;

/// A borrowed source-owner witness, never a command permission or policy issuer.
pub struct OptionalCommandPlan<'a> {
    plan: &'a AgenticPlan,
    task: &'a str,
    tools: &'a [&'a str],
    messages: &'a [ChatMessage],
    recipe: &'a ExecutionRecipe,
    command: String,
}
impl<'a> OptionalCommandPlan<'a> {
    pub(super) fn from_source_plan(
        plan: &'a AgenticPlan,
        task: &'a str,
        tools: &'a [&'a str],
        messages: &'a [ChatMessage],
        recipe: &'a ExecutionRecipe,
    ) -> Option<Self> {
        let catalog = crate::coding::program_language_by_slug(&recipe.language)?;
        let command = catalog
            .execution
            .check_command
            .as_deref()?
            .replace(catalog.save_as.as_ref(), &recipe.path);
        if recipe.commands.first() != Some(&command)
            || !crate::agentic_coding::command_reroute::optional_check_stage(
                messages, tools, recipe,
            )
        {
            return None;
        }
        let AgenticPlan::ToolCalls(calls) = plan else {
            return None;
        };
        let call = calls.first()?;
        if calls.len() != 1
            || Some(call.tool.as_str())
                != crate::agentic_coding::capability_router::tool_for(tools, Capability::Run)
            || call.arguments != serde_json::json!({"command": command}).to_string()
        {
            return None;
        }
        Some(Self {
            plan,
            task,
            tools,
            messages,
            recipe,
            command,
        })
    }
    /// The actual catalog-derived inferred command, not a policy decision.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }
    /// The unchanged actual owner-produced plan.
    #[must_use]
    pub fn original_plan(&self) -> &AgenticPlan {
        self.plan
    }
    /// Recompute from the immutable original borrowed history without issuing a receipt.
    pub fn without_inferred_check(&self) -> Option<AgenticPlan> {
        let mut recipe = self.recipe.clone();
        recipe.commands.remove(0);
        crate::agentic_coding::command_reroute::plan_symbolic_command_reroute(
            self.messages,
            self.tools,
            &SymbolicAnswer {
                intent: "module_function".to_owned(),
                answer: String::new(),
                confidence: 1.0,
                evidence_links: Vec::new(),
                thinking_steps: Vec::new(),
                links_notation: String::new(),
                execution_recipe: Some(Box::new(recipe)),
            },
        )
    }
}
/// Consume only an exact source-owner witness and its original immutable inputs.
pub fn source_owned_optional_command<'w, 'a>(
    owned: &'w OptionalCommandPlan<'a>,
    plan: &AgenticPlan,
    task: &str,
    tools: &[&str],
    messages: &[ChatMessage],
) -> Option<&'w OptionalCommandPlan<'a>> {
    (std::ptr::eq(owned.plan, plan)
        && owned.task == task
        && std::ptr::eq(owned.tools, tools)
        && std::ptr::eq(owned.messages, messages))
    .then_some(owned)
}
