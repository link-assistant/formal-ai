// Standalone validation of the pure (non-walkdir) logic in
// scripts/check-associative-terminology.rs.

const API_ROUTE_PREFIXES: &[&str] = &["/v1/", "/api/formal-ai/v1/"];
const ROUTE_ALIAS_ALLOWLIST: &[&str] = &["/v1/graph", "/api/formal-ai/v1/graph"];
const MODULE_ALLOWLIST: &[&str] = &["knowledge_graph"];
const CITATION_HOSTS: &[&str] = &["codecov.io", "semanticscholar.org"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct RouteViolation { file: String, line: usize, route: String }
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleViolation { file: String, line: usize, name: String, kind: &'static str }

fn identifier_is_graph_named(name: &str) -> bool {
    name.split(['-', '_', '.']).any(|p| p.eq_ignore_ascii_case("graph"))
}
fn line_mentions_citation(line: &str) -> bool {
    CITATION_HOSTS.iter().any(|h| line.contains(h))
}
fn leading_route_segment(s: &str) -> &str {
    let end = s.find(|c: char| !(c.is_ascii_alphanumeric() || c=='.'||c=='_'||c=='-')).unwrap_or(s.len());
    &s[..end]
}
fn collect_route_violations(file: &str, content: &str, out: &mut Vec<RouteViolation>) {
    for (index, line) in content.lines().enumerate() {
        if line_mentions_citation(line) { continue; }
        for prefix in API_ROUTE_PREFIXES {
            let mut start = 0;
            while let Some(rel) = line[start..].find(prefix) {
                let prefix_at = start + rel;
                let segment_at = prefix_at + prefix.len();
                if *prefix == "/v1/" && line[..prefix_at].ends_with("formal-ai") { start = segment_at; continue; }
                let segment = leading_route_segment(&line[segment_at..]);
                if identifier_is_graph_named(segment) {
                    let route = format!("{prefix}{segment}");
                    if !ROUTE_ALIAS_ALLOWLIST.contains(&route.as_str()) {
                        let v = RouteViolation{file:file.to_string(), line:index+1, route};
                        if !out.contains(&v) { out.push(v); }
                    }
                }
                start = segment_at;
            }
        }
    }
}
fn collect_module_declaration_violations(file:&str, content:&str, out:&mut Vec<ModuleViolation>) {
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let declaration = trimmed.strip_prefix("pub(crate) ").or_else(|| trimmed.strip_prefix("pub ")).unwrap_or(trimmed);
        let Some(rest) = declaration.strip_prefix("mod ") else { continue; };
        let name = rest.split(|c: char| c==';'||c=='{'||c.is_whitespace()).next().unwrap_or("");
        if name.is_empty() { continue; }
        if identifier_is_graph_named(name) && !MODULE_ALLOWLIST.contains(&name) {
            out.push(ModuleViolation{file:file.to_string(), line:index+1, name:name.to_string(), kind:"module declaration"});
        }
    }
}

fn main() {
    // graph-word classification
    for t in ["graph","knowledge-graph","source_graph","v1.graph"] { assert!(identifier_is_graph_named(t), "{t}"); }
    for f in ["graphql","ideographic","paragraph","network","self_source_links"] { assert!(!identifier_is_graph_named(f), "{f}"); }

    // alias allowed
    let mut r = Vec::new();
    collect_route_violations("s", "(\"GET\", \"/v1/graph\" | \"/api/formal-ai/v1/graph\") => h,", &mut r);
    assert!(r.is_empty(), "alias {r:?}");

    // network allowed
    let mut r = Vec::new();
    collect_route_violations("s", "\"/v1/network\" \"/api/formal-ai/v1/network\"", &mut r);
    assert!(r.is_empty(), "network {r:?}");

    // fixture flagged
    let mut r = Vec::new();
    collect_route_violations("s", "\"/v1/knowledge-graph\"", &mut r);
    assert_eq!(r, vec![RouteViolation{file:"s".into(), line:1, route:"/v1/knowledge-graph".into()}]);

    // fully-qualified flagged (single, canonical form)
    let mut r = Vec::new();
    collect_route_violations("s", "\"/api/formal-ai/v1/knowledge-graph\"", &mut r);
    assert_eq!(r, vec![RouteViolation{file:"s".into(), line:1, route:"/api/formal-ai/v1/knowledge-graph".into()}], "fq {r:?}");

    // citations exempt
    let mut r = Vec::new();
    collect_route_violations("R", "https://codecov.io/gh/link-assistant/formal-ai/branch/main/graph/badge.svg", &mut r);
    collect_route_violations("c", "https://api.semanticscholar.org/graph/v1/paper/search?query=x", &mut r);
    collect_route_violations("d", "see codecov.io mirror at /v1/knowledge-graph", &mut r);
    assert!(r.is_empty(), "citations {r:?}");

    // module decls
    let mut m = Vec::new();
    collect_module_declaration_violations("lib", "pub mod source_graph;\nmod knowledge_graph;\nmod source_links;\n", &mut m);
    assert_eq!(m, vec![ModuleViolation{file:"lib".into(), line:1, name:"source_graph".into(), kind:"module declaration"}], "mods {m:?}");

    println!("ALL LOGIC CHECKS PASSED");
}
