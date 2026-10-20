//! Shared redacting native diagnostic sink. Protocol stdout is never a sink.
//! The tagged, percent-encoded tree matches JavaScript formal-ai-log-v1.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared protocol envelope identifier used by both the writer and reader.
const WIRE_ENVELOPE: &str = "formal-ai-log-v1";
/// Shallow dialog-only framing tag; the payload remains the original typed tree.
const DIALOG_TREE_TAG: &str = "dialog-tree";
/// Original machine-session evidence prefix shared by producers and readers.
pub const ORCHESTRATION_SESSION_PREFIX: &str = "formal-ai: orchestration-session-json:";

static SILENT: AtomicBool = AtomicBool::new(false);
static SERVE: AtomicBool = AtomicBool::new(false);
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static FILE_DISABLED: AtomicBool = AtomicBool::new(false);
static STDERR_DISABLED: AtomicBool = AtomicBool::new(false);
static FAILURE_REPORTED: AtomicBool = AtomicBool::new(false);
static FILE_HEALTH: OnceLock<Mutex<Value>> = OnceLock::new();

pub fn health_status() -> Value {
    FILE_HEALTH
        .get_or_init(|| Mutex::new(json!({"status":"not_configured"})))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}
fn set_file_health(value: &Value) {
    *FILE_HEALTH
        .get_or_init(|| Mutex::new(Value::Null))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = redact(value);
}
fn open_log_file(directory: &std::path::Path) -> io::Result<std::fs::File> {
    fs::create_dir_all(directory)?;
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(directory.join(format!("serve-{}.lino", std::process::id())))
}
fn file_sink_failed(directory: Option<&std::path::Path>, error: &io::Error) {
    FILE_DISABLED.store(true, Ordering::Relaxed);
    set_file_health(
        &json!({"status":"unwritable","directory":directory,"error":error.to_string(),"error_kind":format!("{:?}", error.kind())}),
    );
    if !FAILURE_REPORTED.swap(true, Ordering::Relaxed) {
        let record = redact(
            &json!({"schema":"formal-ai-diagnostic-v1","level":"error","component":"structured_log","kind":"logging_failure","message":"logging_failed_check_log_directory_permissions_and_space","metadata":health_status()}),
        );
        if let Ok(line) = encode(&record) {
            let _ = writeln!(io::stderr().lock(), "{line}");
        }
    }
}

