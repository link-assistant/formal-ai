//! Discovery on the production path: a rediscoverable coding-procedure cache.
//!
//! Issue #1165 (E130). The machinery for coding by discovery sat behind
//! tests: `research_coding_skill_gap` was called only from test modules, a
//! recognised `write_program` request was answered from stored template sets,
//! and the work-item executor took a template's `execution_recipe` without
//! ever researching. This module puts the cache in front of those paths and
//! makes every stored program a *cache entry*: a row that names the query and
//! the documented source a rediscovery starts from, verifiable by deleting
//! the whole cache and reproducing every row through research.
//!
//! The policy is data, not code: `data/seed/program-cache-policy.lino`
//! (mirrored byte-identical under `rust/embedded/data/seed/`) declares which
//! fields a cache row must carry ([`ProcedureCache::store`] enforces them),
//! the hash that addresses a row's program ([`fnv1a64`], recomputed on load
//! so a stored `content_id` can never drift), and whether the embedded
//! `ORACLE_SNAPSHOTS` bootstrap in `crate::knowledge` still fronts the cache
//! offline. The cache itself is `data/cache/coding-procedure-cache.lino` —
//! runtime state, deletable by design.
//!
//! The wiring points the issue names — the `SelectedRule::WriteProgram`
//! branch of `crate::solver` and `plan_work_item_execution` in
//! `crate::agentic_coding::general_execution` — call
//! [`cached_or_research`] so a cache miss routes to
//! [`research_coding_skill_gap`] instead of a template literal, and the
//! template sets the issue deletes (`hello-world-programs.lino`, the
//! `ORACLE_SNAPSHOTS` bootstrap, the `entry`/`operation`/`ci_setup`/
//! `documented_source` fields) retire once every row they covered is
//! reproduced by a rediscovery run.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::coding_research_learning::{
    CodingResearchApproval, CodingResearchError, CodingResearchGap,
    ResearchedCodingProcedureLedger, research_coding_skill_gap,
};
use crate::seed::parser::parse_lino;
use crate::source_fetch::{CachedSourceClient, SourceTransport};

/// The cache policy seed, mirrored into the embedded bundle.
const POLICY: &str = include_str!("../embedded/data/seed/program-cache-policy.lino");

/// The grammar seed that decides whether a language is renderable at all.
const GRAMMARS: &str = include_str!("../embedded/data/seed/program-cst-grammars.lino");

/// Default location of the deletable cache file, relative to the repository
/// root. Overridable with `FORMAL_AI_PROCEDURE_CACHE` for isolated runs.
pub const DEFAULT_CACHE_FILE: &str = "data/cache/coding-procedure-cache.lino";

/// FNV-1a (64-bit) of the entry program bytes.
///
/// The address of a cached program: stable across runs, cheap to recompute,
/// and independent of the cache file's formatting, so the rediscovery test
/// can compare a reproduced row against the deleted one by content.
#[must_use]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// One rediscovery-verified procedure per `(language, task)` pair.
///
/// A row is accepted only when it records where a rediscovery starts
/// (`rediscovery_query`, `rediscovery_source`), the verified program
/// (`entry`, `verified_output`), and its content address (`content_id`,
/// recomputed from `entry` on load and store — a stored id that disagrees
/// with its program fails the load).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RediscoverableRecipe {
    pub language: String,
    pub task: String,
    pub rediscovery_query: String,
    pub rediscovery_source: String,
    pub entry: String,
    pub verified_output: String,
    pub content_id: u64,
}

impl RediscoverableRecipe {
    /// Address of this row's program.
    #[must_use]
    pub fn content_address(entry: &str) -> u64 {
        fnv1a64(entry.as_bytes())
    }

    /// Whether the row carries every field the policy seed requires (R1165-3:
    /// a procedure without a rediscovery query and source is not stored).
    #[must_use]
    pub fn is_valid_cache_entry(&self) -> bool {
        !self.language.trim().is_empty()
            && !self.task.trim().is_empty()
            && !self.rediscovery_query.trim().is_empty()
            && !self.rediscovery_source.trim().is_empty()
            && !self.entry.trim().is_empty()
    }
}

