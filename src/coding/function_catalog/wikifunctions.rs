//! Wikifunctions label search and `ZObject` extraction.
//!
//! The client uses the shared content-addressed source cache. The same parser
//! therefore consumes a live response and an offline replay, and every part
//! retains the digest and fetch time of the exact bytes that established it.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::links_format::push_lino_node;
use crate::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};

const API: &str = "https://www.wikifunctions.org/w/api.php";
const DEFINITION_LICENSE: &str = "CC0-1.0";
const IMPLEMENTATION_LICENSE: &str = "Apache-2.0";

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionMatch {
    pub page_title: String,
    pub label: String,
    pub match_label: String,
    pub match_lang: String,
    pub match_rate: f64,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionPart {
    pub zid: String,
    pub labels: BTreeMap<String, String>,
    pub argument_types: Vec<String>,
    pub return_type: String,
    pub implementation_zids: Vec<String>,
    pub tester_zids: Vec<String>,
    pub license: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
}

impl FunctionPart {
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "function_part", Some(&self.zid));
        push_lino_node(&mut out, 2, "catalog", Some("wikifunctions"));
        for (language, label) in &self.labels {
            push_lino_node(&mut out, 2, "label", Some(label));
            push_lino_node(&mut out, 4, "language", Some(language));
        }
        for argument_type in &self.argument_types {
            push_lino_node(&mut out, 2, "argument_type", Some(argument_type));
        }
        push_lino_node(&mut out, 2, "return_type", Some(&self.return_type));
        for implementation in &self.implementation_zids {
            push_lino_node(&mut out, 2, "implementation", Some(implementation));
        }
        for tester in &self.tester_zids {
            push_lino_node(&mut out, 2, "tester", Some(tester));
        }
        append_provenance(
            &mut out,
            &self.license,
            &self.source_url,
            &self.sha256,
            &self.fetched_at,
        );
        out.trim_end().to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implementation {
    pub zid: String,
    pub function_zid: String,
    pub label: Option<String>,
    pub language: String,
    pub code: String,
    pub license: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
}

impl Implementation {
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "function_part", Some(&self.zid));
        push_lino_node(&mut out, 2, "catalog", Some("wikifunctions"));
        push_lino_node(&mut out, 2, "function", Some(&self.function_zid));
        if let Some(label) = &self.label {
            push_lino_node(&mut out, 2, "label", Some(label));
        }
        push_lino_node(&mut out, 2, "language", Some(&self.language));
        push_lino_node(&mut out, 2, "code", Some(&self.code));
        append_provenance(
            &mut out,
            &self.license,
            &self.source_url,
            &self.sha256,
            &self.fetched_at,
        );
        out.trim_end().to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionTest {
    pub zid: String,
    pub function_zid: String,
    pub arguments: Vec<String>,
    pub expected: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
}

pub fn search_functions<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    phrase: &str,
    language: &str,
) -> Result<Vec<FunctionMatch>, FetchError> {
    let url = format!(
        "{API}?action=query&format=json&formatversion=2&list=wikilambdasearch_functions&wikilambdasearch_functions_search={}&wikilambdasearch_functions_language={}&wikilambdasearch_functions_limit=5",
        encode_component(phrase),
        encode_component(language),
    );
    let capture = client.fetch(&url)?;
    let value = parse_json(capture.bytes(), "wikifunctions search")?;
    let results: &[Value] = value
        .pointer("/query/wikilambdasearch_functions")
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice);
    Ok(results
        .iter()
        .filter_map(|result| {
            Some(FunctionMatch {
                page_title: string(result, "page_title")?.to_owned(),
                label: string(result, "label")?.to_owned(),
                match_label: string(result, "match_label")?.to_owned(),
                match_lang: string(result, "match_lang")?.to_owned(),
                match_rate: result.get("match_rate")?.as_f64()?,
                source_url: capture.source_url().to_owned(),
                sha256: capture.sha256().to_owned(),
                fetched_at: capture.fetched_at().to_owned(),
            })
        })
        .collect())
}

pub fn fetch_function<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    zid: &str,
) -> Result<FunctionPart, FetchError> {
    let capture = client.fetch(&fetch_url(&[zid]))?;
    let object = fetched_object(&capture, zid)?;
    let function = object
        .pointer("/Z2K2")
        .ok_or_else(|| malformed(zid, "missing Z2K2 function body"))?;
    let argument_types = function
        .get("Z8K1")
        .and_then(Value::as_array)
        .map(|items| reference_list(items, Some("Z17K1")))
        .unwrap_or_default();
    let return_type = string(function, "Z8K2")
        .ok_or_else(|| malformed(zid, "missing Z8K2 return type"))?
        .to_owned();
    Ok(FunctionPart {
        zid: zid.to_owned(),
        labels: multilingual_strings(&object, "Z2K3", "en"),
        argument_types,
        return_type,
        implementation_zids: array_references(function.get("Z8K4"), "Z14"),
        tester_zids: array_references(function.get("Z8K3"), "Z20"),
        license: DEFINITION_LICENSE.to_owned(),
        source_url: capture.source_url().to_owned(),
        sha256: capture.sha256().to_owned(),
        fetched_at: capture.fetched_at().to_owned(),
    })
}

