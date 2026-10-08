//! Generated system diagrams (issue #538, R382).
//!
//! A high-level overview and one view per entry point, rendered as split
//! mermaid parts from live data only: the solver handler registry from the
//! seed's precedence and promotion tables, the CLI subcommands from the clap
//! `Command` enum in `rust/src/main.rs`, and the HTTP routes from
//! `data/meta/server-routes.lino`, the manifest both servers are held to. The
//! wording and the overview's edges live in `data/meta/system-diagrams.lino`,
//! and every edge naming a subcommand or a route is checked against the live
//! data, so a vanished subcommand or route fails the renderer instead of
//! leaving a stale arrow.
//!
//! JavaScript twin: `js/agentic/system_diagram.mjs`, driven by
//! `scripts/generate-system-diagrams.mjs` (`--write`, and `--check` in CI).
//! `rust/tests/unit/system_diagrams.rs` pins this renderer byte-for-byte to the
//! committed `docs/diagrams/*.md` parts.

use crate::seed::parser::{LinoNode, parse_lino};

/// Repository path of the diagram wording and overview edges.
pub const CONFIG_PATH: &str = "data/meta/system-diagrams.lino";

/// Repository directory the generated parts are committed under.
pub const DIAGRAM_DIR: &str = "docs/diagrams";

/// The mermaid class of `browser_only` precedence rows.
const BROWSER_CLASS: &str = "browserOnly";

/// The mermaid style of that class: a dashed outline.
const BROWSER_STYLE: &str = "stroke-dasharray: 5 5";

const FENCE: &str = "```";

/// One `handler` row of the precedence seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerRow {
    /// The handler name.
    pub name: String,
    /// Its dispatch rank (lower is tried first).
    pub rank: u32,
    /// Whether only the browser worker runs it.
    pub browser_only: bool,
}

/// One `promotion` row of the promotions seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRow {
    /// The handler the rule hoists.
    pub handler: String,
    /// The rule's rank among the promotions.
    pub rank: u32,
    /// Why the rule exists.
    pub because: String,
}

/// One subcommand of the clap `Command` enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliSubcommand {
    /// The kebab-case subcommand name clap derives from the variant.
    pub name: String,
    /// The first sentence of the variant's doc comment, or empty.
    pub summary: String,
}

/// The CLI's binary name and its subcommands in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliDefinition {
    /// The `name = "..."` of the clap `#[command]` attribute.
    pub binary: String,
    /// The subcommands, in the order the enum declares them.
    pub subcommands: Vec<CliSubcommand>,
}

/// One route of the server route manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRow {
    /// The route id.
    pub id: String,
    /// The HTTP method.
    pub method: String,
    /// Every path the route serves.
    pub paths: Vec<String>,
    /// `bearer` or `none`.
    pub auth: String,
    /// Whether the manifest marks the route deprecated.
    pub deprecated: bool,
}

/// The first top-level node of a Links Notation document.
fn root_of(text: &str) -> LinoNode {
    parse_lino(text)
        .children
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// The children of `node` named `name`.
fn children_named<'a>(node: &'a LinoNode, name: &'a str) -> impl Iterator<Item = &'a LinoNode> {
    node.children.iter().filter(move |child| child.name == name)
}

/// The value of the first child of `node` named `name`, or `""`.
fn child_value<'a>(node: &'a LinoNode, name: &str) -> &'a str {
    node.find_child_value(name)
}

/// A rank value; a missing or malformed rank reads as zero.
fn rank_of(node: &LinoNode) -> u32 {
    child_value(node, "rank").parse().unwrap_or_default()
}

