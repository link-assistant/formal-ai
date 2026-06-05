//! Real Tree-sitter CST validation for generated programs.
//!
//! Coding handlers first build a language-independent semantic plan, render
//! source from that plan, and then pass the concrete source through the real
//! Tree-sitter Rust binding before accepting it. The grammar metadata lives in
//! `data/seed/meanings-program-cst.lino`; this module is only the native bridge
//! from a seed language slug to the corresponding compiled grammar crate.

use std::fmt::Write as _;

use tree_sitter::{Language, Parser};

use crate::seed::parser::{parse_lino, LinoNode};
use crate::seed::PROGRAM_CST_GRAMMARS_LINO;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstGrammar {
    pub language_slug: String,
    pub grammar_crate: String,
    pub grammar_symbol: String,
    pub expected_root_kind: String,
    pub source_repository: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramCst {
    pub language_slug: String,
    pub grammar_crate: String,
    pub grammar_symbol: String,
    pub expected_root_kind: String,
    pub root_kind: String,
    pub named_child_count: usize,
    pub has_error: bool,
    pub sexp: String,
}

impl ProgramCst {
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("tree_sitter_cst_tree\n");
        let _ = writeln!(out, "  language {}", self.language_slug);
        let _ = writeln!(out, "  parser tree_sitter");
        let _ = writeln!(out, "  binding tree-sitter/lib/binding_rust");
        let _ = writeln!(out, "  grammar_crate {}", self.grammar_crate);
        let _ = writeln!(out, "  grammar_symbol {}", self.grammar_symbol);
        let _ = writeln!(out, "  expected_root_kind {}", self.expected_root_kind);
        let _ = writeln!(out, "  root_kind {}", self.root_kind);
        let _ = writeln!(out, "  named_child_count {}", self.named_child_count);
        let _ = writeln!(out, "  has_error {}", self.has_error);
        let _ = writeln!(out, "  sexp {:?}", self.sexp);
        out.trim_end().to_owned()
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.has_error && self.root_kind == self.expected_root_kind
    }
}

#[must_use]
pub fn grammar_metadata(language_slug: &str) -> Option<CstGrammar> {
    let tree = parse_lino(PROGRAM_CST_GRAMMARS_LINO);
    let result = grammar_nodes(&tree)
        .find(|node| {
            node.id == language_slug || node.find_child_value("program_language") == language_slug
        })
        .map(|node| CstGrammar {
            language_slug: node.find_child_value("program_language").to_owned(),
            grammar_crate: node.find_child_value("grammar_crate").to_owned(),
            grammar_symbol: node.find_child_value("grammar_symbol").to_owned(),
            expected_root_kind: node.find_child_value("root_kind").to_owned(),
            source_repository: node.find_child_value("source_repository").to_owned(),
        });
    result
}

#[cfg(test)]
pub fn grammar_languages() -> Vec<CstGrammar> {
    let tree = parse_lino(PROGRAM_CST_GRAMMARS_LINO);
    grammar_nodes(&tree)
        .map(|node| CstGrammar {
            language_slug: node.find_child_value("program_language").to_owned(),
            grammar_crate: node.find_child_value("grammar_crate").to_owned(),
            grammar_symbol: node.find_child_value("grammar_symbol").to_owned(),
            expected_root_kind: node.find_child_value("root_kind").to_owned(),
            source_repository: node.find_child_value("source_repository").to_owned(),
        })
        .collect()
}

pub fn parse_program_cst(language_slug: &str, source: &str) -> Option<ProgramCst> {
    let grammar = grammar_metadata(language_slug)?;
    let language = tree_sitter_language(language_slug)?;
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    let root = tree.root_node();
    Some(ProgramCst {
        language_slug: grammar.language_slug,
        grammar_crate: grammar.grammar_crate,
        grammar_symbol: grammar.grammar_symbol,
        expected_root_kind: grammar.expected_root_kind,
        root_kind: root.kind().to_owned(),
        named_child_count: root.named_child_count(),
        has_error: root.has_error(),
        sexp: root.to_sexp(),
    })
}

pub fn validated_program_cst(language_slug: &str, source: &str) -> Option<ProgramCst> {
    let cst = parse_program_cst(language_slug, source)?;
    cst.is_valid().then_some(cst)
}

fn grammar_nodes(tree: &LinoNode) -> impl Iterator<Item = &LinoNode> {
    tree.children
        .first()
        .into_iter()
        .flat_map(|root| root.children.iter())
        .filter(|node| node.name == "cst_grammar")
}

fn tree_sitter_language(language_slug: &str) -> Option<Language> {
    match language_slug {
        "javascript" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "ruby" => Some(tree_sitter_ruby::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "csharp" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        "c" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coding::{program_language_by_slug, PROGRAM_LANGUAGES};

    #[test]
    fn every_catalog_language_has_tree_sitter_metadata() {
        for language in PROGRAM_LANGUAGES {
            assert!(
                grammar_metadata(language.slug).is_some(),
                "`{}` must have Tree-sitter CST metadata",
                language.slug
            );
        }
    }

    #[test]
    fn every_tree_sitter_metadata_entry_names_a_catalog_language() {
        for grammar in grammar_languages() {
            assert!(
                program_language_by_slug(&grammar.language_slug).is_some(),
                "`{}` metadata names an unknown language",
                grammar.language_slug
            );
        }
    }

    #[test]
    fn every_catalog_language_loads_tree_sitter_parser() {
        for grammar in grammar_languages() {
            let language = tree_sitter_language(&grammar.language_slug)
                .unwrap_or_else(|| panic!("`{}` has no grammar binding", grammar.language_slug));
            let mut parser = Parser::new();
            parser
                .set_language(&language)
                .unwrap_or_else(|error| panic!("{}: {error:?}", grammar.language_slug));
        }
    }

    #[test]
    fn javascript_source_parses_to_real_tree_sitter_cst() {
        let cst = parse_program_cst(
            "javascript",
            "const numbers = [3, 1, 2];\nconsole.log(numbers.sort((a, b) => a - b));",
        )
        .expect("JavaScript source must produce a Tree-sitter CST");

        assert!(cst.is_valid(), "{cst:#?}");
        assert_eq!(cst.root_kind, "program");
        assert!(!cst.has_error);
        assert!(cst.sexp.contains("lexical_declaration"));
    }
}
