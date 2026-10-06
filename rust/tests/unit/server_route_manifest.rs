//! The route manifest is the server's route table (R1013).
//!
//! `data/meta/server-routes.lino` is the single source of truth for the HTTP
//! surface: the JavaScript server (`js/server/routes.mjs`) routes from it, and
//! these tests hold the Rust server to it from the other side. Every manifest
//! route is served by `handle_api_request_with_auth` (it never answers the
//! `route not found` 404), each route's `auth` column is the bearer rule the
//! Rust dispatcher applies, and no method/path pair outside the manifest is
//! served.

use std::path::{Path, PathBuf};

use formal_ai::{ApiAuthConfig, ApiHttpResponse, handle_api_request_with_auth};
use regex::Regex;

const TOKEN: &str = "server-route-manifest-token";
const ROUTE_NOT_FOUND: &str = r#"{"error":{"message":"route not found","type":"formal_ai_error"}}"#;
const METHODS: &[&str] = &["GET", "POST", "HEAD", "OPTIONS", "PUT", "DELETE"];

/// Paths no route should serve, beside the manifest's own.
const NEAR_MISSES: &[&str] = &[
    "/v1",
    "/v1/",
    "/api",
    "/health/",
    "/mcp/",
    "/telegram",
    "/v1/model",
    "/v1/conversations/",
    "/v1/memory/since/extra",
    "/api/openai/v1/models/extra",
    "/api/anthropic/v1/complete",
    "/api/hello/extra",
    "/api/gemini/v1beta",
    "/api/vertex/v1/projects/p",
    "/api/formal-ai/v1/unknown",
    "/server-route-manifest-missing",
];

#[derive(Debug)]
struct ManifestRoute {
    id: String,
    method: String,
    paths: Vec<String>,
    params: Vec<(String, String)>,
    auth: String,
    example: Option<String>,
}

impl ManifestRoute {
    fn matchers(&self) -> Vec<Regex> {
        self.paths
            .iter()
            .map(|path| {
                if path == "*" {
                    return Regex::new("^.*$").expect("wildcard regex");
                }
                let mut source = String::from("(?s)^");
                let mut rest = path.as_str();
                while let Some(open) = rest.find('{') {
                    let close = rest[open..].find('}').expect("closed parameter") + open;
                    let name = &rest[open + 1..close];
                    let pattern = self
                        .params
                        .iter()
                        .find(|(param, _)| param == name)
                        .map_or(".+", |(_, pattern)| pattern.as_str());
                    source.push_str(&regex::escape(&rest[..open]));
                    source.push('(');
                    source.push_str(pattern);
                    source.push(')');
                    rest = &rest[close + 1..];
                }
                source.push_str(&regex::escape(rest));
                source.push('$');
                Regex::new(&source).expect("manifest path compiles")
            })
            .collect()
    }

    fn serves(&self, method: &str, path: &str) -> bool {
        self.method == method && self.matchers().iter().any(|regex| regex.is_match(path))
    }