/// `{name}` placeholders of `template` filled from `params`; unknown ones stay.
fn fill(template: &str, params: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let name_len = after
            .find(|character: char| !(character.is_ascii_lowercase() || character == '_'))
            .unwrap_or(after.len());
        let value = (name_len > 0 && after[name_len..].starts_with('}'))
            .then(|| params.iter().find(|(key, _)| *key == &after[..name_len]))
            .flatten();
        if let Some((_, value)) = value {
            out.push_str(value);
            rest = &after[name_len + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// A renderer error: `system_diagrams:<parts…>`.
fn config_error(parts: &[&str]) -> String {
    let mut out = String::from("system_diagrams");
    for part in parts {
        out.push(':');
        out.push_str(part);
    }
    out
}

/// A Markdown table cell, `|` escaped.
fn cell(text: &str) -> String {
    text.replace('|', "\\|")
}

/// A Markdown table.
fn table(headers: &[String], rows: &[Vec<String>]) -> String {
    let line = |cells: &[String]| format!("| {} |", cells.join(" | "));
    let mut out = vec![
        line(headers),
        line(
            &headers
                .iter()
                .map(|_| String::from("---"))
                .collect::<Vec<_>>(),
        ),
    ];
    for row in rows {
        out.push(line(&row.iter().map(|text| cell(text)).collect::<Vec<_>>()));
    }
    out.join("\n")
}

/// A fenced mermaid block.
fn mermaid(lines: &[String]) -> String {
    let mut out = vec![format!("{FENCE}mermaid")];
    out.extend(lines.iter().cloned());
    out.push(FENCE.to_owned());
    out.join("\n")
}

/// The `handler` rows of a precedence document in dispatch order: ascending
/// rank, then name, as the seed loader orders them.
#[must_use]
pub fn precedence_rows(text: &str) -> Vec<HandlerRow> {
    let root = root_of(text);
    let mut rows: Vec<HandlerRow> = children_named(&root, "handler")
        .map(|node| HandlerRow {
            name: node.id.clone(),
            rank: rank_of(node),
            browser_only: child_value(node, "browser-only") == "true",
        })
        .collect();
    rows.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.name.cmp(&right.name))
    });
    rows
}

/// The promotion rows of a promotions document in rank order (stable).
#[must_use]
pub fn promotion_rows(text: &str) -> Vec<PromotionRow> {
    let root = root_of(text);
    let mut rows: Vec<PromotionRow> = children_named(&root, "promotion")
        .map(|node| PromotionRow {
            handler: node.id.clone(),
            rank: rank_of(node),
            because: child_value(node, "because").trim().to_owned(),
        })
        .collect();
    rows.sort_by_key(|row| row.rank);
    rows
}

/// A clap variant name as the kebab-case subcommand name clap derives.
fn kebab_case(identifier: &str) -> String {
    let mut out = String::with_capacity(identifier.len() + 4);
    for (index, character) in identifier.chars().enumerate() {
        if index > 0 && character.is_ascii_uppercase() {
            out.push('-');
        }
        out.push(character.to_ascii_lowercase());
    }
    out
}

/// The first sentence of a doc comment's first paragraph.
fn summary(doc: &[String]) -> String {
    let end = doc.iter().position(String::is_empty).unwrap_or(doc.len());
    let paragraph = doc[..end].join(" ");
    let sentence = paragraph
        .find(". ")
        .map_or(paragraph.as_str(), |stop| &paragraph[..=stop]);
    sentence.trim().to_owned()
}

/// The binary name and the subcommands of the clap `Command` enum in
/// `rust/src/main.rs`, in declaration order, each with its doc summary.
#[must_use]
pub fn cli_definition(source: &str) -> CliDefinition {
    let lines: Vec<&str> = source.split('\n').collect();
    let binary = lines
        .iter()
        .find_map(|line| line.trim().strip_prefix("name = \""))
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_default()
        .to_owned();
    let start = lines
        .iter()
        .position(|line| line.trim() == "enum Command {");
    let mut subcommands = Vec::new();
    let mut doc: Vec<String> = Vec::new();
    for line in start.map_or(&[][..], |start| &lines[start + 1..]) {
        if *line == "}" {
            break;
        }
        if let Some(text) = line.strip_prefix("    ///") {
            doc.push(text.trim().to_owned());
        } else if line
            .strip_prefix("    ")
            .and_then(|rest| rest.as_bytes().first())
            .is_some_and(u8::is_ascii_uppercase)
        {
            let identifier: String = line[4..]
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect();
            subcommands.push(CliSubcommand {
                name: kebab_case(&identifier),
                summary: summary(&doc),
            });
            doc.clear();
        }
    }
    CliDefinition {
        binary,
        subcommands,
    }
}

