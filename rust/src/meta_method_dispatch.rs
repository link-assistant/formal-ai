//! Registry-backed method execution for the universal solver.
//!
//! The meta core owns method selection: an impulse is formalized, its
//! `route:`/`handler:` relevants are resolved through [`MethodRegistry`], and the
//! ordered method names are executed here. Handler functions remain ordinary Rust
//! implementations, but they are no longer selected by a separate hardcoded loop
//! in `solver.rs`.

use std::fmt::Write as _;

use crate::capability_routing::{Act, ObjectType, RoutingOutcome};
use crate::engine::{SymbolicAnswer, answer_links_notation};
use crate::event_log::{EventLog, build_evidence_links};
use crate::intent_formalization::IntentFormalization;
use crate::meta_method_answers::{
    capability_gap, established_response_language, explain_previous_turn,
    response_language_demonstration, try_response_language_variant,
};
use crate::method_registry::{
    MethodExecution, MethodRegistry, MethodRuntimeKind, ProjectLookupPhase,
};
use crate::proof_engine::ProofRenderConfig;
use crate::solver::{ConversationTurn, SolverConfig, UniversalSolver};
use crate::solver_diagnostics::append_diagnostic_trace;
use crate::solver_dispatch::{
    ContextualOutcome, ContextualRuntime, PRELUDE_METHOD_NAMES, handler_for_method,
    try_contextual_override,
};
use crate::solver_handlers::{
    CapabilityRuntime, SelfAwarenessRuntime, try_behavior_rules_with_runtime,
    try_explicit_repository_lookup, try_feature_capability, try_learn_from_source,
    try_natural_language_tool_request, try_playwright_script, try_project_lookup,
    try_project_lookup_with_response_language, try_response_language_followup,
    try_routed_calendar_create_event, try_routed_http_fetch_with_offline,
};

