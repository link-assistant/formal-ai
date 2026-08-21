//! Mechanical adapters for pinned upstream Rust reasoning examples.

use serde_json::json;

use super::vocabulary;

pub fn adapt_records(suite: &str, source: &str) -> Result<Vec<String>, String> {
    match suite {
        "egg_math" => egg_math_records(source),
        "ascent_transitive_closure" => ascent_records(source),
        _ => Err(vocabulary::render(
            "external_benchmark_rust_adapter_missing",
            &[("suite", suite)],
        )),
    }
}

fn egg_math_records(source: &str) -> Result<Vec<String>, String> {
    let rules_start = source
        .find("pub fn rules()")
        .ok_or("egg math source has no `rules` function")?;
    let rules_end = source[rules_start..]
        .find("egg::test_fn!")
        .map_or(source.len(), |offset| rules_start + offset);
    let rules = &source[rules_start..rules_end];
    let mut records = Vec::new();
    for body in macro_bodies(rules, "rw!")? {
        if body
            .lines()
            .any(|line| line.trim_start().starts_with("if "))
            || body.contains(" if ")
        {
            continue;
        }
        let strings = quoted_strings(body)?;
        if strings.len() < 3 {
            continue;
        }
        let name = &strings[0];
        let left = instantiate_pattern(&strings[1]);
        let right = instantiate_pattern(&strings[2]);
        records.push(
            json!({
                "id": format!("egg_math/{name}"),
                "prompt": vocabulary::render(
                    "external_benchmark_egg_law_prompt",
                    &[("left", &left), ("right", &right)],
                ),
                "expected": "proven",
            })
            .to_string(),
        );
    }
    if records.len() < 20 {
        return Err(vocabulary::render(
            "external_benchmark_egg_too_few_laws",
            &[("count", &records.len().to_string())],
        ));
    }
    Ok(records)
}

fn instantiate_pattern(pattern: &str) -> String {
    pattern.replace('?', "")
}

fn ascent_records(source: &str) -> Result<Vec<String>, String> {
    let collapsed = source.split_whitespace().collect::<Vec<_>>().join(" ");
    for rule in [
        "reachable(x, y) <-- edge(x, y);",
        "reachable(x, z) <-- reachable(x, y), edge(y, z);",
        "closure_of_a(y) <-- reachable(Node(\"A\"), y);",
    ] {
        if !collapsed.contains(rule) {
            return Err(vocabulary::render(
                "external_benchmark_ascent_missing_rule",
                &[("rule", rule)],
            ));
        }
    }

    let edges = node_pairs(section(source, "prog.edge = vec![", "];")?)?;
    let reachable = node_pairs(section(source, "assert_eq!(reachable, vec![", "]);")?)?;
    let closure = node_names(section(source, "assert_eq!(closure_of_a, vec![", "]);")?);
    if reachable.is_empty() || closure.is_empty() {
        return Err(String::from("Ascent source has no asserted consequences"));
    }

    let facts = edges
        .iter()
        .map(|(left, right)| format!("edge({left},{right})"))
        .collect::<Vec<_>>()
        .join("; ");
    let rules = concat!(
        "reachable(?x,?y) :- edge(?x,?y); ",
        "reachable(?x,?z) :- reachable(?x,?y), edge(?y,?z); ",
        "closure_of_a(?y) :- reachable(a,?y)"
    );
    let prefix = vocabulary::render(
        "external_benchmark_ascent_prompt_prefix",
        &[("facts", &facts), ("rules", rules)],
    );
    let mut records = reachable
        .into_iter()
        .map(|(left, right)| {
            proof_record(
                &format!("ascent/reachable/{left}/{right}"),
                &prefix,
                &format!("reachable({left},{right})"),
            )
        })
        .collect::<Vec<_>>();
    records.extend(closure.into_iter().map(|node| {
        proof_record(
            &format!("ascent/closure_of_a/{node}"),
            &prefix,
            &format!("closure_of_a({node})"),
        )
    }));
    Ok(records)
}

fn proof_record(id: &str, prefix: &str, query: &str) -> String {
    json!({
        "id": id,
        "prompt": vocabulary::render(
            "external_benchmark_proof_query",
            &[("prefix", prefix), ("query", query)],
        ),
        "expected": "proven",
    })
    .to_string()
}

fn section<'a>(source: &'a str, start: &str, end: &str) -> Result<&'a str, String> {
    let start_index = source.find(start).ok_or_else(|| {
        vocabulary::render(
            "external_benchmark_upstream_missing_start",
            &[("start", start)],
        )
    })? + start.len();
    let end_index = source[start_index..].find(end).ok_or_else(|| {
        vocabulary::render(
            "external_benchmark_upstream_missing_end",
            &[("end", end), ("start", start)],
        )
    })? + start_index;
    Ok(&source[start_index..end_index])
}

fn node_pairs(source: &str) -> Result<Vec<(String, String)>, String> {
    let names = node_names(source);
    let (chunks, remainder) = names.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(String::from("upstream Node tuple has odd arity"));
    }
    Ok(chunks
        .iter()
        .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
        .collect())
}

fn node_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut remaining = source;
    while let Some(start) = remaining.find("Node(\"") {
        remaining = &remaining[start + "Node(\"".len()..];
        let Some(end) = remaining.find("\")") else {
            break;
        };
        names.push(remaining[..end].to_ascii_lowercase());
        remaining = &remaining[end + 2..];
    }
    names
}

fn macro_bodies<'a>(source: &'a str, name: &str) -> Result<Vec<&'a str>, String> {
    let marker = format!("{name}(");
    let mut bodies = Vec::new();
    let mut cursor = 0_usize;
    while let Some(relative) = source[cursor..].find(&marker) {
        let invocation = cursor + relative;
        let line_start = source[..invocation]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        if source[line_start..invocation]
            .trim_start()
            .starts_with("//")
        {
            cursor = invocation + marker.len();
            continue;
        }
        let open = invocation + marker.len() - 1;
        let close = matching_parenthesis(source, open)?;
        bodies.push(&source[open + 1..close]);
        cursor = close + 1;
    }
    Ok(bodies)
}

fn matching_parenthesis(source: &str, open: usize) -> Result<usize, String> {
    let mut depth = 0_usize;
    let mut quoted = false;
    let mut escaped = false;
    for (offset, character) in source[open..].char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(open + offset);
                }
            }
            _ => {}
        }
    }
    Err(String::from("unclosed Rust macro invocation"))
}

fn quoted_strings(source: &str) -> Result<Vec<String>, String> {
    let mut strings = Vec::new();
    let mut start = None;
    let mut escaped = false;
    for (index, character) in source.char_indices() {
        if let Some(open) = start {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                let literal = &source[open..=index];
                strings.push(serde_json::from_str(literal).map_err(|error| {
                    vocabulary::render(
                        "external_benchmark_invalid_rust_string",
                        &[("error", &error.to_string())],
                    )
                })?);
                start = None;
            }
        } else if character == '"' {
            start = Some(index);
        }
    }
    if start.is_some() {
        return Err(String::from("unclosed Rust string literal"));
    }
    Ok(strings)
}
