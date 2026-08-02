//! The grounded whole-system self-explanation (issue #558).
//!
//! Issue #558 ("Auto learning") asks that a user be able to *"ask how Formal AI
//! itself works"* and receive an answer *"grounded in its source and data"* rather
//! than in prose docs alone (`R558-08`). The other issue-#558 modules already turn
//! the system's own internals into auditable data: [`crate::self_source_links`]
//! content-addresses every owned source file, [`crate::self_healing`] captures a
//! failure as a `RepairCase`, and [`crate::learning_ledger`] records an approved
//! lesson. This module composes those into a single *explanation* that answers "how
//! does Formal AI work?" by citing the **real** artifacts each claim rests on.
//!
//! The grounding is enforced, not decorative: every [`CitationKind::Source`]
//! citation resolves its `content_id` from the compile-time owned manifest
//! ([`crate::self_source_links::owned_manifest`]) and *panics* if the cited path is
//! not an owned source file. It is therefore impossible to construct a
//! [`SystemExplanation`] that cites a source file the repository does not actually
//! ship — a fabricated citation fails to build the value. Data and test citations
//! are path references into the repository (`data/meta/*.lino`, `tests/**`) whose
//! on-disk existence is checked by the issue-#558 tests.
//!
//! Like [`crate::self_source_links`], the rendered explanation depends on the whole
//! source tree (its per-source `content_id`s and the manifest id change with every
//! edit), so it is a *workspace-only* artifact: never pinned byte-for-byte in a
//! committed `data/meta/*.lino`, only asserted live in tests. Neural inference stays
//! a NON-GOAL: the explanation is a deterministic function of the embedded source
//! and a fixed set of cited paths.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::self_source_links::{owned_file_count, owned_manifest, owned_manifest_content_id};

/// Which layer of the repository a [`Citation`] points at.
///
/// The three kinds mirror issue #558's requirement that a self-explanation cite
/// *"source, data, tests"* — not prose docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CitationKind {
    /// An owned Rust source file (`src/**/*.rs`), grounded against the owned manifest.
    Source,
    /// A generated data artifact (`data/meta/*.lino`).
    Data,
    /// A test that locks the cited behaviour (`tests/**`).
    Test,
}

impl CitationKind {
    /// A stable lower-case slug for the kind (used in the Links Notation artifact).
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Data => "data",
            Self::Test => "test",
        }
    }
}

/// One grounded reference backing a claim in the explanation: a repository path plus,
/// for source files, the content-addressed id proving the file is really in our data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    /// Which repository layer the citation points at.
    pub kind: CitationKind,
    /// The repository-relative path of the cited artifact.
    pub path: String,
    /// The content-addressed id of the artifact — `Some` for [`CitationKind::Source`]
    /// (resolved from the owned manifest), `None` for data/test path references.
    pub content_id: Option<String>,
}

impl Citation {
    /// A source citation, grounded against the owned manifest.
    ///
    /// # Panics
    ///
    /// Panics if `path` is not an owned `src/**/*.rs` file. This is deliberate: a
    /// [`SystemExplanation`] must never cite source the repository does not ship, so
    /// a fabricated or stale citation fails to construct rather than lying at runtime.
    #[must_use]
    pub fn source(path: &str) -> Self {
        let Some(digest) = owned_manifest()
            .into_iter()
            .find(|digest| digest.path == path)
        else {
            panic!("self-explanation cites a source file that is not in the owned manifest: {path}")
        };
        let content_id = digest.content_id;
        Self {
            kind: CitationKind::Source,
            path: path.to_owned(),
            content_id: Some(content_id),
        }
    }

    /// A data-artifact citation (`data/meta/*.lino`). Existence on disk is verified by
    /// the issue-#558 tests (the artifact is generated, not embedded in the binary).
    #[must_use]
    pub fn data(path: &str) -> Self {
        Self {
            kind: CitationKind::Data,
            path: path.to_owned(),
            content_id: None,
        }
    }

    /// A test citation (`tests/**`). Existence on disk is verified by the issue-#558
    /// tests.
    #[must_use]
    pub fn test(path: &str) -> Self {
        Self {
            kind: CitationKind::Test,
            path: path.to_owned(),
            content_id: None,
        }
    }
}

/// One topic of the explanation: a plain-language statement about how a part of
/// Formal AI works, plus the grounded citations it rests on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplanationSection {
    /// A short slug naming the topic (e.g. `deterministic_meta_algorithm`).
    pub topic: String,
    /// The grounded plain-language claim about how this part works.
    pub statement: String,
    /// The real artifacts the statement rests on (at least one).
    pub citations: Vec<Citation>,
}

impl ExplanationSection {
    /// Build a section from a topic slug, a statement, and its citations.
    #[must_use]
    pub fn new(topic: &str, statement: &str, citations: Vec<Citation>) -> Self {
        Self {
            topic: topic.to_owned(),
            statement: statement.to_owned(),
            citations,
        }
    }
}

/// A grounded answer to "how does Formal AI work?".
///
/// An ordered set of [`ExplanationSection`]s, each citing the real source/data/test
/// artifacts it rests on. Every source citation is verified against the owned manifest
/// at construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemExplanation {
    /// The explanation topics, in narrative order.
    pub sections: Vec<ExplanationSection>,
}

