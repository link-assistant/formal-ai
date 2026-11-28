//! Canonical memory-event fields and their stable doublet projection.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::{DoubletLink, MemoryEvent, stable_id};

pub(super) fn ensure_event_id(event: &mut MemoryEvent, sequence: usize) {
    if !event.id.is_empty() {
        return;
    }
    let canonical = canonical_memory_event(event);
    event.id = stable_id("memory_event", &format!("{sequence}:{canonical}"));
}

pub(super) fn event_source_id(event: &MemoryEvent, sequence: usize, canonical: &str) -> String {
    if event.id.is_empty() {
        stable_id("memory_event", &format!("{sequence}:{canonical}"))
    } else {
        event.id.clone()
    }
}

pub(super) fn canonical_memory_event(event: &MemoryEvent) -> String {
    let mut fields = BTreeMap::new();
    push_canonical(&mut fields, "id", Some(event.id.as_str()));
    push_canonical(&mut fields, "kind", event.kind.as_deref());
    push_canonical(&mut fields, "role", event.role.as_deref());
    push_canonical(&mut fields, "intent", event.intent.as_deref());
    push_canonical(&mut fields, "tool", event.tool.as_deref());
    push_canonical(&mut fields, "inputs", event.inputs.as_deref());
    push_canonical(&mut fields, "outputs", event.outputs.as_deref());
    push_canonical(&mut fields, "content", event.content.as_deref());
    push_canonical(&mut fields, "sentAt", event.sent_at.as_deref());
    push_canonical(&mut fields, "demoLabel", event.demo_label.as_deref());
    push_canonical(
        &mut fields,
        "conversationId",
        event.conversation_id.as_deref(),
    );
    push_canonical(
        &mut fields,
        "conversationTitle",
        event.conversation_title.as_deref(),
    );
    for (index, evidence) in event.evidence.iter().enumerate() {
        let key = format!("evidence_{index:04}");
        fields.insert(key, evidence.clone());
    }
    fields.insert(String::from("accessCount"), event.access_count.to_string());
    fields.insert(
        String::from("writeCount"),
        event.write_count.max(1).to_string(),
    );
    let mut out = String::new();
    for (key, value) in fields {
        let _ = write!(out, "{key}={}:{};", value.len(), value);
    }
    out
}

fn push_canonical(fields: &mut BTreeMap<String, String>, key: &str, value: Option<&str>) {
    let Some(value) = value else { return };
    if value.is_empty() {
        return;
    }
    fields.insert(key.to_owned(), value.to_owned());
}

pub(super) fn push_optional_field(
    links: &mut Vec<DoubletLink>,
    record_id: &str,
    key: &str,
    value: Option<&str>,
) {
    let Some(value) = value else { return };
    if value.is_empty() {
        return;
    }
    let field = format!("field:{key}");
    let field_value = format!("value:{value}");
    push_doublet(links, record_id, &field);
    push_doublet(links, &field, &field_value);
}

pub(super) fn push_doublet(links: &mut Vec<DoubletLink>, from: &str, to: &str) {
    links.push(DoubletLink {
        index: stable_id("doublet", &format!("{from}->{to}")),
        from: from.to_owned(),
        to: to.to_owned(),
    });
}
