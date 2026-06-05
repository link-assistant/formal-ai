//! LinksQL — a read-only, GraphQL-flavoured query language over the knowledge
//! link store, plus a Links-Notation REST envelope (`lino-rest-api` style).
//!
//! Issue #347 / R6 asks, "ideally", for two Links-native interfaces alongside
//! the OpenAI REST surface:
//!
//! 1. a [`lino-rest-api`](https://github.com/link-foundation/lino-rest-api)-style
//!    layer that speaks **Links Notation** envelopes (not JSON), and
//! 2. a universal **LinksQL** query language extending the idea behind
//!    [`link-cli`](https://github.com/link-foundation/link-cli) with GraphQL-like
//!    field selection.
//!
//! R7 constrains the *external* REST surface to OpenAI-compatible only, so this
//! is an **internal/adjacent** Links-Notation channel: the query is a string, the
//! request/response envelope is Links Notation, and the same projection JSON the
//! `/v1/graph` endpoint already serves is offered for tooling that wants it.
//!
//! ## Grammar (read-only)
//!
//! ```text
//! MATCH (a)                         RETURN a            # every node
//! MATCH (a)-[:contains]->(b)        RETURN a, b         # edges of one role
//! MATCH (a)-[r]->(b)                RETURN a, r, b      # every edge
//! MATCH (a)-[r:response_link]->(b)  RETURN a, b
//! MATCH (a) WHERE a.id = "rule_greeting"        RETURN a
//! MATCH (a) WHERE a.label CONTAINS "rule"       RETURN a
//! ```
//!
//! The evaluator runs against [`crate::engine::knowledge_graph`], so a LinksQL
//! query is provably consistent with the engine: filtering all edges by a role
//! returns exactly the `/v1/graph` edges of that role (see the tests).

use serde::Serialize;

use crate::engine::{knowledge_graph, GraphEdge, GraphNode, KnowledgeGraph};

/// A parsed LinksQL query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinksQuery {
    pub source: NodePattern,
    pub edge: Option<EdgePattern>,
    pub target: Option<NodePattern>,
    pub filters: Vec<Filter>,
    pub returns: Vec<String>,
}

/// A node pattern such as `(a)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePattern {
    pub var: String,
}

/// An edge pattern such as `-[r:role]->`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgePattern {
    pub var: Option<String>,
    pub role: Option<String>,
}

/// A `WHERE` filter such as `a.id = "rule_greeting"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub var: String,
    pub field: Field,
    pub op: FilterOp,
    pub value: String,
}

/// The field a filter targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Id,
    Label,
    Role,
}

/// The comparison a filter applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Contains,
}

/// A parse or evaluation failure with a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinksQueryError {
    pub message: String,
}

impl std::fmt::Display for LinksQueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for LinksQueryError {}

fn err(message: impl Into<String>) -> LinksQueryError {
    LinksQueryError {
        message: message.into(),
    }
}

/// The nodes and edges a query selected, plus the originating query text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinksQueryResult {
    pub query: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl LinksQueryResult {
    /// Render the result as a Links-Notation envelope (the `lino-rest-api`
    /// response shape). This is the preferred internal transport per R7.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::from("links_query_result\n");
        out.push_str("  query \"");
        out.push_str(&escape(&self.query));
        out.push_str("\"\n");
        for node in &self.nodes {
            out.push_str("  node \"");
            out.push_str(&escape(&node.id));
            out.push_str("\"\n    label \"");
            out.push_str(&escape(&node.label));
            out.push_str("\"\n    links_notation \"");
            out.push_str(&escape(&node.links_notation));
            out.push_str("\"\n");
        }
        for edge in &self.edges {
            out.push_str("  edge\n    from \"");
            out.push_str(&escape(&edge.from));
            out.push_str("\"\n    to \"");
            out.push_str(&escape(&edge.to));
            out.push_str("\"\n    role \"");
            out.push_str(&escape(&edge.role));
            out.push_str("\"\n");
        }
        out
    }
}