/// Execute the single registry-backed method-selection path.
pub fn try_dispatch(
    solver: &UniversalSolver,
    prompt: &str,
    intent_formalization: &IntentFormalization,
    history: &[ConversationTurn],
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let normalized = prompt.to_lowercase();
    let registry = MethodRegistry::shared();
    let promoted_methods: Vec<String> = intent_formalization
        .relevants
        .iter()
        .filter_map(|relevant| {
            let route = relevant
                .strip_prefix("route:")
                .or_else(|| relevant.strip_prefix("handler:"))?;
            registry
                .method_for_route(route)
                .map(|method| method.name.clone())
        })
        .collect();
    let method_names = registry.ordered_method_names_for_relevants(&intent_formalization.relevants);
    let runtime = MethodRuntime::new(solver.config);
    let mut capability_route_checked = false;

    // Issue #556: a response-language follow-up replays the previous request
    // through the whole solver with this language forced onto every localizable
    // answer family, so the retarget generalizes beyond a single handler.
    //
    // Plan 10 leaf 15 (issue #724): a language the conversation has already
    // established binds the same way. The user names it once -- a routed
    // demonstration, a retarget, a plain request -- and the conversation keeps
    // speaking it until they name another; an explicit per-run forcing in the
    // config is the stronger statement and wins.
    let forced_response_language = solver
        .config
        .forced_response_language
        .or_else(|| established_response_language(history));
    for name in method_names {
        let execution = registry.execution_for(&name);
        // Prelude methods keep first refusal: they include explicit natural-
        // language tool requests and policy-like assistant capabilities that
        // already own complete recipes.  The shared capability table then
        // chooses the first regular method, before a promoted web-search
        // handler can reinterpret a URL, path, clock, language or local scope.
        if !capability_route_checked && !PRELUDE_METHOD_NAMES.contains(&name.as_str()) {
            capability_route_checked = true;
            if let Some(answer) =
                try_capability_route(solver, prompt, history, &promoted_methods, log)
            {
                return Some(answer);
            }
        }
        if let Some(answer) = crate::family_method::try_family_method_preempting(
            prompt,
            &normalized,
            history,
            log,
            &name,
            solver.config,
        ) {
            return Some(answer);
        }
        if execution.project_lookup == Some(ProjectLookupPhase::BeforeRuntime)
            && let Some(answer) = try_explicit_repository_lookup(
                prompt,
                &normalized,
                log,
                solver.config.associative_project_promotion,
                intent_formalization.route.as_deref() == Some("identity"),
                forced_response_language,
            )
        {
            return Some(record_method_answer(prompt, log, answer, "project_lookup"));
        }
        if let Some(answer) =
            try_attributed_runtime(solver, execution, &name, prompt, &normalized, log, runtime)
        {
            return Some(answer);
        }
        if solver.config.definition_fusion_by_default
            && execution.definition_fusion
            && let Some(answer) = crate::definition_merge::merge_definitions_by_default(prompt, log)
        {
            return Some(record_method_answer(
                prompt,
                log,
                answer,
                "definition_merge_by_default",
            ));
        }
        // Family interpreters replace the generic unknown/concept fallback,
        // not the grounded methods ordered ahead of it.  This data-selected
        // boundary preserves richer history, source, policy, and execution
        // semantics while still letting a family claim requests none of those
        // methods could solve.
        if execution.project_lookup == Some(ProjectLookupPhase::Fallback)
            && let Some(answer) = crate::family_method::try_family_method(
                prompt,
                &normalized,
                history,
                log,
                solver.config,
            )
        {
            return Some(answer);
        }
        match try_contextual_override(
            &name,
            prompt,
            &normalized,
            history,
            ContextualRuntime::new(
                runtime.proof_render_config,
                runtime.self_awareness_runtime,
                solver.config,
            ),
            log,
        ) {
            ContextualOutcome::Answer(answer) => {
                return Some(record_contextual_method_answer(prompt, log, answer, &name));
            }
            ContextualOutcome::Skip => continue,
            ContextualOutcome::NotHandled => {}
        }
        // Issue #556: when a language is forced, route the concept-lookup family
        // through its response-language variant so a replayed definitional
        // request re-renders in the requested language before the plain handler
        // (which localizes only to the detected prompt language) can claim it.
        if let Some(language) = forced_response_language
            && let Some(answer) = try_response_language_variant(
                execution.response_language,
                prompt,
                &normalized,
                log,
                language,
            )
        {
            return Some(record_method_answer(prompt, log, answer, &name));
        }
        // Issue #1138 B7, plan 07 leaf 6: a learned name is executed as the
        // recipe program its operations project onto, instead of through
        // `handler_for_method` -- there is no compiled handler behind it. This
        // is `MethodRegistry::learned_method`'s first production caller.
        if let Some(answer) =
            try_learned_method(solver, registry, &name, prompt, intent_formalization, log)
        {
            return Some(answer);
        }
        if let Some(handler) = handler_for_method(&name)
            && let Some(answer) = handler.call(prompt, &normalized, log)
        {
            return Some(record_method_answer(prompt, log, answer, &name));
        }
        if execution.project_lookup == Some(ProjectLookupPhase::Fallback) {
            let answer = if let Some(language) = forced_response_language {
                try_project_lookup_with_response_language(
                    prompt,
                    prompt,
                    log,
                    solver.config.associative_project_promotion,
                    intent_formalization.route.as_deref() == Some("identity"),
                    language,
                )
            } else {
                try_project_lookup(
                    prompt,
                    &normalized,
                    log,
                    solver.config.associative_project_promotion,
                    intent_formalization.route.as_deref() == Some("identity"),
                )
            };
            if let Some(answer) = answer {
                return Some(record_method_answer(prompt, log, answer, "project_lookup"));
            }
        }
    }
    None
}

/// Capabilities the symbolic solver can execute without a client-owned
/// workspace tool.  Filesystem capabilities are deliberately absent: when the
/// table selects one, its `HonestGap` is surfaced instead of allowing a later
/// web-search handler to claim the request.
const SOLVER_CAPABILITIES: &[&str] = &[
    "web_fetch",
    "learn_from_source",
    "web_search",
    "calendar_create_event",
    "response_language_demonstration",
    "concept_measurement_lookup",
    "compose_from_sources",
    "explain_previous_turn",
    "report_issue",
    "ask_user",
];

