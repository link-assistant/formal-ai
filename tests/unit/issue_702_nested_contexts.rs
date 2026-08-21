//! Issue #702: nested symbolic contexts and lazy reference resolution.
//!
//! The dialogue world model is also the shared substrate for task, pull-request,
//! issue, repository, and organization scopes. These tests pin the review
//! contract: arbitrary depth, explicit inheritance, nearest-first lookup, and
//! an explicit boundary before any external search.

use formal_ai::LinkPattern;
use formal_ai::agentic_coding::learning_report::context_hierarchy_learning;
use formal_ai::agentic_coding::{
    CONTEXT_HIERARCHY_LEARNING_PATH, CONTEXT_HIERARCHY_LEARNING_TASK, run_agentic_task,
};
use formal_ai::solver::{ConversationRole, ConversationTurn, UniversalSolver};
use formal_ai::world_model::Context;
use formal_ai::world_model::{GeneralMemoryCommitError, GeneralMemoryPermission, WorldModel};
use formal_ai::world_model_context::{
    ContextHierarchy, ExternalLookup, InheritancePolicy, ReferenceResolutionKind,
};
use formal_ai::world_model_dialog::DialogueWorldModel;
use lino_objects_codec::format::parse_indented;

const REAL_AGENT_CLI_TASK: &str = "Use Formal AI auto-learning to inspect the persisted issue 702 nested-context failures as an associative links network, rank the hierarchy observations and amendments, keep promotion human-review gated, and write context-hierarchy-learning-report.lino.";

fn context(id: &str, links: &[(&str, &str)]) -> Context {
    let mut context = Context::new(id);
    for (from, to) in links {
        context.assert_link(from, to);
    }
    context
}

#[test]
fn reference_resolution_has_no_fixed_context_depth_limit_and_is_lazy() {
    let mut hierarchy = ContextHierarchy::new();
    hierarchy
        .insert(context("scope:0", &[("subject", "root")]))
        .expect("root context");

    for depth in 1..=320 {
        let links = if depth == 193 {
            vec![("subject", "nearest")]
        } else {
            Vec::new()
        };
        hierarchy
            .nest(
                context(&format!("scope:{depth}"), &links),
                &format!("scope:{}", depth - 1),
                InheritancePolicy::Full,
            )
            .expect("nested context");
    }

    let resolved = hierarchy
        .resolve("scope:320", "subject", ExternalLookup::Denied)
        .expect("resolution");
    assert_eq!(resolved.kind, ReferenceResolutionKind::Resolved);
    assert_eq!(resolved.context_id.as_deref(), Some("scope:193"));
    assert_eq!(resolved.depth, Some(127));
    assert_eq!(resolved.links[0].to, "nearest");
    assert_eq!(
        resolved.visited.len(),
        128,
        "lazy lookup must stop at the nearest definition"
    );
    assert!(
        !resolved.visited.iter().any(|id| id == "scope:0"),
        "a resolved reference must not scan irrelevant outer scopes"
    );
}

#[test]
fn local_links_shadow_fully_inherited_links() {
    let mut hierarchy = ContextHierarchy::new();
    hierarchy
        .insert(context("repository", &[("owner", "organization")]))
        .expect("root context");
    hierarchy
        .nest(
            context("issue", &[("owner", "issue-team")]),
            "repository",
            InheritancePolicy::Full,
        )
        .expect("issue context");

    let resolved = hierarchy
        .resolve("issue", "owner", ExternalLookup::Denied)
        .expect("resolution");
    assert_eq!(resolved.context_id.as_deref(), Some("issue"));
    assert_eq!(resolved.depth, Some(0));
    assert_eq!(resolved.links[0].to, "issue-team");
    assert_eq!(resolved.visited, ["issue"]);
}

