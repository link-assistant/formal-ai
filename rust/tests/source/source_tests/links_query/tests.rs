use super::*;

#[test]
fn node_only_query_returns_all_nodes() {
    let result = run_links_query("MATCH (a) RETURN a").unwrap();
    let graph = knowledge_graph();
    assert_eq!(result.nodes.len(), graph.nodes.len());
    assert!(result.edges.is_empty());
}

#[test]
fn role_filtered_edges_match_graph_projection() {
    // Acceptance (ROADMAP D3): a LinksQL query is consistent with the
    // engine — filtering edges by role returns exactly the /v1/graph edges
    // of that role.
    let graph = knowledge_graph();
    let result = run_links_query("MATCH (a)-[:contains]->(b) RETURN a, b").unwrap();
    let expected = graph
        .edges
        .iter()
        .filter(|edge| edge.role == "contains")
        .count();
    assert_eq!(result.edges.len(), expected);
    assert!(result.edges.iter().all(|edge| edge.role == "contains"));
}

#[test]
fn where_id_filter_selects_one_node() {
    let result = run_links_query("MATCH (a) WHERE a.id = \"rule_greeting\" RETURN a").unwrap();
    assert_eq!(result.nodes.len(), 1);
    assert_eq!(result.nodes[0].id, "rule_greeting");
}

#[test]
fn where_label_contains_filter() {
    let result = run_links_query("MATCH (a) WHERE a.label CONTAINS \"rule\" RETURN a").unwrap();
    assert!(!result.nodes.is_empty());
    assert!(result.nodes.iter().all(|node| node.label.contains("rule")));
}

#[test]
fn all_edges_query_returns_every_edge() {
    let graph = knowledge_graph();
    let result = run_links_query("MATCH (a)-[r]->(b) RETURN a, r, b").unwrap();
    assert_eq!(result.edges.len(), graph.edges.len());
}

#[test]
fn links_notation_envelope_round_trips_structure() {
    let result = run_links_query("MATCH (a)-[:contains]->(b) RETURN a, b").unwrap();
    let lino = result.to_links_notation();
    assert!(lino.starts_with("links_query_result"));
    assert!(lino.contains("  edge\n"));
    assert!(lino.contains("role \"contains\""));
}

#[test]
fn where_role_value_colliding_with_operator_parses() {
    // Regression: a quoted value of "contains" must not be read as the
    // CONTAINS operator. The role filter should match the graph projection.
    let graph = knowledge_graph();
    let result =
        run_links_query("MATCH (a)-[r]->(b) WHERE r.role = \"contains\" RETURN a, r, b").unwrap();
    let expected = graph
        .edges
        .iter()
        .filter(|edge| edge.role == "contains")
        .count();
    assert_eq!(result.edges.len(), expected);
    assert!(result.edges.iter().all(|edge| edge.role == "contains"));
}

#[test]
fn missing_return_is_an_error() {
    assert!(run_links_query("MATCH (a)").is_err());
}

#[test]
fn missing_match_is_an_error() {
    assert!(run_links_query("RETURN a").is_err());
}
