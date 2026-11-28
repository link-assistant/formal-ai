use super::resolve_who_is;
use crate::engine::normalize_prompt;
use crate::event_log::EventLog;

#[test]
fn unknown_entity_responses_render_the_requested_name_and_any_correction() {
    let cases = [
        ("glimmerwood marivell", None),
        ("ada lovlace", Some("Ada Lovelace")),
    ];
    for (name, correction) in cases {
        let prompt = format!("who is {name}");
        let result = resolve_who_is(&prompt, &normalize_prompt(&prompt), &mut EventLog::new())
            .expect("the entity resolver acknowledges an unknown name");
        assert!(result.answer.contains(name), "{}", result.answer);
        assert!(!result.answer.contains("{term}"), "{}", result.answer);
        if let Some(correction) = correction {
            assert!(result.answer.contains(correction), "{}", result.answer);
        }
        assert!(!result.answer.contains("{corrected}"), "{}", result.answer);
    }
}