/// Execute the object/act/locus decision on the non-agent solver surface.
///
/// This is intentionally a dispatcher, not another recognizer.  All lexical
/// interpretation happened in `capability_routing`; this function records the
/// selected axes and either invokes the matching runtime capability or reports
/// that the chat surface lacks the client-owned tool it needs.
fn try_capability_route(
    solver: &UniversalSolver,
    prompt: &str,
    history: &[ConversationTurn],
    promoted_methods: &[String],
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // A request for unbounded autonomy is the bounded-autonomy policy's to
    // answer, whatever object/act/locus triple the table reads off it.
    // Before the word-boundary fix (issue #1138 plan 03 wave F) such a
    // prompt reached that policy only through a substring false positive
    // ("improve" embedding "prove") promoting a proof handler; with the
    // false positive gone, the table must decline on its own instead of
    // answering a grep capability gap to "improve my codebase forever".
    // An explicit agent opt-in is likewise the agent flow's to serve: the
    // table routes chat-surface requests, and an opted-in agent request
    // never becomes a chat capability gap (#1138).
    let normalized = prompt.to_lowercase();
    if crate::solver_helpers::is_unbounded_autonomy(&normalized)
        && !crate::solver_helpers::is_agent_opt_in(&normalized)
    {
        return None;
    }
    if crate::solver_helpers::is_agent_request(&normalized) {
        return None;
    }
    // A destructive request without an agent opt-in is the bounded-autonomy
    // policy's to answer, whatever object/act/locus triple the table reads
    // off it. The seed's `shell_refusal` rule (rank 650) and the Rust policy
    // gates both speak for it, so routing it reports a capability gap where a
    // guard answers -- the same boundary the autonomy decline above draws
    // (issue #1138: `Please run rm -rf ... on my behalf` lost its refusal to
    // the write gap the table read off the command).
    if crate::solver_helpers::is_destructive_action(&normalized)
        && !crate::solver_helpers::is_agent_opt_in(&normalized)
    {
        return None;
    }
    // A conversion between units the lexicon places in distinct physical
    // dimensions (meters of length, kilobytes of data storage) is the unit
    // specialist's to answer, whatever object/act/locus triple the table reads
    // off it. The quantity rows exist so a magnitude question reaches its
    // measured property; a pair of units that cannot convert has none, and
    // routing it to a web capability reports a capability gap where the honest
    // answer is the recorded incompatibility. Same shape as the autonomy
    // policy above: the table declines on its own instead of a specialist
    // having to win a race it no longer runs (issue #1138).
    if crate::solver_handler_units::names_incompatible_unit_pair(&normalized) {
        return None;
    }
    // Advice to open a URL in a tab is the `url_navigate` row's request. The
    // routed fetch and the compose arm below deliberately do not re-ask a verb
    // recognizer, so they would answer it here; the navigation recognizer is
    // asked instead, and what it claims the table declines -- the row one rank
    // behind this intercept keeps the request, exactly as the ladder pinned it
    // (issue #1138: `Открой страницу github.com` lost its tab advice to the
    // compose arm).
    if crate::solver_handlers::url_navigation_claims(prompt, &normalized) {
        return None;
    }
    // A text operation the manipulation handler recognizes is its turn: the
    // table reads "title case this text: ..." as a write gap, but the edit
    // answers on the chat surface itself, so the table declines and method
    // dispatch claims the edit (issue #1138: text operations).
    if crate::solver_handlers::names_text_operation(&normalized) {
        return None;
    }
    if !crate::capability_routing::table_routing_enabled() {
        return None;
    }
    let decision = crate::capability_routing::route_decision(prompt, SOLVER_CAPABILITIES);
    if !solver_route_is_authoritative(prompt, &decision) {
        return None;
    }

    // A specifically promoted interpreter keeps the request it already
    // grounded.  The table replaces generic fetch/search/clarification
    // precedence that caused #745/#1138's cross-tool misroutes; it does not
    // interrupt translation, text manipulation, installation conversion, or
    // any other complete recipe merely because that recipe mentions a path or
    // a language.  The promotions pipeline hoists every recipe whose
    // conditions fire, and a false promotion is ordinary -- a repo slug
    // promotes `book` as a scheduling act, an artifact word promotes a
    // project reading -- so the table may answer only when it preempts each
    // promoted recipe, not merely the first: the walk behind this intercept
    // still asks every hoisted handler in rank order, and a handler promoted
    // by a false positive declines when actually asked (issue #1138: the
    // trimstray conversion and the Russian pdf plan both lost to the gap the
    // table read off their artifact words behind a falsely promoted
    // preempting handler).
    if !promoted_methods.is_empty()
        && promoted_methods
            .iter()
            .any(|method| !capability_route_preempts(method, &decision))
    {
        return None;
    }

    let (capability, unavailable) = match &decision.outcome {
        RoutingOutcome::Routed { capability } => (capability.as_str(), None),
        RoutingOutcome::Lowered {
            preferred,
            capability,
        } => (capability.as_str(), Some(preferred.as_str())),
        RoutingOutcome::HonestGap { needed, missing } => {
            // An agent client that advertises the workspace tools itself must
            // reach the planner, never dead-end in a refusal (issue #671:
            // `read the file alpha.txt`). That is a statement about tool-bearing
            // requests, which the protocol layer answers before the solver:
            // a prose-only request over the HTTP agent surface carries no tools,
            // falls through to the solver, and the gap refusal naming the
            // requested path is the honest answer it is owed. Restricting the
            // decline to surfaces without a workspace keeps both truths.
            if solver.config.agent_mode
                && solver.config.execution_surface != crate::ExecutionSurface::HttpServer
            {
                return None;
            }
            // A GitHub repository-traffic visibility question is that seeded
            // handler's turn: the table reads "who visited my repository" as
            // workspace retrieval and reports a grep gap, burying the
            // platform's aggregate-traffic answer (issue #497). The four-role
            // recognition is the handler's own, read from the same lexicon.
            if github_repository_traffic_claims(&normalized) {
                return None;
            }
            // An elliptical scheduling prompt the promotion hoisted and the
            // calendar gate accepts is the calendar handler's: the table
            // read a search act off the modal verb, but the clock hour,
            // timezone and participant entities ground the event without
            // the web (issue #595). A routed calendar decision still wins
            // above; only the gap declines.
            if promoted_calendar_claims(promoted_methods, &normalized) {
                return None;
            }
            // A shell task the semantic terminal recognizer can already name
            // is the terminal suggestion's: the table reads "check which
            // processes are running" as a list_dir gap, while the seeded
            // process lexicon answers with the agent suggestion that names
            // the command — in every language the table has no row for
            // (issue #870 parity).
            if crate::agentic_coding::semantic_shell_command_for_task(prompt).is_some() {
                return None;
            }
            // A software-authoring request the lexicon recognizes (authoring
            // verb plus software artifact) is the software-project handler's:
            // the table reads "imports customer records" as a file-read gap,
            // but a scaffolded build is a plan, not a read (issue #1138
            // software-project corpus).
            if crate::solver_handlers::software_project_claims(&normalized) {
                return None;
            }
            if let Some(answer) = crate::family_method::try_family_method_preempting(
                prompt,
                &normalized,
                history,
                log,
                "capability_gap",
                solver.config,
            ) {
                return Some(answer);
            }
            record_capability_route(log, decision.object, decision.act, decision.locus, needed);
            return Some(capability_gap(prompt, log, needed, missing));
        }
        RoutingOutcome::Ask { .. } => return None,
    };
    // Capability routing and handler-family routing are both data-selected
    // interpretations of the same request.  Let the family catalog arbitrate
    // a routed capability by its declared `preempts` relation before the
    // capability executes.  Without this shared boundary, adding a capability
    // can silently steal an older family's requests merely because capability
    // routing runs earlier in the dispatcher.
    if let Some(answer) = crate::family_method::try_family_method_preempting(
        prompt,
        &normalized,
        history,
        log,
        capability,
        solver.config,
    ) {
        return Some(answer);
    }
    record_capability_route(
        log,
        decision.object,
        decision.act,
        decision.locus,
        capability,
    );
    if let Some(preferred) = unavailable {
        log.append_fields(
            "capability_route:lowered",
            &[("preferred", preferred), ("capability", capability)],
        );
    }

    let live_fetch = std::env::var("FORMAL_AI_LIVE_FETCH").is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    });
    match capability {
        "web_fetch" => {
            try_routed_http_fetch_with_offline(prompt, log, solver.config.offline || !live_fetch)
        }
        // Issue #499: a learning directive that names a URL ("learn from … at
        // https://…") asks the engine to adopt from a declared source, not to
        // fetch the URL once, so the learn act outranks the retrieve default
        // and the row lands here. The handler stays gated on the
        // seed-declared learnable-source registry, so a directive the registry
        // declines falls through to the specialized walk unchanged.
        "learn_from_source" => try_learn_from_source(prompt, &normalized, log),
        "calendar_create_event" => try_routed_calendar_create_event(prompt, &normalized, log),
        "response_language_demonstration" => {
            if let Some(answer) =
                try_response_language_followup(prompt, &normalized, log, history, solver.config)
            {
                return Some(record_contextual_method_answer(
                    prompt,
                    log,
                    answer,
                    "response_language_followup",
                ));
            }
            response_language_demonstration(prompt, &normalized, log)
        }
        "explain_previous_turn" => {
            // With no assistant turn behind it the request keeps the honest
            // empty-turn notice the executor states in prose ("There is no
            // previous assistant turn available to explain yet") -- declining
            // here instead lands the comprehension failure in the generic
            // unknown refusal, which answers nothing (issue #1138: the
            // ladder's non-understanding node measured the decline).
            // Issue #556 first: re-rendering the previous turn must not steal
            // an explicit response-language retarget OF that same previous
            // turn ("I don't understand English, write in Hindi" is both a
            // comprehension failure and a retarget; the retarget is the
            // request). Without a marker the act re-renders as usual.
            if let Some(answer) =
                try_response_language_followup(prompt, &normalized, log, history, solver.config)
            {
                return Some(record_contextual_method_answer(
                    prompt,
                    log,
                    answer,
                    "response_language_followup",
                ));
            }
            Some(explain_previous_turn(prompt, log))
        }
        "compose_from_sources" | "concept_measurement_lookup" => {
            // A quantity arithmetic computes — a duration between two clock
            // times, a same-dimension unit conversion — is the calculator's
            // turn, not a measured property a source can look up. The
            // quantity rows read the same interrogative words ("how long",
            // "是多少"), and executing a source capability for them reports a
            // measurement miss where the calculator answers, so the table
            // declines before either arm runs (issue #1138 specification:
            // the calculator-delegation family).
            if crate::solver_handler_units::names_arithmetic_quantity(&normalized) {
                return None;
            }
            // A summarization seed trigger is the summarization method's turn:
            // that handler derives its answer from the concept or project
            // record the trigger references, and composing from sources for
            // it buries the derivation under a fetch the chat surface cannot
            // make (issue #1138: summarization routing).
            if crate::seed::summary_topic_seeds().matches_trigger(&normalized) {
                return None;
            }
            // A text operation the manipulation handler recognizes ("count
            // occurrences of", "title case this text:") is that handler's
            // turn, whatever object/act/locus triple the table reads off it:
            // composing or routing it to a write capability reports a gap
            // where the edit itself answers (issue #1138: the text-operation
            // family).
            if crate::solver_handlers::names_text_operation(&normalized) {
                return None;
            }
            // A question the committed fact store matches is the
            // `fact_lookup` row's turn, not a measured property for a source
            // to look up: the record already carries the Wikidata anchor and
            // the localized summary the question asks for (issue #1138: the
            // factual QnA matrix).
            if crate::solver_handlers::fact_store_resolves(prompt) {
                return None;
            }
            // An elaboration of a procedure this conversation already answered
            // ("give me the exact steps") is that procedure's continuation
            // (issue #444). The rebind is recovered from the prior turns, not
            // from this prompt's own words, so neither a promoted method nor a
            // cue-matched family can claim the turn before the table routes
            // the bare words to a fresh compose — the same blind spot the
            // fetch arms solve for the response-language retarget below. Offer
            // the follow-up its turn before composing from scratch.
            if let Some(answer) =
                crate::solver_handler_how::try_procedural_how_to_followup(prompt, &normalized, log)
            {
                return Some(record_contextual_method_answer(
                    prompt,
                    log,
                    answer,
                    "procedural_how_to_followup",
                ));
            }
            Some(crate::source_capability::execute(
                capability,
                prompt,
                solver.config,
                log,
            ))
        }
        // The fresh-period digest reading (issue #1138 news_07:
        // `relative_period` + `retrieve` routed here) summarises web events a
        // plain chat surface cannot fetch, so the honest answer names the
        // capability the surface lacks instead of falling through to the
        // unknown opener. An agent client that advertises `web_search`
        // executes the digest itself and keeps the fall-through. Weekday and
        // date questions stay `time_expression`, which has no such row, so
        // calendar reasoning keeps answering them. An elliptical scheduling
        // prompt the promotion hoisted (issue #595: "А можешь на 10 часов по
        // Грузии с Марией?") reads as a relative period off its clock hour,
        // but the calendar gate's entities ground the event without the web,
        // so it falls through to the promoted handler instead of a gap.
        "web_search" if decision.object == ObjectType::RelativePeriod => {
            if solver.config.agent_mode || promoted_calendar_claims(promoted_methods, &normalized) {
                None
            } else {
                Some(capability_gap(prompt, log, capability, capability))
            }
        }
        // These capabilities already have general registry methods. Recording
        // the table decision here changes their precedence without duplicating
        // their execution or interrupting a multi-step research recipe; they
        // fall through with everything else.
        // An advertised capability the dispatcher has no dedicated executor
        // for still reaches the planner (issue #671: `read the file alpha.txt`
        // must be answered, not dead-ended). A surface that genuinely lacks
        // the tool never gets here -- that refusal is the `HonestGap` arm
        // above, which names the missing tool instead of the routed one.
        _ => None,
    }
}

