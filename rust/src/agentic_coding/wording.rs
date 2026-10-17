//! Shared agentic wording and cue data reader.

pub(super) fn agentic_message(key: &str, values: &[(&str, &str)]) -> String {
    let root = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/meta/agentic-messages.lino"
    ));
    let mut answer = root
        .children
        .first()
        .and_then(|root| {
            root.children
                .iter()
                .find(|node| node.name == "message" && node.id == key)
        })
        .map_or_else(
            || key.to_owned(),
            |node| node.find_child_value("text").replace("\\n", "\n"),
        );
    for (name, value) in values {
        answer = answer.replace(&format!("{{{name}}}"), value);
    }
    answer
}