    fn concrete_paths(&self) -> Vec<String> {
        let mut out = Vec::new();
        for path in &self.paths {
            if path == "*" || path.contains('{') {
                let example = self
                    .example
                    .clone()
                    .unwrap_or_else(|| panic!("{} needs an example path", self.id));
                if !out.contains(&example) {
                    out.push(example);
                }
            } else {
                out.push(path.clone());
            }
        }
        out
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits above the crate")
        .to_path_buf()
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

fn manifest() -> Vec<ManifestRoute> {
    let text = std::fs::read_to_string(repository_root().join("data/meta/server-routes.lino"))
        .expect("the route manifest is readable");
    let mut routes: Vec<ManifestRoute> = Vec::new();
    let mut current_param: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let (key, value) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        match (indent, key) {
            (2, "route") => routes.push(ManifestRoute {
                id: value.to_owned(),
                method: String::new(),
                paths: Vec::new(),
                params: Vec::new(),
                auth: String::new(),
                example: None,
            }),
            (4, field) => {
                let route = routes.last_mut().expect("a field belongs to a route");
                match field {
                    "method" => route.method = value.to_owned(),
                    "path" => route.paths.push(unquote(value)),
                    "auth" => route.auth = value.to_owned(),
                    "example" => route.example = Some(unquote(value)),
                    "param" => {
                        current_param = Some(value.to_owned());
                    }
                    _ => {}
                }
            }
            (6, "pattern") => {
                let route = routes.last_mut().expect("a pattern belongs to a route");
                let name = current_param.clone().expect("a pattern belongs to a param");
                route.params.push((name, unquote(value)));
            }
            _ => {}
        }
    }
    routes
}

/// Run `run` against a private memory file and dialog log. `name` keeps the
/// directory per test: the tests run in parallel, and only the environment
/// itself is serialized by `temp_env`.
fn with_isolated_state<T>(name: &str, run: impl FnOnce() -> T) -> T {
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-server-route-manifest-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let memory = dir.join("memory.lino");
    let dialogs = dir.join("dialogs");
    let result = temp_env::with_vars(
        [
            ("FORMAL_AI_MEMORY_PATH", Some(memory.as_os_str())),
            ("FORMAL_AI_DIALOG_LOG_DIR", Some(dialogs.as_os_str())),
            ("FORMAL_AI_RECORD_CHAT", Some(std::ffi::OsStr::new("0"))),
        ],
        run,
    );
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// A cheap request: a POST carries a body no route can solve, so every
/// protocol route answers its parse error instead of running the solver.
fn request(method: &str, path: &str, authorized: bool) -> ApiHttpResponse {
    let auth = ApiAuthConfig::bearer_token(TOKEN);
    let bearer = format!("Bearer {TOKEN}");
    let headers: Vec<(&str, &str)> = if authorized {
        vec![("authorization", bearer.as_str())]
    } else {
        Vec::new()
    };
    let body = if method == "POST" { "{not json" } else { "" };
    handle_api_request_with_auth(method, path, &headers, body, &auth)
}

fn is_route_not_found(response: &ApiHttpResponse) -> bool {
    response.status_code == 404 && response.body == ROUTE_NOT_FOUND
}

#[test]
fn the_manifest_lists_the_whole_rust_surface() {
    let routes = manifest();
    assert!(routes.len() >= 29, "the manifest lost routes: {routes:?}");
    for route in &routes {
        assert!(
            ["GET", "POST", "HEAD", "OPTIONS"].contains(&route.method.as_str()),
            "{} has method {}",
            route.id,
            route.method
        );
        assert!(
            ["bearer", "none"].contains(&route.auth.as_str()),
            "{} has auth {}",
            route.id,
            route.auth
        );
        assert!(!route.paths.is_empty(), "{} names no path", route.id);
    }
}

#[test]
fn every_manifest_route_is_served_by_the_rust_dispatcher() {
    with_isolated_state("served", || {
        for route in manifest() {
            for path in route.concrete_paths() {
                let response = request(&route.method, &path, true);
                assert!(
                    !is_route_not_found(&response),
                    "{} {path} ({}) is in the manifest but the Rust server does not serve it",
                    route.method,
                    route.id
                );
                assert_ne!(
                    response.status_code, 401,
                    "{} {path} refused a valid token",
                    route.method
                );
                if route.id == "network_graph_alias" {
                    assert!(response.deprecated, "{path} is the deprecated alias");
                }
            }
        }
    });
}

#[test]
fn the_manifest_auth_column_is_the_rust_bearer_rule() {
    with_isolated_state("auth", || {
        for route in manifest() {
            for path in route.concrete_paths() {
                let response = request(&route.method, &path, false);
                if route.auth == "bearer" {
                    assert_eq!(
                        response.status_code, 401,
                        "{} {path} ({}) is `auth bearer` but answered without a token",
                        route.method, route.id
                    );
                } else {
                    assert_ne!(
                        response.status_code, 401,
                        "{} {path} ({}) is `auth none` but asked for a token",
                        route.method, route.id
                    );
                }
            }
        }
    });
}

#[test]
fn the_rust_dispatcher_serves_nothing_outside_the_manifest() {
    let routes = manifest();
    let mut paths: Vec<String> = routes
        .iter()
        .flat_map(ManifestRoute::concrete_paths)
        .collect();
    paths.extend(NEAR_MISSES.iter().map(|path| (*path).to_owned()));
    paths.sort();
    paths.dedup();
    with_isolated_state("outside", || {
        for path in &paths {
            for method in METHODS {
                let in_manifest = routes.iter().any(|route| route.serves(method, path));
                let response = request(method, path, true);
                if in_manifest {
                    assert!(
                        !is_route_not_found(&response),
                        "{method} {path} matches the manifest but is not served"
                    );
                } else {
                    assert!(
                        is_route_not_found(&response),
                        "{method} {path} is served ({}) but is not in data/meta/server-routes.lino",
                        response.status_code
                    );
                }
            }
        }
    });
}
