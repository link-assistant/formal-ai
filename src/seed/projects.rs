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
    pub fn topic_for(&self, language: &str) -> &str {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_hive_mind_project() {
        let registry = projects_registry();
        let hive = registry
            .by_alias("Hive Mind")
            .expect("hive-mind project must be present");
        assert_eq!(hive.repo_slug(), "link-assistant/hive-mind");
        assert_eq!(hive.url, "https://github.com/link-assistant/hive-mind");
        assert!(!hive.statements.is_empty());
        let purpose = hive
            .statements
            .iter()
            .find(|s| s.kind == "purpose")
            .expect("hive-mind needs a purpose statement");
        assert!(purpose.text.contains("AI"));
    }

    #[test]
    fn alias_lookup_is_case_insensitive() {
        let registry = projects_registry();
        let by_lower = registry.by_alias("hive mind").map(|p| p.slug.clone());
        let by_upper = registry.by_alias("Hive Mind").map(|p| p.slug.clone());
        let by_compact = registry.by_alias("hivemind").map(|p| p.slug.clone());
        assert_eq!(
            by_lower,
            Some("project_link_assistant_hive_mind".to_owned())
        );
        assert_eq!(by_lower, by_upper);
        assert_eq!(by_lower, by_compact);
    }

    #[test]
    fn alias_lookup_handles_hyphen_variants() {
        let registry = projects_registry();
        let with_hyphen = registry.by_alias("hive-mind").map(|p| p.slug.clone());
        let with_underscore = registry.by_alias("hive_mind").map(|p| p.slug.clone());
        assert!(with_hyphen.is_some());
        assert_eq!(with_hyphen, with_underscore);
    }

    #[test]
    fn by_org_returns_only_matching_org() {
        let registry = projects_registry();
        let assistant = registry.by_org("link-assistant");
        let foundation = registry.by_org("link-foundation");
        assert!(!assistant.is_empty());
        assert!(!foundation.is_empty());
        assert!(assistant.iter().all(|p| p.org == "link-assistant"));
        assert!(foundation.iter().all(|p| p.org == "link-foundation"));
    }

    #[test]
    fn localized_russian_overrides_statements_when_present() {
        let registry = projects_registry();
        let hive = registry.by_alias("hive mind").expect("hive-mind present");
        let ru_statements = hive.statements_for("ru");
        assert!(!ru_statements.is_empty());
        assert!(ru_statements
            .iter()
            .any(|s| s.text.contains("ИИ") || s.text.contains("Hive Mind")));
    }

    #[test]
    fn unknown_language_falls_back_to_default() {
        let registry = projects_registry();
        let hive = registry.by_alias("hive mind").expect("hive-mind present");
        // hindi / chinese aren't defined for this entry: default applies
        let hi_statements = hive.statements_for("hi");
        assert_eq!(hi_statements.len(), hive.statements.len());
    }

    #[test]
    fn every_project_carries_url_and_purpose() {
        let registry = projects_registry();
        assert!(
            registry.projects.len() >= 10,
            "expected curated registry of at least 10 projects (got {})",
            registry.projects.len()
        );
        for project in &registry.projects {
            assert!(
                project.url.starts_with("https://github.com/"),
                "{} must point to a GitHub URL",
                project.slug
            );
            assert!(
                project
                    .statements
                    .iter()
                    .any(|s| s.kind == "purpose" && !s.text.is_empty()),
                "{} missing purpose statement",
                project.slug
            );
        }
    }
}
