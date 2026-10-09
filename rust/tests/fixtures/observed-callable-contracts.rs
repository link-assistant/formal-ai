use formal_ai::agentic_coding::module_function::callable_catalog::{
    observe_source_callables, observed_callable_graphs, observed_guarded_graph,
};
use serde_json::{Value, json};
const FIRST: &str = "export function choose(value) { return value === null ? null : 'ready'; } export { choose as publicChoice };";
const SECOND: &str = "export function render(item) { return item === 'ready'; }";
fn observations() -> Vec<Value> {
    vec![
        json!({"path":"producer.mjs","content":FIRST}),
        json!({"path":"consumer.mjs","content":SECOND}),
    ]
}
fn binding(path: &str, exported: &str, content: &str) -> Value {
    json!({"path":path,"exported":exported,"contentId":formal_ai::source_fetch::sha256_hex(content.as_bytes())})
}
#[test]
fn complete_unicode_declarations_keep_both_coordinate_systems() {
    let source = "// 文\nexport function describe(input) { return input; }";
    let catalog = observe_source_callables(source, "source.mjs");
    let entry = &catalog["declarations"][0];
    let start = entry["span"]["byteStart"].as_u64().unwrap();
    let end = entry["span"]["byteEnd"].as_u64().unwrap();
    let observed = &source[usize::try_from(start).unwrap()..usize::try_from(end).unwrap()];
    assert_eq!(entry["source"], observed);
    assert_ne!(entry["span"]["start"], entry["span"]["byteStart"]);
    assert_eq!(
        entry["identity"]["declarationContentId"],
        formal_ai::source_fetch::sha256_hex(observed.as_bytes())
    );
    assert_eq!(
        entry["contract"]["result"],
        json!({"kind":"parameter","name":"input"})
    );
}
#[test]
fn supported_optional_graph_is_not_authored_source_or_a_goal_proof() {
    let first = binding("producer.mjs", "publicChoice", FIRST);
    let second = binding("consumer.mjs", "render", SECOND);
    let graph = observed_guarded_graph(&observations(), &first, &second);
    assert_eq!(graph["kind"], "guarded-call-graph");
    assert_eq!(graph["guard"], json!({"kind":"nonnull","call":0}));
    assert_eq!(
        graph["result"],
        json!({"kind":"optional","value":{"kind":"boolean"}})
    );
    assert_eq!(graph["authored"], false);
    assert!(graph.get("source").is_none());
    assert_eq!(
        graph["bindings"][0],
        json!({"path":"producer.mjs","exported":"publicChoice"})
    );
    assert!(!observed_callable_graphs(&observations()).is_empty());
    let mut changed = observations();
    changed[0]["content"] = json!(format!("{FIRST}\n"));
    assert_eq!(
        observed_guarded_graph(&changed, &first, &second)["reason"],
        "SourceChanged"
    );
}
#[test]
fn unsupported_bodies_and_import_effects_remain_honest_contract_gaps() {
    for (source, reason) in [
        (
            "export function f(x) { return external(x); }",
            "UnobservedCallEffect",
        ),
        (
            "export function f(x) { return x.value; }",
            "MissingStructuralSchema",
        ),
        (
            "export function f(x) { x.value = 3; return x; }",
            "UnsupportedStatement",
        ),
        (
            "export function f(x) { return\n x; }",
            "ReturnLineTerminator",
        ),
        (
            "export function f(x) { return x === null ? true : 3; }",
            "IncompatibleBranches",
        ),
        ("export async function f(x) { return x; }", "AsyncResult"),
    ] {
        let catalog = observe_source_callables(source, "source.mjs");
        assert_eq!(
            catalog["declarations"][0]["contract"]["gap"]["reason"],
            reason
        );
        assert_eq!(catalog["declarations"][0]["contract"]["status"], "unknown");
    }
    let facade =
        "import { render as alias } from './consumer.mjs'; export { alias as publicRender };";
    let mut data = observations();
    data.push(json!({"path":"facade.mjs","content":facade}));
    assert_eq!(
        observed_guarded_graph(
            &data,
            &binding("producer.mjs", "publicChoice", FIRST),
            &binding("facade.mjs", "publicRender", facade)
        )["reason"],
        "MissingContract"
    );
    assert_eq!(
        observe_source_callables("export function f(x) {", "source.mjs")["gaps"][0]["reason"],
        "LexicalFailure"
    );
}