/// Whether the policy seed still serves the embedded bootstrap snapshots.
///
/// Deleting the bootstrap record in the seed stops every
/// `ORACLE_SNAPSHOTS`-backed answer with no Rust change — the deletable-cache
/// contract applied to the compiled-in tier.
#[must_use]
pub fn bootstrap_cache_active() -> bool {
    let policy = parse_lino(POLICY);
    policy
        .children
        .first()
        .and_then(|root| root.children.iter().find(|node| node.name == "bootstrap"))
        .is_some_and(|bootstrap| bootstrap.find_child_value("active") == "true")
}

/// The comment block a written cache file opens with: the policy seed's
/// `cache_file_header`, one `# ` line per header line.
fn cache_file_header() -> String {
    let policy = parse_lino(POLICY);
    let header = policy
        .children
        .first()
        .map_or("", |root| root.find_child_value("cache_file_header"));
    header
        .lines()
        .map(|line| {
            if line.is_empty() {
                "#\n".to_owned()
            } else {
                ["# ", line, "\n"].concat()
            }
        })
        .collect()
}

/// Whether a grammar exists for `language` in the CST grammar seed.
#[must_use]
pub fn grammar_exists(language: &str) -> bool {
    let needle = language.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return false;
    }
    let grammars = parse_lino(GRAMMARS);
    let Some(root) = grammars.children.first() else {
        return false;
    };
    root.children
        .iter()
        .filter(|node| node.name == "cst_grammar")
        .any(|grammar| {
            grammar.id.to_ascii_lowercase() == needle
                || grammar
                    .find_child_value("program_language")
                    .to_ascii_lowercase()
                    == needle
        })
}

/// Whether discovery knows `language` (R1165-4): a grammar exists and
/// discovery found a procedure — a cache row, or the bootstrap record of a
/// discovery that already ran before the cache file existed.
#[must_use]
pub fn knows_language(language: &str) -> bool {
    grammar_exists(language)
        && (bootstrap_cache_active() && crate::knowledge::CodingOracle::knows_language(language)
            || language_has_cache_row(language))
}

fn language_has_cache_row(language: &str) -> bool {
    ProcedureCache::load()
        .recipes()
        .iter()
        .any(|recipe| recipe.language.eq_ignore_ascii_case(language))
}

/// The deletable, rediscoverable procedure cache.
///
/// Backed by `data/cache/coding-procedure-cache.lino`: runtime state the
/// repository ships empty of procedure rows on purpose (rows appear only
/// after a verified research run) and a test may delete wholesale to prove
/// every row reproduces.
#[derive(Debug, Clone, Default)]
pub struct ProcedureCache {
    path: PathBuf,
    recipes: Vec<RediscoverableRecipe>,
}

impl ProcedureCache {
    /// Load the cache from an explicit path (tests isolate their cache this
    /// way); a missing file loads as an empty cache.
    #[must_use]
    pub fn load_at(path: &Path) -> Self {
        let mut cache = Self {
            path: path.to_path_buf(),
            recipes: Vec::new(),
        };
        if let Ok(text) = std::fs::read_to_string(path) {
            cache.parse(&text);
        }
        cache
    }

    /// Load the cache from its default location: `$FORMAL_AI_PROCEDURE_CACHE`
    /// when set, else the repository's `data/cache` directory found by walking
    /// up from the working directory.
    #[must_use]
    pub fn load() -> Self {
        Self::load_at(&default_cache_path())
    }

    /// The rows currently cached.
    #[must_use]
    pub fn recipes(&self) -> &[RediscoverableRecipe] {
        &self.recipes
    }

    /// Look up a `(language, task)` pair, case-insensitive on the language.
    #[must_use]
    pub fn lookup(&self, language: &str, task: &str) -> Option<&RediscoverableRecipe> {
        self.recipes.iter().find(|recipe| {
            recipe.language.eq_ignore_ascii_case(language.trim())
                && recipe.task.eq_ignore_ascii_case(task.trim())
        })
    }

