//! External knowledge oracles, treated as cached APIs (issue #412).
//!
//! The universal solver should not depend on a fixed, hand-written catalogue of
//! coding answers. Instead it treats public knowledge bases — Rosetta Code,
//! Wikifunctions, the Hello World Collection, and Stack Overflow — as external
//! APIs, even when they expose no machine API: a reviewed snippet plus its
//! deterministic output and source attribution is kept as a *cached* example so
//! the answer is served offline, exactly the offline-first contract the
//! Wikidata/Wiktionary caches provide. This cache of popular cases is embedded
//! (`ORACLE_SNAPSHOTS`) so it compiles into both the native binary and the
//! Rust→WASM worker without a runtime fetch; the cache holds only the *popular*
//! cases so offline tests stay fast and the repository stays light, and it
//! never mirrors a whole source. A gated live-refresh path (below) is what
//! would materialise a per-source `data/cache/<source-slug>/` bucket from the
//! live pages; until it runs, the embedded snapshots are the cache of record.
//!
//! Two concerns live here:
//!
//! 1. [`cache_capacity`] — the shared cap policy: never cache more than 1% of a
//!    source, or [`KNOWLEDGE_CACHE_FLOOR`] items when 1% is smaller
//!    (issue #412, R8). The same number bounds every per-source/per-topic cache
//!    so no single external corpus can bloat the merged views.
//! 2. [`CodingOracle`] — an offline-first lookup that resolves a
//!    `(task, language)` coding request to a reviewed snippet plus its source
//!    attribution, generalising the static `crate::coding` catalogue beyond
//!    its built-in languages (R6). The committed snapshots are the popular-case
//!    cache; a gated live-refresh path (mirroring the existing
//!    `FORMAL_AI_LIVE_API` discipline) repopulates them from the live sources.
//!
//! The data is plain Rust so it compiles into both the native binary and the
//! Rust→WASM browser worker without a runtime fetch, and is mirrored verbatim in
//! `js/worker/formal_ai_worker.js` so every reasoning surface agrees byte-for-byte.
//!
//! Issue #1165 (E130) converts this store into the *bootstrap tier* of the
//! rediscoverable procedure cache: the snapshots are the record of discovery
//! that ran before `data/cache/coding-procedure-cache.lino` existed, and
//! [`bootstrap_cache_active`] — which reads
//! `data/seed/program-cache-policy.lino`, not this file — decides whether
//! they are still served. Deleting the bootstrap record in that seed stops
//! every snapshot-backed answer with no Rust change; the production miss
//! path (`crate::discovery_production`) then answers from cache rows that
//! name their rediscovery query and source, and the snapshots retire for good
//! once every language row they covered is reproduced by a rediscovery run.

/// The cache policy seed that governs the bootstrap tier (issue #1165).
const PROCEDURE_CACHE_POLICY: &str =
    include_str!("../embedded/data/seed/program-cache-policy.lino");

/// Whether the embedded snapshot bootstrap still fronts the rediscoverable
/// procedure cache.
///
/// The decision is data, not code: the `bootstrap` record of the policy seed
/// carries `active`, so retiring the bootstrap is a seed edit, and a test can
/// prove the deletion path without recompiling.
#[must_use]
pub fn bootstrap_cache_active() -> bool {
    use std::sync::OnceLock;
    static ACTIVE: OnceLock<bool> = OnceLock::new();
    *ACTIVE.get_or_init(|| {
        let policy = crate::seed::parser::parse_lino(PROCEDURE_CACHE_POLICY);
        policy
            .children
            .first()
            .and_then(|root| root.children.iter().find(|node| node.name == "bootstrap"))
            .is_some_and(|bootstrap| bootstrap.find_child_value("active") == "true")
    })
}

/// Lower bound on the local cache: even for a small source we keep up to this
/// many popular items before the 1% ceiling takes over. Issue #412, R8.
pub const KNOWLEDGE_CACHE_FLOOR: usize = 512;