/// An elliptical scheduling prompt the promotion hoisted and the calendar
/// gate accepts is the calendar handler's (issue #595): the clock hour,
/// timezone and participant entities ground the event without the web, so
/// both a routed `web_search` digest and a `web_search` gap decline it.
fn promoted_calendar_claims(promoted_methods: &[String], normalized: &str) -> bool {
    promoted_methods
        .iter()
        .any(|method| method == "calendar_create_event")
        && crate::solver_handlers::calendar_claims(normalized)
}

/// Whether the lexicon recognises all four roles of the GitHub
/// repository-traffic handler: the platform, a repository reference, a
/// traffic signal, and a visibility question (issue #497). Mirrors the
/// `github_repository_traffic` rule in `data/seed/handler-rules.lino`.
fn github_repository_traffic_claims(normalized: &str) -> bool {
    let lex = crate::seed::lexicon();
    lex.mentions_role("github_repository_platform", normalized)
        && lex.mentions_role("repository_reference", normalized)
        && lex.mentions_role("github_repository_traffic_signal", normalized)
        && lex.mentions_role("github_repository_traffic_question", normalized)
}

/// Whether a grounded capability decision is more specific than a promoted
/// handler's reading of the same prompt.
///
/// Generic retrieval/clarification handlers and concept lookup must yield to a
/// typed URL, path, clock, language or self-surface. Translation is different:
/// a complete translation recipe owns its transform even though it names a
/// target language. Only the `(language_name, demonstrate, dialogue)` reading
/// is a request to render this response in that language.
fn capability_route_preempts(
    promoted_method: &str,
    decision: &crate::capability_routing::RoutingDecision,
) -> bool {
    matches!(
        promoted_method,
        "http_fetch"
            | "url_navigate"
            | "web_search"
            | "calendar_create_event"
            | "clarification"
            | "concept_lookup"
            | "software_project"
            | "conversation_memory"
            | "response_language_followup"
    ) || (promoted_method == "translation"
        && decision.object == ObjectType::LanguageName
        && decision.act == Act::Demonstrate)
}

