//! CST/AST validation for generated programs via the meta-language links network.
//!
//! Coding handlers first build a language-independent semantic plan, render
//! concrete source from that plan, and then validate the source before
//! accepting it. The validation is delegated to the link-foundation
//! [meta-language](https://github.com/link-foundation/meta-language) component
//! (`meta_language::LinkNetwork`) — a single, mutable links-network
//! representation that is the sole CST/AST engine here. We do not re-implement
//! any CST/AST for the supported languages.
//!
//! meta-language ships real tree-sitter grammars for every language we target
//! (C, C++, C#, Go, Java, JavaScript, Kotlin, PHP, Python, R, Ruby, Rust,
//! Scala, Swift, TypeScript), so the same `LinkNetwork::parse` path validates
//! all of them. Which engine validates each language is recorded as data in
//! `data/seed/program-cst-grammars.lino`; this module is only the native
//! bridge from a seed language slug to the corresponding engine, plus the
//! compose→render→parse bridge ([`compose_and_validate`], issue #1167) that
//! lets composition happen in the meta language and render out.

use std::fmt::Write as _;

#[cfg(feature = "meta-language")]
use meta_language::{LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};

use crate::seed::PROGRAM_CST_GRAMMARS_LINO;
use crate::seed::parser::{LinoNode, parse_lino};

/// The slug recorded for languages validated through the meta-language network.
pub const META_LANGUAGE_ENGINE: &str = "meta_language";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstGrammar {
    pub language_slug: String,
    pub engine: String,
    pub meta_language_label: String,
    pub source_repository: String,
}

/// Engine-specific CST evidence captured after a successful parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CstEvidence {
    /// Parsed and verified through the meta-language links network.
    #[cfg_attr(not(feature = "meta-language"), allow(dead_code))]
    MetaLanguage {
        meta_language_label: String,
        syntax_link_count: usize,
        cst_link_count: usize,
        total_link_count: usize,
        text_preserved: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramCst {
    pub language_slug: String,
    pub source_repository: String,
    pub has_error: bool,
    pub evidence: CstEvidence,
}

impl ProgramCst {
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("cst_tree\n");
        let _ = writeln!(out, "  language {}", self.language_slug);
        match &self.evidence {
            CstEvidence::MetaLanguage {
                meta_language_label,
                syntax_link_count,
                cst_link_count,
                total_link_count,
                text_preserved,
            } => {
                let _ = writeln!(out, "  engine {META_LANGUAGE_ENGINE}");
                let _ = writeln!(out, "  component meta-language");
                let _ = writeln!(out, "  source_repository {}", self.source_repository);
                let _ = writeln!(out, "  language_label {meta_language_label}");
                let _ = writeln!(out, "  projection concrete_syntax");
                let _ = writeln!(out, "  syntax_link_count {syntax_link_count}");
                let _ = writeln!(out, "  cst_link_count {cst_link_count}");
                let _ = writeln!(out, "  total_link_count {total_link_count}");
                let _ = writeln!(out, "  has_error {}", self.has_error);
                let _ = writeln!(out, "  text_preserved {text_preserved}");
            }
        }
        out.trim_end().to_owned()
    }

    #[must_use]
    pub const fn is_valid(&self) -> bool {
        if self.has_error {
            return false;
        }
        match &self.evidence {
            CstEvidence::MetaLanguage {
                syntax_link_count,
                text_preserved,
                ..
            } => *syntax_link_count > 0 && *text_preserved,
        }
    }

    /// The engine slug that validated this program (`meta_language`).
    #[must_use]
    pub const fn engine(&self) -> &'static str {
        match self.evidence {
            CstEvidence::MetaLanguage { .. } => META_LANGUAGE_ENGINE,
        }
    }
}

#[must_use]
pub fn grammar_metadata(language_slug: &str) -> Option<CstGrammar> {
    let tree = parse_lino(PROGRAM_CST_GRAMMARS_LINO);
    let found = grammar_nodes(&tree).find(|node| {
        node.id == language_slug || node.find_child_value("program_language") == language_slug
    });
    found.map(grammar_from_node)
}

fn grammar_from_node(node: &LinoNode) -> CstGrammar {
    CstGrammar {
        language_slug: node.find_child_value("program_language").to_owned(),
        engine: node.find_child_value("engine").to_owned(),
        meta_language_label: node.find_child_value("meta_language_label").to_owned(),
        source_repository: node.find_child_value("source_repository").to_owned(),
    }
}

