//! Policy gates extracted from `solver.rs` (issue #1138, plan 09 migration
//! ratchet): the refusal and confirmation surfaces that run before the
//! specialized handlers. Every gate here is additive — it either answers the
//! prompt with a policy intent or returns `None` so dispatch continues.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::Language;
use crate::seed;
use crate::solver_config::ExecutionSurface;
use crate::solver_handlers::{finalize_simple, try_agent_workspace_task};
use crate::solver_helpers::{
    is_agent_opt_in, is_agent_request, is_cache_flush_request, is_destructive_action,
    is_forget_request, is_inappropriate_content, is_unbounded_autonomy, is_unbounded_loop,
};

/// The policy gates for one prompt: inappropriate content, unbounded autonomy
/// without an opt-in, forget and cache-flush requests, and the agent-mode
/// confirmations. `execution_surface` is the caller's surface, because the
/// HTTP server must stay declarative where the CLI may execute.
pub fn try_policy_gates(
    execution_surface: ExecutionSurface,
    prompt: &str,
    log: &mut EventLog,
    language: Language,
) -> Option<SymbolicAnswer> {
    let normalized = prompt.to_lowercase();

    if is_inappropriate_content(&normalized) {
        log.append("policy:inappropriate_content", prompt.to_owned());
        let lang_slug = language.slug();
        let fallback = "That message contains inappropriate content. Please keep the conversation respectful.";
        let body = seed::response_for("inappropriate_content", lang_slug)
            .unwrap_or_else(|| String::from(fallback));
        return Some(finalize_policy(
            prompt,
            log,
            "inappropriate_content",
            &body,
        ));
    }

    if is_unbounded_autonomy(&normalized) && !is_agent_opt_in(&normalized) {
        log.append("policy:chat_bounded_autonomy", prompt.to_owned());
        return Some(finalize_policy(
            prompt,
            log,
            "bounded_autonomy",
            concat!(
                "I can only run a bounded chat reply per message. To take repeated, ",
                "open-ended actions I need an explicit opt-in to agent mode, and agent ",
                "mode runs in an isolated sandbox so the host stays safe."
            ),
        ));
    }

    if is_forget_request(&normalized) {
        log.append("policy:add_only_history", prompt.to_owned());
        return Some(finalize_policy(
            prompt,
            log,
            "add_only_history",
            concat!(
                "The link network is append-only. To retract a fact, send the explicit ",
                "retraction protocol; it will append a superseding event without erasing ",
                "history."
            ),
        ));
    }

    if is_cache_flush_request(&normalized) {
        log.append(
            "policy:cache_flush_requires_confirmation",
            prompt.to_owned(),
        );
        return Some(finalize_policy(
            prompt,
            log,
            "cache_flush_requires_confirmation",
            "Flushing the source cache is an auditable action. Confirm explicitly.",
        ));
    }

    if is_agent_request(&normalized) && is_destructive_action(&normalized) {
        log.append("agent_mode:opted_in", prompt.to_owned());
        log.append(
            "policy:destructive_action_requires_confirmation",
            prompt.to_owned(),
        );
        return Some(finalize_policy(
            prompt,
            log,
            "destructive_action_requires_confirmation",
            concat!(
                "Destructive agent actions require an explicit human confirmation. ",
                "The action will run inside an isolated sandbox once confirmed."
            ),
        ));
    }

    if is_agent_request(&normalized) && is_unbounded_loop(&normalized) {
        log.append("agent_mode:opted_in", prompt.to_owned());
        log.append("policy:agent_time_budget", prompt.to_owned());
        return Some(finalize_policy(
            prompt,
            log,
            "agent_time_budget",
            concat!(
                "Agent execution is bounded by a documented time budget; unbounded ",
                "loops are refused. Re-send a bounded version inside an isolated sandbox."
            ),
        ));
    }

    if is_agent_request(&normalized) {
        // The HTTP surface is embedded in an agentic CLI harness. Executing
        // here would mutate the server's private temporary workspace while
        // the caller sees no tool call and cannot audit or approve it. API
        // requests therefore stay declarative; `protocol` routes concrete
        // actions through the tools advertised by the client.
        if execution_surface != ExecutionSurface::HttpServer
            && let Some(answer) = try_agent_workspace_task(prompt, &normalized, log)
        {
            return Some(answer);
        }
        log.append("agent_mode:opted_in", prompt.to_owned());
        log.append("agent_mode:active", prompt.to_owned());
        log.append("action_log", prompt.to_owned());
        return Some(finalize_policy(
            prompt,
            log,
            "agent_action",
            concat!(
                "Agent mode is opted in for this message. The action will run inside ",
                "an isolated sandbox (docker, webvm or sandbox-equivalent) and every ",
                "step will be appended to the action log."
            ),
        ));
    }

    None
}

fn finalize_policy(
    prompt: &str,
    log: &mut EventLog,
    intent_slug: &str,
    body: &str,
) -> SymbolicAnswer {
    let intent = format!("policy_{intent_slug}");
    let response_link = format!("response:policy:{intent_slug}");
    finalize_simple(prompt, log, &intent, &response_link, body, 0.5)
}
