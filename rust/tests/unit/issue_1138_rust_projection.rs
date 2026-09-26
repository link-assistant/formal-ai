//! Plan 16 L7 (issue #1138): rust rendering from the pivot, landed at the
//! fidelity the engine already guarantees. The meta-language dependency
//! ships a lossless network serialization (`to_lino`/`from_lino`), and
//! `reconstruct_text` is byte-identical (issue #558), so the meta → rust
//! leg is wiring: every owned Rust module survives
//! source → network lino → source byte-for-byte through the public
//! translate surface, and a document in any other lino dialect — the
//! self-AST census, the ES token-tree pivot — is refused with the expected
//! dialect named, never mis-parsed into invented source. The legs that
//! render another grammar's source text (rust → js/ts, js/ts → rust) stay
//! owed to L8 until grammar projection rules exist.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};

/// A small, ordinary Rust module the dialect probes round-trip — real
/// syntax, not a special case.
const PROBE: &str =
    "fn dialect_probe(sealed: bool) -> u32 {\n    if sealed { 7 } else { 41 + 1 }\n}\n";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

/// The owned corpus is the census set, derived: every committed census
/// document names its source module one-to-one, so walking the census tree
/// is walking `rust/src` exactly as the repository defines ownership.
fn collect_owned(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_owned(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("lino") {
            out.push(path);
        }
    }
    out.sort();
}

fn rendered(outcome: TranslationOutcome, context: &str) -> String {
    match outcome {
        TranslationOutcome::Rendered { target, .. } => target,
        TranslationOutcome::Refused { refusals } => {
            panic!("{context} must not refuse: {refusals:?}")
        }
        TranslationOutcome::Pending { plan_leaf } => {
            panic!("{context} must be live, not pending on {plan_leaf}")
        }
        TranslationOutcome::Invalid { reason } => {
            panic!("{context} must render from the network document: {reason}")
        }
    }
}

/// The leaf's own sentence: the full-fidelity rust leg is the lossless
/// serialization the engine already guarantees, proven corpus-wide. Every
/// owned module — derived from the committed census tree, not selected —
/// goes source → `to_lino` → `from_lino` + `reconstruct_text` through the
/// public translate surface and must come back byte-identical. The biggest
/// modules are the point, not an exception: a serialization that drops a
/// span anywhere fails on the file that has the most of them.
#[test]
fn every_owned_rust_module_round_trips_through_the_network_serialization() {
    let root = root();
    let mut census = Vec::new();
    collect_owned(
        &root.join("data").join("meta").join("self-ast"),
        &mut census,
    );
    assert!(
        census.len() >= 600,
        "the owned corpus is the census set and is never empty: {count} files",
        count = census.len()
    );
    for census_path in &census {
        let relative = census_path
            .strip_prefix(root.join("data").join("meta").join("self-ast"))
            .expect("the census document lives under the census tree")
            .with_extension("rs");
        let source_path = root.join("rust").join(&relative);
        let display = relative.to_string_lossy().replace('\\', "/");
        let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
            panic!("rust/{display}: every census document names a source module: {error}")
        });
        let lino = formal_ai::agentic_coding::self_ast::network_lino(&source);
        let back = rendered(
            meta_translate::translate(SourceRoot::Meta, SourceRoot::Rust, &display, &lino),
            &display,
        );
        assert_eq!(
            back, source,
            "rust/{display}: the network round trip must be byte-identical"
        );
    }
}

/// The Meta root carries three lino dialects and the rust leg reads exactly
/// one: a census document (what rust → meta renders) and a token-tree pivot
/// document (what the ES legs carry) are both refused with the expected
/// dialect named — the leaf's honest-gap contract.
#[test]
fn meta_to_rust_refuses_every_other_lino_dialect_by_name() {
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Meta, SourceRoot::Rust),
        None
    );
    let census = rendered(
        meta_translate::translate(SourceRoot::Rust, SourceRoot::Meta, "probe.rs", PROBE),
        "the census probe",
    );
    let pivot = rendered(
        meta_translate::translate(
            SourceRoot::JavaScript,
            SourceRoot::Meta,
            "probe.js",
            "value;\n",
        ),
        "the pivot probe",
    );
    for (dialect, document) in [("census", &census), ("token-tree pivot", &pivot)] {
        match meta_translate::translate(SourceRoot::Meta, SourceRoot::Rust, "probe", document) {
            TranslationOutcome::Invalid { reason } => {
                assert!(
                    reason.contains("network serialization"),
                    "the {dialect} refusal must name the expected dialect: {reason}"
                );
            }
            other => panic!("the {dialect} document must be refused, not {other:?}"),
        }
    }
}