    /// Store one row, enforcing the policy seed's requirements.
    ///
    /// A row without a rediscovery query and source is refused (it would be
    /// memorization, not a cache entry); `content_id` is recomputed from
    /// `entry` so a stored address can never drift from its program; a row
    /// for the same pair is replaced, never duplicated.
    pub fn store(&mut self, mut recipe: RediscoverableRecipe) -> Result<(), String> {
        if !recipe.is_valid_cache_entry() {
            return Err(String::from(
                "procedure_cache_entry_requires_rediscovery_query_and_source",
            ));
        }
        recipe.content_id = RediscoverableRecipe::content_address(&recipe.entry);
        self.recipes.retain(|present| {
            !(present.language.eq_ignore_ascii_case(&recipe.language)
                && present.task.eq_ignore_ascii_case(&recipe.task))
        });
        self.recipes.push(recipe);
        self.write()
    }

    /// Delete every row (R1165-7: the rediscovery test starts here).
    /// The file survives as an empty cache; only knowledge leaves.
    pub fn delete_all(&mut self) {
        self.recipes.clear();
        let _unused = self.write();
    }

    fn parse(&mut self, text: &str) {
        let root = parse_lino(text);
        let Some(document) = root.children.first() else {
            return;
        };
        for record in &document.children {
            if !record.name.starts_with("procedure_") {
                continue;
            }
            let recipe = RediscoverableRecipe {
                language: record.find_child_value("language").to_owned(),
                task: record.find_child_value("task").to_owned(),
                rediscovery_query: record.find_child_value("rediscovery_query").to_owned(),
                rediscovery_source: record.find_child_value("rediscovery_source").to_owned(),
                entry: record.find_child_value("entry").to_owned(),
                verified_output: record.find_child_value("verified_output").to_owned(),
                content_id: record
                    .find_child_value("content_id")
                    .trim()
                    .strip_prefix("0x")
                    .and_then(|hex| u64::from_str_radix(hex, 16).ok())
                    .unwrap_or_default(),
            };
            if !recipe.is_valid_cache_entry() {
                continue;
            }
            // A stored address that disagrees with its program is corruption,
            // not a cache row; drop it and let the miss path rediscover.
            if recipe.content_id != RediscoverableRecipe::content_address(&recipe.entry) {
                continue;
            }
            self.recipes.push(recipe);
        }
    }

    fn write(&self) -> Result<(), String> {
        let mut out = cache_file_header();
        crate::links_format::push_lino_field(&mut out, 0, "coding_procedure_cache", None);
        crate::links_format::push_lino_field(&mut out, 2, "version", Some("\"1\""));
        for recipe in &self.recipes {
            let slug = format!("procedure_{}_{}", recipe.language, recipe.task)
                .chars()
                .map(|character| {
                    if character.is_ascii_alphanumeric() || character == '_' {
                        character.to_ascii_lowercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>();
            let _ = writeln!(out, "  {slug}");
            let _ = writeln!(out, "    language {}", quote(&recipe.language));
            let _ = writeln!(out, "    task {}", quote(&recipe.task));
            let _ = writeln!(
                out,
                "    rediscovery_query {}",
                quote(&recipe.rediscovery_query)
            );
            let _ = writeln!(
                out,
                "    rediscovery_source {}",
                quote(&recipe.rediscovery_source)
            );
            let _ = writeln!(out, "    entry {}", quote(&recipe.entry));
            let _ = writeln!(
                out,
                "    verified_output {}",
                quote(&recipe.verified_output)
            );
            let _ = writeln!(out, "    content_id \"0x{:016x}\"", recipe.content_id);
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("procedure_cache_create_dir_failed:{error}"))?;
        }
        std::fs::write(&self.path, out)
            .map_err(|error| format!("procedure_cache_write_failed:{error}"))
    }
}

/// What [`cached_or_research`] returned: a cache row, or a row research just
/// verified and stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachedOrDiscovered {
    Cached(RediscoverableRecipe),
    Discovered(RediscoverableRecipe),
}

impl CachedOrDiscovered {
    /// The row either way.
    #[must_use]
    pub const fn recipe(&self) -> &RediscoverableRecipe {
        match self {
            Self::Cached(recipe) | Self::Discovered(recipe) => recipe,
        }
    }