/// The route manifest's rows in manifest order.
#[must_use]
pub fn route_rows(text: &str) -> Vec<RouteRow> {
    let root = root_of(text);
    children_named(&root, "route")
        .map(|node| {
            let auth = child_value(node, "auth");
            RouteRow {
                id: node.id.clone(),
                method: child_value(node, "method").to_owned(),
                paths: children_named(node, "path")
                    .map(|path| path.id.clone())
                    .collect(),
                auth: if auth.is_empty() { "bearer" } else { auth }.to_owned(),
                deprecated: child_value(node, "deprecated") == "true",
            }
        })
        .collect()
}

/// One `source` of the config: a key and a repository path.
struct Source {
    key: String,
    path: String,
}

/// One `part` of the config: a renderer key, its file and the sources it reads.
struct Part {
    key: String,
    file: String,
    uses: Vec<String>,
}

/// One overview `node`.
struct Node {
    id: String,
    kind: String,
    label: String,
    detail: String,
    count: String,
}

/// One overview `edge`, optionally naming a live subcommand or route.
struct Edge {
    id: String,
    from: String,
    to: String,
    subcommand: String,
    route: String,
}

/// The parsed `data/meta/system-diagrams.lino`.
struct Config {
    chunk: usize,
    sources: Vec<Source>,
    parts: Vec<Part>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    messages: Vec<(String, String)>,
}

impl Config {
    fn parse(text: &str) -> Self {
        let root = root_of(text);
        let value = |node: &LinoNode, name: &str| child_value(node, name).to_owned();
        Self {
            chunk: child_value(&root, "chunk").parse().unwrap_or(1).max(1),
            sources: children_named(&root, "source")
                .map(|node| Source {
                    key: node.id.clone(),
                    path: value(node, "path"),
                })
                .collect(),
            parts: children_named(&root, "part")
                .map(|node| Part {
                    key: node.id.clone(),
                    file: value(node, "file"),
                    uses: children_named(node, "uses")
                        .map(|used| used.id.clone())
                        .collect(),
                })
                .collect(),
            nodes: children_named(&root, "node")
                .map(|node| Node {
                    id: node.id.clone(),
                    kind: value(node, "kind"),
                    label: value(node, "label"),
                    detail: value(node, "detail"),
                    count: value(node, "count"),
                })
                .collect(),
            edges: children_named(&root, "edge")
                .map(|node| Edge {
                    id: node.id.clone(),
                    from: value(node, "from"),
                    to: value(node, "to"),
                    subcommand: value(node, "subcommand"),
                    route: value(node, "route"),
                })
                .collect(),
            messages: children_named(&root, "message")
                .map(|node| (node.id.clone(), value(node, "text")))
                .collect(),
        }
    }

    fn message(&self, key: &str, params: &[(&str, String)]) -> Result<String, String> {
        self.messages
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, text)| fill(text, params))
            .ok_or_else(|| config_error(&["missing_message", key]))
    }

    fn text(&self, key: &str) -> Result<String, String> {
        self.message(key, &[])
    }

    fn source_path(&self, key: &str) -> Option<&str> {
        self.sources
            .iter()
            .find(|source| source.key == key)
            .map(|source| source.path.as_str())
    }
}

/// The live data every part is drawn from.
struct Data {
    handlers: Vec<HandlerRow>,
    promotions: Vec<PromotionRow>,
    cli: CliDefinition,
    routes: Vec<RouteRow>,
}

/// The generated header (title and provenance comment), then the blocks.
fn document(
    config: &Config,
    part: &Part,
    title_key: &str,
    blocks: Vec<String>,
) -> Result<String, String> {
    let mut sources = Vec::with_capacity(part.uses.len());
    for key in &part.uses {
        let path = config
            .source_path(key)
            .ok_or_else(|| config_error(&["part", &part.key, "unknown_source", key]))?;
        sources.push(path);
    }
    let mut all = vec![
        config.text(title_key)?,
        config.message("document_comment", &[("sources", sources.join(", "))])?,
    ];
    all.extend(blocks);
    Ok(format!("{}\n", all.join("\n\n")))
}

