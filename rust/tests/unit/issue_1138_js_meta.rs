//! Plan 16 L2 acceptance (issue #1138): the js ↔ meta ↔ ts pivot — extract,
//! document, parse back, render, and the identity round trip over the whole
//! committed `./js` tree, under the projection rules the seed carries.
//!
//! The engine/data split is the point of the leaf, so the corpus test checks
//! behavior while the seed tests check where the behavior lives: removing a
//! `carries_to` row from the seed text must make the renderer refuse, not
//! find a second opinion in code.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::es_meta::{
    OwnedPart, OwnedTree, PivotParseError, ProjectionTarget, SourceLanguage, extract,
    parse_document, projection_rules, render_document, render_source,
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

/// The pivot's own copy of the tree, for equality against a re-extracted one.
fn owned_of_source(language: SourceLanguage, source: &str) -> Vec<OwnedTree> {
    extract("probe.js", language, source)
        .expect("the fixture must extract")
        .trees
}

#[test]
fn the_whole_js_corpus_round_trips_through_meta_to_ts_and_back() {
    let root = repository_root();
    let mut files = Vec::new();
    walk_js_files(&root.join("js"), &mut files);
    assert!(
        !files.is_empty(),
        "the committed ./js corpus must be present"
    );

    for path in &files {
        let source = fs::read_to_string(path).expect("the corpus file must be readable");
        let display_path = path
            .strip_prefix(&root)
            .expect("the corpus file lives under the repository root")
            .display()
            .to_string();
        let original = owned_of_source(SourceLanguage::JavaScript, &source);

        // js → meta → ts: the document parses back, ts renders it, and
        // re-extracting the ts (as TypeScript) yields the same tree.
        let document = extract(&display_path, SourceLanguage::JavaScript, &source)
            .expect("the corpus file must extract");
        let rendered_document = render_document(&document);
        let parsed =
            parse_document(&rendered_document).expect("a rendered pivot document must parse back");
        assert_eq!(
            parsed, document,
            "{display_path}: the document must parse back to itself"
        );
        let ts = render_source(&parsed, ProjectionTarget::TypeScript);
        assert!(
            ts.report.refused.is_empty(),
            "{display_path}: js → ts must refuse nothing: {:?}",
            ts.report.refused
        );
        let ts_source = ts.output.expect("zero refusals means output");
        let ts_tree = extract(&display_path, SourceLanguage::TypeScript, &ts_source)
            .expect("the rendered ts must tokenize")
            .trees;
        assert_eq!(
            ts_tree, original,
            "{display_path}: the ts leg must be CST-equal"
        );

        // → js: the identity round trip. Canonical spacing differs from the
        // original text; the token tree must not.
        let js = render_source(&parsed, ProjectionTarget::JavaScript);
        assert!(
            js.report.refused.is_empty(),
            "{display_path}: the js leg must refuse nothing: {:?}",
            js.report.refused
        );
        let js_source = js.output.expect("zero refusals means output");
        let js_tree = owned_of_source(SourceLanguage::JavaScript, &js_source);
        assert_eq!(
            js_tree, original,
            "{display_path}: js → ts → js must be CST-equal"
        );
    }
}

#[test]
fn pivot_documents_are_canonical_across_the_round_trip() {
    let source = "const re = /a\\d+/g ;\nconst tpl = `sum ${ 1 + 2 } units`;\n";
    let document =
        extract("probe.js", SourceLanguage::JavaScript, source).expect("the fixture must extract");
    let rendered = render_source(&document, ProjectionTarget::JavaScript)
        .output
        .expect("nothing is refused");
    let re_document = extract("probe.js", SourceLanguage::JavaScript, &rendered)
        .expect("the rendered source must extract");
    assert_eq!(
        render_document(&re_document),
        render_document(&document),
        "extract → render → extract must reproduce the same document: {rendered}"
    );
}

#[test]
fn projection_rules_live_in_the_seed_not_the_engine() {
    let rules = projection_rules();
    // The engine's whole emission vocabulary — every token class plus the
    // three structural nodes — must be exactly the seed's class list, in
    // both directions: a class only the engine knows is an unruled carry,
    // and a class only the seed knows is dead data.
    let classes: Vec<&str> = rules
        .classes
        .iter()
        .map(|rule| rule.class.as_str())
        .collect();
    for class in [
        "identifier",
        "numeric",
        "string",
        "template_chunk",
        "regexp",
        "punctuator",
        "group",
        "template",
        "interpolation",
    ] {
        assert!(
            classes.contains(&class),
            "the seed must carry the {class} class, found {classes:?}"
        );
    }
    assert_eq!(
        classes.len(),
        9,
        "the seed carries no class the engine cannot emit: {classes:?}"
    );

    // Every construct the seed declares refuses to js only; ts is its own
    // superset and accepts every construct.
    let constructs = rules
        .constructs
        .iter()
        .map(|rule| rule.construct.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        constructs,
        vec![
            "interface_declaration",
            "enum_declaration",
            "type_alias",
            "satisfies_expression",
        ],
        "the refused list is exactly what real recognition supports — no `as` \
         cast signature, because `export {{ x as y }}` is plain ES module syntax"
    );
    for rule in &rules.constructs {
        assert_eq!(
            rule.refuses_to,
            vec!["js".to_owned()],
            "{} refuses to js and nothing else",
            rule.construct
        );
        assert!(
            !rule.signature.is_empty(),
            "{} must name a signature, not a vibe",
            rule.construct
        );
    }
}