pub fn parse_program_cst(language_slug: &str, source: &str) -> Option<ProgramCst> {
    let grammar = grammar_metadata(language_slug)?;
    #[cfg(not(feature = "meta-language"))]
    let _ = source;
    match grammar.engine.as_str() {
        #[cfg(feature = "meta-language")]
        META_LANGUAGE_ENGINE => Some(parse_with_meta_language(&grammar, source)),
        #[cfg(not(feature = "meta-language"))]
        META_LANGUAGE_ENGINE => None,
        _ => None,
    }
}

pub fn validated_program_cst(language_slug: &str, source: &str) -> Option<ProgramCst> {
    let cst = parse_program_cst(language_slug, source)?;
    cst.is_valid().then_some(cst)
}

/// Parse `source` in `language_slug` and serialize the resulting links
/// network into the network dialect of lino — the composition-side half of
/// the render leg (issue #1167, R3).
///
/// `to_lino` is the dependency's documented lossless serialization
/// (`from_lino(to_lino(n))` is isomorphic for any network, spans included),
/// so this is the wire a composition travels on its way to
/// [`compose_and_validate`]. Returns `None` when the language has no grammar
/// entry or the engine is disabled.
#[must_use]
pub fn network_lino(language_slug: &str, source: &str) -> Option<String> {
    let grammar = grammar_metadata(language_slug)?;
    match grammar.engine.as_str() {
        #[cfg(feature = "meta-language")]
        META_LANGUAGE_ENGINE => Some(network_for(&grammar, source).to_lino()),
        #[cfg(not(feature = "meta-language"))]
        _ => {
            let _ = source;
            None
        }
        #[cfg(feature = "meta-language")]
        _ => None,
    }
}

/// The named reason a compose→render→parse round trip refused (issue #1167,
/// R1/R6): a gap is stated, never a silent skip, and CST inequality is a
/// refusal rather than a shrug.
#[allow(dead_code)] // which variants construct depends on the `meta-language` feature
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeGap {
    /// The render leg refused; the reason names
    /// [`crate::meta_translate::CstRenderGap`]'s finding.
    Render { reason: String },
    /// The rendered source did not parse into a valid CST for the language.
    SyntaxInvalid { language_slug: String },
    /// The rendered source parses, but its own network serialization differs
    /// from the composed one — the round trip lost or invented structure, so
    /// the composition is not accepted (R6).
    NotCstEqual { language_slug: String },
    /// The optional parsing engine is compiled out, so nothing can compose.
    EngineDisabled,
}

impl ComposeGap {
    /// The stable, human-readable name of the gap for error paths.
    #[must_use]
    // `coding` is crate-private and no in-crate path formats the gap yet; the
    // issue #1167 tests read it through the tests/source mirror.
    #[allow(dead_code)]
    pub fn describe(&self) -> String {
        match self {
            Self::Render { reason } => {
                crate::seed::report_text("compose_gap_render", &[("reason", reason)])
            }
            Self::SyntaxInvalid { language_slug } => crate::seed::report_text(
                "compose_gap_syntax_invalid",
                &[("language", language_slug)],
            ),
            Self::NotCstEqual { language_slug } => crate::seed::report_text(
                "compose_gap_not_cst_equal",
                &[("language", language_slug)],
            ),
            Self::EngineDisabled => {
                "the meta-language engine (feature `meta-language`) is disabled, so \
                 compose_and_validate cannot run"
                    .to_owned()
            }
        }
    }
}

/// Render a meta-language network to `language_slug` source and validate it
/// through the CST engine — the bridge that makes the render legs reachable
/// from the coding path (issue #1167, R3).
///
/// The input is the network serialization dialect of lino (the output of
/// [`network_lino`]); it renders through
/// [`crate::meta_translate::render_cst_source`], parses back through
/// [`parse_program_cst`], and — before success is returned — requires the
/// rendered source's own network serialization to equal the composed one,
/// which is the CST-equality assertion R6 puts in front of every
/// composition. Returns `None` when the language has no grammar entry, no
/// render leg, or the round trip is not CST-equal; the named gap is
/// available from [`try_compose_and_validate`].
#[must_use]
pub fn compose_and_validate(network_text: &str, language_slug: &str) -> Option<ProgramCst> {
    try_compose_and_validate(network_text, language_slug).ok()
}