/// Maximum number of items we may cache locally for a source that publishes
/// `source_total` items.
///
/// The policy from issue #412 is "never cache more than 1% — or 512 items when
/// 1% is smaller than 512". So the cap is 1% of the source, rounded up, raised
/// to the [`KNOWLEDGE_CACHE_FLOOR`] floor, and finally clamped to the source's
/// own size (you can never cache more rows than exist).
///
/// The crate's Rust 1.77 baseline supports `div_ceil`, so the 1% calculation
/// stays explicit without carrying a manual rounding formula.
#[must_use]
pub const fn cache_capacity(source_total: usize) -> usize {
    let one_percent = source_total.div_ceil(100);
    let floored = if one_percent > KNOWLEDGE_CACHE_FLOOR {
        one_percent
    } else {
        KNOWLEDGE_CACHE_FLOOR
    };
    if floored < source_total {
        floored
    } else {
        source_total
    }
}

/// Whether keeping `cached` items from a source of `source_total` rows stays
/// within the [`cache_capacity`] cap.
///
/// Used by the ratchet test that guards the committed snapshot set from
/// silently growing into a mirror.
#[must_use]
pub const fn within_cache_capacity(cached: usize, source_total: usize) -> bool {
    cached <= cache_capacity(source_total)
}

/// Public knowledge sources the coding oracle draws on.
///
/// Each is a public, reviewable corpus we treat as an external API even when it
/// exposes none: a fetched page is parsed into a snippet and cached locally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnowledgeSource {
    /// <https://rosettacode.org> — the same task implemented in hundreds of
    /// languages; our primary code corpus for idioms we do not template.
    RosettaCode,
    /// <https://www.wikifunctions.org> — Abstract Wikipedia's function library;
    /// evaluates `Z7` calls and returns a `Z22` result, so it doubles as a
    /// *result oracle* that can cross-check an in-solver computation.
    Wikifunctions,
    /// <http://helloworldcollection.de> — "Hello, World!" in ~600 languages; the
    /// canonical first program for any language we do not yet template.
    HelloWorldCollection,
    /// <https://stackoverflow.com> — community answers, treated read-only and
    /// only for snippets under a compatible licence.
    StackOverflow,
    /// A documentation page captured byte for byte under
    /// `data/seed/coding-documentation-captures.lino` (issue #1165): the
    /// documentation route rediscovers a program from it before any snapshot
    /// answers.
    DocumentationCapture,
}

impl KnowledgeSource {
    /// Stable, filesystem-safe identifier used as the `data/cache/<slug>/`
    /// bucket name and in trace evidence.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::RosettaCode => "rosetta-code",
            Self::Wikifunctions => "wikifunctions",
            Self::HelloWorldCollection => "hello-world-collection",
            Self::StackOverflow => "stack-overflow",
            Self::DocumentationCapture => "documentation-capture",
        }
    }

    /// Human-readable name for attribution in answers and the case study.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::RosettaCode => "Rosetta Code",
            Self::Wikifunctions => "Wikifunctions",
            Self::HelloWorldCollection => "Hello World Collection",
            Self::StackOverflow => "Stack Overflow",
            Self::DocumentationCapture => "Documentation capture",
        }
    }

    /// Landing URL for the source.
    #[must_use]
    pub const fn base_url(self) -> &'static str {
        match self {
            Self::RosettaCode => "https://rosettacode.org",
            Self::Wikifunctions => "https://www.wikifunctions.org",
            Self::HelloWorldCollection => "http://helloworldcollection.de",
            Self::StackOverflow => "https://stackoverflow.com",
            Self::DocumentationCapture => "",
        }
    }

    /// Conservative published size of the source, used only by
    /// [`cache_capacity`] to bound how much we may cache. The exact figure does
    /// not need to be precise — it just feeds the 1% ceiling — and is refreshed
    /// by the gated live-refresh tool. Figures are public approximations:
    /// Rosetta Code lists ~1,300 tasks, Wikifunctions a few thousand functions,
    /// and the Hello World Collection ~600 languages.
    #[must_use]
    pub const fn approximate_catalog_size(self) -> usize {
        match self {
            Self::RosettaCode => 1_300,
            Self::Wikifunctions => 3_000,
            Self::HelloWorldCollection => 600,
            Self::StackOverflow => 24_000_000,
            Self::DocumentationCapture => 0,
        }
    }
}