/// The registry row for the landed leg — `network` fidelity, live — and the
/// leg it declares really renders through the public surface.
#[test]
fn meta_to_rust_is_declared_network_and_live() {
    let row = meta_translate::root_projections()
        .into_iter()
        .find(|row| row.from == SourceRoot::Meta && row.to == SourceRoot::Rust)
        .expect("meta → rust is declared");
    assert_eq!(row.fidelity, "network");
    assert_eq!(row.owed_by, None);
    let lino = formal_ai::agentic_coding::self_ast::network_lino(PROBE);
    let back = rendered(
        meta_translate::translate(SourceRoot::Meta, SourceRoot::Rust, "probe.rs", &lino),
        "the probe",
    );
    assert_eq!(back, PROBE);
}

/// The L8 machinery probe: the walk follows the data, built through the
/// same public builder path the seed generator uses. One tiny module
/// exercises every placeholder form the walk expands — the variadic
/// `{*cap|sep}` over all bindings, recursive `{cap}`, the `{cap:text}`
/// span splice, `{.:text}`, the `{{`/`}}` escapes — and the declared
/// refusal that splices an integer literal through verbatim.
#[test]
fn grammar_projection_walk_follows_the_data() {
    use formal_ai::rust_projection::{ProjectionOutcome, project, projection_from};
    use meta_language::{LinkQuery, LinkType, TranslationRule, TranslationRuleSet};

    let probe_set = || {
        // The template literals below are projection placeholders, not
        // format strings.
        #[allow(clippy::literal_string_with_formatting_args, clippy::type_complexity)]
        let rows: &[(&str, &str, &[(&str, &str)])] = &[
            (
                "source_file",
                "(source_file (_)* @item)",
                &[("", "{*item|\n}")],
            ),
            ("identifier", "(identifier)", &[("", "{.:text}")]),
            (
                "function_item",
                "(function_item name: (identifier) @name parameters: (parameters) @parameters body: (block) @body)",
                &[("", "function {name}({parameters}) {{\n{body}\n}}")],
            ),
            (
                "parameters",
                "(parameters (_)* @param)",
                &[("", "{*param|, }")],
            ),
            (
                // A bare parameter is a type_identifier child of
                // parameters — there is no parameter node at all.
                "type_identifier",
                "(type_identifier)",
                &[("", "{.:text}")],
            ),
            (
                "block",
                "(block (_)* @statement)",
                &[("", "{*statement|\n}")],
            ),
            (
                "expression_statement",
                "(expression_statement (_) @expression)",
                &[("", "{expression};")],
            ),
            (
                "binary_expression",
                "(binary_expression left: (_) @left operator: (_) @operator right: (_) @right)",
                &[("", "{left} {operator:text} {right}")],
            ),
        ];
        let mut rule_set = TranslationRuleSet::new("grammar-projection-probe");
        for (name, sexpression, templates) in rows {
            let query = LinkQuery::from_sexpression(sexpression)
                .unwrap_or_else(|error| panic!("{name}: {error}"))
                .with_link_type(LinkType::Syntax);
            let mut rule = TranslationRule::new(format!("probe:{name}"), query);
            for (_, template) in *templates {
                rule = rule.with_template("javascript", *template);
            }
            rule_set = rule_set.with_rule(rule);
        }
        rule_set.to_lino()
    };

    // The declared refusal rides the same document as an extra root child
    // of the network round-trip — the composition the generator performs.
    let lino = probe_set();
    let network = meta_language::LinkNetwork::from_lino(&lino).expect("the probe serializes");
    let rule_root = network
        .links()
        .find(|link| link.metadata().term() == Some("translation-rule-set"))
        .expect("the rule set root is present")
        .id();
    let mut network = network;
    network.insert_link(
        [rule_root],
        meta_language::LinkMetadata::new()
            .with_link_type(LinkType::Semantic)
            .with_named(true)
            .with_term("integer_literal")
            .with_language("grammar-projection-refusal")
            .with_definition("any"),
    );
    let seed = network.to_lino();

    let projection = projection_from(&seed).expect("the probe seed loads");
    // The semicoloned body keeps the expression_statement wrapper a block
    // tail expression would drop.
    let source = "fn add(a, b) { a + 1; }";
    let parsed = formal_ai::grammar_kinds::parse_network("rust", source);
    let rendered = projection
        .render(&parsed, source, "javascript")
        .expect("the probe module is fully covered");
    assert_eq!(rendered, "function add(a, b) {\na + 1;\n}");

    // The public entry composes the same walk over the probe seed's
    // contract: without the refusal row, the walk refuses naming the kind.
    let bare = projection_from(&probe_set()).expect("the bare probe loads");
    let kinds: std::collections::BTreeSet<String> =
        formal_ai::grammar_kinds::named_source_kinds("rust", source)
            .into_iter()
            .map(|(kind, _)| kind)
            .collect();
    let refusals = bare.coverage_kinds(&kinds, "javascript");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.construct == "integer_literal"),
        "the undeclared literal must be refused by name: {refusals:?}"
    );
    assert!(
        refusals
            .iter()
            .all(|refusal| refusal.construct != "source_file"
                && refusal.construct != "function_item"
                && refusal.construct != "block"),
        "a ruled kind is covered, so an empty ruled table cannot pass: {refusals:?}"
    );
    match project("rust", "javascript", source) {
        ProjectionOutcome::Rendered { .. } => {}
        refused @ ProjectionOutcome::Refused { .. } => {
            panic!("the embedded seed decides this leg, not the probe: {refused:?}")
        }
    }
}

