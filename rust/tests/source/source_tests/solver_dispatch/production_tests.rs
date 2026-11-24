use super::{
    CONTEXTUAL_HANDLER_NAMES, ContextualOutcome, ContextualRuntime, ProofRenderConfig,
    SelfAwarenessRuntime, SolverConfig, try_contextual_override,
};
use crate::event_log::EventLog;

#[test]
fn every_declared_contextual_handler_reaches_its_dispatch_arm() {
    let runtime = ContextualRuntime::new(
        ProofRenderConfig::default(),
        SelfAwarenessRuntime::default(),
        SolverConfig {
            offline: true,
            ..SolverConfig::default()
        },
    );
    for handler in CONTEXTUAL_HANDLER_NAMES {
        let outcome = try_contextual_override(handler, "", "", &[], runtime, &mut EventLog::new());
        assert!(
            !matches!(outcome, ContextualOutcome::NotHandled),
            "{handler} is registered but has no contextual dispatch"
        );
    }
    let absent = try_contextual_override(
        "unregistered_contextual_handler",
        "",
        "",
        &[],
        runtime,
        &mut EventLog::new(),
    );
    assert!(matches!(absent, ContextualOutcome::NotHandled));
}