/// Whether the table should preempt regular symbolic-handler precedence.
///
/// A typed/named object is authoritative.  A bare term is not: definitions and
/// other local symbolic answers keep their turn unless the prompt explicitly
/// names the open web, lands in the workspace, or asks to compose from sources.
/// A prompt that is *only* a seeded clarification utterance is not either: the
/// table reads its short words as a self-surface question, and the seed's own
/// class keeps the turn (issue #29).
fn solver_route_is_authoritative(
    prompt: &str,
    decision: &crate::capability_routing::RoutingDecision,
) -> bool {
    if crate::capability_routing::is_bare_clarification(prompt) {
        return false;
    }
    match decision.object {
        ObjectType::BareTerm => {
            decision.locus == crate::capability_routing::Locus::Workspace
                || decision.act == crate::capability_routing::Act::Compose
                || crate::capability_routing::names_open_web(prompt)
        }
        ObjectType::None => false,
        _ => true,
    }
}

fn record_capability_route(
    log: &mut EventLog,
    object: ObjectType,
    act: crate::capability_routing::Act,
    locus: crate::capability_routing::Locus,
    capability: &str,
) {
    log.append_fields(
        "capability_route",
        &[
            ("object", object.slug()),
            ("act", act.slug()),
            ("locus", locus.slug()),
            ("capability", capability),
        ],
    );
}