pub fn configure(silent: bool, serve: bool) {
    SILENT.store(silent, Ordering::Relaxed);
    SERVE.store(serve, Ordering::Relaxed);
    FILE_DISABLED.store(false, Ordering::Relaxed);
    STDERR_DISABLED.store(false, Ordering::Relaxed);
    FAILURE_REPORTED.store(false, Ordering::Relaxed);
    if serve && !silent {
        if let Some(directory) = directory() {
            match open_log_file(&directory).and_then(|mut file| file.flush()) {
                Ok(()) => set_file_health(&json!({"status":"writable","directory":directory})),
                Err(error) => file_sink_failed(Some(&directory), &error),
            }
        } else {
            file_sink_failed(None, &io::Error::other("log directory unavailable"));
        }
    } else {
        set_file_health(&json!({"status":"disabled"}));
    }
}
fn rank(level: &str) -> u8 {
    match level {
        "error" => 1,
        "warn" => 2,
        "info" => 3,
        "trace" => 5,
        "silent" | "off" => 0,
        _ => 4,
    }
}
#[must_use]
pub fn enabled(level: &str) -> bool {
    !SILENT.load(Ordering::Relaxed)
        && !crate::solver_config::flag_enabled("FORMAL_AI_SILENT")
        && rank(level)
            <= rank(&std::env::var("FORMAL_AI_LOG_LEVEL").unwrap_or_else(|_| "debug".into()))
}
fn secret_key(key: &str) -> bool {
    let key: String = key
        .chars()
        .filter(|c| *c != '_' && *c != '-')
        .flat_map(char::to_lowercase)
        .collect();
    matches!(
        key.as_str(),
        "authorization"
            | "proxyauthorization"
            | "password"
            | "passwd"
            | "cookie"
            | "setcookie"
            | "privatekey"
    ) || key.ends_with("token")
        || key.ends_with("secret")
        || key.ends_with("apikey")
}
/// Redact before formatting, hashing, or persistence, including nested errors.
#[must_use]
pub fn redact(value: &Value) -> Value {
    fn walk(value: &Value, secrets: &[String]) -> Value {
        let scrub = |text: &str| {
            secrets.iter().fold(text.to_owned(), |text, secret| {
                text.replace(secret, "[REDACTED]")
            })
        };
        match value {
            Value::String(text) => Value::String(scrub(text)),
            Value::Array(items) => {
                Value::Array(items.iter().map(|item| walk(item, secrets)).collect())
            }
            Value::Object(fields) => {
                let mut output = serde_json::Map::new();
                for (key, value) in fields {
                    let key_safe = scrub(key);
                    if output.contains_key(&key_safe) {
                        return json!({"redaction_failure":"key_collision"});
                    }
                    output.insert(
                        key_safe,
                        if secret_key(key) {
                            json!("[REDACTED]")
                        } else {
                            walk(value, secrets)
                        },
                    );
                }
                Value::Object(output)
            }
            other => other.clone(),
        }
    }
    let mut secrets: Vec<String> = std::env::vars()
        .filter(|(key, value)| secret_key(key) && !value.is_empty())
        .map(|(_, value)| value)
        .collect();
    secrets.sort_by_key(|value| std::cmp::Reverse(value.len()));
    walk(value, &secrets)
}
fn word(text: &str) -> String {
    if text.is_empty() {
        return "%".into();
    }
    let mut output = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"._~*/<>=!&|+-".contains(&byte) {
            output.push(char::from(byte));
        } else {
            write!(&mut output, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    output
}
fn typed(value: &Value, preserve_numbers: bool) -> io::Result<String> {
    Ok(match value {
        Value::Null => "(null)".into(),
        Value::Bool(value) => format!("(boolean {value})"),
        Value::String(value) => format!("(string {})", word(value)),
        Value::Number(value) => {
            let unsafe_integer = value
                .as_i64()
                .is_some_and(|n| n.unsigned_abs() > 9_007_199_254_740_991)
                || value.as_u64().is_some_and(|n| n > 9_007_199_254_740_991);
            if value.is_f64() {
                if !preserve_numbers {
                    return Err(io::Error::other("unsupported_diagnostic_float"));
                }
                format!("(float 64 {value})")
            } else if unsafe_integer {
                if !preserve_numbers {
                    return Err(io::Error::other("unsupported_diagnostic_integer"));
                }
                format!("(integer {value})")
            } else {
                format!("(number {value})")
            }
        }
        Value::Array(items) => format!(
            "(array{})",
            items
                .iter()
                .map(|v| typed(v, preserve_numbers).map(|v| format!(" {v}")))
                .collect::<io::Result<String>>()?
        ),
        Value::Object(fields) => {
            let mut fields: Vec<_> = fields.iter().collect();
            fields.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            let fields = fields
                .into_iter()
                .map(|(key, value)| {
                    typed(value, preserve_numbers)
                        .map(|value| format!(" (field {} {value})", word(key)))
                })
                .collect::<io::Result<String>>()?;
            format!("(object{fields})")
        }
    })
}
/// Validate every physical record with the actual dependency grammar.
pub fn encode(value: &Value) -> io::Result<String> {
    encode_with_numbers(value, false)
}
/// Lossless dialog records opt into integer/f64 typed extensions.
pub fn encode_dialog(value: &Value) -> io::Result<String> {
    let line = encode_with_numbers(value, true)?;
    // Validate the same reader before callers append: accepted rows remain readable.
    let decoded = decode_with_numbers(&line, true)?;
    if &decoded != value {
        return Err(io::Error::other("dialog_roundtrip_failed"));
    }
    Ok(line)
}
fn encode_with_numbers(value: &Value, preserve_numbers: bool) -> io::Result<String> {
    let tree = typed(value, preserve_numbers)?;
    // The pinned dependency grammar permits at most 64 physical nesting levels.
    // Dialogs retain deeper typed trees as a single encoded word, not JSON.
    let mut depth = 1_usize;
    let mut maximum_depth = depth;
    for byte in tree.bytes() {
        if byte == b'(' {
            depth += 1;
            maximum_depth = maximum_depth.max(depth);
        }
        if byte == b')' {
            depth -= 1;
        }
    }
    let line = if preserve_numbers && maximum_depth > 64 {
        format!("({WIRE_ENVELOPE} ({DIALOG_TREE_TAG} {}))", word(&tree))
    } else {
        format!("({WIRE_ENVELOPE} {tree})")
    };
    links_notation::parser::parse_document_with_diagnostics(&line)
        .map_err(|_| io::Error::other("invalid_log_notation"))?;
    Ok(line)
}
fn timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = elapsed.as_secs();
    // Gregorian civil date from Unix days; arithmetic follows 400-year eras.
    let z = i64::try_from(seconds / 86_400).unwrap_or(0) + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:09}Z",
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60,
        elapsed.subsec_nanos()
    )
}
fn directory() -> Option<PathBuf> {
    std::env::var_os("FORMAL_AI_LOG_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".formal-ai/logs"))
        })
}
/// Render the legacy request trace from its declarative wire header.
#[must_use]
pub fn request_trace_message(method: &str, path: &str, body: &str) -> String {
    static HEADER: OnceLock<String> = OnceLock::new();
    let header = HEADER.get_or_init(|| {
        let tree = crate::seed::parser::parse_lino(include_str!(
            "../embedded/data/meta/structured-log-wire.lino"
        ));
        tree.children
            .iter()
            .find(|record| record.name == "structured-log-wire")
            .expect("embedded structured log wire policy")
            .find_child_value("request-trace-header")
            .to_owned()
    });
    let bytes = body.len().to_string();
    let header = crate::seed::fill_template_once(
        header,
        &[("method", method), ("path", path), ("body_bytes", &bytes)],
    );
    format!("{header}\n{body}")
}

