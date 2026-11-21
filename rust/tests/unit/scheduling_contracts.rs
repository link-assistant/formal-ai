//! Issue #1580: replay the existing calendar and capability-routing contracts.

#[path = "solver/issue_869_meeting_scheduling.rs"]
mod meeting_scheduling;
#[path = "issue_1138_place_timezone.rs"]
mod place_timezone;
#[path = "capability-routing/issue_1138_self_use_intent_routing.rs"]
mod self_use_intent_routing;
#[path = "specification/spoken_hour_scheduling.rs"]
mod spoken_hour_scheduling;
