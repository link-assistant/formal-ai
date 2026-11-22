//! Bounded JSON Schema validation for output values. Required keys must be
//! present. Their schemas determine whether a null or empty value is valid.
use serde_json::Value;
use std::collections::HashSet;
use std::sync::LazyLock;

const MAXIMUM_DEPTH: usize = 32;
const MAXIMUM_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
const TYPES: [&str; 7] = [
    "object", "array", "string", "number", "integer", "boolean", "null",
];

fn reference_pointer(pointer: &str) -> Option<String> {
    let mut bytes = pointer.strip_prefix('#')?.bytes();
    let mut decoded = Vec::new();
    while let Some(byte) = bytes.next() {
        decoded.push(if byte == b'%' {
            let high = char::from(bytes.next()?).to_digit(16)?;
            let low = char::from(bytes.next()?).to_digit(16)?;
            u8::try_from(high * 16 + low).ok()?
        } else {
            byte
        });
    }
    let pointer = String::from_utf8(decoded).ok()?;
    (pointer.is_empty() || pointer.starts_with('/')).then_some(pointer)
}

pub(super) fn reference<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    root.pointer(&reference_pointer(pointer)?)
}

struct Pattern {
    regex: regex::Regex,
    forbidden: Vec<regex::Regex>,
}

impl Pattern {
    fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text) && self.forbidden.iter().all(|guard| !guard.is_match(text))
    }
}

fn pattern(text: &str) -> Option<Pattern> {
    if text.len() > 128
        || !text.bytes().all(|byte| matches!(byte, b' '..=b'~'))
        || ["[^", "&&", "--", "~~"]
            .into_iter()
            .any(|op| text.contains(op))
    {
        return None;
    }
    let needs_anchored_subset = text.contains(['.', '(', ')', '|', '{', '}', '\\'])
        || text
            .bytes()
            .filter(|byte| matches!(byte, b'+' | b'*' | b'?'))
            .count()
            > 1;
    if !needs_anchored_subset {
        return Some(Pattern {
            regex: regex::Regex::new(text).ok()?,
            forbidden: Vec::new(),
        });
    }
    let mut tail = text.strip_prefix('^')?;
    let mut forbidden = Vec::new();
    while let Some(guard) = NEGATIVE_PREFIX.captures(tail) {
        forbidden.push(regex::Regex::new(&format!("^{}", &guard[1])).ok()?);
        tail = &tail[guard.get(0)?.end()..];
    }
    if !ANCHORED_PATTERN.is_match(tail) {
        return None;
    }
    Some(Pattern {
        regex: regex::Regex::new(&format!("^{tail}")).ok()?,
        forbidden,
    })
}

// Same anchored grammar as server/structured-pattern.mjs. The regex crate
// handles matching without backtracking.
static ANCHORED_PATTERN: LazyLock<regex::Regex> = LazyLock::new(|| {
    let atom = r"(?:\[(?:\\[-.]|[A-Za-z0-9 _'+@-])+\]|\\\.|[A-Za-z0-9 _'@-])";
    let quantifier = r"(?:[?*+]|\{[1-8],\})";
    let term = format!("{atom}{quantifier}?");
    regex::Regex::new(&format!(r"\A(?:{term}|\((?:\?:)?(?:{term})+\)[?*+]?)+\$\z"))
        .expect("anchored structured pattern grammar")
});
static NEGATIVE_PREFIX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\A\(\?!((?:\.\*)?(?:\\\.|[A-Za-z0-9_@-])+)\)")
        .expect("anchored structured pattern prefix grammar")
});

fn supported_dialect(value: &Value) -> bool {
    value
        .as_str()
        .and_then(|uri| {
            uri.strip_prefix("https://json-schema.org/")
                .or_else(|| uri.strip_prefix("http://json-schema.org/"))
        })
        .is_some_and(|dialect| {
            matches!(
                dialect.strip_suffix('#').unwrap_or(dialect),
                "draft-07/schema" | "draft/2019-09/schema" | "draft/2020-12/schema"
            )
        })
}

fn child_pointer(pointer: &str, key: &str) -> String {
    format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1"))
}

pub(super) struct SchemaIssue<'a> {
    keyword: &'a str,
    pointer: String,
    unsupported: bool,
}

impl SchemaIssue<'_> {
    pub(super) fn message(&self) -> String {
        let key = if self.unsupported {
            "structured-output-schema-unsupported-keyword"
        } else {
            "structured-output-schema-invalid-keyword"
        };
        let keyword = serde_json::to_string(self.keyword).expect("serialize schema keyword");
        let pointer = if self.pointer.is_empty() {
            "\"\""
        } else {
            self.pointer.as_str()
        };
        crate::seed::agentic_message(key, &[("keyword", &keyword), ("pointer", pointer)])
    }
}

