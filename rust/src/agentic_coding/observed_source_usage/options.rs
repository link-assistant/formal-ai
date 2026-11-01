use super::Facts;
use super::view::{children, concrete, form, identifier, literal, one, trivia};
use serde_json::{Value, json};
impl Facts<'_> {
    pub(super) fn options(&mut self, node: Option<&Value>) -> Vec<Value> {
        let Some(node) = node.filter(|node| {
            matches!(
                form(node),
                "object_expression" | "object" | "field_initializer_list"
            )
        }) else {
            return Vec::new();
        };
        let mut output = Vec::new();
        for child in children(node) {
            if trivia(child) || matches!(concrete(child).as_str(), "{" | "}" | ",") {
                continue;
            }
            if !matches!(
                form(child),
                "object_property"
                    | "pair"
                    | "field_initializer"
                    | "shorthand_field_initializer"
                    | "shorthand_property_identifier"
            ) {
                self.gap(
                    "option-spread-or-field-grammar-unresolved",
                    Some(child),
                    None,
                );
                continue;
            }
            let key = one(child, &["key", "field", "name"]);
            let expression = one(child, &["value"]);
            let name = key.map_or_else(
                || identifier(child),
                |node| identifier(node).or_else(|| literal(node)),
            );
            let Some(name) = name else {
                self.gap("option-key-resolution-unavailable", Some(child), None);
                continue;
            };
            if output.len() >= 16 {
                self.truncated = true;
                break;
            }
            let mut item = self.view.observation(child);
            item["name"] = json!(name);
            item["valueKind"] = json!(expression.map_or("shorthand-reference", form));
            output.push(item);
        }
        output
    }
}
