// Scratch proposal to be consumed by the existing seeded leading-action detector.
use crate::seed::{self, Slot};
fn word(character: Option<char>) -> bool {
    character.is_some_and(|value| value.is_alphanumeric() || value == '_' || value == '-')
}
pub(super) fn first_operation_owner(request: &str) -> Option<&'static str> {
    let mut view = request.to_owned();
    for span in crate::normal_markov::quoted_segment_spans(request) {
        view.replace_range(span.start..span.end, &" ".repeat(span.end - span.start));
    }
    let sentence = crate::agentic_coding::shell_command_policy::prose_sentences(&view)
        .into_iter()
        .next()?;
    let scope = view.get(sentence.span)?;
    let mut found = Vec::new();
    for role in [
        "coding_request_object",
        "software_artifact_kind",
        "software_artifact",
        "capability_web_scope",
    ] {
        for form in seed::lexicon().role_word_forms(role) {
            let text = match form.slot() {
                Slot::Bare => form.text.as_str(),
                Slot::Prefix => form.before_slot(),
                _ => continue,
            };
            let needle = text.trim().to_lowercase();
            if needle.is_empty() {
                continue;
            }
            for (start, _) in scope.char_indices() {
                let mut end = start;
                let mut lowered = String::new();
                for point in scope[start..].chars() {
                    if lowered.len() >= needle.len() {
                        break;
                    }
                    lowered.extend(point.to_lowercase());
                    end += point.len_utf8();
                }
                let bounded = crate::coding::contains_cjk(&needle)
                    || !word(scope[..start].chars().next_back())
                        && !word(scope[end..].chars().next());
                if lowered == needle && bounded {
                    found.push((start, end, role));
                }
            }
        }
    }
    found.sort_by_key(|(start, end, _)| (*start, std::cmp::Reverse(*end)));
    found.first().map(|(_, _, role)| *role)
}
