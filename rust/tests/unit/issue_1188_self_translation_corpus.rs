//! The Rust side of the shared self-translation corpus (R1024).
//!
//! `rust/tests/fixtures/self-translation/cases.lino` is one contract both
//! runtimes consume -- the practice relative-meta-logic keeps in
//! `test-corpus/` and meta-language PR #196 in `parity/self-translation/`.
//! `rust/tests/web/self-translation.test.mjs` runs the JavaScript side of
//! every `(call ...)` row; this suite runs the Rust side. It compiles the
//! committed JavaScript-to-Rust translation (`expected/ratings-to-rust.rs`)
//! and the hand-written Rust source of the Rust-to-JavaScript case
//! (`sources/geometry.rs`) as modules, so CI's build, clippy and rustfmt hold
//! translated Rust to the standard of hand-written Rust, and every call
//! must print the result the corpus records.

use std::fs;
use std::path::Path;

#[path = "../fixtures/self-translation/expected/ratings-to-rust.rs"]
mod ratings_to_rust;

#[path = "../fixtures/self-translation/sources/geometry.rs"]
mod geometry;

/// One argument or result of a `(call ...)` row.
#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    Boolean(bool),
    Text(String),
}

impl Value {
    fn number(&self) -> f64 {
        match self {
            Self::Number(value) => *value,
            other => panic!("expected a number, got {other:?}"),
        }
    }

    fn boolean(&self) -> bool {
        match self {
            Self::Boolean(value) => *value,
            other => panic!("expected a boolean, got {other:?}"),
        }
    }

    fn text(&self) -> &str {
        match self {
            Self::Text(value) => value,
            other => panic!("expected a text, got {other:?}"),
        }
    }

    /// The printed form both runtimes compare: JavaScript's `String(value)`
    /// and Rust's `Display` agree on these finite values.
    fn printed(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Text(value) => value.clone(),
        }
    }
}

/// A Links Notation node: a word or a parenthesized link.
#[derive(Debug, Clone)]
enum Node {
    Word(String),
    Link(Vec<Node>),
}

impl Node {
    fn word(&self) -> &str {
        match self {
            Self::Word(word) => word,
            Self::Link(items) => panic!("expected a word, got a link of {}", items.len()),
        }
    }

    fn items(&self) -> &[Self] {
        match self {
            Self::Link(items) => items,
            Self::Word(word) => panic!("expected a link, got {word}"),
        }
    }

    /// The nested link whose head is `head`.
    fn field(&self, head: &str) -> &Self {
        self.items()
            .iter()
            .find(|item| match item {
                Self::Link(items) => {
                    matches!(items.first(), Some(Self::Word(word)) if word == head)
                }
                Self::Word(_) => false,
            })
            .unwrap_or_else(|| panic!("no ({head} ...) in {self:?}"))
    }
}

/// Parse one line of the corpus into its link.
fn parse_link(line: &str) -> Node {
    fn read(chars: &[char], at: &mut usize) -> Node {
        while chars[*at] == ' ' {
            *at += 1;
        }
        if chars[*at] == '(' {
            *at += 1;
            let mut items = Vec::new();
            loop {
                while chars[*at] == ' ' {
                    *at += 1;
                }
                if chars[*at] == ')' {
                    *at += 1;
                    return Node::Link(items);
                }
                items.push(read(chars, at));
            }
        }
        let start = *at;
        while *at < chars.len() && !matches!(chars[*at], ' ' | '(' | ')') {
            *at += 1;
        }
        Node::Word(chars[start..*at].iter().collect())
    }
    let chars: Vec<char> = line.chars().collect();
    read(&chars, &mut 0)
}

/// Decode a percent-encoded word (`%` alone is the empty text).
fn decode(word: &str) -> String {
    if word == "%" {
        return String::new();
    }
    let bytes = word.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' {
            out.push(u8::from_str_radix(&word[at + 1..at + 3], 16).expect("a percent escape"));
            at += 3;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8(out).expect("UTF-8 text")
}

fn value(node: &Node) -> Value {
    let items = node.items();
    let text = items[1].word();
    match items[0].word() {
        "number" => Value::Number(text.parse().expect("a number")),
        "boolean" => Value::Boolean(text == "true"),
        "text" => Value::Text(decode(text)),
        other => panic!("unknown value type {other}"),
    }
}