/// A reviewed code snippet discovered from an external knowledge source.
///
/// `language_slug` is the lowercase identifier a prompt uses (`kotlin`),
/// `language_label` is the display form (`Kotlin`). `expected_output` is the
/// deterministic stdout the snippet prints, so the solver can show "code + the
/// result" exactly as the built-in catalogue does.
#[derive(Clone, Copy, Debug)]
pub struct OracleSnippet {
    pub task_slug: &'static str,
    pub language_slug: &'static str,
    pub language_label: &'static str,
    pub source: KnowledgeSource,
    pub source_url: &'static str,
    pub code: &'static str,
    pub expected_output: &'static str,
}

/// The committed popular-case cache for the coding oracle.
///
/// These are the "Hello, World!" programs for languages no grammar row lets
/// the documentation route rediscover (Bash, Haskell), plus a Rosetta-Code
/// factorial in Kotlin to exercise a non-trivial task. Issue #1165 retired
/// the Swift, Lua, Kotlin and PHP Hello World snapshots: the oracle answers
/// Swift and Lua from their captured documentation (the Swift book, lua.org),
/// and Kotlin and PHP are answered by the catalog from theirs. The set is intentionally tiny — well under [`cache_capacity`] for every
/// source — and is the offline accelerator a live refresh would repopulate.
const ORACLE_SNAPSHOTS: &[OracleSnippet] = &[
    OracleSnippet {
        task_slug: "hello_world",
        language_slug: "bash",
        language_label: "Bash",
        source: KnowledgeSource::HelloWorldCollection,
        source_url: "http://helloworldcollection.de/#Bash",
        code: "echo \"Hello, World!\"",
        expected_output: "Hello, World!",
    },
    OracleSnippet {
        task_slug: "hello_world",
        language_slug: "haskell",
        language_label: "Haskell",
        source: KnowledgeSource::HelloWorldCollection,
        source_url: "http://helloworldcollection.de/#Haskell",
        code: "main :: IO ()\nmain = putStrLn \"Hello, World!\"",
        expected_output: "Hello, World!",
    },
    OracleSnippet {
        task_slug: "factorial",
        language_slug: "kotlin",
        language_label: "Kotlin",
        source: KnowledgeSource::RosettaCode,
        source_url: "https://rosettacode.org/wiki/Factorial#Kotlin",
        code: "fun factorial(n: Int): Long =\n    if (n <= 1) 1L else n * factorial(n - 1)\n\nfun main() {\n    println(factorial(5))\n}",
        expected_output: "120",
    },
];

/// A snippet the oracle answers a `(task, language)` request with, owned so
/// a program rediscovered from documentation and a cached snapshot render
/// alike.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleAnswer {
    pub task_slug: String,
    pub language_slug: String,
    pub language_label: String,
    pub source: KnowledgeSource,
    pub source_url: String,
    pub code: String,
    pub expected_output: String,
}

/// Offline-first lookup that generalises the built-in coding catalogue using
/// the external knowledge sources' cached snapshots.
pub struct CodingOracle;