/// The live count an overview node shows.
fn count_of(data: &Data, kind: &str) -> Result<usize, String> {
    match kind {
        "cli_subcommands" => Ok(data.cli.subcommands.len()),
        "routes" => Ok(data.routes.len()),
        "browser_handlers" => Ok(data.handlers.iter().filter(|row| row.browser_only).count()),
        "promotions" => Ok(data.promotions.len()),
        "handlers" => Ok(data.handlers.len()),
        _ => Err(config_error(&["unknown_count", kind])),
    }
}

/// The live subcommand or route an edge names, as its label, or `""`.
fn edge_label(data: &Data, edge: &Edge) -> Result<String, String> {
    if !edge.subcommand.is_empty() {
        if !data
            .cli
            .subcommands
            .iter()
            .any(|sub| sub.name == edge.subcommand)
        {
            return Err(config_error(&[
                "edge",
                &edge.id,
                "unknown_subcommand",
                &edge.subcommand,
            ]));
        }
        return Ok(format!("{} {}", data.cli.binary, edge.subcommand));
    }
    if !edge.route.is_empty() {
        let route = data
            .routes
            .iter()
            .find(|row| row.id == edge.route)
            .ok_or_else(|| config_error(&["edge", &edge.id, "unknown_route", &edge.route]))?;
        let first = route.paths.first().map_or("", String::as_str);
        return Ok(format!("{} {first}", route.method));
    }
    Ok(String::new())
}

/// Part: entry points to layers.
fn render_overview(config: &Config, data: &Data, part: &Part) -> Result<String, String> {
    let mut intro = vec![config.text("overview_intro")?];
    for other in config.parts.iter().filter(|other| other.key != part.key) {
        let title = config.text(&format!("{}_title", other.key))?;
        intro.push(config.message(
            "overview_part_link",
            &[
                ("title", title.trim_start_matches('#').trim().to_owned()),
                ("file", other.file.clone()),
            ],
        )?);
    }
    intro.push(config.text("overview_recipes_link")?);
    let mut lines = vec![String::from("flowchart LR")];
    for (kind, group) in [("entry", "entries"), ("layer", "layers")] {
        lines.push(format!(
            "    subgraph {group}[\"{}\"]",
            config.text(&format!("overview_{group}"))?
        ));
        for node in config.nodes.iter().filter(|node| node.kind == kind) {
            let detail = if node.detail.is_empty() {
                String::new()
            } else {
                let count = if node.count.is_empty() {
                    String::new()
                } else {
                    count_of(data, &node.count)?.to_string()
                };
                format!("<br/>{}", fill(&node.detail, &[("count", count)]))
            };
            lines.push(format!("        {}[\"{}{detail}\"]", node.id, node.label));
        }
        lines.push(String::from("    end"));
    }
    for edge in &config.edges {
        for end in [&edge.from, &edge.to] {
            if !config.nodes.iter().any(|node| &node.id == end) {
                return Err(config_error(&["edge", &edge.id, "unknown_node", end]));
            }
        }
        let label = edge_label(data, edge)?;
        lines.push(if label.is_empty() {
            format!("    {} --> {}", edge.from, edge.to)
        } else {
            format!("    {} -->|\"{label}\"| {}", edge.from, edge.to)
        });
    }
    document(
        config,
        part,
        "overview_title",
        vec![
            intro.join("\n"),
            config.text("overview_heading")?,
            mermaid(&lines),
        ],
    )
}

/// A precedence node label: `rank: name`.
fn handler_label(row: &HandlerRow) -> String {
    format!("{}: {}", row.rank, row.name)
}

