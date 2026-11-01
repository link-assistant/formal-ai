use serde_json::{Value, json};
use std::collections::HashMap;
pub(crate) fn field<'a>(node: &'a Value, name: &str) -> Option<&'a str> {
    let rows = node
        .as_array()?
        .iter()
        .filter_map(Value::as_array)
        .filter(|row| row.first().and_then(Value::as_str) == Some(name))
        .collect::<Vec<_>>();
    (rows.len() == 1)
        .then(|| rows[0].get(1)?.get("quoted")?.as_str())
        .flatten()
}
pub(crate) fn form(node: &Value) -> &str {
    field(node, "form").unwrap_or("")
}
pub(crate) fn role(node: &Value, name: &str) -> bool {
    field(node, "role").is_some_and(|value| value.split('|').any(|part| part == name))
}
pub(crate) fn children(node: &Value) -> Vec<&Value> {
    node.as_array()
        .into_iter()
        .flatten()
        .filter(|item| {
            item.as_array()
                .and_then(|row| row.first())
                .and_then(Value::as_str)
                .is_some_and(|head| {
                    !matches!(
                        head,
                        "form" | "role" | "text" | "language" | "path" | "meaning" | "semantic-gap"
                    )
                })
        })
        .collect()
}
pub(crate) fn concrete(node: &Value) -> String {
    field(node, "text").map(str::to_owned).unwrap_or_else(|| {
        children(node)
            .into_iter()
            .map(concrete)
            .collect::<Vec<_>>()
            .concat()
    })
}
pub(crate) fn named<'a>(node: &'a Value, names: &[&str]) -> Vec<&'a Value> {
    children(node)
        .into_iter()
        .filter(|child| names.iter().any(|name| role(child, name)))
        .collect()
}
pub(crate) fn one<'a>(node: &'a Value, names: &[&str]) -> Option<&'a Value> {
    let found = named(node, names);
    (found.len() == 1).then(|| found[0])
}
pub(crate) fn identifier(node: &Value) -> Option<String> {
    (matches!(
        form(node),
        "identifier"
            | "type_identifier"
            | "field_identifier"
            | "property_identifier"
            | "crate"
            | "self"
            | "super"
    ) && field(node, "text").is_some())
    .then(|| concrete(node))
}
pub(crate) fn trivia(node: &Value) -> bool {
    matches!(
        form(node),
        "whitespace" | "comment" | "line_comment" | "block_comment"
    )
}
pub(crate) fn literal(node: &Value) -> Option<String> {
    if !matches!(form(node), "string_literal" | "string") {
        return None;
    }
    let raw = concrete(node);
    let quote = raw.chars().next()?;
    if !matches!(quote, '\'' | '"') || !raw.ends_with(quote) || raw.len() < 2 {
        return None;
    }
    let body = &raw[1..raw.len() - 1];
    (!body.contains('\\') && !body.chars().any(|character| character < ' '))
        .then(|| body.to_owned())
}
pub(crate) fn path_segments(node: &Value) -> Option<Vec<String>> {
    if let Some(name) = identifier(node) {
        return Some(vec![name]);
    }
    if !matches!(form(node), "scoped_identifier" | "scoped_type_identifier") {
        return None;
    }
    let tail = one(node, &["name"])?;
    let mut prefix = one(node, &["path"]).map_or_else(|| Some(Vec::new()), path_segments)?;
    prefix.extend(path_segments(tail)?);
    Some(prefix)
}
pub(crate) struct Record<'a> {
    pub(crate) node: &'a Value,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) ancestors: Vec<&'a Value>,
    pub(crate) subtree_end: usize,
}
pub(crate) struct View<'a> {
    source: &'a str,
    pub(crate) records: Vec<Record<'a>>,
    index: HashMap<*const Value, usize>,
}
impl<'a> View<'a> {
    pub(crate) fn new(source: &'a str, document: &'a Value) -> Result<Self, String> {
        let mut view = Self {
            source,
            records: Vec::new(),
            index: HashMap::new(),
        };
        if view.visit(document, 0, Vec::new())? != source.len() {
            return Err("SourceUsageSpanReconstructionMismatch".to_owned());
        }
        Ok(view)
    }
    fn visit(
        &mut self,
        node: &'a Value,
        start: usize,
        ancestors: Vec<&'a Value>,
    ) -> Result<usize, String> {
        if ancestors.len() > 256 || self.records.len() >= 50000 {
            return Err("SourceUsageCstBudgetExceeded".to_owned());
        }
        let index = self.records.len();
        self.index.insert(std::ptr::from_ref(node), index);
        self.records.push(Record {
            node,
            start,
            end: start,
            ancestors: ancestors.clone(),
            subtree_end: 0,
        });
        let end = if let Some(text) = field(node, "text") {
            start + text.len()
        } else {
            let mut cursor = start;
            for child in children(node) {
                let mut nested = ancestors.clone();
                nested.push(node);
                cursor = self.visit(child, cursor, nested)?;
            }
            cursor
        };
        self.records[index].end = end;
        self.records[index].subtree_end = self.records.len();
        Ok(end)
    }
    pub(crate) fn record(&self, node: &Value) -> Option<&Record<'a>> {
        self.index
            .get(&std::ptr::from_ref(node))
            .map(|index| &self.records[*index])
    }
    pub(crate) fn observation(&self, node: &Value) -> Value {
        let Some(record) = self.record(node) else {
            return json!({});
        };
        let mut end = record.end.min(record.start + 512);
        while end > record.start && !self.source.is_char_boundary(end) {
            end -= 1;
        }
        json!({"span":[record.start,record.end],"source":&self.source[record.start..end],"sourceTruncated":end<record.end})
    }
    pub(crate) fn descendants(&self, node: &Value) -> &[Record<'a>] {
        self.index
            .get(&std::ptr::from_ref(node))
            .map_or(&[], |index| {
                &self.records[*index..self.records[*index].subtree_end]
            })
    }
    pub(crate) fn in_import(record: &Record<'_>) -> bool {
        record.ancestors.iter().any(|node| {
            matches!(
                form(node),
                "import_declaration" | "import_statement" | "use_declaration"
            )
        })
    }
}