// Producer budgets mirror js/logging/diagnostic-metadata.mjs, not reader policy.
const DIAGNOSTIC_FIELD_BYTES: usize = 4096;
const DIAGNOSTIC_METADATA_BYTES: usize = 64 * 1024;
const DIAGNOSTIC_PREVIEW_BYTES: usize = 256;

fn diagnostic_metadata_summary(value: &Value, canonical: &str) -> Value {
    let content = value.as_str().unwrap_or(canonical);
    let mut end = content.len().min(DIAGNOSTIC_PREVIEW_BYTES);
    while !content.is_char_boundary(end) {
        end -= 1;
    }
    json!({"diagnostic_summary":"post-redaction/v1","encoding":if value.is_string(){"utf8"}else{"formal-ai-log-v1"},"bytes":content.len(),"sha256":crate::source_fetch::hex_lower(&Sha256::digest(content.as_bytes())),"preview":&content[..end],"preview_truncated":end<content.len()})
}

fn bounded_diagnostic_value(value: &Value) -> Value {
    // Raw UTF-8 size proves oversized encoded cost without parsing huge CST strings.
    if value
        .as_str()
        .is_some_and(|text| text.len() > DIAGNOSTIC_FIELD_BYTES)
    {
        return diagnostic_metadata_summary(value, "");
    }
    let Ok(canonical) = encode(value) else {
        return value.clone();
    };
    if canonical.len() > DIAGNOSTIC_FIELD_BYTES {
        diagnostic_metadata_summary(value, &canonical)
    } else {
        value.clone()
    }
}

