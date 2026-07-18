//! Is a raw newline safe inside a *nested* Links Notation record? (#715)
//!
//! `issue_715_multiline_value_probe.rs` shows the grammar round-trips a value
//! carrying a raw newline when the record is flat. The learning report is not
//! flat: it is an indented tree, and a raw newline inside a value means the
//! value's continuation line starts at column 0. Indentation is what the tree
//! structure is *made of*, so the question is whether the continuation line is
//! read as part of the value or as a new sibling at the root.
//!
//! This decides the encoding for every renderer:
//!
//! * if a raw newline survives nesting, it is the notation's own answer and a
//!   value can be carried verbatim;
//! * if it does not, a newline must be escaped, and the escape is only ever
//!   readable by the repository's own parser, never by the grammar.
//!
//! Run it with:
//!
//! ```sh
//! rust-script experiments/issue_715_nested_newline_probe.rs
//! ```
//!
//! ```cargo
//! [dependencies]
//! links-notation = "0.13.0"
//! ```

use links_notation::LiNo;

const VALUE: &str = "first line\nsecond line";

fn main() {
    // The report's real shape: a value nested two levels down.
    let nested = format!(
        "report\n  expression\n    text \"{VALUE}\"\n    reads \"9\"\n  expression\n    text \"other\"\n"
    );
    probe("nested record, raw newline in the value", &nested);

    // The same, with the newline escaped the way `links_format` does.
    let escaped = format!(
        "report\n  expression\n    text \"{}\"\n    reads \"9\"\n  expression\n    text \"other\"\n",
        VALUE.replace('\n', "\\n")
    );
    probe("nested record, newline escaped as a literal", &escaped);
}

fn probe(label: &str, document: &str) {
    println!("=== {label}");
    println!("--- document:\n{document}");
    match links_notation::parse_lino(document) {
        Ok(tree) => {
            let mut fields = Vec::new();
            collect(&tree, &mut fields);
            println!("--- parses: yes");
            println!("--- fields the grammar found:");
            for (name, value) in &fields {
                println!("      {name} = {value:?}");
            }
            let text_values: Vec<&String> = fields
                .iter()
                .filter(|(n, _)| n == "text")
                .map(|(_, v)| v)
                .collect();
            println!("--- `text` fields found: {}", text_values.len());
            match text_values.first() {
                Some(v) if v.as_str() == VALUE => println!("--- first `text`: ROUND-TRIPS"),
                Some(v) => println!("--- first `text`: CHANGED -> {v:?}"),
                None => println!("--- first `text`: MISSING — the field did not survive"),
            }
            // `reads` proves the fields *after* the multi-line value are still
            // attached where they belong.
            println!(
                "--- sibling `reads` still present: {}",
                fields.iter().any(|(n, _)| n == "reads")
            );
        }
        Err(error) => println!("--- parses: NO -> {error:?}"),
    }
    println!();
}

/// Collect every `name value` pair the grammar produced.
fn collect(node: &LiNo<String>, out: &mut Vec<(String, String)>) {
    let LiNo::Link { values, .. } = node else {
        return;
    };
    let refs: Vec<&str> = values
        .iter()
        .filter_map(|v| match v {
            LiNo::Ref(r) => Some(r.as_str()),
            LiNo::Link { .. } => None,
        })
        .collect();
    if refs.len() == 2 {
        out.push((refs[0].to_string(), refs[1].to_string()));
    }
    for value in values {
        collect(value, out);
    }
}
