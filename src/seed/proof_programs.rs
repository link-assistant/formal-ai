//! Proof statement and executable-program presentations loaded from seed data.

use std::sync::OnceLock;

use super::PROOF_PROGRAM_TEMPLATES_LINO;
use super::parser::parse_lino;

/// Seed-defined projections for one programming language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofLanguageTemplates {
    pub slug: String,
    pub satisfiable: String,
    pub unsatisfiable: String,
}

/// Seed-defined presentations of a language-neutral formal proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofProgramTemplates {
    pub statement: String,
    pub languages: Vec<ProofLanguageTemplates>,
}

impl ProofProgramTemplates {
    #[must_use]
    pub fn language(&self, slug: &str) -> Option<&ProofLanguageTemplates> {
        self.languages.iter().find(|language| language.slug == slug)
    }
}

/// Return the proof projection catalog parsed once from Links Notation.
#[must_use]
pub fn proof_program_templates() -> &'static ProofProgramTemplates {
    static TEMPLATES: OnceLock<ProofProgramTemplates> = OnceLock::new();
    TEMPLATES.get_or_init(parse_proof_program_templates)
}

fn parse_proof_program_templates() -> ProofProgramTemplates {
    let tree = parse_lino(PROOF_PROGRAM_TEMPLATES_LINO);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "proof_program_templates")
        .expect("data/seed/proof-program-templates.lino must declare proof_program_templates");
    let statement = root.find_child_value("statement").to_owned();
    assert!(
        !statement.is_empty(),
        "proof statement template is required"
    );
    let languages = root
        .children
        .iter()
        .filter(|node| node.name == "language")
        .map(|node| ProofLanguageTemplates {
            slug: node.id.clone(),
            satisfiable: node.find_child_value("satisfiable").to_owned(),
            unsatisfiable: node.find_child_value("unsatisfiable").to_owned(),
        })
        .collect();
    ProofProgramTemplates {
        statement,
        languages,
    }
}