/// Execute one adopted learned method as the recipe program its learned
/// operations project onto (issue #1138 B7, plan 07 leaf 6).
///
/// A learned abstraction has no compiled handler: its `operations` are the
/// recorder event kinds `crate::recipe_interpreter` already dispatches on, so
/// the method runs by executing that program rather than by calling Rust that
/// was written for it. A name that is not a learned record, an operation that
/// binds to no recorder, and a misordered program are all reported in the trace
/// under `method:learned` and then declined, never silently skipped.
fn try_learned_method(
    solver: &UniversalSolver,
    registry: &MethodRegistry,
    name: &str,
    prompt: &str,
    intent_formalization: &IntentFormalization,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let learned = registry.learned_method(name)?;
    let execution = match learned.execute(intent_formalization, solver.config) {
        Ok(execution) => execution,
        Err(reason) => {
            log.append("method:learned", format!("{name}:{reason}"));
            return None;
        }
    };
    log.append("method:learned", name.to_owned());
    log.append(
        "method:learned:operations_verified",
        execution.operations_verified.to_string(),
    );
    let rendered = execution.answer;
    let intent = format!("learned_method:{name}");
    log.append("intent", intent.clone());
    let response_link = format!("response:learned_method:{name}");
    log.append("response", response_link.clone());
    let trace_id = log.append("trace", intent.clone());
    let evidence_links = build_evidence_links(prompt, log, &response_link);
    let links_notation = answer_links_notation(prompt, &intent, &rendered, log, &trace_id);
    let thinking_steps = log.thinking_steps_for_answer(&rendered);
    Some(record_method_answer(
        prompt,
        log,
        SymbolicAnswer {
            intent,
            answer: rendered,
            execution_recipe: None,
            confidence: solver.config.guess_probability,
            evidence_links,
            thinking_steps,
            links_notation,
        },
        name,
    ))
}

