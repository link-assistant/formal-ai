//! Plan 16 L2a acceptance: the general ES tokenizer over the committed
//! `./js` corpus (issue #1138,
//! `docs/case-studies/issue-1138/plans/16-js-ts-rust-cycle.md`).
//!
//! The tokenizer is engine code under the engine/data split; these tests pin
//! the engine's honest behaviors: the whole committed corpus normalizes into
//! a balanced token tree, the regex-versus-division ambiguity resolves by the
//! previous-significant-token heuristic, template literals carry their chunks
//! and interpolations as structure, and every unterminated construct reports
//! its opening byte instead of guessing.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::es_tokenizer::{
    Delimiter, TemplatePart, Token, TokenKind, TokenizeErrorKind, Tree, tokenize,
};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .map(PathBuf::from)
        .expect("the crate root sits one level below the repository root")
}

fn walk_js_files(directory: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("the js root must be readable") {
        let entry = entry.expect("the js root must be walkable");
        let path = entry.path();
        if path.is_dir() {
            walk_js_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("js") {
            out.push(path);
        }
    }
}

/// Collect every leaf token in tree order, interpolations included.
fn leaves<'src>(trees: &[Tree<'src>], out: &mut Vec<Token<'src>>) {
    for tree in trees {
        match tree {
            Tree::Leaf(token) => out.push(token.clone()),
            Tree::Group { trees, .. } => leaves(trees, out),
            Tree::Template { parts, .. } => {
                for part in parts {
                    if let TemplatePart::Interpolation { trees, .. } = part {
                        leaves(trees, out);
                    }
                }
            }
        }
    }
}

fn leaf_texts(source: &str) -> Vec<&str> {
    let trees = tokenize(source).expect("the fixture must tokenize");
    let mut collected = Vec::new();
    leaves(&trees, &mut collected);
    collected.into_iter().map(|token| token.text).collect()
}

#[test]
fn the_committed_js_corpus_tokenizes_with_balanced_structure() {
    let root = repository_root().join("js");
    let mut files = Vec::new();
    walk_js_files(&root, &mut files);
    files.sort();
    assert!(
        files.len() >= 10,
        "expected the committed corpus under js/, found {} files",
        files.len()
    );
    for path in &files {
        let source = fs::read_to_string(path).unwrap_or_default();
        let display = path
            .strip_prefix(repository_root())
            .unwrap_or(path)
            .display();
        match tokenize(&source) {
            Ok(trees) => {
                let mut collected = Vec::new();
                leaves(&trees, &mut collected);
                assert!(
                    !collected.is_empty(),
                    "{display} produced an empty token tree"
                );
                for token in &collected {
                    assert_eq!(
                        &source[token.span.start..token.span.end],
                        token.text,
                        "{display} leaf span must slice its own text"
                    );
                }
            }
            Err(error) => panic!("{display}: {error}"),
        }
    }
}

#[test]
fn regex_literals_and_division_split_by_previous_token() {
    let source =
        "const re = /a\\/b[/:x]/gimsuy;\nconst half = total / 2 / count;\nreturn /x/.test(s);\n";
    let trees = tokenize(source).expect("the fixture must tokenize");
    let mut collected = Vec::new();
    leaves(&trees, &mut collected);

    assert_eq!(
        collected
            .iter()
            .find(|token| token.kind == TokenKind::RegExp)
            .map(|token| token.text),
        Some("/a\\/b[/:x]/gimsuy"),
        "a regex after `=` must lex with its body, class, and flags"
    );
    let divisions = collected
        .iter()
        .filter(|token| token.text == "/" && token.kind == TokenKind::Punctuator)
        .count();
    assert_eq!(
        divisions, 2,
        "`/` after identifiers and numbers is division"
    );
    let trailing = collected
        .iter()
        .filter(|token| token.text == "/x/" && token.kind == TokenKind::RegExp)
        .count();
    assert_eq!(
        trailing, 1,
        "a `/` after `return` starts a regex, not a division"
    );
    assert!(formal_ai::es_tokenizer::is_keyword("return"));
    assert!(!formal_ai::es_tokenizer::is_keyword("half"));
    assert!(formal_ai::es_tokenizer::regex_may_follow("typeof"));
    assert!(!formal_ai::es_tokenizer::regex_may_follow("this"));
}

