//! Seed-owned complete compound clauses. Parsed obligations never grant effects.
use super::code_task::valid_identifier;
use crate::seed::parser::LinoNode;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundCapture {
    pub text: Option<String>,
    pub span: Option<[usize; 2]>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundClause {
    pub kind: String,
    pub span: [usize; 2],
    pub captures: BTreeMap<String, CompoundCapture>,
    pub status: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundTask {
    pub original_request: String,
    pub unit: &'static str,
    pub whole_request_consumed: bool,
    pub clauses: Vec<CompoundClause>,
    pub destinations: Vec<String>,
    pub write_authority: bool,
    pub unresolved_needs: Vec<String>,
    pub compilation: &'static str,
    pub effects: &'static str,
}
fn children<'a>(node: &'a LinoNode, name: &str) -> Vec<&'a LinoNode> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .collect()
}
fn utf16_offset(source: &str, byte: usize) -> Option<usize> {
    Some(source.get(..byte)?.encode_utf16().count())
}
fn capture_text<'a>(clauses: &'a [CompoundClause], kind: &str, name: &str) -> Option<&'a str> {
    clauses
        .iter()
        .find(|clause| clause.kind == kind)?
        .captures
        .get(name)?
        .text
        .as_deref()
}
fn safe_path(value: &str) -> bool {
    !value.starts_with('/')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}
/// Compile one complete raw request against exactly one supplied grammar root.
/// Regex/capture offsets are projected from UTF-8 bytes to JS-compatible UTF-16.
#[must_use]
pub fn compile_compound_task(request: &str, document: &LinoNode) -> Option<CompoundTask> {
    let roots = if document.name == "compound-task-grammar" {
        vec![document]
    } else {
        children(document, "compound-task-grammar")
    };
    if roots.len() != 1 {
        return None;
    }
    let mut forms = Vec::new();
    for form in children(roots[0], "form") {
        let kinds = children(form, "kind");
        let patterns = children(form, "pattern");
        if kinds.len() != 1 || patterns.len() != 1 {
            return None;
        }
        // The current seed uses the common JS/Rust regex subset. Unsupported
        // lookaround/backreferences fail Regex::new rather than gain a match.
        let expression = regex::Regex::new(&patterns[0].id.replace("(?<", "(?P<")).ok()?;
        forms.push((kinds[0].id.clone(), expression));
    }
    if forms.is_empty() {
        return None;
    }
    let mut clauses = Vec::new();
    let mut cursor = 0;
    while cursor < request.len() {
        while request
            .as_bytes()
            .get(cursor)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        {
            cursor += 1;
        }
        if cursor == request.len() {
            break;
        }
        let matches: Vec<_> = forms
            .iter()
            .filter_map(|(kind, expression)| {
                let captures = expression.captures(request.get(cursor..)?)?;
                let whole = captures.get(0)?;
                (whole.start() == 0 && whole.end() > 0).then_some((kind, expression, captures))
            })
            .collect();
        if matches.len() != 1 {
            return None;
        }
        let (kind, expression, matched) = &matches[0];
        let end = cursor + matched.get(0)?.end();
        let mut captures = BTreeMap::new();
        for name in expression.capture_names().flatten() {
            let observed = matched.name(name);
            let span = if let Some(value) = observed {
                Some([
                    utf16_offset(request, cursor + value.start())?,
                    utf16_offset(request, cursor + value.end())?,
                ])
            } else {
                None
            };
            captures.insert(
                name.to_owned(),
                CompoundCapture {
                    text: observed.map(|value| value.as_str().to_owned()),
                    span,
                },
            );
        }
        clauses.push(CompoundClause {
            kind: (*kind).clone(),
            span: [utf16_offset(request, cursor)?, utf16_offset(request, end)?],
            captures,
            status: "unresolved",
        });
        cursor = end;
    }
    let kinds: BTreeSet<_> = forms.iter().map(|(kind, _)| kind.as_str()).collect();
    if kinds.len() != forms.len()
        || kinds
            .iter()
            .any(|kind| clauses.iter().filter(|clause| clause.kind == *kind).count() != 1)
    {
        return None;
    }
    let source = capture_text(&clauses, "private-constant-rename", "sourcePath")?;
    let previous = capture_text(&clauses, "private-constant-rename", "previous")?;
    let replacement = capture_text(&clauses, "private-constant-rename", "replacement")?;
    if source != capture_text(&clauses, "tracked-owner-condition", "trackedPath")?
        || !valid_identifier(previous)
        || !valid_identifier(replacement)
        || previous == replacement
        || replacement != capture_text(&clauses, "exact-field-receipt", "requiredIdentifier")?
    {
        return None;
    }
    for clause in &clauses {
        for (name, value) in &clause.captures {
            if name.ends_with("Path") && name != "nodePath" && !safe_path(value.text.as_deref()?) {
                return None;
            }
        }
    }
    let destinations = vec![
        source.to_owned(),
        capture_text(&clauses, "exact-field-receipt", "receiptPath")?.to_owned(),
        capture_text(&clauses, "proof-target", "proofPath")?.to_owned(),
    ];
    if destinations.iter().collect::<BTreeSet<_>>().len() != destinations.len() {
        return None;
    }
    let unresolved_needs = clauses.iter().map(|clause| clause.kind.clone()).collect();
    Some(CompoundTask {
        original_request: request.to_owned(),
        unit: "utf16",
        whole_request_consumed: true,
        clauses,
        destinations,
        write_authority: false,
        unresolved_needs,
        compilation: "pending",
        effects: "not-executed",
    })
}

#[cfg(test)]
mod tests {
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
        let plain =
            compile_compound_task("asrcOLDNEW bsrc creceiptNEW dproof", &grammar()).unwrap();
        assert_eq!(plain.clauses[0].captures["label"].text, None);
        assert_eq!(plain.clauses[0].captures["label"].span, None);
        let astral =
            compile_compound_task("😀asrcOLDNEW bsrc creceiptNEW dproof", &grammar()).unwrap();
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
        assert!(
            compile_compound_task("asrcOLDNEW bother creceiptNEW dproof", &grammar()).is_none()
        );
        assert!(compile_compound_task("asrcOLDNEW bsrc csrcNEW dproof", &grammar()).is_none());
        let mut duplicate = grammar();
        duplicate.children.push(duplicate.children[0].clone());
        assert!(compile_compound_task("asrcOLDNEW bsrc creceiptNEW dproof", &duplicate).is_none());
        assert!(
            compile_compound_task("asrcOLDNEW\u{a0}bsrc creceiptNEW dproof", &grammar()).is_none()
        );
    }
}