/// Bound already-redacted diagnostic fields using their actual percent-encoded cost.
/// Dialogs, payload custody, control records and the shared codecs do not use this.
fn bounded_diagnostic_metadata(value: &Value) -> Value {
    let Some(fields) = value.as_object() else {
        return bounded_diagnostic_value(value);
    };
    let mut candidates = Vec::new();
    for (key, item) in fields {
        let projected = bounded_diagnostic_value(item);
        let singleton = Value::Object(std::iter::once((key.clone(), projected.clone())).collect());
        let cost = if key.len() > DIAGNOSTIC_METADATA_BYTES {
            DIAGNOSTIC_METADATA_BYTES + 1
        } else {
            let Ok(encoded) = encode(&singleton) else {
                return value.clone();
            };
            encoded.len()
        };
        candidates.push((cost, key.clone(), projected));
    }
    candidates.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.as_bytes().cmp(right.1.as_bytes()))
    });
    let mut remaining = DIAGNOSTIC_METADATA_BYTES - 2048;
    let mut omitted = 0;
    let mut fields = serde_json::Map::new();
    for (cost, key, value) in candidates {
        if cost > remaining {
            omitted += 1;
            continue;
        }
        fields.insert(key, value);
        remaining -= cost;
    }
    if omitted == 0 {
        Value::Object(fields)
    } else {
        let Ok(canonical) = encode(value) else {
            return value.clone();
        };
        json!({"diagnostic_metadata":"bounded-post-redaction/v1","fields":fields,"omitted_fields":omitted,"omitted":diagnostic_metadata_summary(value, &canonical)})
    }
}

/// Fields and message closures run only after level filtering.
pub fn event(
    level: &str,
    component: &str,
    message: impl FnOnce() -> String,
    fields: impl FnOnce() -> Value,
) {
    if !enabled(level) {
        return;
    }
    let message = redact(&json!(message()));
    let mut record = json!({"schema":"formal-ai-diagnostic-v1", "timestamp":timestamp(), "timestamp_precision":"nanoseconds", "pid":std::process::id(), "sequence":SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1, "level":level, "component":component, "dialog_id":crate::dialog_log::current_dialog_id(), "message":message, "kind":"native_diagnostic", "metadata":redact(&fields())});
    if let Some(message) = record["message"].as_str().filter(|text| text.len() > 4096)
        && (level != "trace"
            || !crate::solver_config::flag_enabled("FORMAL_AI_LOG_PAYLOAD_CUSTODY"))
    {
        let digest = crate::source_fetch::hex_lower(&Sha256::digest(message.as_bytes()));
        let bytes = message.len();
        record["payload"] = json!({"sha256":digest,"bytes":bytes});
        record["message"] = json!("large_native_diagnostic");
    }
    record["metadata"] = bounded_diagnostic_metadata(&record["metadata"]);
    record["component"] = bounded_diagnostic_metadata(&record["component"]);
    record["dialog_id"] = bounded_diagnostic_metadata(&record["dialog_id"]);
    write_record(&record);
}

/// Emit machine evidence independently of diagnostic verbosity and payload elision.
/// This preserves the existing orchestration control channel; it grants no authority.
pub fn control(kind: &str, component: &str, message: &str, metadata: &Value) {
    let record = json!({"schema":"formal-ai-control-v1", "timestamp":timestamp(), "timestamp_precision":"nanoseconds", "pid":std::process::id(), "sequence":SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1, "level":"info", "component":component, "dialog_id":crate::dialog_log::current_dialog_id(), "message":message, "kind":kind, "metadata":metadata});
    write_record(&record);
}

