//! Offline concept knowledge base used by the universal solver's
//! `try_concept_lookup` handler. Records are compiled into the binary so the
//! solver can answer "what is X?" prompts without a network round trip, and
//! every record cites its source so the answer remains auditable.

/// A single record in the offline concept knowledge base. The solver looks
/// these up from the `CONCEPTS` static table when answering "what is X?" style
/// prompts before any network call is considered.
#[derive(Debug, Clone, Copy)]
pub struct ConceptRecord {
    pub slug: &'static str,
    pub term: &'static str,
    pub aliases: &'static [&'static str],
    pub category: &'static str,
    pub summary: &'static str,
    pub source: &'static str,
    pub source_kind: &'static str,
}

/// Offline concept knowledge base loaded at compile time from
/// `data/seed/concepts.lino`. Records here ground "what is X?" answers in
/// inspectable text instead of fabricated prose.
pub const CONCEPTS: &[ConceptRecord] = &[
    ConceptRecord {
        slug: "concept_universal_solver",
        term: "universal solver",
        aliases: &["the universal solver", "universal problem solver"],
        category: "algorithm",
        summary: "The universal solver is formal-ai's deterministic 11-step \
                  loop: impulse, formalization, context, history, decomposition, \
                  TDD, synthesis, combination, verification, simplification, \
                  documentation. Every interface routes through the same loop.",
        source: "docs/case-studies/issue-12/README.md",
        source_kind: "project-docs",
    },
    ConceptRecord {
        slug: "concept_event_log",
        term: "event log",
        aliases: &["the event log", "eventlog", "append-only log"],
        category: "data-structure",
        summary: "The event log is formal-ai's append-only system of record. \
                  Every step in the universal solver loop appends an Event with \
                  a stable content-addressed id; the user-facing answer is, by \
                  construction, a projection of this log.",
        source: "docs/NON-GOALS.md",
        source_kind: "project-docs",
    },
    ConceptRecord {
        slug: "concept_links_notation",
        term: "Links Notation",
        aliases: &["links notation", "lino", "the links notation format"],
        category: "data-format",
        summary: "Links Notation is an indentation-based, untyped serialization \
                  format used by the Deep Theory project to represent links and \
                  link networks as portable text.",
        source: "https://github.com/linksplatform/Documentation",
        source_kind: "project-docs",
    },
    ConceptRecord {
        slug: "concept_doublet",
        term: "doublet",
        aliases: &["doublet link", "a doublet", "two-link"],
        category: "data-structure",
        summary: "A doublet is a link with exactly two endpoints. In Deep \
                  Theory it is the canonical reduction target for higher-arity \
                  links because every higher arity can be encoded as a chain of \
                  doublets.",
        source: "docs/VISION.md",
        source_kind: "project-docs",
    },
    ConceptRecord {
        slug: "concept_wikipedia",
        term: "Wikipedia",
        aliases: &["wikipedia", "the wikipedia", "en.wikipedia"],
        category: "encyclopedia",
        summary: "Wikipedia is a free, multilingual online encyclopedia \
                  written and maintained by a community of volunteer \
                  contributors through a model of open collaboration.",
        source: "https://en.wikipedia.org/wiki/Wikipedia",
        source_kind: "wikipedia",
    },
    ConceptRecord {
        slug: "concept_wikidata",
        term: "Wikidata",
        aliases: &["wikidata", "the wikidata knowledge graph"],
        category: "structured-knowledge",
        summary: "Wikidata is a collaboratively edited multilingual knowledge \
                  graph hosted by the Wikimedia Foundation. It stores \
                  structured data items that power Wikipedia infoboxes and \
                  external knowledge applications.",
        source: "https://en.wikipedia.org/wiki/Wikidata",
        source_kind: "wikipedia",
    },
    ConceptRecord {
        slug: "concept_wiktionary",
        term: "Wiktionary",
        aliases: &["wiktionary", "the wiktionary dictionary"],
        category: "dictionary",
        summary: "Wiktionary is a multilingual, web-based free-content \
                  dictionary, available in many languages and including \
                  thesaurus, rhymes, translations, audio pronunciations, \
                  etymologies, and definitions.",
        source: "https://en.wikipedia.org/wiki/Wiktionary",
        source_kind: "wikipedia",
    },
    ConceptRecord {
        slug: "concept_webassembly",
        term: "WebAssembly",
        aliases: &["webassembly", "wasm", "the wasm runtime"],
        category: "runtime",
        summary: "WebAssembly (Wasm) is a binary instruction format for a \
                  stack-based virtual machine. It is designed as a portable \
                  compilation target for programming languages, enabling \
                  deployment on the web for client and server applications.",
        source: "https://en.wikipedia.org/wiki/WebAssembly",
        source_kind: "wikipedia",
    },
    ConceptRecord {
        slug: "concept_rust",
        term: "Rust",
        aliases: &[
            "rust",
            "rust programming language",
            "the rust language",
            "rust-lang",
        ],
        category: "programming-language",
        summary: "Rust is a multi-paradigm, general-purpose programming \
                  language that emphasises performance, type safety, and \
                  concurrency. It enforces memory safety without using a \
                  garbage collector.",
        source: "https://en.wikipedia.org/wiki/Rust_(programming_language)",
        source_kind: "wikipedia",
    },
];

/// Extract the concept term from a "what is X" style prompt. Returns `None`
/// when the prompt does not look like a definition request, which lets the
/// solver fall through to other handlers (greeting, arithmetic, etc.).
pub fn extract_concept_term(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let trimmed = lower.trim();
    let prefixes = [
        "what is a ",
        "what is an ",
        "what is the ",
        "what is ",
        "what's a ",
        "what's an ",
        "what's the ",
        "what's ",
        "what does ",
        "tell me about ",
        "tell me what ",
        "define ",
        "explain ",
        "describe ",
        "who is ",
        "who was ",
    ];
    let mut body: Option<&str> = None;
    for prefix in prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            body = Some(rest);
            break;
        }
    }
    let body = body?;
    let body = body.trim_end_matches(['?', '.', '!', ',', ';', ':']).trim();
    if body.is_empty() {
        return None;
    }
    let body = body
        .strip_suffix(" mean")
        .or_else(|| body.strip_suffix(" stand for"))
        .unwrap_or(body)
        .trim();
    if body.is_empty() {
        return None;
    }
    Some(body.to_owned())
}

/// Look up a concept by term, alias, or slug. Comparison is case-insensitive
/// and ignores leading articles ("the", "a", "an") so "the universal solver"
/// matches "universal solver".
#[must_use]
pub fn lookup_concept(term: &str) -> Option<&'static ConceptRecord> {
    let normalized = normalize_concept_term(term);
    if normalized.is_empty() {
        return None;
    }
    CONCEPTS.iter().find(|record| {
        if normalize_concept_term(record.term) == normalized
            || normalize_concept_term(record.slug) == normalized
        {
            return true;
        }
        record
            .aliases
            .iter()
            .any(|alias| normalize_concept_term(alias) == normalized)
    })
}

fn normalize_concept_term(value: &str) -> String {
    let lower = value.to_lowercase();
    let mut stripped = lower.as_str();
    for prefix in ["the ", "a ", "an "] {
        if let Some(rest) = stripped.strip_prefix(prefix) {
            stripped = rest;
            break;
        }
    }
    stripped
        .trim()
        .trim_end_matches(['?', '.', '!', ',', ';', ':'])
        .trim()
        .to_owned()
}