#[derive(Clone, Copy)]
struct MethodRuntime {
    proof_render_config: ProofRenderConfig,
    capability_runtime: CapabilityRuntime,
    self_awareness_runtime: SelfAwarenessRuntime,
}

impl MethodRuntime {
    const fn new(config: SolverConfig) -> Self {
        Self {
            proof_render_config: ProofRenderConfig {
                guess_probability: config.guess_probability,
                follow_up_probability: config.follow_up_probability,
            },
            capability_runtime: CapabilityRuntime::new(
                config.offline,
                config.agent_mode,
                config.diagnostic_mode,
                config.definition_fusion_by_default,
            ),
            self_awareness_runtime: SelfAwarenessRuntime::new(
                config.execution_surface,
                config.offline,
                config.agent_mode,
                config.diagnostic_mode,
                config.definition_fusion_by_default,
                config.blueprint_composition,
            ),
        }
    }
}

fn try_attributed_runtime(
    solver: &UniversalSolver,
    execution: MethodExecution,
    method_name: &str,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: MethodRuntime,
) -> Option<SymbolicAnswer> {
    let kind = execution.runtime?;
    let answer = match kind {
        MethodRuntimeKind::Diagnostic => try_diagnostic(solver, prompt, normalized, log),
        MethodRuntimeKind::NaturalLanguageTool => {
            try_natural_language_tool_request(prompt, normalized, log, solver.config.agent_mode)
        }
        MethodRuntimeKind::BehaviorRules => {
            try_behavior_rules_with_runtime(prompt, normalized, log, runtime.self_awareness_runtime)
        }
        MethodRuntimeKind::FeatureCapability => {
            try_feature_capability(prompt, normalized, log, runtime.capability_runtime)
        }
        MethodRuntimeKind::PlaywrightScript => {
            try_playwright_script(prompt, normalized, log, solver.config.guess_probability)
        }
    }?;
    Some(record_method_answer(prompt, log, answer, method_name))
}