/// The named-gap shape of [`compose_and_validate`].
pub fn try_compose_and_validate(
    network_text: &str,
    language_slug: &str,
) -> Result<ProgramCst, ComposeGap> {
    try_compose_and_validate_impl(network_text, language_slug)
}

#[cfg(feature = "meta-language")]
fn try_compose_and_validate_impl(
    network_text: &str,
    language_slug: &str,
) -> Result<ProgramCst, ComposeGap> {
    let rendered = crate::meta_translate::try_render_cst_source(network_text, language_slug)
        .map_err(|gap| ComposeGap::Render {
            reason: gap.describe(),
        })?;
    let cst =
        parse_program_cst(language_slug, &rendered).ok_or_else(|| ComposeGap::SyntaxInvalid {
            language_slug: language_slug.to_owned(),
        })?;
    if !cst.is_valid() {
        return Err(ComposeGap::SyntaxInvalid {
            language_slug: language_slug.to_owned(),
        });
    }
    // R6: the parsed network equals the composed one. The rendered source is
    // serialized back and compared byte-for-byte with the input wire, so any
    // structure the render lost or invented refuses the composition by name.
    let round_trip =
        network_lino(language_slug, &rendered).ok_or_else(|| ComposeGap::SyntaxInvalid {
            language_slug: language_slug.to_owned(),
        })?;
    if round_trip != network_text {
        return Err(ComposeGap::NotCstEqual {
            language_slug: language_slug.to_owned(),
        });
    }
    Ok(cst)
}

/// The engine-disabled shape: without the optional parsing engine the bridge
/// cannot run, and the honest answer is the named gap, not a guess.
#[cfg(not(feature = "meta-language"))]
fn try_compose_and_validate_impl(
    _network_text: &str,
    _language_slug: &str,
) -> Result<ProgramCst, ComposeGap> {
    Err(ComposeGap::EngineDisabled)
}

/// The meta-language label a grammar's language parses under: the recorded
/// label when present, the slug itself otherwise.
#[cfg(feature = "meta-language")]
const fn grammar_label(grammar: &CstGrammar) -> &str {
    if grammar.meta_language_label.is_empty() {
        grammar.language_slug.as_str()
    } else {
        grammar.meta_language_label.as_str()
    }
}

/// Parse `source` in the grammar's language into the links network — the one
/// parse every path here starts from, whether it validates
/// ([`parse_with_meta_language`]) or serializes ([`network_lino`]).
#[cfg(feature = "meta-language")]
fn network_for(grammar: &CstGrammar, source: &str) -> LinkNetwork {
    let label = grammar_label(grammar);
    LinkNetwork::parse(source, label, ParseConfiguration::default())
}

/// Validate `source` through the meta-language links network and capture the
/// resulting CST evidence. A real grammar parse populates `LinkType::Syntax`
/// links; the lossless text fallback does not, so `syntax_link_count` doubles as
/// a guard that meta-language really understood the language.
#[cfg(feature = "meta-language")]
fn parse_with_meta_language(grammar: &CstGrammar, source: &str) -> ProgramCst {
    let label = grammar_label(grammar);
    let network = network_for(grammar, source);
    let verification = network.verify_full_match(None);
    let syntax_link_count = network
        .projected_links(NetworkProjection::ConcreteSyntax)
        .filter(|link| link.metadata().link_type() == Some(LinkType::Syntax))
        .count();
    let cst_link_count = network
        .projected_links(NetworkProjection::ConcreteSyntax)
        .count();
    let total_link_count = network.len();
    let text_preserved = network.reconstruct_text() == source;
    ProgramCst {
        language_slug: grammar.language_slug.clone(),
        source_repository: grammar.source_repository.clone(),
        has_error: !verification.is_clean(),
        evidence: CstEvidence::MetaLanguage {
            meta_language_label: label.to_owned(),
            syntax_link_count,
            cst_link_count,
            total_link_count,
            text_preserved,
        },
    }
}

fn grammar_nodes(tree: &LinoNode) -> impl Iterator<Item = &LinoNode> {
    tree.children
        .first()
        .into_iter()
        .flat_map(|root| root.children.iter())
        .filter(|node| node.name == "cst_grammar")
}