    /// Whether this answer came from the cache.
    #[must_use]
    pub const fn was_cached(&self) -> bool {
        matches!(self, Self::Cached(_))
    }
}

/// The production miss path (R1165-1/R1165-2).
///
/// Answer a `(language, task)`
/// request from the cache when a row exists, otherwise run
/// [`research_coding_skill_gap`] with the caller's transport, store the
/// verified procedure, and return it.
///
/// `candidate_source` and `expected_output` are passed to research exactly as
/// that function defines them (the procedure to verify and the result it must
/// produce); `rediscovery_source` is the documented source URL the candidate
/// came from, recorded on the row so a later rediscovery starts from the same
/// place. The offline bootstrap tier of [`crate::knowledge`] answers only
/// when no cache row exists and the caller did not supply research inputs.
#[allow(clippy::too_many_arguments)]
pub fn cached_or_research<T: SourceTransport>(
    cache: &mut ProcedureCache,
    gap: &mut CodingResearchGap,
    ledger: &mut ResearchedCodingProcedureLedger,
    client: &CachedSourceClient<T>,
    candidate_source: &str,
    expected_output: &str,
    approval: &CodingResearchApproval,
    rediscovery_source: &str,
) -> Result<CachedOrDiscovered, CodingResearchError> {
    let language = gap_field(gap, "language");
    let task = gap_field(gap, "task");
    if let Some(recipe) = cache.lookup(&language, &task) {
        return Ok(CachedOrDiscovered::Cached(recipe.clone()));
    }
    // The query a rediscovery of this row starts from is the one this gap
    // opened with, captured before research advances the gap's schedule.
    let rediscovery_query = gap.next_query().to_owned();
    let execution = research_coding_skill_gap(
        gap,
        ledger,
        client,
        candidate_source,
        expected_output,
        approval,
    )?;
    // `execution.output` is the verified produced text (for a rewrite task,
    // the rewritten program; for a template, the completed program), so it is
    // the program a cache row reuses; `expected_output` is what research
    // verified it against.
    let recipe = RediscoverableRecipe {
        rediscovery_query,
        rediscovery_source: rediscovery_source.to_owned(),
        entry: execution.output.clone(),
        verified_output: expected_output.to_owned(),
        content_id: RediscoverableRecipe::content_address(&execution.output),
        language,
        task,
    };
    if let Err(reason) = cache.store(recipe.clone()) {
        return Err(CodingResearchError {
            reason,
            cycle: execution.cycle,
        });
    }
    Ok(CachedOrDiscovered::Discovered(recipe))
}

/// A field of a gap (`CodingResearchGap` keeps them private; the cache key
/// needs the `language` and `task` identity). The durable projection the
/// ledger already publishes carries both, so read the pair from the notation
/// instead of widening the gap's API.
fn gap_field(gap: &CodingResearchGap, field: &str) -> String {
    let notation = gap.links_notation();
    let prefix = format!("{field} ");
    for line in notation.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            return value.trim_matches('"').to_owned();
        }
    }
    String::new()
}

/// Resolve the default cache path: the environment override, else the
/// repository's `data/cache` found by walking up from the working directory.
#[must_use]
pub fn default_cache_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("FORMAL_AI_PROCEDURE_CACHE")
        && !explicit.trim().is_empty()
    {
        return PathBuf::from(explicit);
    }
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..6 {
        let candidate = current.join(DEFAULT_CACHE_FILE);
        if candidate.exists() || current.join("data/seed").is_dir() {
            return candidate;
        }
        if !current.pop() {
            break;
        }
    }
    PathBuf::from(DEFAULT_CACHE_FILE)
}

/// Quote a value for the cache file, using the parser's backslash dialect.
fn quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}
