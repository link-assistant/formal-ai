//! Source declaration syntax is checked by round-tripping the maintained producer.
pub(super) fn canonical_creation_path(raw: &str) -> Option<&str> {
    let rest = raw.strip_prefix("*** Begin Patch\n*** Add File: ")?;
    let (path, body) = rest.split_once('\n')?;
    let body = body.strip_suffix("*** End Patch\n")?;
    if path.is_empty() || !body.ends_with('\n') {
        return None;
    }
    let mut content = String::with_capacity(body.len());
    for line in body.split_terminator('\n') {
        content.push_str(line.strip_prefix('+')?);
        content.push('\n');
    }
    let arguments = serde_json::json!({"path":path,"content":content}).to_string();
    (crate::protocol_responses::apply_patch_input(&arguments)?.as_str() == raw).then_some(path)
}