/// The embedded seed is well-formed and loads through the public loader,
/// and the entry refuses unknown or same-label directions honestly.
#[test]
fn grammar_projection_seed_loads_and_entry_refuses_honestly() {
    use formal_ai::rust_projection::{ProjectionOutcome, project};

    let projection =
        formal_ai::rust_projection::projection_from(formal_ai::seed::GRAMMAR_PROJECTION_RULES_LINO)
            .expect("the embedded seed is well-formed");
    assert!(projection.rule_count() > 0, "the seed carries rules");
    assert!(projection.refusal_count() > 0, "the seed carries refusals");
    for (from, target) in [("nope", "rust"), ("rust", "nope"), ("rust", "rust")] {
        match project(from, target, "x") {
            ProjectionOutcome::Refused { refusals } => {
                assert_eq!(refusals.len(), 1, "{from} -> {target} refuses once");
            }
            rendered @ ProjectionOutcome::Rendered { .. } => {
                panic!("{from} -> {target} must be refused, not {rendered:?}")
            }
        }
    }
}

/// The corpus ratchet: the seed's authored rows grow against the measured
/// corpus inventories. A direction goes live only when its remaining count
/// reaches zero; until then the pins here only ever move down.
#[test]
fn grammar_projection_corpus_ratchet() {
    use std::collections::BTreeSet;

    use formal_ai::grammar_kinds::{corpus_sources, named_corpus_inventory};
    use formal_ai::rust_projection::projection_from;

    // Pins, tightened to the measured counts as the authored table grows.
    const RUST_TO_JAVASCRIPT_REMAINING: usize = 61;
    const RUST_TO_TYPESCRIPT_REMAINING: usize = 61;
    const JAVASCRIPT_TO_RUST_REMAINING: usize = 42;
    const TYPESCRIPT_TO_RUST_REMAINING: usize = 45;

    let projection = projection_from(formal_ai::seed::GRAMMAR_PROJECTION_RULES_LINO)
        .expect("the embedded seed is well-formed");
    let root = root();
    let pins: [((&str, &str), usize); 4] = [
        (("rust", "javascript"), RUST_TO_JAVASCRIPT_REMAINING),
        (("rust", "typescript"), RUST_TO_TYPESCRIPT_REMAINING),
        (("javascript", "rust"), JAVASCRIPT_TO_RUST_REMAINING),
        (("typescript", "rust"), TYPESCRIPT_TO_RUST_REMAINING),
    ];
    for ((from, target), pin) in pins {
        let corpus = formal_ai::grammar_kinds::corpus_by_label(from)
            .expect("the direction's corpus is owned");
        let sources = corpus_sources(&root, *corpus);
        let kinds: BTreeSet<String> = named_corpus_inventory(from, &sources).into_keys().collect();
        let remaining = projection.coverage_kinds(&kinds, target);
        assert!(
            remaining.len() <= pin,
            "{from} -> {target}: {remaining:?} must shrink to {pin} or below"
        );
    }
}