/// Run one call on the Rust side; a row without a dispatch arm fails.
fn call(case: &str, function: &str, arguments: &[Value]) -> Value {
    let number = |index: usize| arguments[index].number();
    let text = |index: usize| arguments[index].text();
    match (case, function) {
        ("ratings-to-rust", "add") => Value::Number(ratings_to_rust::add(number(0), number(1))),
        ("ratings-to-rust", "average") => {
            Value::Number(ratings_to_rust::average(number(0), number(1)))
        }
        ("ratings-to-rust", "distance") => {
            Value::Number(ratings_to_rust::distance(number(0), number(1)))
        }
        ("ratings-to-rust", "score_band") => Value::Text(ratings_to_rust::score_band(number(0))),
        ("ratings-to-rust", "is_rating_unit") => {
            Value::Boolean(ratings_to_rust::is_rating_unit(text(0)))
        }
        ("ratings-to-rust", "mentions_rating") => {
            Value::Boolean(ratings_to_rust::mentions_rating(text(0)))
        }
        ("ratings-to-rust", "rating_label") => {
            Value::Text(ratings_to_rust::rating_label(number(0)))
        }
        ("ratings-to-rust", "is_rated") => {
            Value::Boolean(ratings_to_rust::is_rated(number(0), arguments[1].boolean()))
        }
        ("ratings-to-rust", "spread") => {
            Value::Number(ratings_to_rust::spread(number(0), number(1)))
        }
        ("ratings-to-rust", "whole_stars") => {
            Value::Number(ratings_to_rust::whole_stars(number(0)))
        }
        ("geometry-to-javascript", "squared") => Value::Number(geometry::squared(number(0))),
        ("geometry-to-javascript", "hypotenuse") => {
            Value::Number(geometry::hypotenuse(number(0), number(1)))
        }
        ("geometry-to-javascript", "quadrant") => {
            Value::Text(geometry::quadrant(number(0), number(1)))
        }
        ("geometry-to-javascript", "is_unit") => Value::Boolean(geometry::is_unit(text(0))),
        (case, function) => panic!("no Rust dispatch for ({case}, {function})"),
    }
}

fn corpus_path(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/self-translation")
        .join(path)
}

#[test]
fn every_corpus_call_returns_the_committed_result_in_rust() {
    let cases = fs::read_to_string(corpus_path("cases.lino")).expect("the corpus");
    let mut checked = 0_usize;
    for line in cases.lines().filter(|line| line.starts_with("(call ")) {
        let link = parse_link(line);
        let case = link.items()[1].word();
        let function = link.field("rust").items()[1].word();
        let arguments: Vec<Value> = link.field("arguments").items()[1..]
            .iter()
            .map(value)
            .collect();
        let expected = value(&link.field("result").items()[1]);
        let actual = call(case, function, &arguments);
        assert_eq!(
            actual.printed(),
            expected.printed(),
            "{function}({arguments:?}) of {case}"
        );
        assert_eq!(
            std::mem::discriminant(&actual),
            std::mem::discriminant(&expected),
            "{function} of {case} returns a {expected:?}-typed value"
        );
        checked += 1;
    }
    assert_eq!(checked, 30, "every (call ...) row of the corpus ran");
}

#[test]
fn the_translated_rust_records_its_source_and_the_lints_it_needs() {
    let translated =
        fs::read_to_string(corpus_path("expected/ratings-to-rust.rs")).expect("the translation");
    let header = translated.lines().next().expect("a header line");
    assert!(
        header
            .starts_with("// formal-ai:self-translation:v1 source=JavaScript target=Rust sha256="),
        "{header}"
    );
    let prelude: Vec<&str> = translated
        .lines()
        .skip_while(|line| *line != "// formal-ai:prelude begin")
        .take_while(|line| *line != "// formal-ai:prelude end")
        .collect();
    assert_eq!(
        prelude,
        [
            "// formal-ai:prelude begin",
            "#![allow(",
            "    clippy::float_cmp,",
            "    clippy::imprecise_flops,",
            "    clippy::missing_const_for_fn,",
            "    clippy::suboptimal_flops",
            ")]",
        ]
    );
}