/// The promotions diagram and table.
fn promotion_blocks(config: &Config, data: &Data) -> Result<Vec<String>, String> {
    let mut lines = vec![
        String::from("flowchart LR"),
        format!("    prompt([\"{}\"])", config.text("handlers_prompt")?),
    ];
    let mut previous = String::from("prompt");
    for (index, row) in data.promotions.iter().enumerate() {
        let node = format!("p_{}", index + 1);
        lines.push(format!(
            "    {previous} --> {node}[\"{}: {}\"]",
            row.rank, row.handler
        ));
        previous = node;
    }
    lines.push(format!(
        "    {previous} --> precedence[[\"{}\"]]",
        config.text("handlers_precedence_node")?
    ));
    let rows: Vec<Vec<String>> = data
        .promotions
        .iter()
        .map(|row| {
            vec![
                row.rank.to_string(),
                format!("`{}`", row.handler),
                row.because.clone(),
            ]
        })
        .collect();
    Ok(vec![
        config.text("handlers_intro")?,
        config.text("handlers_promotions_heading")?,
        mermaid(&lines),
        table(
            &[
                config.text("column_rank")?,
                config.text("column_handler")?,
                config.text("column_because")?,
            ],
            &rows,
        ),
    ])
}

/// One precedence chunk: its heading, diagram and table.
fn precedence_blocks(
    config: &Config,
    chunk: &[HandlerRow],
    part_number: usize,
    has_more: bool,
) -> Result<Vec<String>, String> {
    let (Some(first), Some(last)) = (chunk.first(), chunk.last()) else {
        return Ok(Vec::new());
    };
    let mut lines = vec![String::from("flowchart TD")];
    let mut prior = String::new();
    if part_number > 2 {
        prior = format!("before_{part_number}");
        lines.push(format!(
            "    {prior}([\"{}\"])",
            config.message("handlers_previous", &[("rank", first.rank.to_string())])?
        ));
    }
    for row in chunk {
        let node = format!("h_{}", row.name);
        lines.push(if prior.is_empty() {
            format!("    {node}[\"{}\"]", handler_label(row))
        } else {
            format!("    {prior} --> {node}[\"{}\"]", handler_label(row))
        });
        prior = node;
    }
    if has_more {
        lines.push(format!(
            "    {prior} --> after_{part_number}([\"{}\"])",
            config.message("handlers_next", &[("rank", last.rank.to_string())])?
        ));
    }
    let browser: Vec<String> = chunk
        .iter()
        .filter(|row| row.browser_only)
        .map(|row| format!("h_{}", row.name))
        .collect();
    if !browser.is_empty() {
        lines.push(format!("    classDef {BROWSER_CLASS} {BROWSER_STYLE}"));
        lines.push(format!("    class {} {BROWSER_CLASS}", browser.join(",")));
    }
    let mut rows = Vec::with_capacity(chunk.len());
    for row in chunk {
        let surface = if row.browser_only {
            "surface_browser"
        } else {
            "surface_native"
        };
        rows.push(vec![
            row.rank.to_string(),
            format!("`{}`", row.name),
            config.text(surface)?,
        ]);
    }
    Ok(vec![
        config.message(
            "handlers_precedence_heading",
            &[
                ("part", part_number.to_string()),
                ("first", first.rank.to_string()),
                ("last", last.rank.to_string()),
            ],
        )?,
        mermaid(&lines),
        table(
            &[
                config.text("column_rank")?,
                config.text("column_handler")?,
                config.text("column_surface")?,
            ],
            &rows,
        ),
    ])
}

/// Part: the solver handler registry (promotions, then precedence chunks).
fn render_handlers(config: &Config, data: &Data, part: &Part) -> Result<String, String> {
    let mut blocks = promotion_blocks(config, data)?;
    let chunks: Vec<&[HandlerRow]> = data.handlers.chunks(config.chunk).collect();
    for (index, chunk) in chunks.iter().enumerate() {
        blocks.extend(precedence_blocks(
            config,
            chunk,
            index + 2,
            index + 1 < chunks.len(),
        )?);
    }
    document(config, part, "handlers_title", blocks)
}

/// The mermaid id of a subcommand node.
fn cli_node(name: &str) -> String {
    format!("c_{}", name.replace('-', "_"))
}