#[test]
fn template_literals_carry_chunks_and_nested_interpolations() {
    let source = "const s = `a${ `b${c}` }d${e}`;";
    let trees = tokenize(source).expect("the fixture must tokenize");
    let Some(Tree::Template { parts, .. }) = trees
        .iter()
        .find(|tree| matches!(tree, Tree::Template { .. }))
    else {
        panic!("the fixture contains one outer template");
    };
    // Chunk "a", an interpolation holding a nested template, chunk "d", and
    // a final interpolation — four parts in source order.
    assert_eq!(parts.len(), 4);
    let (
        TemplatePart::Chunk(first),
        TemplatePart::Interpolation { trees: inner, .. },
        TemplatePart::Chunk(second),
        TemplatePart::Interpolation { trees: tail, .. },
    ) = (&parts[0], &parts[1], &parts[2], &parts[3])
    else {
        panic!("the template alternates chunks and interpolations");
    };
    assert_eq!(first.text, "a");
    assert_eq!(first.kind, TokenKind::TemplateChunk);
    assert_eq!(second.text, "d");
    let Some(Tree::Template {
        parts: nested_parts,
        ..
    }) = inner
        .iter()
        .find(|tree| matches!(tree, Tree::Template { .. }))
    else {
        panic!("the first interpolation holds a nested template");
    };
    let (TemplatePart::Chunk(chunk), TemplatePart::Interpolation { trees: leaf, .. }) =
        (&nested_parts[0], &nested_parts[1])
    else {
        panic!("the nested template is a chunk then an interpolation");
    };
    assert_eq!(chunk.text, "b");
    let mut collected = Vec::new();
    leaves(leaf, &mut collected);
    assert_eq!(collected[0].text, "c");
    let mut collected = Vec::new();
    leaves(tail, &mut collected);
    assert_eq!(collected[0].text, "e");
}

#[test]
fn unterminated_constructs_report_their_opening_byte() {
    let cases: [(&str, TokenizeErrorKind, usize); 7] = [
        ("\"abc", TokenizeErrorKind::UnterminatedString, 0),
        ("'abc", TokenizeErrorKind::UnterminatedString, 0),
        ("`abc${x", TokenizeErrorKind::UnterminatedTemplate, 0),
        ("x = `a", TokenizeErrorKind::UnterminatedTemplate, 4),
        ("const re = /ab", TokenizeErrorKind::UnterminatedRegExp, 11),
        ("/* forever", TokenizeErrorKind::UnterminatedComment, 0),
        (
            "f(a, [b",
            TokenizeErrorKind::UnclosedGroup(Delimiter::Bracket),
            5,
        ),
    ];
    for (source, kind, start) in cases {
        let error =
            tokenize(source).expect_err("an unterminated construct is an error, not a guess");
        assert_eq!(error.kind, kind, "for source {source:?}");
        assert_eq!(error.span.start, start, "for source {source:?}");
    }

    let closers: [(&str, Delimiter); 3] = [
        ("a)", Delimiter::Paren),
        ("a]", Delimiter::Bracket),
        ("(a}", Delimiter::Brace),
    ];
    for (source, delim) in closers {
        let error =
            tokenize(source).expect_err("a stray or mismatched closer is an error, not a guess");
        assert_eq!(
            error.kind,
            TokenizeErrorKind::UnexpectedClosing(delim),
            "for source {source:?}"
        );
    }
}

#[test]
fn punctuators_match_longest_first_and_split_decimal_ternaries() {
    let source = "a >>>= b; c ??= d; e?.f; g **= 2;";
    let texts = leaf_texts(source);
    for longest in [">>>=", "??=", "?.", "**="] {
        assert!(
            texts.contains(&longest),
            "{longest} must survive as one token"
        );
    }

    // `?.` followed by a digit is a ternary over a decimal, never optional
    // chaining — the lexer must not eat the decimal's leading dot.
    let ternary = leaf_texts("a?.3:b");
    assert!(
        ternary.contains(&"?"),
        "the ternary question mark stands alone"
    );
    assert!(ternary.contains(&".3"), "the decimal keeps its leading dot");
    assert!(
        !ternary.contains(&"?."),
        "no optional chaining was invented"
    );
}

#[test]
fn private_names_hashbang_and_comments_normalize() {
    let texts = leaf_texts(
        "#!/usr/bin/env node\nclass A { #field = 1; /* drop */ get() { return this.#field; } }\n",
    );
    assert!(texts.contains(&"#field"), "private names lex as one token");
    assert!(
        !texts.iter().any(|text| text.contains("env")),
        "the hashbang and comments are not part of the normalized tree"
    );
}