impl KnowledgeGraph {
    /// Render the graph as a `knowledge_graph` Links-Notation document — the
    /// Links-native projection served from `/v1/links` (R6/R7). It carries the
    /// same nodes and edges as the JSON `/v1/graph` view, only in Links
    /// Notation, the project's preferred internal format.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::from("knowledge_graph\n");
        for node in &self.nodes {
            out.push_str("  node \"");
            out.push_str(&escape(&node.id));
            out.push_str("\"\n    label \"");
            out.push_str(&escape(&node.label));
            out.push_str("\"\n    links_notation \"");
            out.push_str(&escape(&node.links_notation));
            out.push_str("\"\n");
        }
        for edge in &self.edges {
            out.push_str("  edge\n    from \"");
            out.push_str(&escape(&edge.from));
            out.push_str("\"\n    to \"");
            out.push_str(&escape(&edge.to));
            out.push_str("\"\n    role \"");
            out.push_str(&escape(&edge.role));
            out.push_str("\"\n");
        }
        out
    }
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Parse and evaluate a LinksQL query against the engine's knowledge graph.
///
/// # Errors
/// Returns [`LinksQueryError`] when the query cannot be parsed.
pub fn run_links_query(query: &str) -> Result<LinksQueryResult, LinksQueryError> {
    run_links_query_against(query, &knowledge_graph())
}

/// Evaluate a query against an explicit graph (used by tests and tooling).
///
/// # Errors
/// Returns [`LinksQueryError`] when the query cannot be parsed.
pub fn run_links_query_against(
    query: &str,
    graph: &KnowledgeGraph,
) -> Result<LinksQueryResult, LinksQueryError> {
    let parsed = parse_links_query(query)?;
    Ok(evaluate(&parsed, query, graph))
}

/// Parse a LinksQL query string into a [`LinksQuery`].
///
/// # Errors
/// Returns [`LinksQueryError`] when the `MATCH` / `RETURN` structure is invalid.
pub fn parse_links_query(query: &str) -> Result<LinksQuery, LinksQueryError> {
    let trimmed = query.trim();
    let upper = trimmed.to_uppercase();
    let match_start = upper
        .find("MATCH")
        .ok_or_else(|| err("query must contain a MATCH clause"))?;
    let return_start = upper
        .find("RETURN")
        .ok_or_else(|| err("query must contain a RETURN clause"))?;
    if return_start < match_start {
        return Err(err("RETURN must follow MATCH"));
    }

    let after_match = &trimmed[match_start + "MATCH".len()..return_start];
    let returns_text = &trimmed[return_start + "RETURN".len()..];

    let (pattern_text, where_text) =
        upper[match_start..return_start]
            .find("WHERE")
            .map_or((after_match, None), |relative| {
                let absolute = match_start + relative;
                (
                    &trimmed[match_start + "MATCH".len()..absolute],
                    Some(&trimmed[absolute + "WHERE".len()..return_start]),
                )
            });

    let (source, edge, target) = parse_pattern(pattern_text.trim())?;
    let filters = match where_text {
        Some(text) => parse_filters(text.trim())?,
        None => Vec::new(),
    };
    let returns = returns_text
        .split(',')
        .map(|item| {
            item.trim()
                .split('.')
                .next()
                .unwrap_or("")
                .trim()
                .to_owned()
        })
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if returns.is_empty() {
        return Err(err("RETURN must name at least one variable"));
    }

    Ok(LinksQuery {
        source,
        edge,
        target,
        filters,
        returns,
    })
}

