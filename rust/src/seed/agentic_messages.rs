use std::collections::BTreeMap;

/// Shared agentic/protocol diagnostics are immutable seed data, parsed once per process.
pub fn agentic_message(key: &str, values: &[(&str, &str)]) -> String {
    static MESSAGES: std::sync::OnceLock<BTreeMap<String, String>> = std::sync::OnceLock::new();
    let messages = MESSAGES.get_or_init(|| {
        let root = super::parser::parse_lino(include_str!(
            "../../embedded/data/meta/agentic-messages.lino"
        ));
        root.children.first().map_or_else(BTreeMap::new, |root| {
            root.children
                .iter()
                .filter(|node| node.name == "message")
                .map(|node| {
                    (
                        node.id.clone(),
                        node.find_child_value("text").replace("\\n", "\n"),
                    )
                })
                .collect()
        })
    });
    super::fill_template_once(messages.get(key).map_or(key, String::as_str), values)
}