fn write_record(record: &Value) {
    let Ok(line) = encode(&redact(record)) else {
        sink_failure("encoding_failed");
        return;
    };
    let _guard = WRITE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !STDERR_DISABLED.load(Ordering::Relaxed) && writeln!(io::stderr().lock(), "{line}").is_err()
    {
        STDERR_DISABLED.store(true, Ordering::Relaxed);
        sink_failure("stderr_write_failed");
    }
    if SERVE.load(Ordering::Relaxed) && !FILE_DISABLED.load(Ordering::Relaxed) {
        if let Some(directory) = directory() {
            let write = || -> io::Result<()> {
                let mut file = open_log_file(&directory)?;
                writeln!(file, "{line}")?;
                file.flush()
            };
            if let Err(error) = write() {
                file_sink_failed(Some(&directory), &error);
            }
        } else {
            file_sink_failed(None, &io::Error::other("log directory unavailable"));
        }
    }
}
/// A bounded nonrecursive fallback: no caller text, paths, or IO errors enter it.
fn sink_failure(reason: &str) {
    if FAILURE_REPORTED.swap(true, Ordering::Relaxed) {
        return;
    }
    let record = json!({"schema":"formal-ai-diagnostic-v1", "level":"error", "component":"structured_log", "message":"logging_failed_check_log_directory_permissions_and_space", "kind":"logging_failure", "timestamp":timestamp(), "timestamp_precision":"nanoseconds", "pid":std::process::id(), "sequence":SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1, "dialog_id":null, "reason":reason});
    if let Ok(line) = encode(&record) {
        let _ = writeln!(io::stderr().lock(), "{line}");
    }
}
/// Compatibility migration: formatting stays lazy and all paths share redaction.
#[macro_export]
macro_rules! diagnostic {
    ($level:literal; $($argument:tt)*) => {
        $crate::dialog_log::structured_log::event($level, module_path!(), || format!($($argument)*), || serde_json::json!({}))
    };
    ($($argument:tt)*) => {
        $crate::dialog_log::structured_log::event("debug", module_path!(), || format!($($argument)*), || serde_json::json!({}))
    };
}

