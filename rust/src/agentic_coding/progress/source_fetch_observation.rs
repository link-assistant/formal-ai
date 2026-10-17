//! Host observations feed one generated policy; reports never grant authority.
use super::super::coding_research::kernel as capture_policy;
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

fn report_record<'a>(
    value: &'a Value,
    allowed: &[&str],
) -> Option<&'a serde_json::Map<String, Value>> {
    value
        .as_object()
        .filter(|fields| fields.keys().all(|key| allowed.contains(&key.as_str())))
}

// JSON is the common numeric observation boundary; policy count limits are canonical.
fn observed_count(value: usize) -> f64 {
    json!(value).as_f64().unwrap_or(f64::NAN)
}

fn observed_text(value: &Value) -> (bool, f64) {
    value.as_str().map_or((false, 0.0), |text| {
        (true, observed_count(text.chars().count()))
    })
}

fn reported(value: &Value, valid: bool) -> Value {
    if valid {
        json!({"status":"ProviderReported","value":value})
    } else {
        json!({"status":"Unknown"})
    }
}

pub(super) fn source_fetch_observation(
    messages: &[ChatMessage],
    start: usize,
    url: &str,
    text: &str,
) -> Value {
    let digest = crate::source_fetch::sha256_hex(text.as_bytes());
    let mut observation = json!({"schema":"formal-ai-source-observation-v1","url":url,"sha256":digest,
        "binding":{"status":"Unknown","callId":null,"tool":null},"authority":"Unknown",
        "fetchedAt":{"status":"Unknown"},"cached":{"status":"Unknown"},
        "trust":{"status":"Unknown"},"licence":{"status":"Unknown"}});
    let declarations: Vec<_> = messages
        .iter()
        .enumerate()
        .skip(start)
        .filter(|(_, message)| message.role.eq_ignore_ascii_case("assistant"))
        .flat_map(|(index, message)| message.tool_calls.iter().map(move |call| (index, call)))
        .collect();
    let receipts: Vec<_> = messages
        .iter()
        .enumerate()
        .skip(start)
        .filter(|(_, message)| message.role.eq_ignore_ascii_case("tool"))
        .collect();
    let matches: Vec<_> = declarations
        .iter()
        .filter_map(|(index, call)| {
            let declaration = capture_policy::source_capture_declaration_accepted(
                !call.id.is_empty(),
                super::classify_tool(&call.function.name) == Some(super::Capability::Fetch),
                super::argument_url(&call.function.arguments).as_deref() == Some(url),
                observed_count(
                    declarations
                        .iter()
                        .filter(|(_, other)| other.id == call.id)
                        .count(),
                ),
            );
            let results: Vec<_> = receipts
                .iter()
                .filter(|(_, message)| message.tool_call_id.as_deref() == Some(call.id.as_str()))
                .collect();
            let receipt = results.first();
            let receipt_identity = capture_policy::source_capture_receipt_identity_accepted(
                observed_count(results.len()),
                receipt.is_some_and(|result| result.0 > *index),
                receipt.is_some_and(|result| {
                    result.1.name.as_deref() == Some(call.function.name.as_str())
                }),
            );
            let outcome = capture_policy::source_capture_all_observed(
                receipt.is_some_and(|result| !result.1.is_error),
                receipt.is_some_and(|result| result.1.content.plain_text() == text),
                true,
            );
            let eligible =
                capture_policy::source_capture_all_observed(declaration, receipt_identity, outcome);
            if eligible {
                Some((*call, receipt?.1))
            } else {
                None
            }
        })
        .collect();
    if !capture_policy::source_capture_unique_accepted(observed_count(matches.len())) {
        return observation;
    }
    let (call, message) = matches[0];
    observation["binding"] =
        json!({"status":"TranscriptBound","callId":call.id,"tool":call.function.name});
    let Some(capture_value) = message.source_capture.as_ref() else {
        return observation;
    };
    let capture = report_record(
        capture_value,
        &[
            "schema",
            "callId",
            "url",
            "sha256",
            "fetchedAt",
            "cached",
            "trust",
            "licence",
        ],
    );
    let identity = capture_policy::source_capture_all_observed(
        capture_value["schema"].as_str() == Some("formal-ai-source-capture-v1"),
        capture_value["callId"].as_str() == Some(call.id.as_str()),
        capture_value["url"].as_str() == Some(url),
    );
    if !capture_policy::source_capture_all_observed(
        capture.is_some(),
        identity,
        capture_value["sha256"].as_str() == Some(digest.as_str()),
    ) {
        return observation;
    }
    let Some(capture) = capture else {
        return observation;
    };
    let value = |key: &str| capture.get(key).unwrap_or(&Value::Null);
    let fetched_at = value("fetchedAt");
    let timestamp = fetched_at.as_str();
    let timestamp_syntax = capture_policy::source_capture_timestamp_text_accepted(
        timestamp.is_some(),
        timestamp.is_some_and(|text| text.bytes().all(|byte| byte.is_ascii_digit())),
        timestamp.is_some_and(|text| text.starts_with('0')),
        observed_text(fetched_at).1,
    );
    observation["fetchedAt"] = reported(
        fetched_at,
        capture_policy::source_capture_all_observed(
            timestamp_syntax,
            timestamp.is_some_and(|text| text.parse::<u64>().is_ok()),
            true,
        ),
    );
    let cached = value("cached");
    observation["cached"] = reported(cached, cached.is_boolean());
    let trust = value("trust");
    let trust_record = report_record(trust, &["score", "policy"]);
    let score = trust["score"].as_f64();
    let (policy_observed, policy_length) = observed_text(&trust["policy"]);
    let valid_trust = capture_policy::source_capture_trust_accepted(
        trust_record.is_some(),
        observed_count(trust_record.map_or(0, serde_json::Map::len)),
        score.is_some(),
        score.unwrap_or(f64::NAN),
        policy_observed,
        policy_length,
    );
    let mut canonical_trust = if valid_trust {
        trust.clone()
    } else {
        Value::Null
    };
    if valid_trust
        && let Some(score) = score.filter(|score| score.fract() == 0.0)
        && let Ok(integer) = format!("{score:.0}").parse::<i64>()
    {
        canonical_trust["score"] = json!(integer);
    }
    observation["trust"] = reported(&canonical_trust, valid_trust);
    let licence = value("licence");
    let licence_record = report_record(licence, &["name", "url"]);
    let (name_observed, name_length) = observed_text(&licence["name"]);
    let (url_observed, url_length) = observed_text(&licence["url"]);
    observation["licence"] = reported(
        licence,
        capture_policy::source_capture_licence_accepted(
            licence_record.is_some(),
            observed_count(licence_record.map_or(0, serde_json::Map::len)),
            name_observed,
            name_length,
            url_observed,
            url_length,
        ),
    );
    observation
}

#[cfg(test)]
#[path = "../../../tests/fixtures/source-fetch-provenance.rs"]
mod tests;