#[test]
fn isolated_contexts_stop_inheritance_and_gate_external_lookup() {
    let mut hierarchy = ContextHierarchy::new();
    hierarchy
        .insert(context("repository", &[("release", "0.306.0")]))
        .expect("root context");
    hierarchy
        .nest(
            Context::new("private-task"),
            "repository",
            InheritancePolicy::Isolated,
        )
        .expect("isolated context");

    let local_only = hierarchy
        .resolve("private-task", "release", ExternalLookup::Denied)
        .expect("local resolution");
    assert_eq!(local_only.kind, ReferenceResolutionKind::Unresolved);
    assert_eq!(local_only.visited, ["private-task"]);

    let externally_allowed = hierarchy
        .resolve("private-task", "release", ExternalLookup::Allowed)
        .expect("external resolution decision");
    assert_eq!(
        externally_allowed.kind,
        ReferenceResolutionKind::ExternalLookupRequired
    );
    assert!(
        externally_allowed.links.is_empty(),
        "the resolver is a policy boundary; it never fabricates outside facts"
    );
}

#[test]
fn conditional_inheritance_filters_links_at_every_boundary() {
    let fact_pattern = LinkPattern::parse("fact:$name -> $value").expect("fact pattern");
    let public_pattern =
        LinkPattern::parse("fact:public_$name -> $value").expect("public fact pattern");

    let mut hierarchy = ContextHierarchy::new();
    hierarchy
        .insert(context(
            "organization",
            &[
                ("fact:public_policy", "retained"),
                ("fact:private_token", "hidden"),
                ("secret", "never-inherit"),
            ],
        ))
        .expect("organization context");
    hierarchy
        .nest(
            Context::new("repository"),
            "organization",
            InheritancePolicy::Conditional(vec![fact_pattern]),
        )
        .expect("repository context");
    hierarchy
        .nest(
            Context::new("issue"),
            "repository",
            InheritancePolicy::Conditional(vec![public_pattern]),
        )
        .expect("issue context");

    let public = hierarchy
        .resolve("issue", "fact:public_policy", ExternalLookup::Denied)
        .expect("public lookup");
    assert_eq!(public.kind, ReferenceResolutionKind::Resolved);
    assert_eq!(public.context_id.as_deref(), Some("organization"));

    for hidden in ["fact:private_token", "secret"] {
        let resolution = hierarchy
            .resolve("issue", hidden, ExternalLookup::Denied)
            .expect("hidden lookup");
        assert_eq!(
            resolution.kind,
            ReferenceResolutionKind::Unresolved,
            "{hidden} must satisfy every conditional inheritance boundary"
        );
    }
}

#[test]
fn hierarchy_rejects_parent_cycles() {
    let mut hierarchy = ContextHierarchy::new();
    hierarchy
        .insert(Context::new("organization"))
        .expect("organization context");
    hierarchy
        .nest(
            Context::new("repository"),
            "organization",
            InheritancePolicy::Full,
        )
        .expect("repository context");

    assert!(
        hierarchy
            .set_parent("organization", "repository", InheritancePolicy::Full)
            .is_err(),
        "scope inheritance must remain acyclic"
    );
}

#[test]
fn nested_context_trace_uses_links_notation_without_graph_jargon() {
    let mut hierarchy = ContextHierarchy::new();
    hierarchy
        .insert(context("repository", &[("language", "rust")]))
        .expect("repository context");
    hierarchy
        .nest(Context::new("task"), "repository", InheritancePolicy::Full)
        .expect("task context");

    let resolution = hierarchy
        .resolve("task", "language", ExternalLookup::Denied)
        .expect("resolution");
    let trace = resolution.links_notation();
    assert!(trace.contains("reference_resolution"));
    assert!(trace.contains("resolved_in \"repository\""));
    assert!(trace.contains("visited \"task|repository\""));
    assert!(!trace.contains("vertex"));
    assert!(!trace.contains("edge"));
}

#[test]
fn context_traces_quote_arbitrary_link_values_with_the_links_notation_codec() {
    let context_id = "task:\"quoted\" and 'single'";
    let reference = "requirement:\"exact\" and 'literal'";
    let mut hierarchy = ContextHierarchy::new();
    hierarchy
        .insert(context(context_id, &[(reference, "value:\"kept\"")]))
        .expect("quoted context");

    let resolution = hierarchy
        .resolve(context_id, reference, ExternalLookup::Denied)
        .expect("quoted resolution");
    parse_indented(&resolution.links_notation()).expect("resolution must be valid Links Notation");
    parse_indented(&hierarchy.links_notation()).expect("hierarchy must be valid Links Notation");
}

