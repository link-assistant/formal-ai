//! External knowledge oracles, treated as cached APIs (issue #412).
//!
//! The universal solver should not depend on a fixed, hand-written catalogue of
//! coding answers. Instead it treats public knowledge bases — Rosetta Code,
//! Wikifunctions, the Hello World Collection, and Stack Overflow — as external
//! APIs, even when they expose no machine API: a reviewed snippet plus its
//! deterministic output and source attribution is kept as a *cached* example so
//! the answer is served offline, exactly the offline-first contract the
//! Wikidata/Wiktionary caches provide. This cache of popular cases is embedded
//! ([`ORACLE_SNAPSHOTS`]) so it compiles into both the native binary and the
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
//!    attribution, generalising the static [`crate::coding`] catalogue beyond
//!    its built-in languages (R6). The committed snapshots are the popular-case
//!    cache; a gated live-refresh path (mirroring the existing
//!    `FORMAL_AI_LIVE_API` discipline) repopulates them from the live sources.
//!
//! The data is plain Rust so it compiles into both the native binary and the
//! Rust→WASM browser worker without a runtime fetch, and is mirrored verbatim in
//! `src/web/formal_ai_worker.js` so every reasoning surface agrees byte-for-byte.

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
/// `div_ceil` is avoided to keep the crate buildable on the declared MSRV
/// (Rust 1.70), so the 1% is computed as `(source_total + 99) / 100`.
#[must_use]
pub const fn cache_capacity(source_total: usize) -> usize {
    let one_percent = (source_total + 99) / 100;
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
/// These are the "Hello, World!" programs for languages the built-in
/// [`crate::coding::catalog`] does not template (Kotlin, Swift, PHP, Bash, Lua,
/// Haskell), plus a Rosetta-Code factorial in Kotlin to exercise a non-trivial
/// task. The set is intentionally tiny — well under [`cache_capacity`] for every
/// source — and is the offline accelerator a live refresh would repopulate.
const ORACLE_SNAPSHOTS: &[OracleSnippet] = &[
    OracleSnippet {
        task_slug: "hello_world",
        language_slug: "kotlin",
        language_label: "Kotlin",
        source: KnowledgeSource::HelloWorldCollection,
        source_url: "http://helloworldcollection.de/#Kotlin",
        code: "fun main() {\n    println(\"Hello, World!\")\n}",
        expected_output: "Hello, World!",
    },
    OracleSnippet {
        task_slug: "hello_world",
        language_slug: "swift",
        language_label: "Swift",
        source: KnowledgeSource::HelloWorldCollection,
        source_url: "http://helloworldcollection.de/#Swift",
        code: "print(\"Hello, World!\")",
        expected_output: "Hello, World!",
    },
    OracleSnippet {
        task_slug: "hello_world",
        language_slug: "php",
        language_label: "PHP",
        source: KnowledgeSource::HelloWorldCollection,
        source_url: "http://helloworldcollection.de/#PHP",
        code: "<?php\necho \"Hello, World!\\n\";",
        expected_output: "Hello, World!",
    },
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
        language_slug: "lua",
        language_label: "Lua",
        source: KnowledgeSource::HelloWorldCollection,
        source_url: "http://helloworldcollection.de/#Lua",
        code: "print(\"Hello, World!\")",
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
    /// no cached answer — the caller then stays on its existing path (the static
    /// catalogue or, ultimately, the `unknown` opener), so this is purely
    /// additive.
    #[must_use]
    pub fn lookup(task_slug: &str, language: &str) -> Option<&'static OracleSnippet> {
        let needle = language.trim().to_ascii_lowercase();
        ORACLE_SNAPSHOTS.iter().find(|snippet| {
            snippet.task_slug == task_slug
                && (snippet.language_slug == needle
                    || snippet.language_label.to_ascii_lowercase() == needle)
        })
    }

    /// Whether the oracle can answer for `language` (any task), used to decide
    /// when to generalise beyond the static catalogue.
    #[must_use]
    pub fn knows_language(language: &str) -> bool {
        let needle = language.trim().to_ascii_lowercase();
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

#[path = "source_tests/knowledge/tests.rs"]
mod tests;
