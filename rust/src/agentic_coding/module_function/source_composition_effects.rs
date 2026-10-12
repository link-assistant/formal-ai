//! Close operation inventory over imported source graph; called effects remain conditional.
use super::source_contract::text;
use serde_json::{Value, json};
fn same(node: &Value, expected: &str) -> Result<(), &'static str> {
    if text(node) == expected {
        Ok(())
    } else {
        Err("UnexpectedSourceOperation")
    }
}
fn group<'source>(
    node: &'source Value,
    delimiter: &str,
) -> Result<&'source Vec<Value>, &'static str> {
    if node["$"] != "group" || node["delim"] != delimiter {
        return Err("UnexpectedSourceGroup");
    }
    node["trees"].as_array().ok_or("MissingSourceGroup")
}
pub fn checked_composition_source(
    source: &str,
    request: &Value,
    imports: &[Value],
) -> Result<Value, &'static str> {
    let raw = crate::es_tokenizer::tokenize(source).map_err(|_| "SourceLexicalFailure")?;
    let trees: Vec<_> = raw
        .iter()
        .map(super::callable_catalog::tree_value)
        .collect();
    if trees.len() != 15 || imports.len() != 2 {
        return Err("ExtraModuleOperation");
    }
    let parameter = request["parameters"]
        .as_array()
        .filter(|value| value.len() == 1)
        .and_then(|value| value[0].as_str())
        .ok_or("UnsupportedDeclaration")?;
    let name = request["name"].as_str().ok_or("UnsupportedDeclaration")?;
    let mut locals = Vec::new();
    for (index, imported) in imports.iter().enumerate() {
        let at = index * 5;
        same(&trees[at], "import")?;
        let binding = group(&trees[at + 1], "brace")?;
        if binding.len() != 3 {
            return Err("UnknownImportBinding");
        }
        same(
            &binding[0],
            imported["exported"]
                .as_str()
                .ok_or("MissingExportBinding")?,
        )?;
        same(&binding[1], "as")?;
        if binding[2]["kind"] != "identifier" {
            return Err("UnsupportedLocalBinding");
        }
        locals.push(text(&binding[2]).to_owned());
        same(&trees[at + 2], "from")?;
        if trees[at + 3]["kind"] != "string" {
            return Err("UnsupportedImportURL");
        }
        let parsed: Value =
            serde_json::from_str(text(&trees[at + 3])).map_err(|_| "UnsupportedImportURL")?;
        if parsed != imported["moduleURL"] {
            return Err("ForeignImportURL");
        }
        same(&trees[at + 4], ";")?;
    }
    same(&trees[10], "export")?;
    same(&trees[11], "function")?;
    same(&trees[12], name)?;
    let parameters = group(&trees[13], "paren")?;
    if parameters.len() != 1 {
        return Err("UnsupportedDeclaration");
    }
    same(&parameters[0], parameter)?;
    let body = group(&trees[14], "brace")?;
    if body.len() != 16 {
        return Err("ExtraFunctionOperation");
    }
    same(&body[0], "const")?;
    if body[1]["kind"] != "identifier" {
        return Err("UnsupportedLocalBinding");
    }
    let selected = text(&body[1]);
    let unique = [
        locals[0].as_str(),
        locals[1].as_str(),
        selected,
        name,
        parameter,
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    if unique.len() != 5 {
        return Err("BindingCollision");
    }
    same(&body[2], "=")?;
    same(&body[3], &locals[0])?;
    let arguments = group(&body[4], "paren")?;
    if arguments.len() != 1 {
        return Err("UnsupportedSourceCall");
    }
    same(&arguments[0], parameter)?;
    same(&body[5], ";")?;
    same(&body[6], "return")?;
    same(&body[7], selected)?;
    same(&body[8], "===")?;
    same(&body[9], "null")?;
    same(&body[10], "?")?;
    same(&body[11], "null")?;
    same(&body[12], ":")?;
    same(&body[13], &locals[1])?;
    let arguments = group(&body[14], "paren")?;
    if arguments.len() != 1 {
        return Err("UnsupportedSourceCall");
    }
    same(&arguments[0], selected)?;
    same(&body[15], ";")?;
    Ok(
        json!({"kind":"checked-source-import-composition","imports":imports.iter().map(|value|
        json!({"moduleURL":value["moduleURL"],"exported":value["exported"],"sourceIdentity":value["sourceIdentity"]}))
        .collect::<Vec<_>>(),"calls":[{"source":0,"argument":parameter,"result":selected},
        {"source":1,"argument":selected,"guard":"nonnull"}],"output":{"absent":null,"present":"source-call-1"},
        "localMutation":false,"additionalModuleInitialization":false,
        "calledSourceEffects":"conditional; inherited, not pure","scope":"same declared Node acceptance operation"}),
    )
}
#[must_use]
pub fn source_null_absence(ir: &Value) -> bool {
    fn proven(node: &Value) -> bool {
        node["op"] == "literal" && node.get("value").is_some_and(Value::is_null)
            || node["op"] == "??"
                && node["right"]["op"] == "literal"
                && node["right"].get("value").is_some_and(Value::is_null)
            || node["op"] == "conditional" && proven(&node["yes"]) && proven(&node["no"])
    }
    ir["status"] == "parsed"
        && ir["ir"].as_array().is_some_and(|statements| {
            statements.len() == 1
                && statements[0]["op"] == "return"
                && proven(&statements[0]["value"])
        })
}
