//! Replay reasoning, contextual follow-ups, debug sessions and sourced web
//! requests without compiling the monolithic unit target. Existing test modules
//! are included intact.

#[path = "specification/debug_session.rs"]
mod debug_session;
#[path = "specification/event_source_origin.rs"]
mod event_source_origin;
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