#[test]
fn typescript_only_constructs_are_refused_by_name() {
    // A TS interface cannot be valid plain JavaScript; the seed's signature
    // says so and the js render refuses the whole document by name.
    let document = extract(
        "iface.ts",
        SourceLanguage::TypeScript,
        "interface Foo { x : number }\nconst plain = 1;\n",
    )
    .expect("the fixture must extract");
    let js = render_source(&document, ProjectionTarget::JavaScript);
    assert!(js.output.is_none(), "a refused document produces no output");
    assert!(
        js.report
            .refused
            .iter()
            .any(|refusal| refusal.construct == "interface_declaration"),
        "the interface must be refused by name: {:?}",
        js.report.refused
    );

    // The same construct carries to ts, which accepts it.
    let ts = render_source(&document, ProjectionTarget::TypeScript);
    assert!(
        ts.report.refused.is_empty(),
        "ts accepts its own construct: {:?}",
        ts.report.refused
    );

    // A plain-subset ts file carries to js untouched.
    let plain = extract(
        "plain.ts",
        SourceLanguage::TypeScript,
        "export function add ( a , b ) { return a + b }\n",
    )
    .expect("the fixture must extract");
    let plain_js = render_source(&plain, ProjectionTarget::JavaScript);
    assert!(
        plain_js.report.refused.is_empty() && plain_js.output.is_some(),
        "the plain subset carries: {:?}",
        plain_js.report.refused
    );

    // Every construct signature the seed declares refuses its construct. No
    // `as` cast fixture: `export { x as y }` is plain ES, so the seed
    // deliberately declares no such signature (see the seed header).
    for fixture in [
        ("enum_declaration", "enum Color { A , B }\n"),
        ("type_alias", "type Id = string ;\n"),
        ("satisfies_expression", "const c = { } satisfies Id ;\n"),
    ] {
        let document = extract("probe.ts", SourceLanguage::TypeScript, fixture.1)
            .expect("the fixture must extract");
        let js = render_source(&document, ProjectionTarget::JavaScript);
        assert!(
            js.report
                .refused
                .iter()
                .any(|refusal| refusal.construct == fixture.0),
            "{} must be refused by name: {:?}",
            fixture.0,
            js.report.refused
        );
    }
}

#[test]
fn templates_and_regexps_survive_canonical_spacing() {
    let source = "const tpl = `a ${ x } b ${ `nested ${ y }` } c`;\nreturn /a b/g . test ( x )\n";
    let document =
        extract("probe.js", SourceLanguage::JavaScript, source).expect("the fixture must extract");
    let rendered = render_source(&document, ProjectionTarget::TypeScript)
        .output
        .expect("nothing is refused");
    let re_extracted = extract("probe.ts", SourceLanguage::TypeScript, &rendered)
        .expect("the rendered source must extract");
    assert_eq!(
        re_extracted.trees, document.trees,
        "spacing must not merge or split tokens"
    );
    // The regexp stayed one token and every template part kept its shape.
    let texts: Vec<&str> = collected_texts(&document.trees);
    assert!(
        texts.contains(&"/a b/g"),
        "the regexp is one token: {texts:?}"
    );
    assert!(
        texts.contains(&"a "),
        "the first chunk is intact: {texts:?}"
    );
    assert!(
        texts.contains(&"nested "),
        "the nested chunk is intact: {texts:?}"
    );
}

fn collected_texts(trees: &[OwnedTree]) -> Vec<&str> {
    let mut out = Vec::new();
    collect_texts(trees, &mut out);
    out
}

fn collect_texts<'a>(trees: &'a [OwnedTree], out: &mut Vec<&'a str>) {
    for tree in trees {
        match tree {
            OwnedTree::Leaf(token) => out.push(&token.text),
            OwnedTree::Group { trees, .. } => collect_texts(trees, out),
            OwnedTree::Template { parts } => {
                for part in parts {
                    match part {
                        OwnedPart::Chunk(text) => out.push(text),
                        OwnedPart::Interpolation { trees } => collect_texts(trees, out),
                    }
                }
            }
        }
    }
}

#[test]
fn documents_that_are_not_pivot_documents_are_rejected_by_name() {
    assert_eq!(parse_document(""), Err(PivotParseError::MissingRecord));
    assert_eq!(
        parse_document("token_tree\n  language javascript\n"),
        Err(PivotParseError::MissingField("target"))
    );
    assert_eq!(
        parse_document("token_tree\n  target \"x.js\"\n"),
        Err(PivotParseError::MissingField("language"))
    );
    assert_eq!(
        parse_document("token_tree\n  target \"x.js\"\n  language javascript\n"),
        Err(PivotParseError::MissingField("token_count"))
    );
    let with_bad_tree = concat!(
        "token_tree\n",
        "  target \"x.js\"\n",
        "  language javascript\n",
        "  token_count 0\n",
        "  trees\n",
        "    leaf nope \"x\"\n",
    );
    assert_eq!(
        parse_document(with_bad_tree),
        Err(PivotParseError::UnknownKind("nope".to_owned()))
    );
}