fn parse_pattern(
    text: &str,
) -> Result<(NodePattern, Option<EdgePattern>, Option<NodePattern>), LinksQueryError> {
    let source_end = text
        .find(')')
        .ok_or_else(|| err("pattern must open with a node like (a)"))?;
    let source = parse_node(&text[..=source_end])?;
    let rest = text[source_end + 1..].trim();
    if rest.is_empty() {
        return Ok((source, None, None));
    }

    // Expect an edge `-[...]->` followed by a target node `(b)`.
    let arrow = "->";
    let arrow_pos = rest
        .find(arrow)
        .ok_or_else(|| err("edge pattern must end with ->"))?;
    let edge_text = rest[..arrow_pos].trim();
    let target_text = rest[arrow_pos + arrow.len()..].trim();
    let edge = parse_edge(edge_text)?;
    let target = parse_node(target_text)?;
    Ok((source, Some(edge), Some(target)))
}

fn parse_node(text: &str) -> Result<NodePattern, LinksQueryError> {
    let inner = text
        .trim()
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| err(format!("invalid node pattern: {text}")))?;
    let var = inner.split(':').next().unwrap_or("").trim().to_owned();
    if var.is_empty() {
        return Err(err("node pattern needs a variable name"));
    }
    Ok(NodePattern { var })
}

fn parse_edge(text: &str) -> Result<EdgePattern, LinksQueryError> {
    let inner = text
        .trim()
        .strip_prefix('-')
        .unwrap_or(text)
        .trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| err(format!("invalid edge pattern: {text}")))?;
    let inner = inner.trim();
    let (var_text, role_text) = if let Some((var, role)) = inner.split_once(':') {
        (var.trim(), Some(role.trim()))
    } else {
        (inner, None)
    };
    let var = (!var_text.is_empty()).then(|| var_text.to_owned());
    let role = role_text.and_then(|role| (!role.is_empty()).then(|| role.to_owned()));
    Ok(EdgePattern { var, role })
}

fn parse_filters(text: &str) -> Result<Vec<Filter>, LinksQueryError> {
    let mut filters = Vec::new();
    for clause in split_filters(text) {
        let clause = clause.trim();
        if clause.is_empty() {
            continue;
        }
        filters.push(parse_filter(clause)?);
    }
    Ok(filters)
}

fn split_filters(text: &str) -> Vec<String> {
    // Split on AND / and, but never inside a quoted value.
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
            index += 1;
            continue;
        }
        if !in_quotes {
            let remaining: String = chars[index..].iter().collect();
            let upper = remaining.to_uppercase();
            if upper.starts_with("AND") && is_boundary(chars.get(index + 3).copied()) {
                parts.push(std::mem::take(&mut current));
                index += 3;
                continue;
            }
        }
        current.push(ch);
        index += 1;
    }
    parts.push(current);
    parts
}

fn is_boundary(ch: Option<char>) -> bool {
    ch.map_or(true, char::is_whitespace)
}

fn parse_filter(clause: &str) -> Result<Filter, LinksQueryError> {
    // Locate the operator outside of any quoted value, so a value such as
    // `"contains"` is never mistaken for the CONTAINS operator and a `=` inside
    // a quoted string never splits the clause.
    let (op, split_at, op_len) = find_operator(clause)
        .ok_or_else(|| err(format!("filter needs `=` or CONTAINS: {clause}")))?;
    let lhs = clause[..split_at].trim();
    let rhs = clause[split_at + op_len..].trim();

    let (var, field_text) = lhs
        .split_once('.')
        .ok_or_else(|| err(format!("filter target must be `var.field`: {clause}")))?;
    let field = match field_text.trim().to_lowercase().as_str() {
        "id" => Field::Id,
        "label" => Field::Label,
        "role" => Field::Role,
        other => return Err(err(format!("unknown field `{other}`"))),
    };
    let value = rhs.trim().trim_matches('"').to_owned();
    Ok(Filter {
        var: var.trim().to_owned(),
        field,
        op,
        value,
    })
}