/// The authored tranche, walked end to end through the public entry the
/// way a caller gets it. Each probe is chosen to walk the rows it names —
/// a rule whose query never matches is dead weight the coverage table
/// cannot see — and every render must re-parse under the target grammar
/// with no error flags, because a projection that emits target syntax the
/// target parser rejects is not a projection.
#[test]
fn grammar_projection_tranche_renders_and_reparses() {
    use formal_ai::rust_projection::{ProjectionOutcome, project};

    // The template literals below are projection placeholders, not
    // format strings.
    #[allow(clippy::literal_string_with_formatting_args, clippy::type_complexity)]
    let probes: &[(&str, &str, &str, &str)] = &[
        // visibility drops, the parameter keeps its name for javascript
        // and its annotation for typescript, the block tail expression
        // rides the block row.
        (
            "rust",
            "javascript",
            "pub fn area(w: u32) -> u32 { w * w }",
            "function area(w) {\nw * w\n}",
        ),
        (
            "rust",
            "typescript",
            "pub fn area(w: u32) -> u32 { w * w }",
            "function area(w: u32) {\nw * w\n}",
        ),
        // field access, the scoped path as property access, the call,
        // and the borrow dropped from both the type and the value.
        (
            "rust",
            "javascript",
            "fn go(x: u32, r: &u32) { let a = x.field; let b = std::next(x); let c = &x; }",
            "function go(x, r) {\nlet a = x.field;\nlet b = std.next(x);\nlet c = x;\n}",
        ),
        (
            "rust",
            "typescript",
            "fn go(x: u32, r: &u32) { let a = x.field; let b = std::next(x); let c = &x; }",
            "function go(x: u32, r: u32) {\nlet a = x.field;\nlet b = std.next(x);\nlet c = x;\n}",
        ),
        // the generic type and the turbofish both drop; the let omits
        // the annotation in every target.
        (
            "rust",
            "javascript",
            "fn make() { let v: Vec<u32> = build::<u32>(); }",
            "function make() {\nlet v = build();\n}",
        ),
        // the function declaration, its parameters, its block owning the
        // braces, and the valued return.
        (
            "javascript",
            "rust",
            "function add(a, b) { return a + b; }",
            "fn add(a, b) {\nreturn a + b;\n}",
        ),
        // the valued declarator under the one declaration kind, the
        // member access, and the call with its argument list.
        (
            "javascript",
            "rust",
            "var total = obj.compute(1, 2);",
            "let total = obj.compute(1, 2);",
        ),
        // the bare declarator, the bare return, the assignment, the
        // parenthesized expression, and the general expression statement
        // carrying a call.
        (
            "javascript",
            "rust",
            "var x;\nfunction f() { return; }\nx = (x + 1);\nconsole.log(x);",
            "let x;\n\nfn f() {\nreturn;\n}\n\nx = (x + 1);\n\nconsole.log(x);",
        ),
        // the if/else pair: conditions take target parens, blocks own
        // their braces, and the else clause wraps each shape it holds.
        (
            "rust",
            "javascript",
            "fn a(x: u32) -> u32 { if x > 0 { 1 } else { 2 } }",
            "function a(x) {\nif (x > 0) {\n1\n} else {\n2\n};\n}",
        ),
        (
            "rust",
            "typescript",
            "fn a(x: u32) -> u32 { if x > 0 { 1 } else { 2 } }",
            "function a(x: u32) {\nif (x > 0) {\n1\n} else {\n2\n};\n}",
        ),
        // while, the else-if chain, break and continue.
        (
            "rust",
            "javascript",
            "fn b(x: u32) { while x > 0 { if x > 1 { break; } else if x == 1 { continue; } else { break; } } }",
            "function b(x) {\nwhile (x > 0) {\nif (x > 1) {\nbreak;\n} else if (x == 1) {\ncontinue;\n} else {\nbreak;\n};\n};\n}",
        ),
        // an attribute drops to the visibility marker's empty render, the
        // loop crosses to while (true), and the macro call keeps its
        // identifier and token tree.
        (
            "rust",
            "javascript",
            "#[derive(Debug)]\nfn g() { loop { println!(\"hi {}\", 1); } }",
            "\n\nfunction g() {\nwhile (true) {\nprintln(\"hi {}\", 1);\n};\n}",
        ),
        // closures cross to arrows — move drops the way borrows do — and
        // returns keep their valued/bare split.
        (
            "rust",
            "javascript",
            "fn c() { let f = |a, b| a + b; let g = move |x| x * 2; }\n\nfn d(x: u32) -> u32 { return x + 1; }\n\nfn e() { return; }",
            "function c() {\nlet f = (a, b) => a + b;\nlet g = (x) => x * 2;\n}\n\nfunction d(x) {\nreturn x + 1;\n}\n\nfunction e() {\nreturn;\n}",
        ),
        // the receiver renders no parameter slot and self becomes this.
        (
            "rust",
            "javascript",
            "fn s(&self) -> u32 { self.width }",
            "function s() {\nthis.width\n}",
        ),
        // let-else: the tuple struct pattern crosses to object
        // destructuring and the else block drops with the constructor.
        (
            "rust",
            "javascript",
            "fn h(o: Option<u32>) -> u32 { let Some(x) = o else { return 0 }; x }",
            "function h(o) {\nlet { x } = o;\nx\n}",
        ),
        // unary expressions splice their glyph sequence, the struct
        // expression lowers to an object literal, and tuple and array
        // literals ride their own spans.
        (
            "rust",
            "javascript",
            "fn p(f: bool, x: u32) -> u32 { if !f { return -1; } x }\n\nfn q() { let s = Point { x: 1, y: 2 }; let t = (1, 2); let arr = [3, 4]; }",
            "function p(f, x) {\nif (!f) {\nreturn -1;\n};\nx\n}\n\nfunction q() {\nlet s = { x: 1, y: 2 };\nlet t = (1, 2);\nlet arr = [3, 4];\n}",
        ),
        // lifetimes drop inside a reference type and a scoped type path
        // crosses to dotted access in annotation position; the type
        // arguments drop the way the turbofish does.
        (
            "rust",
            "typescript",
            "fn l(x: &'a u32) { }\n\nfn m(v: std::vec::Vec<u32>) { }",
            "function l(x: u32) {\n\n}\n\nfunction m(v: std.vec.Vec) {\n\n}",
        ),
        // a const item drops its annotation, a struct crosses to a class
        // shell, and a tuple pattern destructures as an array pattern.
        (
            "rust",
            "javascript",
            "const MAX: u32 = 5;\n\nstruct P { width: u32 }\n\nfn r(t: u32) { let (a, b) = t; }",
            "const MAX = 5;\n\nclass P {\nwidth;\n}\n\nfunction r(t) {\nlet [a, b] = t;\n}",
        ),
        (
            "rust",
            "typescript",
            "const MAX: u32 = 5;\n\nstruct P { width: u32 }\n\nfn r(t: u32) { let (a, b) = t; }",
            "const MAX = 5;\n\nclass P {\nwidth: u32;\n}\n\nfunction r(t: u32) {\nlet [a, b] = t;\n}",
        ),
        // the javascript if spine with both else shapes, the unary
        // operators, null, the ternary, the subscript, and this.
        (
            "javascript",
            "rust",
            "if (a > 0) { b(); } else if (a < 9) { c(); } else { d(); }\n\nif (a) { b(); }\n\nlet x = !a;\n\nlet n = null;\n\nlet t = a ? b : c;\n\nlet s = a[i];\n\nlet u = this.x;",
            "if (a > 0) {\nb();\n} else if (a < 9) {\nc();\n} else {\nd();\n}\n\nif (a) {\nb();\n}\n\nlet x = !a;\n\nlet n = None;\n\nlet t = if a { b } else { c };\n\nlet s = a[i];\n\nlet u = self.x;",
        ),
        // arrows in both parameter shapes, the anonymous function
        // expression as a closure, the array and sequence literals, the
        // constructor call, and the augmented assignment splice.
        (
            "javascript",
            "rust",
            "const f = (a, b) => a + b;\n\nconst g = x => x * 2;\n\nconst h = function(a) { return a; };\n\nlet arr = [1, 2, 3];\n\nlet seq = (a, b, c);\n\nlet v = new Foo(1);\n\nlet w = 0;\n\nw += 2;",
            "let f = |a, b| a + b;\n\nlet g = |x| x * 2;\n\nlet h = |a| {\nreturn a;\n};\n\nlet arr = [1, 2, 3];\n\nlet seq = (a, b, c);\n\nlet v = Foo(1);\n\nlet w = 0;\n\nw += 2;",
        ),
        // the loop statements, the for-of binding, and the switch as a
        // match with case arms and the default wildcard.
        (
            "javascript",
            "rust",
            "while (a > 0) { a = a - 1; continue; }\n\nfor (const x of items) { b(x); }\n\nswitch (k) { case 1: c(); break; default: d(); }",
            "while (a > 0) {\na = a - 1;\ncontinue;\n}\n\nfor x in items {\nb(x);\n}\n\nmatch (k) {\n1 => {\nc();\nbreak;\n},\n_ => {\nd();\n},\n}",
        ),
        // typescript parameters: the annotated wrapper splices its
        // annotation span and the bare wrapper keeps the pattern alone.
        (
            "typescript",
            "rust",
            "function f(a: number, b: string) { return a; }\n\nfunction g(a) { return a; }",
            "fn f(a: number, b: string) {\nreturn a;\n}\n\nfn g(a) {\nreturn a;\n}",
        ),
        // struct patterns cross to object destructuring in both field
        // shapes — renamed and shorthand.
        (
            "rust",
            "javascript",
            "fn c(p: Point) { let Point { x, y } = p; }\n\nfn d(p: Point) { let Point { x: a, y } = p; }",
            "function c(p) {\nlet { x, y } = p;\n}\n\nfunction d(p) {\nlet { x: a, y } = p;\n}",
        ),
        // the array type takes typescript brackets and the tuple type
        // takes its tuple list.
        (
            "rust",
            "typescript",
            "fn b(v: [u32; 3], t: (u32, u32)) { }",
            "function b(v: u32[], t: [u32, u32]) {\n\n}",
        ),
    ];
    for (from, target, source, expected) in probes {
        match project(from, target, source) {
            ProjectionOutcome::Rendered { source: output, .. } => {
                assert_eq!(output, *expected, "{from} -> {target} over {source}");
                let reparsed = formal_ai::grammar_kinds::parse_network(target, &output);
                let flagged: Vec<String> = reparsed
                    .links()
                    .filter(|link| {
                        let flags = link.metadata().flags();
                        flags.is_error() || flags.has_error()
                    })
                    .filter_map(|link| link.metadata().term().map(str::to_owned))
                    .collect();
                assert!(
                    flagged.is_empty(),
                    "{from} -> {target} render does not re-parse under the target \
                     grammar: {output} flagged {flagged:?}"
                );
            }
            refused @ ProjectionOutcome::Refused { .. } => {
                panic!("{from} -> {target} must render {source}: {refused:?}")
            }
        }
    }
}