impl CodingOracle {
    /// Every committed snapshot.
    #[must_use]
    pub const fn snapshots() -> &'static [OracleSnippet] {
        ORACLE_SNAPSHOTS
    }

    /// Resolve a `(task, language)` request to a cached snippet.
    ///
    /// The language is matched by slug or case-insensitive display label so a
    /// bare `kotlin` / `Kotlin` both resolve. Returns `None` when the oracle has
    /// no cached answer — or when the bootstrap tier has been retired in the
    /// policy seed, which is the deletable-cache contract of issue #1165: the
    /// caller then stays on its existing path (the static catalogue, the
    /// procedure cache, or ultimately the `unknown` opener), so this is purely
    /// additive.
    #[must_use]
    pub fn lookup(task_slug: &str, language: &str) -> Option<&'static OracleSnippet> {
        if !bootstrap_cache_active() {
            return None;
        }
        let needle = language.trim().to_ascii_lowercase();
        ORACLE_SNAPSHOTS.iter().find(|snippet| {
            snippet.task_slug == task_slug
                && (snippet.language_slug == needle
                    || snippet.language_label.to_ascii_lowercase() == needle)
        })
    }

    /// The snippet the oracle answers a pair with (issue #1165 R1165-4).
    ///
    /// The program the documentation route rediscovers comes first (the
    /// Swift book), credited to its captured page; a cached snapshot answers
    /// only a pair no captured page covers. The browser twin is
    /// `codingOracleAnswer` with `codingOracleDocumentedSnippet`.
    #[must_use]
    pub fn answer(task_slug: &str, language: &str) -> Option<OracleAnswer> {
        if let Some(program) =
            crate::discovery_production::documented_oracle_program(task_slug, language)
        {
            let slug = language.trim().to_ascii_lowercase();
            let label = if program.language_name.is_empty() {
                slug.clone()
            } else {
                program.language_name
            };
            return Some(OracleAnswer {
                task_slug: task_slug.to_owned(),
                language_slug: slug,
                language_label: label,
                source: KnowledgeSource::DocumentationCapture,
                source_url: program.recipe.rediscovery_source,
                code: program.recipe.entry,
                expected_output: program.recipe.verified_output,
            });
        }
        let snippet = Self::lookup(task_slug, language)?;
        Some(OracleAnswer {
            task_slug: snippet.task_slug.to_owned(),
            language_slug: snippet.language_slug.to_owned(),
            language_label: snippet.language_label.to_owned(),
            source: snippet.source,
            source_url: snippet.source_url.to_owned(),
            code: snippet.code.to_owned(),
            expected_output: snippet.expected_output.to_owned(),
        })
    }

    /// Whether the oracle can answer for `language` (any task), used to decide
    /// when to generalise beyond the static catalogue.
    ///
    /// Issue #1165 R4 replaces the reading "a snapshot exists" with "a grammar
    /// exists and discovery found a procedure": while the bootstrap tier is
    /// active the snapshots are the record of discoveries that already ran, so
    /// they still answer; retiring the bootstrap in the policy seed leaves
    /// `crate::discovery_production::knows_language` — cache row plus grammar —
    /// as the authority.
    #[must_use]
    pub fn knows_language(language: &str) -> bool {
        if !bootstrap_cache_active() {
            return false;
        }
        let needle = language.trim().to_ascii_lowercase();
        if crate::discovery_production::language_has_documented_procedure(&needle) {
            return true;
        }
        ORACLE_SNAPSHOTS.iter().any(|snippet| {
            snippet.language_slug == needle || snippet.language_label.to_ascii_lowercase() == needle
        })
    }

    /// Distinct language labels the oracle covers that the built-in catalogue
    /// does not, in committed order, for diagnostics and the case study.
    #[must_use]
    pub fn languages() -> Vec<&'static str> {
        let mut labels: Vec<&'static str> = Vec::new();
        for snippet in ORACLE_SNAPSHOTS {
            if !labels.contains(&snippet.language_label) {
                labels.push(snippet.language_label);
            }
        }
        labels
    }

    /// Number of snapshots cached for `source`, for the cache-cap ratchet test.
    #[must_use]
    pub fn cached_count(source: KnowledgeSource) -> usize {
        ORACLE_SNAPSHOTS
            .iter()
            .filter(|snippet| snippet.source == source)
            .count()
    }
}
