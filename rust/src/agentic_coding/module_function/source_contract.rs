//! Supported return expressions establish symbolic constraints; unknown stays unknown.
use serde_json::{Value, json};

pub(super) fn text(tree: &Value) -> &str {
    if tree["$"].as_str() == Some("leaf") {
        tree["text"].as_str().unwrap_or("")
    } else {
        ""
    }
}
fn optional(value: Value) -> Value {
    if value["kind"] == "optional" {
        value
    } else {
        json!({"kind":"optional","value":value})
    }
}
fn merge(left: Value, right: Value) -> Result<Value, &'static str> {
    if left == right {
        Ok(left)
    } else if left["kind"] == "null" {
        Ok(optional(right))
    } else if right["kind"] == "null" {
        Ok(optional(left))
    } else {
        Err("IncompatibleBranches")
    }
}
fn decimal_number(value: &str) -> bool {
    let mut exponent = value.split(['e', 'E']);
    let main = exponent.next().unwrap_or("");
    if let Some(suffix) = exponent.next() {
        let digits = suffix.strip_prefix(['+', '-']).unwrap_or(suffix);
        if digits.is_empty()
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
            || exponent.next().is_some()
        {
            return false;
        }
    }
    let mut fraction = main.split('.');
    let integer = fraction.next().unwrap_or("");
    if integer.is_empty()
        || (integer.len() > 1 && integer.starts_with('0'))
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
    {
        return false;
    }
    if let Some(digits) = fraction.next() {
        if digits.is_empty()
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
            || fraction.next().is_some()
        {
            return false;
        }
    }
    true
}
fn expression(trees: &[Value], parameters: &[String]) -> Result<Value, &'static str> {
    if trees.is_empty() {
        return Err("MissingReturnExpression");
    }
    if let Some(question) = trees.iter().position(|tree| text(tree) == "?") {
        let mut depth = 0;
        let mut colon = None;
        for (at, tree) in trees.iter().enumerate().skip(question + 1) {
            if text(tree) == "?" {
                depth += 1;
            }
            if text(tree) == ":" {
                if depth == 0 {
                    colon = Some(at);
                    break;
                }
                depth -= 1;
            }
        }
        let colon = colon.ok_or("UnclosedConditional")?;
        if expression(&trees[..question], parameters)?["kind"] != "boolean" {
            return Err("MissingConditionType");
        }
        return merge(
            expression(&trees[question + 1..colon], parameters)?,
            expression(&trees[colon + 1..], parameters)?,
        );
    }
    if let Some(at) = trees.iter().position(|tree| text(tree) == "??") {
        let left = expression(&trees[..at], parameters)?;
        let right = expression(&trees[at + 1..], parameters)?;
        if left["kind"] != "optional" {
            return Err("MissingOptionalOperand");
        }
        return if right["kind"] == "null" {
            Ok(left)
        } else {
            merge(left["value"].clone(), right)
        };
    }
    if let Some(at) = trees
        .iter()
        .position(|tree| matches!(text(tree), "===" | "!=="))
    {
        expression(&trees[..at], parameters)?;
        expression(&trees[at + 1..], parameters)?;
        return Ok(json!({"kind":"boolean"}));
    }
    if trees.iter().any(|tree| {
        matches!(text(tree), "." | "?.") || (tree["$"] == "group" && tree["delim"] == "bracket")
    }) {
        return Err("MissingStructuralSchema");
    }
    if let [tree] = trees {
        if tree["$"] == "group" && tree["delim"] == "paren" {
            return expression(tree["trees"].as_array().expect("token group"), parameters);
        }
        if parameters.iter().any(|name| name == text(tree)) {
            return Ok(json!({"kind":"parameter","name":text(tree)}));
        }
        if text(tree) == "null" {
            return Ok(json!({"kind":"null"}));
        }
        if matches!(text(tree), "true" | "false") {
            return Ok(json!({"kind":"boolean"}));
        }
        if tree["kind"] == "numeric" && decimal_number(text(tree)) {
            return Ok(json!({"kind":"number"}));
        }
        if tree["kind"] == "string" && !text(tree).contains(['\\', '\r', '\n']) {
            return Ok(json!({"kind":"text"}));
        }
    }
    Err(
        if trees
            .iter()
            .any(|tree| tree["$"] == "group" && tree["delim"] == "paren")
        {
            "UnobservedCallEffect"
        } else {
            "UnsupportedExpression"
        },
    )
}

