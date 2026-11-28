#[test]
fn calculator_folding_retains_each_origin_and_clears_it_between_clusters() {
    let mut log = formal_ai::event_log::EventLog::new();
    log.append_with_origin("calculation:request", "request:first", "1 + 1");
    log.append_with_origin("calculation:engine", "engine:first", "engine");
    log.append_with_origin("calculation:lino", "expression:first", "1+1");
    log.append_with_origin("calculation:steps", "steps:first", "3");
    log.append_with_origin("calculation", "result:first", "2");
    log.append("intent", "arithmetic");
    log.append_with_origin("calculation:request", "request:second", "2 + 2");
    log.append("calculation:engine", "engine");
    let steps = log.thinking_steps();
    assert_eq!(
        steps
            .iter()
            .map(|step| step.source_origin.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("result:first"),
            Some("engine:first"),
            Some("expression:first"),
            Some("steps:first"),
            None,
            Some("request:second"),
            None
        ]
    );
    assert_eq!(steps[1].parent_id.as_deref(), Some(steps[0].id.as_str()));
    assert_eq!(steps[6].parent_id.as_deref(), Some(steps[5].id.as_str()));
}
