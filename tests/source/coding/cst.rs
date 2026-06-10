//! CST/AST validation for generated programs via the meta-language links network.
//!
//! Coding handlers first build a language-independent semantic plan, render
//! concrete source from that plan, and then validate the source before
//! accepting it. The validation is delegated to the link-foundation
//! [meta-language](https://github.com/link-foundation/meta-language) component
//! (`meta_language::LinkNetwork`) — a single, mutable links-network
//! representation that is the primary CST/AST engine here. We do not
//! re-implement any CST/AST for the supported languages.
//!
//! meta-language ships real tree-sitter grammars for most of our targets
//! (JavaScript, Python, Rust, Java, C, C++, C#). For the few it does not cover
//! yet — TypeScript, Go, and Ruby — we keep a thin direct tree-sitter bridge and
//! track the gap upstream. Which engine validates each language, and the
//! upstream request for the bridged ones, is recorded as data in
//! `data/seed/program-cst-grammars.lino`; this module is only the native bridge
//! from a seed language slug to the corresponding engine.

use std::fmt::Write as _;

use meta_language::{LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};
use tree_sitter::{Language, Parser};

use crate::seed::parser::{parse_lino, LinoNode};
use crate::seed::PROGRAM_CST_GRAMMARS_LINO;

/// The slug recorded for languages validated through the meta-language network.
pub const META_LANGUAGE_ENGINE: &str = "meta_language";
/// The slug recorded for languages still validated through a direct tree-sitter
/// grammar while meta-language gains coverage.
pub const TREE_SITTER_BRIDGE_ENGINE: &str = "tree_sitter_bridge";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstGrammar {
    pub language_slug: String,
    pub engine: String,
    pub meta_language_label: String,
    pub grammar_crate: String,
    pub grammar_symbol: String,
    pub expected_root_kind: String,
    pub source_repository: String,
    pub upstream_grammar_request: String,
}

/// Engine-specific CST evidence captured after a successful parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CstEvidence {
    /// Parsed and verified through the meta-language links network.
    MetaLanguage {
        meta_language_label: String,
        syntax_link_count: usize,
        cst_link_count: usize,
        total_link_count: usize,
        text_preserved: bool,
    },
    /// Parsed through a direct tree-sitter grammar bridge (languages
    /// meta-language does not cover yet).
    TreeSitterBridge {
        grammar_crate: String,
        grammar_symbol: String,
        expected_root_kind: String,
        root_kind: String,
        named_child_count: usize,
        upstream_grammar_request: String,
        sexp: String,
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
            CstEvidence::TreeSitterBridge {
                grammar_crate,
                grammar_symbol,
                expected_root_kind,
                root_kind,
                named_child_count,
                upstream_grammar_request,
                sexp,
            } => {
                let _ = writeln!(out, "  engine {TREE_SITTER_BRIDGE_ENGINE}");
                let _ = writeln!(out, "  component tree-sitter");
                let _ = writeln!(out, "  source_repository {}", self.source_repository);
                let _ = writeln!(out, "  grammar_crate {grammar_crate}");
                let _ = writeln!(out, "  grammar_symbol {grammar_symbol}");
                let _ = writeln!(out, "  upstream_grammar_request {upstream_grammar_request}");
                let _ = writeln!(out, "  expected_root_kind {expected_root_kind}");
                let _ = writeln!(out, "  root_kind {root_kind}");
                let _ = writeln!(out, "  named_child_count {named_child_count}");
                let _ = writeln!(out, "  has_error {}", self.has_error);
                let _ = writeln!(out, "  sexp {sexp:?}");
            }
        }
        out.trim_end().to_owned()
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.has_error {
            return false;
        }
        match &self.evidence {
            CstEvidence::MetaLanguage {
                syntax_link_count,
                text_preserved,
                ..
            } => *syntax_link_count > 0 && *text_preserved,
            CstEvidence::TreeSitterBridge {
                expected_root_kind,
                root_kind,
                ..
            } => root_kind == expected_root_kind,
        }
    }

    /// The engine slug that validated this program (`meta_language` or
    /// `tree_sitter_bridge`).
    #[must_use]
    pub const fn engine(&self) -> &'static str {
        match self.evidence {
            CstEvidence::MetaLanguage { .. } => META_LANGUAGE_ENGINE,
            CstEvidence::TreeSitterBridge { .. } => TREE_SITTER_BRIDGE_ENGINE,
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
        grammar_crate: node.find_child_value("grammar_crate").to_owned(),
        grammar_symbol: node.find_child_value("grammar_symbol").to_owned(),
        expected_root_kind: node.find_child_value("root_kind").to_owned(),
        source_repository: node.find_child_value("source_repository").to_owned(),
        upstream_grammar_request: node.find_child_value("upstream_grammar_request").to_owned(),
    }
}

pub fn parse_program_cst(language_slug: &str, source: &str) -> Option<ProgramCst> {
    let grammar = grammar_metadata(language_slug)?;
    match grammar.engine.as_str() {
        META_LANGUAGE_ENGINE => Some(parse_with_meta_language(&grammar, source)),
        TREE_SITTER_BRIDGE_ENGINE => parse_with_tree_sitter_bridge(&grammar, source),
        _ => None,
    }
}

pub fn validated_program_cst(language_slug: &str, source: &str) -> Option<ProgramCst> {
    let cst = parse_program_cst(language_slug, source)?;
    cst.is_valid().then_some(cst)
}

/// Validate `source` through the meta-language links network and capture the
/// resulting CST evidence. A real grammar parse populates `LinkType::Syntax`
/// links; the lossless text fallback does not, so `syntax_link_count` doubles as
/// a guard that meta-language really understood the language.
fn parse_with_meta_language(grammar: &CstGrammar, source: &str) -> ProgramCst {
    let label = if grammar.meta_language_label.is_empty() {
        grammar.language_slug.as_str()
    } else {
        grammar.meta_language_label.as_str()
    };
    let network = LinkNetwork::parse(source, label, ParseConfiguration::default());
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

/// Validate `source` through a direct tree-sitter grammar for a language
/// meta-language does not cover yet. Tracked upstream via
/// `grammar.upstream_grammar_request`.
fn parse_with_tree_sitter_bridge(grammar: &CstGrammar, source: &str) -> Option<ProgramCst> {
    let language = tree_sitter_bridge_language(&grammar.language_slug)?;
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    let root = tree.root_node();
    Some(ProgramCst {
        language_slug: grammar.language_slug.clone(),
        source_repository: grammar.source_repository.clone(),
        has_error: root.has_error(),
        evidence: CstEvidence::TreeSitterBridge {
            grammar_crate: grammar.grammar_crate.clone(),
            grammar_symbol: grammar.grammar_symbol.clone(),
            expected_root_kind: grammar.expected_root_kind.clone(),
            root_kind: root.kind().to_owned(),
            named_child_count: root.named_child_count(),
            upstream_grammar_request: grammar.upstream_grammar_request.clone(),
            sexp: root.to_sexp(),
        },
    })
}

fn grammar_nodes(tree: &LinoNode) -> impl Iterator<Item = &LinoNode> {
    tree.children
        .first()
        .into_iter()
        .flat_map(|root| root.children.iter())
        .filter(|node| node.name == "cst_grammar")
}

fn tree_sitter_bridge_language(language_slug: &str) -> Option<Language> {
    match language_slug {
        "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "ruby" => Some(tree_sitter_ruby::LANGUAGE.into()),
        _ => None,
    }
}

#[path = "../source_tests/coding/cst/tests.rs"]
mod tests;