pub(super) fn check(root: &Value) -> Result<(), SchemaIssue<'_>> {
    fn visit<'a>(
        schema: &'a Value,
        root: &'a Value,
        pointer: &str,
        depth: usize,
        keyword: &'a str,
        visited: &mut HashSet<*const Value>,
    ) -> Result<(), SchemaIssue<'a>> {
        let invalid = |keyword| SchemaIssue {
            keyword,
            pointer: pointer.to_owned(),
            unsupported: false,
        };
        let location_depth = pointer.bytes().filter(|byte| *byte == b'/').count();
        if depth > MAXIMUM_DEPTH || location_depth > MAXIMUM_DEPTH {
            return Err(invalid(keyword));
        }
        if !visited.insert(std::ptr::from_ref(schema)) || schema.is_boolean() {
            return Ok(());
        }
        let fields = schema.as_object().ok_or_else(|| invalid(keyword))?;
        let kind = |value: &Value| value.as_str().is_some_and(|kind| TYPES.contains(&kind));
        for (key, value) in fields {
            let key = key.as_str();
            if matches!(
                key,
                "title"
                    | "description"
                    | "default"
                    | "examples"
                    | "$id"
                    | "$comment"
                    | "deprecated"
                    | "readOnly"
                    | "writeOnly"
                    | "contentMediaType"
                    | "contentEncoding"
                    | "format"
            ) {
                continue;
            }
            let field_depth = location_depth + 1;
            if field_depth > MAXIMUM_DEPTH {
                return Err(invalid(key));
            }
            let mut nested = |part, at: &str| visit(part, root, at, depth + 1, key, visited);
            let valid = match key {
                "$schema" => supported_dialect(value),
                "$ref" => {
                    let target_pointer = value
                        .as_str()
                        .and_then(reference_pointer)
                        .ok_or_else(|| invalid(key))?;
                    let target = root.pointer(&target_pointer).ok_or_else(|| invalid(key))?;
                    nested(target, &target_pointer)?;
                    continue;
                }
                "$defs" | "definitions" | "properties" => {
                    let entries = value.as_object().ok_or_else(|| invalid(key))?;
                    let at = child_pointer(pointer, key);
                    for (name, part) in entries {
                        nested(part, &child_pointer(&at, name))?;
                    }
                    continue;
                }
                "additionalProperties" | "items" | "not" => {
                    nested(value, &child_pointer(pointer, key))?;
                    continue;
                }
                "anyOf" | "allOf" | "oneOf" => {
                    let items = value
                        .as_array()
                        .filter(|items| !items.is_empty())
                        .ok_or_else(|| invalid(key))?;
                    let at = child_pointer(pointer, key);
                    for (index, part) in items.iter().enumerate() {
                        nested(part, &format!("{at}/{index}"))?;
                    }
                    continue;
                }
                "required" => value
                    .as_array()
                    .is_some_and(|items| items.iter().all(Value::is_string)),
                "enum" => value.as_array().is_some_and(|items| !items.is_empty()),
                "type" => {
                    kind(value)
                        || value
                            .as_array()
                            .is_some_and(|items| !items.is_empty() && items.iter().all(kind))
                }
                "minLength" | "maxLength" | "minItems" | "maxItems" | "minProperties"
                | "maxProperties" => value
                    .as_f64()
                    .is_some_and(|n| n.fract() == 0.0 && (0.0..=MAXIMUM_SAFE_INTEGER).contains(&n)),
                "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" => {
                    value.is_number()
                }
                "pattern" => value.as_str().and_then(pattern).is_some(),
                "const" => true,
                _ => {
                    return Err(SchemaIssue {
                        unsupported: true,
                        ..invalid(key)
                    });
                }
            };
            if !valid || !portable(value, field_depth) {
                return Err(invalid(key));
            }
        }
        Ok(())
    }
    if !root.is_object() {
        return Err(SchemaIssue {
            keyword: "schema",
            pointer: String::new(),
            unsupported: false,
        });
    }
    visit(root, root, "", 0, "schema", &mut HashSet::new())
}

fn json_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(_), Value::Number(_)) => left.as_f64() == right.as_f64(),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len() && left.iter().zip(right).all(|(a, b)| json_equal(a, b))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .all(|(key, a)| right.get(key).is_some_and(|b| json_equal(a, b)))
        }
        _ => left == right,
    }
}

