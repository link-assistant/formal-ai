//! Replay reasoning, contextual follow-ups, debug sessions and sourced web
//! requests without compiling the monolithic unit target. Existing test modules
//! are included intact.

#[path = "specification/benchmarks.rs"]
mod benchmarks;
#[path = "specification/debug_session.rs"]
mod debug_session;
#[path = "specification/event_source_origin.rs"]
mod event_source_origin;
#[path = "issue_1173_fallback_executes_search.rs"]
mod external_search_fallback;
#[path = "specification/formalization.rs"]
mod formalization;
#[path = "specification/pronoun_topic_follow_up.rs"]
mod pronoun_topic_follow_up;
#[path = "specification/reasoning_loop.rs"]
mod reasoning_loop;

#[path = "page_formalization_route.rs"]
mod page_formalization_route;
mod web_requests;

#[path = "seed/issue_1172_live_fact_answer.rs"]
mod live_fact_answer;

#[path = "issue_498_google_trends.rs"]
mod google_trends;

#[path = "solver/issue_1138_self_use_concept_lookup.rs"]
mod concept_lookup_self_use;
#[path = "specification/reasoning_paths_procedures.rs"]
mod procedures;
#[path = "issue_1138_retrieval_method.rs"]
mod retrieval_method;

#[path = "agentic-coding/issue_498_google_trends_catalog.rs"]
mod google_trends_catalog;
#[path = "specification/unknown_reasoning.rs"]
mod unknown_reasoning;

#[path = "offline_instruction_research.rs"]
mod offline_instruction_research;
