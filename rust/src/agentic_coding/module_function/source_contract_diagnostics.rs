//! Source-owned diagnostic rendering; no request or effect authorization.
use crate::seed::parser::parse_lino;
const MEANINGS: &str =
    include_str!("../../../embedded/data/seed/meanings-source-authoring-grammar.lino");
pub fn source_contract_diagnostic(code: &str, language: &str) -> Option<String> {
    let root = parse_lino(MEANINGS);
    let meanings: Vec<_> = root
        .children
        .iter()
        .filter(|node| node.name == "meanings")
        .collect();
    if meanings.len() != 1 {
        return None;
    }
    let records: Vec<_> = meanings[0]
        .children
        .iter()
        .filter(|node| {
            node.name == code && node.find_child_value("role") == "source-contract-diagnostic"
        })
        .collect();
    if records.len() != 1 {
        return None;
    }
    let lexemes: Vec<_> = records[0]
        .children
        .iter()
        .filter(|node| node.name == "lexeme" && node.id == language)
        .collect();
    if lexemes.len() != 1 {
        return None;
    }
    let surfaces: Vec<_> = lexemes[0]
        .children
        .iter()
        .filter(|node| node.name == "surface")
        .collect();
    if surfaces.len() != 1 {
        return None;
    }
    let text = surfaces[0].find_child_value("text");
    (!text.is_empty()).then(|| text.to_owned())
}
