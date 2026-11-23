//! JSON member spans shared by npm collection and exact manifest rewrites.
use super::*;

pub(super) struct NpmMember {
    pub name: String,
    pub declared: String,
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub blocked: Option<String>,
}

// serde_json owns JSON syntax and escaping; byte offsets retain the original layout.
fn object_members(text: &str, offset: usize) -> Option<Vec<(String, usize, usize)>> {
    if !serde_json::from_str::<serde_json::Value>(text)
        .ok()?
        .is_object()
    {
        return None;
    }
    let mut cursor = text.find('{')? + 1;
    let mut members: Vec<(String, usize, usize)> = Vec::new();
    loop {
        cursor += text[cursor..].len() - text[cursor..].trim_start().len();
        if text.as_bytes().get(cursor) == Some(&b'}') {
            // JSON objects use their last occurrence of a repeated key.
            let mut seen = std::collections::BTreeSet::new();
            let mut unique: Vec<_> = members
                .into_iter()
                .rev()
                .filter(|(name, _, _)| seen.insert(name.clone()))
                .collect();
            unique.reverse();
            return Some(unique);
        }
        let mut key = serde_json::Deserializer::from_str(&text[cursor..]).into_iter::<String>();
        let name = key.next()?.ok()?;
        cursor += key.byte_offset();
        cursor += text[cursor..].len() - text[cursor..].trim_start().len();
        if text.as_bytes().get(cursor) != Some(&b':') {
            return None;
        }
        cursor += 1;
        cursor += text[cursor..].len() - text[cursor..].trim_start().len();
        let start = cursor;
        let mut value =
            serde_json::Deserializer::from_str(&text[cursor..]).into_iter::<serde_json::Value>();
        value.next()?.ok()?;
        cursor += value.byte_offset();
        members.push((name, offset + start, offset + cursor));
        cursor += text[cursor..].len() - text[cursor..].trim_start().len();
        if text.as_bytes().get(cursor) == Some(&b',') {
            cursor += 1;
        }
    }
}

pub(super) fn dependency_members(text: &str) -> Vec<NpmMember> {
    let Some(root) = object_members(text, 0) else {
        return Vec::new();
    };
    let mut members = Vec::new();
    let mut notes = root.clone();
    for (section, start, end) in root {
        if matches!(
            section.as_str(),
            "dependencies" | "devDependencies" | "optionalDependencies"
        ) && let Some(entries) = object_members(&text[start..end], start)
        {
            notes.extend(entries.clone());
            members.extend(entries);
        }
    }
    let blocked: BTreeMap<String, String> = notes
        .into_iter()
        .filter_map(|(name, start, end)| {
            let name = name.strip_suffix("//")?;
            let note = serde_json::from_str::<String>(&text[start..end]).ok()?;
            let reason = note.trim().strip_prefix("blocked:")?;
            Some((name.to_string(), tracking_reference(reason)?))
        })
        .collect();
    members
        .into_iter()
        .filter_map(|(name, start, end)| {
            if name.ends_with("//") {
                return None;
            }
            let declared = serde_json::from_str::<String>(&text[start..end]).ok()?;
            Some(NpmMember {
                blocked: blocked.get(&name).cloned(),
                name,
                declared,
                line: text[..start].bytes().filter(|byte| *byte == b'\n').count() + 1,
                start,
                end,
            })
        })
        .collect()
}
