//! Public knowledge-source identifiers, cache size policy and coding discovery.
//!
//! The distribution registers sources. Coding examples are resolved from
//! publisher bytes acquired on demand or verified in the runtime source cache.

/// The distribution carries no pre-populated procedure cache.
#[must_use]
pub const fn bootstrap_cache_active() -> bool {
    false
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

/// A coding example decomposed from publisher documentation.
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

/// Compatibility API for callers of the former bundled example lookup.
pub struct CodingOracle;

impl CodingOracle {
    #[must_use]
    pub const fn snapshots() -> &'static [OracleSnippet] {
        &[]
    }

    #[must_use]
    pub fn lookup(_task_slug: &str, _language: &str) -> Option<&'static OracleSnippet> {
        None
    }

    /// Resolve a coding example from previously verified publisher bytes.
    #[must_use]
    pub fn answer(task_slug: &str, language: &str) -> Option<OracleAnswer> {
        let program = crate::discovery_production::documented_oracle_program(task_slug, language)?;
        let slug = language.trim().to_ascii_lowercase();
        let label = if program.language_name.is_empty() {
            slug.clone()
        } else {
            program.language_name
        };
        Some(OracleAnswer {
            task_slug: task_slug.to_owned(),
            language_slug: slug,
            language_label: label,
            source: KnowledgeSource::DocumentationCapture,
            source_url: program.recipe.rediscovery_source,
            code: program.recipe.entry,
            expected_output: program.recipe.verified_output,
        })
    }

    #[must_use]
    pub fn knows_language(language: &str) -> bool {
        crate::discovery_production::language_has_documented_procedure(language)
    }

    /// Languages represented in the legacy bundled example cache.
    #[must_use]
    pub fn languages() -> Vec<&'static str> {
        Vec::new()
    }

    #[must_use]
    pub fn cached_count(_source: KnowledgeSource) -> usize {
        0
    }
}