/// Find the filter operator (`=` or `CONTAINS`) outside any quoted region.
///
/// Returns the operator, its byte offset, and its byte length. Scanning the
/// unquoted spans only means a quoted value such as `"contains"` is never read
/// as the CONTAINS keyword.
fn find_operator(clause: &str) -> Option<(FilterOp, usize, usize)> {
    let bytes = clause.as_bytes();
    let upper = clause.to_uppercase();
    let upper_bytes = upper.as_bytes();
    let mut in_quotes = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' {
            in_quotes = !in_quotes;
            index += 1;
            continue;
        }
        if !in_quotes {
            if byte == b'=' {
                return Some((FilterOp::Eq, index, 1));
            }
            if upper_bytes[index..].starts_with(b"CONTAINS") {
                return Some((FilterOp::Contains, index, "CONTAINS".len()));
            }
        }
        index += 1;
    }
    None
}

fn evaluate(query: &LinksQuery, source_text: &str, graph: &KnowledgeGraph) -> LinksQueryResult {
    let mut result = LinksQueryResult {
        query: source_text.trim().to_owned(),
        nodes: Vec::new(),
        edges: Vec::new(),
    };

    if query.edge.is_none() {
        // Node-only query: filter all nodes by the filters that target the
        // source variable.
        for node in &graph.nodes {
            if node_matches(node, &query.source.var, &query.filters) {
                push_node(&mut result.nodes, node.clone());
            }
        }
        return result;
    }

    let edge_pattern = query.edge.as_ref().expect("edge present");
    let target = query.target.as_ref().expect("target present");
    let want_source = query.returns.iter().any(|var| var == &query.source.var);
    let want_target = query.returns.iter().any(|var| var == &target.var);

    for edge in &graph.edges {
        if let Some(role) = &edge_pattern.role {
            if &edge.role != role {
                continue;
            }
        }
        let from_node = graph.nodes.iter().find(|node| node.id == edge.from);
        let to_node = graph.nodes.iter().find(|node| node.id == edge.to);

        if !node_filters_match(from_node, &query.source.var, &query.filters)
            || !node_filters_match(to_node, &target.var, &query.filters)
            || !edge_filters_match(edge, edge_pattern.var.as_deref(), &query.filters)
        {
            continue;
        }

        result.edges.push(edge.clone());
        if want_source {
            if let Some(node) = from_node {
                push_node(&mut result.nodes, node.clone());
            }
        }
        if want_target {
            if let Some(node) = to_node {
                push_node(&mut result.nodes, node.clone());
            }
        }
    }
    result
}

fn push_node(nodes: &mut Vec<GraphNode>, node: GraphNode) {
    if !nodes.iter().any(|existing| existing.id == node.id) {
        nodes.push(node);
    }
}

fn node_matches(node: &GraphNode, var: &str, filters: &[Filter]) -> bool {
    filters
        .iter()
        .filter(|filter| filter.var == var)
        .all(|filter| apply_node_filter(node, filter))
}

fn node_filters_match(node: Option<&GraphNode>, var: &str, filters: &[Filter]) -> bool {
    let relevant: Vec<&Filter> = filters
        .iter()
        .filter(|filter| filter.var == var && !matches!(filter.field, Field::Role))
        .collect();
    if relevant.is_empty() {
        return true;
    }
    let Some(node) = node else {
        return false;
    };
    relevant
        .iter()
        .all(|filter| apply_node_filter(node, filter))
}

fn edge_filters_match(edge: &GraphEdge, var: Option<&str>, filters: &[Filter]) -> bool {
    let Some(var) = var else {
        return !filters
            .iter()
            .any(|filter| matches!(filter.field, Field::Role) && filter.var != "__none__");
    };
    filters
        .iter()
        .filter(|filter| filter.var == var)
        .all(|filter| match filter.op {
            FilterOp::Eq => edge.role == filter.value,
            FilterOp::Contains => edge.role.contains(&filter.value),
        })
}

fn apply_node_filter(node: &GraphNode, filter: &Filter) -> bool {
    let haystack = match filter.field {
        Field::Id => &node.id,
        Field::Label => &node.label,
        Field::Role => return true,
    };
    match filter.op {
        FilterOp::Eq => haystack == &filter.value,
        FilterOp::Contains => haystack.contains(&filter.value),
    }
}

#[path = "source_tests/links_query/tests.rs"]
mod tests;
