//! Manual native manifest observation projection; no native execution claimed.
use crate::source_fetch::sha256_hex;
use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};
use std::fmt;
use toml_edit::{DocumentMut, Item, Table, Value as TomlValue};

struct UniqueJson(Value);
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueJson;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON with recursively unique object keys")
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Bool(value)))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(value)
                    .map(|number| UniqueJson(Value::Number(number)))
                    .ok_or_else(|| E::custom("nonfinite JSON number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::String(value.to_owned())))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::String(value)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Null))
            }
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(UniqueJson(value)) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(UniqueJson(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut mapping: A) -> Result<Self::Value, A::Error> {
                let mut values = Map::new();
                while let Some(name) = mapping.next_key::<String>()? {
                    if values.contains_key(&name) {
                        return Err(de::Error::custom("duplicate JSON object key"));
                    }
                    let UniqueJson(value) = mapping.next_value()?;
                    values.insert(name, value);
                }
                Ok(UniqueJson(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}

fn validate_json_depth(value: &Value, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("manifest depth limit".to_owned());
    }
    match value {
        Value::Number(value) => {
            let number = value.as_f64().ok_or("unsupported manifest numeric range")?;
            if !number.is_finite()
                || (number.fract() == 0.0 && number.abs() > 9_007_199_254_740_991.0)
            {
                return Err("unsupported manifest numeric range".to_owned());
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_json_depth(value, depth + 1)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_json_depth(value, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_toml_profile(source: &str) -> Result<(), String> {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut previous = None;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if byte == b'#' {
            while cursor < bytes.len() && bytes[cursor] != b'\n' {
                cursor += 1;
            }
            continue;
        }
        if byte == b'"' || byte == b'\'' {
            let quote = byte;
            if bytes
                .get(cursor..cursor + 3)
                .is_some_and(|value| value == [quote, quote, quote])
            {
                return Err("unsupported multiline manifest string".to_owned());
            }
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor] != quote {
                if bytes[cursor] < b' ' {
                    return Err("unsupported manifest string control".to_owned());
                }
                if bytes[cursor] == b'\\' && quote == b'"' {
                    cursor += 1;
                    if bytes.get(cursor) == Some(&b'U') {
                        return Err("unsupported eight-digit manifest escape".to_owned());
                    }
                }
                cursor += 1;
            }
            cursor += 1;
            previous = Some(b's');
            continue;
        }
        if byte == b'}' && previous == Some(b',') {
            return Err("unsupported inline manifest trailing comma".to_owned());
        }
        if !byte.is_ascii_whitespace() {
            previous = Some(byte);
        }
        cursor += 1;
    }
    Ok(())
}

fn string_array(value: &TomlValue) -> Result<Vec<String>, String> {
    let array = value.as_array().ok_or("manifest string array required")?;
    array
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "manifest array member must be a string".to_owned())
        })
        .collect()
}

fn validate_literal(value: &TomlValue, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("manifest depth limit".to_owned());
    }
    match value {
        TomlValue::Datetime(_) => Err("unsupported manifest datetime".to_owned()),
        TomlValue::Integer(_) => {
            let rendered = value.to_string();
            let literal = rendered.trim_start().trim_start_matches(['+', '-']);
            if ["0x", "0o", "0b"]
                .iter()
                .any(|prefix| literal.starts_with(prefix))
            {
                Err("unsupported nondecimal manifest integer".to_owned())
            } else {
                Ok(())
            }
        }
        TomlValue::Float(_) if !value.as_float().is_some_and(f64::is_finite) => {
            Err("unsupported nonfinite manifest number".to_owned())
        }
        TomlValue::String(_) => {
            let rendered = value.to_string();
            let rendered = rendered.trim_start();
            if rendered.starts_with("\"\"\"") || rendered.starts_with("'''") {
                return Err("unsupported multiline manifest string".to_owned());
            }
            let quote = rendered.as_bytes().first().copied();
            let mut escaped = false;
            for byte in rendered.bytes().skip(1) {
                if escaped {
                    if byte == b'U' {
                        return Err("unsupported eight-digit manifest escape".to_owned());
                    }
                    escaped = false;
                } else if Some(byte) == quote {
                    break;
                } else if byte == b'\t' {
                    return Err("unsupported literal manifest string tab".to_owned());
                } else if byte == b'\\' && quote == Some(b'"') {
                    escaped = true;
                }
            }
            Ok(())
        }
        TomlValue::Array(array) => {
            for entry in array {
                validate_literal(entry, depth + 1)?;
            }
            Ok(())
        }
        TomlValue::InlineTable(table) => {
            for (_, entry) in table {
                validate_literal(entry, depth + 1)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
fn validate_table(table: &Table) -> Result<(), String> {
    for (_, item) in table {
        match item {
            Item::Value(value) => validate_literal(value, 0)?,
            Item::Table(nested) => validate_table(nested)?,
            Item::ArrayOfTables(tables) => {
                for nested in tables {
                    validate_table(nested)?;
                }
            }
            Item::None => return Err("missing manifest value".to_owned()),
        }
    }
    Ok(())
}
fn dependency_properties(item: &Item) -> Result<Vec<(String, &TomlValue)>, String> {
    if let Some(table) = item.as_inline_table() {
        return Ok(table
            .iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect());
    }
    if let Some(table) = item.as_table() {
        return table
            .iter()
            .map(|(name, item)| {
                item.as_value()
                    .map(|value| (name.to_owned(), value))
                    .ok_or_else(|| "dependency property must be a literal value".to_owned())
            })
            .collect();
    }
    Err("dependency must be a version string or literal table".to_owned())
}
fn dependency_details(item: &Item) -> Result<(Option<String>, Map<String, Value>), String> {
    if let Some(version) = item.as_str() {
        return Ok((Some(version.to_owned()), Map::new()));
    }
    let mut version = None;
    let mut options = Map::new();
    for (name, value) in dependency_properties(item)? {
        if name == "version" {
            version = Some(
                value
                    .as_str()
                    .ok_or("dependency version must be string")?
                    .to_owned(),
            );
        } else {
            let converted = match name.as_str() {
                "package" | "path" | "git" | "branch" | "tag" | "rev" | "registry" => {
                    Value::String(
                        value
                            .as_str()
                            .ok_or("dependency option must be string")?
                            .to_owned(),
                    )
                }
                "default-features" | "optional" | "workspace" => {
                    Value::Bool(value.as_bool().ok_or("dependency option must be boolean")?)
                }
                "features" => json!(string_array(value)?),
                _ => return Err(format!("unsupported dependency property: {name}")),
            };
            options.insert(name, converted);
        }
    }
    if version.is_none()
        && !["path", "git"]
            .iter()
            .any(|name| options.contains_key(*name))
        && options.get("workspace") != Some(&Value::Bool(true))
    {
        return Err("dependency origin required".to_owned());
    }
    Ok((version, options))
}
fn dependency_rows(
    table: &Table,
    manifest: &str,
    identity: &str,
    target: Option<&str>,
    output: &mut Vec<Value>,
) -> Result<(), String> {
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(item) = table.get(section) else {
            continue;
        };
        let dependencies = item
            .as_table()
            .ok_or("dependency section must be a table")?;
        for (name, item) in dependencies {
            let (version, options) = dependency_details(item)?;
            let mut row = json!({"kind":"manifest-declaration", "manifest":manifest,
                "sourceIdentity":identity,"section":section,"name":name,"version":version});
            if let Some(target) = target {
                row["target"] = json!(target);
            }
            if !options.is_empty() {
                row["options"] = Value::Object(options);
            }
            output.push(row);
        }
    }
    Ok(())
}

/// Observe literal declarations only. A declaration is not proof of executable use.
pub(super) fn observed_manifest_facts(owner: &str, source: &str) -> Result<Vec<Value>, String> {
    if source.len() > 262_144 {
        return Err("manifest source exceeds finite observation bound".to_owned());
    }
    let valid_first = owner
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
    if !valid_first
        || !owner
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
        || owner
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("relative manifest owner required".to_owned());
    }
    let identity = sha256_hex(source.as_bytes());
    let mut output = Vec::new();
    if owner.ends_with(".json") {
        let document = observed_source_json(source)?;
        let object = document
            .as_object()
            .ok_or("package manifest must be object")?;
        for section in [
            "dependencies",
            "devDependencies",
            "optionalDependencies",
            "peerDependencies",
        ] {
            let Some(value) = object.get(section) else {
                continue;
            };
            let dependencies = value
                .as_object()
                .ok_or("package dependency section must be object")?;
            for (name, value) in dependencies {
                let version = value
                    .as_str()
                    .ok_or("package dependency version must be string")?;
                output.push(json!({"kind":"manifest-declaration","manifest":owner,
                    "sourceIdentity":identity,"section":section,"name":name,"version":version}));
            }
        }
    } else if owner.ends_with(".toml") {
        validate_toml_profile(source)?;
        let document = source
            .parse::<DocumentMut>()
            .map_err(|error| error.to_string())?;
        validate_table(document.as_table())?;
        dependency_rows(document.as_table(), owner, &identity, None, &mut output)?;
        if let Some(item) = document.get("target") {
            let targets = item.as_table().ok_or("Cargo targets must be table")?;
            for (target, item) in targets {
                let table = item.as_table().ok_or("Cargo target must be table")?;
                dependency_rows(table, owner, &identity, Some(target), &mut output)?;
            }
        }
        if let Some(item) = document.get("features") {
            let features = item.as_table().ok_or("Cargo features must be table")?;
            for (name, item) in features {
                let value = item
                    .as_value()
                    .ok_or("Cargo feature must be literal array")?;
                output.push(
                    json!({"kind":"cargo-feature","manifest":owner,"sourceIdentity":identity,
                    "section":"features","name":name,"enables":string_array(value)?}),
                );
            }
        }
    } else {
        return Err("unsupported dependency manifest path".to_owned());
    }
    output.sort_by_cached_key(|value| {
        json!([
            value["kind"],
            value["section"],
            value.get("target").and_then(Value::as_str).unwrap_or(""),
            value["name"]
        ])
        .to_string()
    });
    Ok(output)
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/source_manifest_facts.rs"]
mod tests;

/// Complete bounded JSON observations shared by manifest and scalar bindings.
pub(super) fn observed_source_json(source: &str) -> Result<Value, String> {
    if source.len() > 262_144 {
        return Err("manifest source exceeds finite observation bound".to_owned());
    }
    let UniqueJson(document) =
        serde_json::from_str::<UniqueJson>(source).map_err(|error| error.to_string())?;
    validate_json_depth(&document, 0)?;
    Ok(document)
}
