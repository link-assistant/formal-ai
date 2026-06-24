//! Curated GitHub-project registry loaded from `data/seed/projects.lino`.
//!
//! Each `project_*` entry encodes one open-source project from the promoted
//! repository organizations. The entry carries
//! everything that project lookup call sites need to
//! describe the project at any summarization length:
//!
//! - identity metadata (`org`, `name`, `display_name`, `url`),
//! - the primary programming `language`,
//! - a coarse `category` and a list of `aliases` (pipe-delimited),
//! - a 1–5 word `topic` used by the topic-mode summarizer,
//! - any number of `statement "..."` blocks, each tagged with a `kind`
//!   (purpose, feature, language, identity, install, example, misc) and a
//!   numeric `weight` (0–100) so the summarizer can keep the highest-weighted
//!   statements when the caller asks for a tighter response.
//!
//! Optional `localized "<lang>"` blocks override `display_name` and the
//! statement list per language. Empty fields fall back to the outer English
//! values so a localization only needs to differ where it actually does.
//!
//! `project_lookup` treats this registry as the authoritative source for
//! promoted project answers: when the prompt asks about `Hive Mind` /
//! `hive-mind` (or any other registered project), the answer starts from the
//! curated statements here before any fallback web search.

use super::parser::{parse_lino, split_pipe_list, LinoNode};
use super::PROJECTS_LINO;

/// A localized variant of a single project statement.
#[derive(Debug, Clone, Default)]
pub struct ProjectStatement {
    pub text: String,
    pub kind: String,
    pub weight: u8,
}

impl ProjectStatement {
    fn parse(node: &LinoNode) -> Option<Self> {
        if node.name != "statement" {
            return None;
        }
        let text = node.id.trim().to_string();
        if text.is_empty() {
            return None;
        }
        let kind = node.find_child_value("kind").trim().to_string();
        let weight = node
            .find_child_value("weight")
            .trim()
            .parse::<u8>()
            .unwrap_or(50);
        Some(Self { text, kind, weight })
    }
}

/// A per-language override (display name + replacement statements).
#[derive(Debug, Clone, Default)]
pub struct LocalizedProject {
    pub language: String,
    pub display_name: String,
    pub statements: Vec<ProjectStatement>,
}

/// A single curated project entry from `data/seed/projects.lino`.
#[derive(Debug, Clone)]
pub struct ProjectRecord {
    pub slug: String,
    pub org: String,
    pub name: String,
    pub display_name: String,
    pub url: String,
    pub language: String,
    pub category: String,
    pub aliases: Vec<String>,
    pub topic: String,
    pub statements: Vec<ProjectStatement>,
    pub localized: Vec<LocalizedProject>,
}

impl ProjectRecord {
    /// `org/name` repository slug as displayed in URLs and prompts.
    #[must_use]
    pub fn repo_slug(&self) -> String {
        format!("{}/{}", self.org, self.name)
    }

    /// Pick the localized variant matching `language`, falling back to the
    /// explicit English variant when one is defined, otherwise `None`.
    #[must_use]
    pub fn localized_for(&self, language: &str) -> Option<&LocalizedProject> {
        self.localized
            .iter()
            .find(|loc| loc.language == language)
            .or_else(|| self.localized.iter().find(|loc| loc.language == "en"))
    }

    /// Statements for `language` — either the localized override (when
    /// present and non-empty) or the default English statements.
    #[must_use]
    pub fn statements_for(&self, language: &str) -> &[ProjectStatement] {
        self.localized_for(language)
            .map(|loc| loc.statements.as_slice())
            .filter(|stmts| !stmts.is_empty())
            .unwrap_or(self.statements.as_slice())
    }

    /// Localized display name when defined, otherwise the default.
    #[must_use]
    pub fn display_name_for(&self, language: &str) -> &str {
        self.localized_for(language)
            .map(|loc| loc.display_name.as_str())
            .filter(|name| !name.is_empty())
            .unwrap_or(self.display_name.as_str())
    }