impl SystemExplanation {
    /// The canonical grounded explanation of how Formal AI works.
    ///
    /// Each section cites *real* artifacts: source files resolved through the owned
    /// manifest (so the `content_id`s are genuine), the generated data artifacts they
    /// emit, and the tests that lock the behaviour. Because [`Citation::source`]
    /// panics on an unknown path, this function is also a build-time assertion that
    /// every cited module still exists.
    #[must_use]
    pub fn canonical() -> Self {
        let sections = vec![
            ExplanationSection::new(
                "deterministic_meta_algorithm",
                "Formal AI answers agentic requests with a deterministic planner state machine: a \
                 keyword-routed recipe walk (write, verify, final answer), never neural inference, \
                 so every step is a reproducible function of its inputs.",
                vec![
                    Citation::source("src/agentic_coding/planner.rs"),
                    Citation::source("src/agentic_coding/mod.rs"),
                ],
            ),
            ExplanationSection::new(
                "source_to_links_round_trip",
                "The entire owned source tree is embedded as content-addressed data and projected \
                 through the single CST/AST engine (tree-sitter); every owned module round-trips \
                 source to links and back byte-for-byte, so the system's own code is present in its \
                 data as links.",
                vec![
                    Citation::source("src/self_source_links.rs"),
                    Citation::source("src/agentic_coding/self_ast.rs"),
                    Citation::test("tests/unit/issue_558_source_links.rs"),
                ],
            ),
            ExplanationSection::new(
                "self_healing_loop",
                "When Formal AI cannot answer, it does not simply fail: it captures the failure as a \
                 structured RepairCase (unknown trace, a verified source-to-links mapping, and a \
                 benchmark gate) that never advances past awaiting human review.",
                vec![
                    Citation::source("src/self_healing.rs"),
                    Citation::data("data/meta/self-healing-case.lino"),
                    Citation::test("tests/unit/issue_558_self_healing.rs"),
                ],
            ),
            ExplanationSection::new(
                "human_gated_promotion_ledger",
                "An approved lesson is promoted into a durable learning ledger only when the \
                 benchmark gate is green and a human approves; a repeated failure is then answered \
                 from the ledger instead of being re-derived, which is the concrete payoff of auto \
                 learning.",
                vec![
                    Citation::source("src/learning_ledger.rs"),
                    Citation::data("data/meta/learning-ledger.lino"),
                    Citation::test("tests/unit/issue_558_learning_ledger.rs"),
                ],
            ),
            ExplanationSection::new(
                "agentic_interface",
                "Every capability is reachable over the OpenAI-compatible agentic interface used by \
                 external CLIs (Codex, OpenCode, Gemini, Agent CLI), driven by the same deterministic \
                 planner and proven over the wire by server integration tests.",
                vec![
                    Citation::source("src/agentic_coding/driver.rs"),
                    Citation::test("tests/integration/issue_558_learning_ledger.rs"),
                ],
            ),
        ];
        Self { sections }
    }

    /// How many topics the explanation covers.
    #[must_use]
    pub const fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Every citation across every section, in document order.
    #[must_use]
    pub fn citations(&self) -> Vec<&Citation> {
        self.sections
            .iter()
            .flat_map(|section| section.citations.iter())
            .collect()
    }

    /// The total number of grounded citations.
    #[must_use]
    pub fn citation_count(&self) -> usize {
        self.sections.iter().map(|s| s.citations.len()).sum()
    }

    /// The citations of a given [`CitationKind`] across all sections.
    #[must_use]
    pub fn citations_of(&self, kind: CitationKind) -> Vec<&Citation> {
        self.citations()
            .into_iter()
            .filter(|citation| citation.kind == kind)
            .collect()
    }

    /// A one-line human-readable summary of the grounded explanation.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Explained how Formal AI works across {sections} grounded topics with {citations} \
             citations into its own source ({files} owned files), generated data, and tests.",
            sections = self.section_count(),
            citations = self.citation_count(),
            files = owned_file_count(),
        )
    }

    /// Render the grounded explanation as Links Notation — the auditable artifact of
    /// the whole-system self-explanation. Ends trimmed of trailing whitespace.
    ///
    /// The header ties the explanation to the source-to-links graph via
    /// [`owned_manifest_content_id`] (one id for the entire source tree), so the
    /// answer is explicitly anchored to the same data the round-trip proves lossless.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("system_explanation\n");
        let _ = writeln!(out, "  engine meta_language");
        let _ = writeln!(out, "  question \"how does Formal AI work?\"");
        let _ = writeln!(out, "  source_file_count {}", owned_file_count());
        let _ = writeln!(
            out,
            "  source_manifest_content_id \"{}\"",
            owned_manifest_content_id()
        );
        let _ = writeln!(out, "  section_count {}", self.section_count());
        let _ = writeln!(out, "  citation_count {}", self.citation_count());
        let _ = writeln!(out, "  sections");
        for section in &self.sections {
            let _ = writeln!(out, "    section");
            let _ = writeln!(out, "      topic \"{}\"", quote(&section.topic));
            let _ = writeln!(out, "      statement \"{}\"", quote(&section.statement));
            let _ = writeln!(out, "      citations");
            for citation in &section.citations {
                let _ = writeln!(out, "        citation");
                let _ = writeln!(out, "          kind {}", citation.kind.slug());
                let _ = writeln!(out, "          path \"{}\"", quote(&citation.path));
                if let Some(content_id) = &citation.content_id {
                    let _ = writeln!(out, "          content_id \"{}\"", quote(content_id));
                }
            }
        }
        out.trim_end().to_owned()
    }

    /// A single stable content-addressed id for the whole grounded explanation.
    #[must_use]
    pub fn content_id(&self) -> String {
        stable_id("system_explanation", &self.links_notation())
    }
}

/// The canonical grounded explanation of how Formal AI works.
///
/// Convenience wrapper over [`SystemExplanation::canonical`] mirroring
/// [`crate::self_healing::canonical_case`] and [`crate::learning_ledger::canonical_ledger`].
#[must_use]
pub fn canonical_explanation() -> SystemExplanation {
    SystemExplanation::canonical()
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