#[test]
fn dialogue_state_requires_explicit_permission_before_entering_general_memory() {
    let mut model = WorldModel::new();
    model.current.assert_link("private_fact", "dialogue_only");

    let denied = model.commit_current_to_general(GeneralMemoryPermission::Denied);
    assert_eq!(denied, Err(GeneralMemoryCommitError::PermissionDenied));
    assert!(
        !model.general().holds("private_fact", "dialogue_only"),
        "a denied promotion must leave general memory unchanged"
    );

    model
        .commit_current_to_general(GeneralMemoryPermission::Allowed)
        .expect("explicitly allowed promotion");
    assert!(model.general().holds("private_fact", "dialogue_only"));
}

#[test]
fn dialogue_turns_are_real_nested_contexts_not_only_a_standalone_utility() {
    let mut dialogue = DialogueWorldModel::new();
    dialogue.observe_user("the door is closed");
    dialogue.observe(ConversationRole::Assistant, "I recorded that state.");
    dialogue.observe_user("the table is sturdy");

    let resolved = dialogue
        .resolve_reference("door", ExternalLookup::Denied)
        .expect("dialogue scope resolution");
    assert_eq!(resolved.kind, ReferenceResolutionKind::Resolved);
    assert_eq!(resolved.context_id.as_deref(), Some("dialogue:turn:1"));
    assert_eq!(resolved.links[0].to, "closed");
    assert_eq!(
        dialogue
            .context_hierarchy()
            .parent("dialogue:turn:3")
            .map(|parent| (
                parent.parent_id.as_str(),
                parent.policy == InheritancePolicy::Full
            )),
        Some(("dialogue:turn:2", true))
    );
}

#[test]
fn runtime_coreference_searches_nearest_relevant_ancestor_turn() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("I love Rust."),
        ConversationTurn::assistant("Rust is a systems programming language."),
        ConversationTurn::user("The weather is pleasant."),
        ConversationTurn::assistant("It is."),
    ];

    let response = solver.solve_with_history("Why is it safer than C?", &history);
    assert_eq!(response.intent, "coreference_rust");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("context_resolution:")),
        "runtime use must expose the inherited-scope decision: {:?}",
        response.evidence_links
    );
}

#[test]
fn rust_and_browser_coding_inheritance_are_cycle_safe_not_depth_capped() {
    let rust = include_str!("../../src/solver_handlers/numeric_list/codegen.rs");
    let browser = include_str!("../../src/web/worker/formal_ai_worker_07.js");

    assert!(!rust.contains("MAX_INHERITANCE_DEPTH"));
    assert!(rust.contains("seen.insert"));
    assert!(!browser.contains("CODING_IDIOMS_MAX_INHERITANCE_DEPTH"));
    assert!(browser.contains("seen.has"));
}

#[test]
fn nested_context_auto_learning_is_derived_and_review_gated() {
    let baseline = include_str!("../../data/meta/issue-702-context-hierarchy-learning.lino");
    let changed = baseline.replace("accessCount \"8\"", "accessCount \"18\"");
    let first = context_hierarchy_learning::render_document_from(baseline);
    let second = context_hierarchy_learning::render_document_from(&changed);

    assert_ne!(first, second, "the ranking must be derived from memory");
    assert!(first.contains("decision \"awaiting_human_review\""));
    assert!(first.contains("promotion_gate \"nested_context_runtime_and_parity_fixtures_pass\""));
    assert!(first.contains("lesson:shared-context-hierarchy"));
    assert!(first.contains("lesson:lazy-nearest-first"));
    assert!(first.contains("lesson:explicit-inheritance"));
    assert!(first.contains("lesson:cycle-safe-unbounded"));
}

#[test]
fn formal_ai_executes_the_nested_context_learning_task() {
    assert!(REAL_AGENT_CLI_TASK.contains(CONTEXT_HIERARCHY_LEARNING_TASK));
    let outcome = run_agentic_task(REAL_AGENT_CLI_TASK).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], CONTEXT_HIERARCHY_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        context_hierarchy_learning::render_document()
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}

#[test]
fn real_agent_cli_learning_artifact_is_byte_reproducible() {
    let committed = include_str!(
        "../../dev/log/issues/702/pulls/818/agent-cli/context-hierarchy-learning-report.lino"
    );
    assert_eq!(committed, context_hierarchy_learning::render_document());
}