    /// Localized topic (1–5 word) label when defined, otherwise the default.
    #[must_use]
    pub const fn topic_for(&self, language: &str) -> &str {
        // Statements may move, but the topic stays per-record — localized
        // overrides only differ for prose, not for the bare topic label.
        let _ = language;
        if self.topic.is_empty() {
            self.display_name.as_str()
        } else {
            self.topic.as_str()
        }
    }

    /// Case-insensitive alias match against `term`. Aliases are normalized
    /// (lowercased, hyphens/underscores -> spaces, whitespace collapsed)
    /// at load time so substring comparison is straightforward.
    #[must_use]
    pub fn matches_alias(&self, term: &str) -> bool {
        let normalized = normalize_alias(term);
        if normalized.is_empty() {
            return false;
        }
        self.aliases.iter().any(|alias| alias == &normalized)
    }
}

/// Convenience wrapper around the loaded registry.
#[derive(Debug, Clone, Default)]
pub struct ProjectsRegistry {
    pub projects: Vec<ProjectRecord>,
}

impl ProjectsRegistry {
    /// Look up a project whose `slug` (`project_*` identifier) matches `slug`.
    #[must_use]
    pub fn by_slug(&self, slug: &str) -> Option<&ProjectRecord> {
        self.projects.iter().find(|p| p.slug == slug)
    }

    /// Look up a project by alias match (term in `aliases` or display name).
    #[must_use]
    pub fn by_alias(&self, term: &str) -> Option<&ProjectRecord> {
        if term.is_empty() {
            return None;
        }
        let normalized = normalize_alias(term);
        if normalized.is_empty() {
            return None;
        }
        self.projects.iter().find(|p| {
            p.aliases.iter().any(|alias| alias == &normalized)
                || normalize_alias(&p.display_name) == normalized
                || normalize_alias(&p.name) == normalized
        })
    }

    /// Filter projects by organization (`link-assistant`, `link-foundation`, ...).
    #[must_use]
    pub fn by_org<'a>(&'a self, org: &str) -> Vec<&'a ProjectRecord> {
        self.projects.iter().filter(|p| p.org == org).collect()
    }
}

/// Load and parse `data/seed/projects.lino` into the in-memory registry.
#[must_use]
pub fn projects_registry() -> ProjectsRegistry {
    let tree = parse_lino(PROJECTS_LINO);
    let entries: &[LinoNode] = if tree.name.is_empty() {
        tree.children.as_slice()
    } else {
        std::slice::from_ref(&tree)
    };
    let mut out = Vec::new();
    for entry in entries {
        if !entry.name.starts_with("project_") {
            continue;
        }
        let aliases = split_pipe_list(entry.find_child_value("aliases"))
            .into_iter()
            .map(|s| normalize_alias(&s))
            .filter(|s| !s.is_empty())
            .collect();
        let statements: Vec<ProjectStatement> = entry
            .children
            .iter()
            .filter_map(ProjectStatement::parse)
            .collect();
        let localized = entry
            .children
            .iter()
            .filter(|c| c.name == "localized")
            .map(|child| {
                let language = child.id.clone();
                let display_name = child.find_child_value("display_name").to_string();
                let stmts: Vec<ProjectStatement> = child
                    .children
                    .iter()
                    .filter_map(ProjectStatement::parse)
                    .collect();
                LocalizedProject {
                    language,
                    display_name,
                    statements: stmts,
                }
            })
            .filter(|loc| !loc.language.is_empty())
            .collect();
        out.push(ProjectRecord {
            slug: entry.name.clone(),
            org: entry.find_child_value("org").to_string(),
            name: entry.find_child_value("name").to_string(),
            display_name: entry.find_child_value("display_name").to_string(),
            url: entry.find_child_value("url").to_string(),
            language: entry.find_child_value("language").to_string(),
            category: entry.find_child_value("category").to_string(),
            aliases,
            topic: entry.find_child_value("topic").to_string(),
            statements,
            localized,
        });
    }
    ProjectsRegistry { projects: out }
}

/// Normalize an alias for case-insensitive matching: lowercased, hyphens and
/// underscores replaced with spaces, collapse whitespace.
fn normalize_alias(value: &str) -> String {
    value
        .to_lowercase()
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
