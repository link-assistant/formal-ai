//! Browser adapter for the shared language-neutral proof representation.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::proof_program_core::FormalProof;
use crate::seed_parser::parse_lino;
use crate::web_engine_core::stable_id;
use crate::{decode_uri_component, push_json_string};

/// Translate `statement\ntarget\nseed`, whose fields are URI encoded, into a
/// complete worker answer. Invalid or unsupported inputs deliberately return an
/// empty string so the JavaScript router can continue to another handler.
pub(crate) fn answer(payload: &str) -> String {
    let decoded = payload.lines().map(decode_uri_component).collect::<Vec<_>>();
    if decoded.len() != 3 {
        return String::new();
    }
    let statement = &decoded[0];
    let target = &decoded[1];
    let Some(proof) = FormalProof::from_statement(statement) else {
        return String::new();
    };
    let tree = parse_lino(&decoded[2]);
    let Some(catalog) = tree
        .children
        .iter()
        .find(|node| node.name == "proof_program_templates")
    else {
        return String::new();
    };
    let Some(language) = catalog
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == *target)
    else {
        return String::new();
    };
    let state = if proof.is_satisfiable() {
        "satisfiable"
    } else {
        "unsatisfiable"
    };
    let template = language.find_child_value(state);
    if template.is_empty() {
        return String::new();
    }
    let program = proof.render_template(template);
    let content = format!(
        "Translated `{statement}` from proof to {target}:\n\n```{target}\n{program}\n```"
    );
    serialize_answer(target, &content, &stable_id("meaning", &proof.slug()))
}

fn serialize_answer(target: &str, content: &str, meaning_id: &str) -> String {
    let mut output = String::from("{\"intent\":");
    push_json_string(&mut output, &format!("translate_proof_to_{target}"));
    output.push_str(",\"content\":");
    push_json_string(&mut output, content);
    output.push_str(",\"confidence\":1.0,\"evidence\":[");
    for (index, evidence) in [
        "handler:translation".to_string(),
        "language_from:proof".to_string(),
        format!("language_to:{target}"),
        format!("meaning:{meaning_id}"),
    ]
    .iter()
    .enumerate()
    {
        if index > 0 {
            output.push(',');
        }
        push_json_string(&mut output, evidence);
    }
    output.push_str("]}");
    output
}
