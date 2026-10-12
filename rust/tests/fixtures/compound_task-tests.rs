#![cfg(test)]

use super::*;
fn grammar() -> LinoNode {
    let forms = [
        (
            "private-constant-rename",
            "^(?<label>😀)?a(?<sourcePath>[a-z]+)(?<previous>OLD)(?<replacement>NEW)",
        ),
        ("tracked-owner-condition", "^b(?<trackedPath>[a-z]+)"),
        (
            "exact-field-receipt",
            "^c(?<receiptPath>[a-z]+)(?<requiredIdentifier>NEW)",
        ),
        ("proof-target", "^d(?<proofPath>[a-z]+)"),
    ];
    LinoNode {
        name: "compound-task-grammar".to_owned(),
        id: String::new(),
        children: forms
            .into_iter()
            .map(|(kind, pattern)| LinoNode {
                name: "form".to_owned(),
                id: String::new(),
                children: vec![
                    LinoNode {
                        name: "kind".to_owned(),
                        id: kind.to_owned(),
                        children: vec![],
                    },
                    LinoNode {
                        name: "pattern".to_owned(),
                        id: pattern.to_owned(),
                        children: vec![],
                    },
                ],
            })
            .collect(),
    }
}
#[test]
fn complete_source_and_utf16_optional_capture() {
    let plain = compile_compound_task("asrcOLDNEW bsrc creceiptNEW dproof", &grammar()).unwrap();
    assert_eq!(plain.clauses[0].captures["label"].text, None);
    assert_eq!(plain.clauses[0].captures["label"].span, None);
    let astral = compile_compound_task("😀asrcOLDNEW bsrc creceiptNEW dproof", &grammar()).unwrap();
    assert_eq!(astral.clauses[0].captures["label"].span, Some([0, 2]));
    assert!(!astral.write_authority);
    assert_eq!(astral.unresolved_needs.len(), 4);
}
#[test]
fn unknown_tail_duplicate_kind_and_collision_refuse() {
    assert!(
        compile_compound_task(
            include_str!("../../tests/fixtures/compound-task/native-span-request.txt"),
            &grammar()
        )
        .is_none()
    );
    assert!(compile_compound_task("asrcOLDNEW bother creceiptNEW dproof", &grammar()).is_none());
    assert!(compile_compound_task("asrcOLDNEW bsrc csrcNEW dproof", &grammar()).is_none());
    let mut duplicate = grammar();
    duplicate.children.push(duplicate.children[0].clone());
    assert!(compile_compound_task("asrcOLDNEW bsrc creceiptNEW dproof", &duplicate).is_none());
    assert!(compile_compound_task("asrcOLDNEW\u{a0}bsrc creceiptNEW dproof", &grammar()).is_none());
}