pub(super) fn valid(value: &Value, schema: &Value, root: &Value) -> bool {
    struct Validator<'a> {
        root: &'a Value,
        remaining: usize,
        exhausted: bool,
    }
    impl Validator<'_> {
        fn valid(&mut self, value: &Value, schema: &Value, depth: usize) -> bool {
            if self.remaining == 0 || depth > MAXIMUM_DEPTH {
                self.exhausted = true;
                return false;
            }
            self.remaining -= 1;
            if let Some(allowed) = schema.as_bool() {
                return allowed;
            }
            if let Some(pointer) = schema.get("$ref").and_then(Value::as_str)
                && !reference(self.root, pointer)
                    .is_some_and(|part| self.valid(value, part, depth + 1))
            {
                return false;
            }
            let mut nested = |part: &Value| self.valid(value, part, depth + 1);
            if schema
                .get("allOf")
                .and_then(Value::as_array)
                .is_some_and(|parts| !parts.iter().all(&mut nested))
                || schema
                    .get("anyOf")
                    .and_then(Value::as_array)
                    .is_some_and(|parts| !parts.iter().any(&mut nested))
                || schema
                    .get("oneOf")
                    .and_then(Value::as_array)
                    .is_some_and(|parts| parts.iter().filter(|part| nested(part)).count() != 1)
                || schema.get("not").is_some_and(nested)
            {
                return false;
            }
            let matches = |kind: &Value| match kind.as_str() {
                Some("object") => value.is_object(),
                Some("array") => value.is_array(),
                Some("string") => value.is_string(),
                Some("number") => value.is_number(),
                Some("integer") => value.as_f64().is_some_and(|n| n.fract() == 0.0),
                Some("boolean") => value.is_boolean(),
                Some("null") => value.is_null(),
                _ => false,
            };
            if schema.get("type").is_some_and(|kind| {
                kind.as_array()
                    .map_or_else(|| !matches(kind), |kinds| !kinds.iter().any(matches))
            }) || schema
                .get("enum")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.iter().any(|item| json_equal(item, value)))
                || schema
                    .get("const")
                    .is_some_and(|item| !json_equal(item, value))
            {
                return false;
            }
            let bound =
                |key: &str, default| schema.get(key).and_then(Value::as_f64).unwrap_or(default);
            let count_valid = |length: usize, min, max| {
                let length = u32::try_from(length).map_or(f64::INFINITY, f64::from);
                length >= bound(min, 0.0) && length <= bound(max, f64::INFINITY)
            };
            match value {
                Value::Number(n) => n.as_f64().is_some_and(|n| {
                    n >= bound("minimum", f64::NEG_INFINITY)
                        && n <= bound("maximum", f64::INFINITY)
                        && n > bound("exclusiveMinimum", f64::NEG_INFINITY)
                        && n < bound("exclusiveMaximum", f64::INFINITY)
                }),
                Value::String(text) => {
                    let length = text.chars().count();
                    count_valid(length, "minLength", "maxLength")
                        && schema
                            .get("pattern")
                            .and_then(Value::as_str)
                            .is_none_or(|rule| {
                                length <= 16384
                                    && pattern(rule).is_some_and(|regex| regex.is_match(text))
                            })
                }
                Value::Array(items) => {
                    count_valid(items.len(), "minItems", "maxItems")
                        && schema.get("items").is_none_or(|field| {
                            items.iter().all(|item| self.valid(item, field, depth + 1))
                        })
                }
                Value::Object(fields) => {
                    count_valid(fields.len(), "minProperties", "maxProperties")
                        && schema
                            .get("required")
                            .and_then(Value::as_array)
                            .is_none_or(|keys| {
                                keys.iter()
                                    .filter_map(Value::as_str)
                                    .all(|key| fields.contains_key(key))
                            })
                        && fields.iter().all(|(key, item)| {
                            schema
                                .get("properties")
                                .and_then(|fields| fields.get(key))
                                .or_else(|| schema.get("additionalProperties"))
                                .is_none_or(|field| self.valid(item, field, depth + 1))
                        })
                }
                _ => true,
            }
        }
    }
    let mut validator = Validator {
        root,
        remaining: 8192,
        exhausted: false,
    };
    validator.valid(value, schema, 0) && !validator.exhausted
}

pub(super) fn portable(value: &Value, depth: usize) -> bool {
    if depth > MAXIMUM_DEPTH {
        return false;
    }
    match value {
        Value::Object(fields) => fields.values().all(|value| portable(value, depth + 1)),
        Value::Array(items) => items.iter().all(|value| portable(value, depth + 1)),
        Value::Number(number) => number
            .as_f64()
            .is_some_and(|number| number.fract() != 0.0 || number.abs() <= MAXIMUM_SAFE_INTEGER),
        _ => true,
    }
}