/// Part: the CLI subcommands.
fn render_cli(config: &Config, data: &Data, part: &Part) -> Result<String, String> {
    let binary = &data.cli.binary;
    let mut lines = vec![
        String::from("flowchart LR"),
        format!("    cli([\"{binary}\"])"),
    ];
    for sub in &data.cli.subcommands {
        lines.push(format!(
            "    cli --> {}[\"{}\"]",
            cli_node(&sub.name),
            sub.name
        ));
    }
    let rows: Vec<Vec<String>> = data
        .cli
        .subcommands
        .iter()
        .map(|sub| vec![format!("`{binary} {}`", sub.name), sub.summary.clone()])
        .collect();
    document(
        config,
        part,
        "cli_title",
        vec![
            config.text("cli_intro")?,
            config.text("cli_heading")?,
            mermaid(&lines),
            table(
                &[
                    config.text("column_subcommand")?,
                    config.text("column_summary")?,
                ],
                &rows,
            ),
        ],
    )
}

/// Part: the HTTP routes, grouped by method.
fn render_routes(config: &Config, data: &Data, part: &Part) -> Result<String, String> {
    let mut lines = vec![
        String::from("flowchart LR"),
        format!("    server([\"{}\"])", config.text("routes_server")?),
    ];
    let mut methods: Vec<&str> = Vec::new();
    for row in &data.routes {
        if !methods.contains(&row.method.as_str()) {
            methods.push(&row.method);
        }
    }
    for method in methods {
        lines.push(format!("    subgraph m_{method}[\"{method}\"]"));
        for row in data.routes.iter().filter(|row| row.method == method) {
            let name = if row.deprecated {
                config.message("routes_deprecated", &[("route", row.id.clone())])?
            } else {
                row.id.clone()
            };
            let mut label = vec![name];
            label.extend(row.paths.iter().cloned());
            lines.push(format!("        r_{}[\"{}\"]", row.id, label.join("<br/>")));
        }
        lines.push(String::from("    end"));
    }
    for row in &data.routes {
        let arrow = if row.auth == "none" { "-.->" } else { "-->" };
        lines.push(format!("    server {arrow}|\"{}\"| r_{}", row.auth, row.id));
    }
    let rows: Vec<Vec<String>> = data
        .routes
        .iter()
        .map(|row| {
            let paths: Vec<String> = row.paths.iter().map(|path| format!("`{path}`")).collect();
            vec![
                format!("`{}`", row.id),
                row.method.clone(),
                paths.join(", "),
                row.auth.clone(),
            ]
        })
        .collect();
    document(
        config,
        part,
        "routes_title",
        vec![
            config.text("routes_intro")?,
            config.text("routes_heading")?,
            mermaid(&lines),
            table(
                &[
                    config.text("column_route")?,
                    config.text("column_method")?,
                    config.text("column_paths")?,
                    config.text("column_auth")?,
                ],
                &rows,
            ),
        ],
    )
}

/// Every generated part as `(file name under docs/diagrams, document)`, in
/// config order, reading repository files through `read`.
///
/// # Errors
/// Returns a `system_diagrams:…` key when a file is unreadable, the config
/// names an unknown part, source, count or message, or an overview edge names
/// a subcommand, route or node the live data lacks.
pub fn render_system_diagrams(
    read: impl Fn(&str) -> Option<String>,
) -> Result<Vec<(String, String)>, String> {
    let config = Config::parse(
        &read(CONFIG_PATH).ok_or_else(|| config_error(&["unreadable", CONFIG_PATH]))?,
    );
    let source_text = |key: &str| -> Result<String, String> {
        let path = config
            .source_path(key)
            .ok_or_else(|| config_error(&["missing_source", key]))?;
        read(path).ok_or_else(|| config_error(&["unreadable", path]))
    };
    let data = Data {
        handlers: precedence_rows(&source_text("handler_precedence")?),
        promotions: promotion_rows(&source_text("handler_promotions")?),
        cli: cli_definition(&source_text("cli")?),
        routes: route_rows(&source_text("routes")?),
    };
    let mut out = Vec::with_capacity(config.parts.len());
    for part in &config.parts {
        let text = match part.key.as_str() {
            "overview" => render_overview(&config, &data, part)?,
            "handlers" => render_handlers(&config, &data, part)?,
            "cli" => render_cli(&config, &data, part)?,
            "routes" => render_routes(&config, &data, part)?,
            other => return Err(config_error(&["unknown_part", other])),
        };
        out.push((part.file.clone(), text));
    }
    Ok(out)
}