pub fn fetch_implementations<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    zids: &[String],
) -> Result<Vec<Implementation>, FetchError> {
    let requested = zids.iter().map(String::as_str).collect::<Vec<_>>();
    let capture = client.fetch(&fetch_url(&requested))?;
    let outer = outer_objects(&capture)?;
    let mut out = Vec::new();
    for zid in zids {
        let object = decoded_object(&outer, zid)?;
        let body = object
            .pointer("/Z2K2")
            .ok_or_else(|| malformed(zid, "missing Z2K2 implementation body"))?;
        let Some(code) = body.pointer("/Z14K3/Z16K2").and_then(Value::as_str) else {
            continue;
        };
        let Some(language_zid) = body.pointer("/Z14K3/Z16K1").and_then(Value::as_str) else {
            continue;
        };
        let language = match language_zid {
            "Z610" => "python",
            "Z600" => "javascript",
            other => other,
        };
        out.push(Implementation {
            zid: zid.clone(),
            function_zid: string(body, "Z14K1").unwrap_or_default().to_owned(),
            label: multilingual_strings(&object, "Z2K3", "en").remove("en"),
            language: language.to_owned(),
            code: code.to_owned(),
            license: IMPLEMENTATION_LICENSE.to_owned(),
            source_url: capture.source_url().to_owned(),
            sha256: capture.sha256().to_owned(),
            fetched_at: capture.fetched_at().to_owned(),
        });
    }
    Ok(out)
}

pub fn fetch_testers<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    zids: &[String],
) -> Result<Vec<FunctionTest>, FetchError> {
    let requested = zids.iter().map(String::as_str).collect::<Vec<_>>();
    let capture = client.fetch(&fetch_url(&requested))?;
    let outer = outer_objects(&capture)?;
    let mut out = Vec::new();
    for zid in zids {
        let object = decoded_object(&outer, zid)?;
        let body = object
            .pointer("/Z2K2")
            .ok_or_else(|| malformed(zid, "missing Z2K2 tester body"))?;
        let Some(function_zid) = string(body, "Z20K1") else {
            continue;
        };
        let Some(call) = body.get("Z20K2").and_then(Value::as_object) else {
            continue;
        };
        let mut arguments = call
            .iter()
            .filter(|(key, _)| key.starts_with(function_zid) && key.as_str() != "Z7K1")
            .filter_map(|(key, value)| literal(value).map(|literal| (key.clone(), literal)))
            .collect::<Vec<_>>();
        arguments.sort_by(|left, right| left.0.cmp(&right.0));
        let Some(expected) = body.pointer("/Z20K3/Z13522K2").and_then(literal) else {
            continue;
        };
        out.push(FunctionTest {
            zid: zid.clone(),
            function_zid: function_zid.to_owned(),
            arguments: arguments.into_iter().map(|(_, value)| value).collect(),
            expected,
            source_url: capture.source_url().to_owned(),
            sha256: capture.sha256().to_owned(),
            fetched_at: capture.fetched_at().to_owned(),
        });
    }
    Ok(out)
}

fn fetch_url(zids: &[&str]) -> String {
    format!(
        "{API}?action=wikilambda_fetch&format=json&zids={}&language=en",
        zids.join("%7C")
    )
}

fn outer_objects(capture: &SourceCapture) -> Result<Value, FetchError> {
    parse_json(capture.bytes(), "wikilambda_fetch response")
}

fn fetched_object(capture: &SourceCapture, zid: &str) -> Result<Value, FetchError> {
    decoded_object(&outer_objects(capture)?, zid)
}

fn decoded_object(outer: &Value, zid: &str) -> Result<Value, FetchError> {
    let encoded = outer
        .get(zid)
        .and_then(|entry| entry.get("wikilambda_fetch"))
        .and_then(Value::as_str)
        .ok_or_else(|| malformed(zid, "object absent from wikilambda_fetch response"))?;
    serde_json::from_str(encoded).map_err(|error| malformed(zid, &error.to_string()))
}

fn parse_json(bytes: &[u8], context: &str) -> Result<Value, FetchError> {
    serde_json::from_slice(bytes).map_err(|error| malformed(context, &error.to_string()))
}

fn malformed(context: &str, detail: &str) -> FetchError {
    FetchError::Cache(format!("wikifunctions_malformed:{context}:{detail}"))
}

fn string<'a>(object: &'a Value, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

fn array_references(value: Option<&Value>, sentinel: &str) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|reference| *reference != sentinel)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn reference_list(items: &[Value], key: Option<&str>) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| {
            key.map_or_else(
                || item.as_str(),
                |key| item.get(key).and_then(Value::as_str),
            )
        })
        .map(str::to_owned)
        .collect()
}

fn multilingual_strings(
    object: &Value,
    key: &str,
    requested_language: &str,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(items) = object
        .get(key)
        .and_then(|value| value.get("Z12K1"))
        .and_then(Value::as_array)
    {
        for item in items {
            if let Some(text) = item.get("Z11K2").and_then(Value::as_str) {
                out.entry(requested_language.to_owned())
                    .or_insert_with(|| text.to_owned());
            }
        }
    }
    out
}

fn literal(value: &Value) -> Option<String> {
    if let Some(value) = value.as_str() {
        return Some(value.to_owned());
    }
    let object = value.as_object()?;
    for key in ["Z13518K1", "Z6K1"] {
        if let Some(value) = object.get(key).and_then(Value::as_str) {
            return Some(value.to_owned());
        }
    }
    match object.get("Z40K1").and_then(Value::as_str) {
        Some("Z41") => Some("true".to_owned()),
        Some("Z42") => Some("false".to_owned()),
        _ => None,
    }
}

fn append_provenance(out: &mut String, license: &str, url: &str, sha256: &str, fetched_at: &str) {
    push_lino_node(out, 2, "license", Some(license));
    push_lino_node(out, 2, "source_url", Some(url));
    push_lino_node(out, 2, "sha256", Some(sha256));
    push_lino_node(out, 2, "fetched_at", Some(fetched_at));
}

fn encode_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push('%');
            out.push(char::from(HEX[usize::from(byte >> 4)]));
            out.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    out
}