/// Mirrors inferReturnContract: no unsupported body is promoted to a type/effect proof.
pub(super) fn infer_return_contract(parameters: &[String], body: &[Value], source: &str) -> Value {
    let mut contract = json!({"inputs":parameters.iter().map(|name| json!({"name":name,"type":{"kind":"parameter","name":name}})).collect::<Vec<_>>(),
        "result":null,"callEffects":"unknown","preconditions":[],"status":"unknown","gap":null});
    let checked = (|| -> Result<Value, &'static str> {
        let unique: std::collections::BTreeSet<_> = parameters.iter().collect();
        if unique.len() != parameters.len() {
            return Err("DuplicateParameter");
        }
        if body.first().map_or("", text) != "return" {
            return Err("UnsupportedStatement");
        }
        let mut terms = &body[1..];
        if terms.last().is_some_and(|tree| text(tree) == ";") {
            terms = &terms[..terms.len() - 1];
        }
        if terms.iter().any(|tree| text(tree) == ";") {
            return Err("UnsupportedStatement");
        }
        let from = usize::try_from(body[0]["span"]["end"].as_u64().expect("token span"))
            .expect("host span");
        let to = terms
            .first()
            .and_then(|tree| tree["span"]["start"].as_u64())
            .map_or(from, |start| usize::try_from(start).expect("host span"));
        if source[from..to].contains(['\r', '\n', '\u{2028}', '\u{2029}']) {
            return Err("ReturnLineTerminator");
        }
        expression(terms, parameters)
    })();
    match checked {
        Ok(value) => {
            contract["result"] = value;
            contract["callEffects"] = json!("none");
            contract["status"] = json!("supported");
        }
        Err(reason) => {
            contract["gap"] =
                json!({"reason":reason,"span":body.first().map(|tree| &tree["span"])});
        }
    }
    contract
}
fn instantiate(value: &Value, operand: &Value) -> Value {
    if value["kind"] == "parameter" {
        operand.clone()
    } else if value["kind"] == "optional" {
        optional(instantiate(&value["value"], operand))
    } else {
        value.clone()
    }
}

/// Mirrors guardedCallGraph: every unknown effect/type blocks a graph operand.
pub(super) fn guarded_call_graph(first: &Value, second: &Value) -> Value {
    if [first, second].iter().any(|entry| {
        entry["contract"]["status"] != "supported"
            || entry["contract"]["callEffects"] != "none"
            || entry["moduleEffects"] != "none"
    }) {
        return json!({"kind":"gap","reason":"MissingContract"});
    }
    if first["parameters"].as_array().map_or(0, Vec::len) != 1
        || second["parameters"].as_array().map_or(0, Vec::len) != 1
    {
        return json!({"kind":"gap","reason":"UnsupportedArity"});
    }
    if first["contract"]["result"]["kind"] != "optional" {
        return json!({"kind":"gap","reason":"MissingOptionalResult"});
    }
    if second["contract"]["inputs"][0]["type"]["kind"] != "parameter" {
        return json!({"kind":"gap","reason":"IncompatibleOperand"});
    }
    let result = instantiate(
        &second["contract"]["result"],
        &first["contract"]["result"]["value"],
    );
    json!({"kind":"guarded-call-graph","calls":[first["identity"],second["identity"]],"bindings":[first["binding"],second["binding"]],
        "guard":{"kind":"nonnull","call":0},"result":optional(result),"effects":"none","authored":false})
}