fn try_diagnostic(
    solver: &UniversalSolver,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !normalized.contains("[diagnostic]") {
        return None;
    }
    log.append("diagnostic_mode", "active".to_owned());
    let stripped = prompt.replace("[diagnostic]", "").trim().to_owned();
    let inner_solver = UniversalSolver::new(solver.config);
    let inner = inner_solver.solve(&stripped);
    let mut decorated = inner.answer.clone();
    decorated.push_str("\n\n[diagnostic]\n");
    decorated.push_str(inner.links_notation.trim_end());
    decorated.push('\n');
    for link in &inner.evidence_links {
        let _ = writeln!(decorated, "evidence: {link}");
    }
    let _ = writeln!(decorated, "trace: {}", inner.intent);
    log.append("intent", inner.intent.clone());
    let response_link = format!("response:diagnostic:{}", inner.intent);
    log.append("response", response_link.clone());
    let trace_id = log.append("trace", inner.intent.clone());
    let evidence_links = build_evidence_links(prompt, log, &response_link);
    let links_notation = answer_links_notation(prompt, &inner.intent, &decorated, log, &trace_id);
    let thinking_steps = log.thinking_steps_for_answer(&inner.answer);
    let answer = append_diagnostic_trace(solver.config.diagnostic_mode, decorated, &links_notation);
    Some(SymbolicAnswer {
        intent: inner.intent,
        answer,
        execution_recipe: inner.execution_recipe,
        confidence: inner.confidence,
        evidence_links,
        thinking_steps,
        links_notation,
    })
}

fn record_method(log: &mut EventLog, name: &str) {
    log.append("method", name.to_owned());
}

fn record_method_answer(
    prompt: &str,
    log: &mut EventLog,
    answer: SymbolicAnswer,
    name: &str,
) -> SymbolicAnswer {
    record_method(log, name);
    refresh_answer_projection(prompt, log, answer)
}

fn record_contextual_method_answer(
    prompt: &str,
    log: &mut EventLog,
    answer: SymbolicAnswer,
    name: &str,
) -> SymbolicAnswer {
    log.append("method", name.to_owned());
    refresh_answer_projection(prompt, log, answer)
}

fn refresh_answer_projection(
    prompt: &str,
    log: &EventLog,
    mut answer: SymbolicAnswer,
) -> SymbolicAnswer {
    let Some(response_link) = log.last_of("response").map(|event| event.payload.clone()) else {
        return answer;
    };
    let Some(trace_id) = log.last_of("trace").map(|event| event.id.clone()) else {
        return answer;
    };
    answer.evidence_links = build_evidence_links(prompt, log, &response_link);
    answer.links_notation =
        answer_links_notation(prompt, &answer.intent, &answer.answer, log, &trace_id);
    answer.thinking_steps = log.thinking_steps_for_answer(&answer.answer);
    answer
}
