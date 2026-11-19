//! Replay candidate validation, interpretation, contextual follow-ups and debug
//! sessions without compiling the monolithic unit target. Existing test modules
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
