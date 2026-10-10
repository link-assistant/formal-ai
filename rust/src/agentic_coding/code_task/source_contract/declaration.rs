use std::collections::BTreeMap;

/// Bind the complete supported Rust declaration without executing its code.
pub(super) fn source_declaration_slots(
    content: &str,
) -> Option<(&'static str, BTreeMap<&'static str, String>)> {
    let profiles = [
        (
            "integer",
            r"^pub fn ([A-Za-z_][A-Za-z_0-9]*)\(\) -> i64 \{\n    (-?\d+)\n\}\n$",
            vec!["identifier", "value"],
        ),
        (
            "float-division",
            r"^pub fn ([A-Za-z_][A-Za-z_0-9]*)\(([A-Za-z_][A-Za-z_0-9]*): (f64)\) -> (f64) \{\n    ([A-Za-z_][A-Za-z_0-9]*) / (-?[0-9]+\.[0-9]+)\n\}\n$",
            vec![
                "identifier",
                "parameter",
                "parameter-type",
                "return-type",
                "operand",
                "value",
            ],
        ),
        (
            "string-constant",
            r#"^pub const ([A-Za-z_][A-Za-z_0-9]*): (&str) = "([A-Za-z_][A-Za-z_0-9]*)";\n$"#,
            vec!["identifier", "constant-type", "value"],
        ),
        (
            "equality-test",
            r"^#\[test\]\nfn ([A-Za-z_][A-Za-z_0-9]*)\(\) \{\n    assert_eq!\((-?\d+), (-?\d+)\);\n\}\n$",
            vec!["identifier", "left", "right"],
        ),
        (
            "addition-test",
            r"^#\[test\]\nfn ([A-Za-z_][A-Za-z_0-9]*)\(\) \{\n    assert_eq!\((-?\d+) \+ (-?\d+), (-?\d+)\);\n\}\n$",
            vec!["identifier", "left", "right", "expected"],
        ),
    ];
    for (kind, pattern, roles) in profiles {
        let expression = regex::Regex::new(pattern).ok()?;
        let Some(captures) = expression.captures(content) else {
            continue;
        };
        let mut slots = BTreeMap::new();
        for (index, role) in roles.into_iter().enumerate() {
            slots.insert(role, captures.get(index + 1)?.as_str().to_owned());
        }
        if !super::super::identifier_domain::rust_identifier_is_valid(slots.get("identifier")?) {
            return None;
        }
        match kind {
            "integer" => {
                slots.get("value")?.parse::<i64>().ok()?;
            }
            "float-division" => {
                let parameter = slots.remove("parameter")?;
                let operand = slots.remove("operand")?;
                let return_type = slots.remove("return-type")?;
                let value = slots.get("value")?.parse::<f64>().ok()?;
                if parameter != operand
                    || !super::super::identifier_domain::rust_identifier_is_valid(&parameter)
                    || slots.get("parameter-type")? != &return_type
                    || !value.is_finite()
                    || value == 0.0
                {
                    return None;
                }
            }
            "equality-test" | "addition-test" => {
                for role in ["left", "right", "expected"] {
                    if let Some(value) = slots.get(role) {
                        value.parse::<i32>().ok()?;
                    }
                }
            }
            _ => {}
        }
        return Some((kind, slots));
    }
    None
}
