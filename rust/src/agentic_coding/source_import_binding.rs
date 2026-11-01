//! Finite source-derived import addition. Qualification runs only in CI.
use super::observed_source_usage::view::{View, concrete, field, form, literal, one};
use super::source_model::{
    owned_source_links, source_from_links, source_to_links, verify_source_links,
};
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::sync::OnceLock;
const GRAMMAR: &str = include_str!("../../embedded/data/meta/source-import-request.json");
pub(super) fn source_import_contract(task: &str) -> Option<Value> {
    if task.len() > 8192 {
        return None;
    }
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        let grammar: Value = serde_json::from_str(GRAMMAR).expect("StaticImportGrammar");
        regex::RegexBuilder::new(
            grammar["expression"]
                .as_str()
                .expect("StaticImportExpression"),
        )
        .case_insensitive(true)
        .build()
        .expect("StaticImportPattern")
    });
    let found = pattern.captures(task)?;
    let owner = found.name("owner")?.as_str();
    if owner.len() > 4096 || owner.split('/').any(|part| matches!(part, "" | "." | "..")) {
        return None;
    }
    Some(
        json!({"operation":"add-named-import","authority":"owned","owner":owner,
        "module":found.name("module")?.as_str(),"binding":found.name("binding")?.as_str()}),
    )
}
struct Selected {
    existing: bool,
    position: usize,
    trailing_comma: bool,
}
fn inspect(
    source: &str,
    document: &Value,
    module: &str,
    binding: &str,
) -> Result<Selected, String> {
    let view = View::new(source, document)?;
    let imports = view
        .records
        .iter()
        .filter(|record| matches!(form(record.node), "import_declaration" | "import_statement"))
        .filter(|record| one(record.node, &["source"]).and_then(literal).as_deref() == Some(module))
        .collect::<Vec<_>>();
    if imports.len() != 1 {
        return Err("NamedImportDeclarationAmbiguous".to_owned());
    }
    let selected = imports[0];
    let inside = view
        .records
        .iter()
        .filter(|record| {
            record
                .ancestors
                .iter()
                .any(|ancestor| std::ptr::eq(*ancestor, selected.node))
        })
        .collect::<Vec<_>>();
    if inside.iter().any(|record| {
        matches!(
            form(record.node),
            "import_namespace_specifier" | "namespace_import"
        )
    }) {
        return Err("NamedImportNamespaceUnsupported".to_owned());
    }
    let specifiers = inside
        .iter()
        .filter(|record| form(record.node) == "import_specifier")
        .collect::<Vec<_>>();
    if specifiers.is_empty() {
        return Err("NamedImportListRequired".to_owned());
    }
    let existing = specifiers.iter().any(|record| {
        let imported = one(record.node, &["imported", "name"]);
        let local = one(record.node, &["local", "alias"]).or(imported);
        imported.is_some_and(|node| concrete(node) == binding)
            && local.is_some_and(|node| concrete(node) == binding)
    });
    let module_node = one(selected.node, &["source"]).ok_or("NamedImportSourceRequired")?;
    let module_start = view
        .record(module_node)
        .ok_or("NamedImportSourceRequired")?
        .start;
    let last = specifiers
        .iter()
        .map(|record| record.end)
        .max()
        .ok_or("NamedImportListRequired")?;
    let closing = inside
        .iter()
        .filter(|record| {
            field(record.node, "text").is_some()
                && concrete(record.node) == "}"
                && record.start >= last
                && record.end <= module_start
        })
        .collect::<Vec<_>>();
    if closing.len() != 1 {
        return Err("NamedImportDelimiterAmbiguous".to_owned());
    }
    let position = closing[0].start;
    let trailing_comma = inside.iter().any(|record| {
        field(record.node, "text").is_some()
            && concrete(record.node) == ","
            && record.start >= last
            && record.end <= position
    });
    Ok(Selected {
        existing,
        position,
        trailing_comma,
    })
}
pub(super) fn derive_named_import_edit(
    owner: &str,
    source: &str,
    module: &str,
    binding: &str,
) -> Result<Value, String> {
    static BINDING: OnceLock<regex::Regex> = OnceLock::new();
    if source.len() > 262144
        || module.is_empty()
        || !BINDING
            .get_or_init(|| {
                regex::Regex::new(r"^[A-Za-z_$][A-Za-z0-9_$]{0,127}$").expect("StaticImportBinding")
            })
            .is_match(binding)
        || ![".mjs", ".js"].iter().any(|suffix| owner.ends_with(suffix))
        || owner.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err("NamedImportInputRequired".to_owned());
    }
    let document = owned_source_links(owner, &source_to_links(source, "JavaScript")?)?;
    verify_source_links(&document)?;
    if source_from_links(&document)? != source {
        return Err("NamedImportPreimageReconstructionMismatch".to_owned());
    }
    let selected = inspect(source, &document, module, binding)?;
    let insertion = if selected.existing {
        String::new()
    } else {
        format!(
            "{}{binding}{}",
            if selected.trailing_comma { " " } else { ", " },
            if selected.trailing_comma { "," } else { "" }
        )
    };
    let prefix = &source[..selected.position];
    let candidate = format!("{prefix}{insertion}{}", &source[selected.position..]);
    if candidate.len() > 262144 {
        return Err("NamedImportCandidateBudget".to_owned());
    }
    let changed = owned_source_links(owner, &source_to_links(&candidate, "JavaScript")?)?;
    verify_source_links(&changed)?;
    if source_from_links(&changed)? != candidate
        || !inspect(&candidate, &changed, module, binding)?.existing
    {
        return Err("NamedImportCandidateBindingMismatch".to_owned());
    }
    let restored = format!(
        "{}{}",
        &candidate[..prefix.len()],
        &candidate[prefix.len() + insertion.len()..]
    );
    if restored != source
        || owned_source_links(owner, &source_to_links(&restored, "JavaScript")?)? != document
    {
        return Err("NamedImportInverseMismatch".to_owned());
    }
    let grammar: Value = serde_json::from_str(GRAMMAR).map_err(|_| "NamedImportInputRequired")?;
    Ok(json!({"schema":"observed-source-edit/v1","owner":owner,
        "candidate":{"path":owner,"source":candidate,"beforeSha256":sha256_hex(source.as_bytes()),"afterSha256":sha256_hex(candidate.as_bytes()),"changed":!selected.existing},
        "binding":{"module":module,"name":binding,"insertionSpan":[selected.position,selected.position+insertion.len()]},
        "verification":{"completeConcreteResidual":true,"independentGrammarReparse":true,"exactInverse":true,"commentsAndWhitespacePreserved":true,"arbitrarySemanticEquivalence":false,"exportResolution":"not-checked"},
        "knowledge":{"schema":grammar["schema"],"scope":grammar["scope"]},"independentAcceptance":false}))
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/source_import_binding.rs"]
mod tests;