/// Decode canonical shared diagnostic records, refusing alternate spellings.
/// The dependency validates grammar before this typed interpretation runs.
pub fn decode(line: &str) -> io::Result<Value> {
    decode_with_numbers(line, false)
}
pub fn decode_dialog(line: &str) -> io::Result<Value> {
    decode_with_numbers(line, true)
}
/// Decode an already captured control line without relaxing diagnostic defaults.
/// Numeric/depth refusals remain those of the narrow shared codec.
pub fn decode_control(line: &str) -> io::Result<Value> {
    let record = decode_with_policy(line, false, Some(line.len()))?;
    if record["schema"] != "formal-ai-control-v1" {
        return Err(io::Error::other("invalid_control_schema"));
    }
    Ok(record)
}
fn decode_with_numbers(line: &str, preserve_numbers: bool) -> io::Result<Value> {
    decode_with_policy(
        line,
        preserve_numbers,
        if preserve_numbers {
            None
        } else {
            Some(1024 * 1024)
        },
    )
}
fn decode_with_policy(
    line: &str,
    preserve_numbers: bool,
    maximum_record_bytes: Option<usize>,
) -> io::Result<Value> {
    fn invalid() -> io::Error {
        io::Error::other("invalid_typed_log_record")
    }
    fn unword(token: &str) -> io::Result<String> {
        if token == "%" {
            return Ok(String::new());
        }
        let bytes = token.as_bytes();
        let mut output = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' {
                let hex = token.get(index + 1..index + 3).ok_or_else(invalid)?;
                output.push(u8::from_str_radix(hex, 16).map_err(|_| invalid())?);
                index += 3;
            } else {
                output.push(bytes[index]);
                index += 1;
            }
        }
        String::from_utf8(output).map_err(|_| invalid())
    }
    fn expect(tokens: &[&str], position: &mut usize, expected: &str) -> io::Result<()> {
        if tokens.get(*position) != Some(&expected) {
            return Err(invalid());
        }
        *position += 1;
        Ok(())
    }
    fn item(
        tokens: &[&str],
        position: &mut usize,
        depth: usize,
        preserve_numbers: bool,
    ) -> io::Result<Value> {
        if !preserve_numbers && depth > 64 {
            return Err(invalid());
        }
        expect(tokens, position, "(")?;
        let tag = *tokens.get(*position).ok_or_else(invalid)?;
        *position += 1;
        let value = match tag {
            "null" => Value::Null,
            "string" | "number" | "boolean" => {
                let word = *tokens.get(*position).ok_or_else(invalid)?;
                *position += 1;
                match tag {
                    "string" => Value::String(unword(word)?),
                    "boolean" if word == "true" => Value::Bool(true),
                    "boolean" if word == "false" => Value::Bool(false),
                    "number" => serde_json::from_str::<Value>(word)
                        .ok()
                        .filter(Value::is_number)
                        .ok_or_else(invalid)?,
                    _ => return Err(invalid()),
                }
            }
            "integer" if preserve_numbers => {
                let word = *tokens.get(*position).ok_or_else(invalid)?;
                *position += 1;
                serde_json::from_str::<Value>(word)
                    .ok()
                    .filter(|value| value.is_i64() || value.is_u64())
                    .ok_or_else(invalid)?
            }
            "float" if preserve_numbers => {
                expect(tokens, position, "64")?;
                let word = *tokens.get(*position).ok_or_else(invalid)?;
                *position += 1;
                let number: f64 = word.parse().map_err(|_| invalid())?;
                serde_json::Number::from_f64(number)
                    .map(Value::Number)
                    .ok_or_else(invalid)?
            }
            "array" => {
                let mut values = Vec::new();
                while tokens.get(*position) != Some(&")") {
                    values.push(item(tokens, position, depth + 1, preserve_numbers)?);
                }
                Value::Array(values)
            }
            "object" => {
                let mut values = serde_json::Map::new();
                while tokens.get(*position) != Some(&")") {
                    expect(tokens, position, "(")?;
                    expect(tokens, position, "field")?;
                    let key = unword(tokens.get(*position).ok_or_else(invalid)?)?;
                    *position += 1;
                    let value = item(tokens, position, depth + 1, preserve_numbers)?;
                    expect(tokens, position, ")")?;
                    if values.insert(key, value).is_some() {
                        return Err(invalid());
                    }
                }
                Value::Object(values)
            }
            _ => return Err(invalid()),
        };
        expect(tokens, position, ")")?;
        Ok(value)
    }
    if maximum_record_bytes.is_some_and(|limit| line.len() > limit) || line.contains(['\n', '\r']) {
        return Err(invalid());
    }
    links_notation::parser::parse_document_with_diagnostics(line).map_err(|_| invalid())?;
    let prefix = format!("({WIRE_ENVELOPE} ({DIALOG_TREE_TAG} ");
    let expanded = if preserve_numbers && line.starts_with(&prefix) {
        let token = line
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix("))"))
            .ok_or_else(invalid)?;
        format!("({WIRE_ENVELOPE} {})", unword(token)?)
    } else {
        line.to_owned()
    };
    let spaced = expanded.replace('(', " ( ").replace(')', " ) ");
    let tokens: Vec<_> = spaced.split_whitespace().collect();
    let mut position = 0;
    expect(&tokens, &mut position, "(")?;
    expect(&tokens, &mut position, WIRE_ENVELOPE)?;
    let value = item(&tokens, &mut position, 0, preserve_numbers)?;
    expect(&tokens, &mut position, ")")?;
    if position != tokens.len() || encode_with_numbers(&value, preserve_numbers)? != line {
        return Err(invalid());
    }
    Ok(value)
}

#[cfg(test)]
#[path = "../tests/source/source_tests/structured_log/tests.rs"]
mod tests;
