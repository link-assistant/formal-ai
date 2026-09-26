//! Bounded, function-free Datalog inference over controlled formal programs.

use std::collections::{BTreeMap, BTreeSet};

use crate::proof_engine::types::{Proof, ProofMethod, ProofOutcome, ProofStep, StepKind};

use super::render_proof_text;

const MAX_ROUNDS: usize = 256;
const MAX_FACTS: usize = 10_000;
const MAX_CLAUSES: usize = 512;
const MAX_ARITY: usize = 16;
const MAX_SUBSTITUTIONS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GroundAtom {
    predicate: String,
    terms: Vec<String>,
}

impl GroundAtom {
    fn notation(&self) -> String {
        format!("{}({})", self.predicate, self.terms.join(","))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Atom {
    predicate: String,
    terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    head: Atom,
    body: Vec<Atom>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Program {
    facts: BTreeSet<GroundAtom>,
    rules: Vec<Rule>,
    query: GroundAtom,
}

struct Evaluation {
    facts: BTreeSet<GroundAtom>,
    rounds: usize,
    derived: usize,
}

#[must_use]
pub fn has_rule_program(claim: &str) -> bool {
    ["facts", "rules", "query"]
        .iter()
        .all(|section| claim.contains(&format!("{section} {{")))
}

pub fn attempt_rule_inference(claim: &str, language: &str) -> Option<ProofOutcome> {
    if !has_rule_program(claim) {
        return None;
    }
    let program = match parse_program(claim) {
        Ok(program) => program,
        Err(reason) => {
            return Some(ProofOutcome::Inconclusive {
                reason: format!("datalog_parse({reason})"),
            });
        }
    };
    let initial_facts = program.facts.len();
    let rule_count = program.rules.len();
    let query = program.query.notation();
    let evaluation = match evaluate(&program) {
        Ok(evaluation) => evaluation,
        Err(reason) => {
            return Some(ProofOutcome::Inconclusive {
                reason: format!("datalog_limit({reason})"),
            });
        }
    };
    let certificate = format!(
        "datalog(facts={initial_facts},rules={rule_count},rounds={},derived={})",
        evaluation.rounds, evaluation.derived,
    );
    if evaluation.facts.contains(&program.query) {
        return Some(ProofOutcome::Proven {
            proof: Proof {
                statement: claim.to_string(),
                steps: vec![
                    ProofStep {
                        kind: StepKind::Hypothesis,
                        text: format!("datalog_query({query})"),
                    },
                    ProofStep {
                        kind: StepKind::Inference,
                        text: certificate,
                    },
                ],
                conclusion: render_proof_text(
                    "proof_datalog_conclusion",
                    language,
                    &[("query", &query)],
                ),
                method: ProofMethod::DecisionProcedure,
            },
        });
    }
    Some(ProofOutcome::Disproven {
        counterexample: render_proof_text(
            "proof_datalog_counterexample",
            language,
            &[("query", &query), ("certificate", &certificate)],
        ),
        method: ProofMethod::DecisionProcedure,
        partial_proof: None,
    })
}

fn parse_program(claim: &str) -> Result<Program, String> {
    let fact_block = extract_block(claim, "facts")?;
    let rule_block = extract_block(claim, "rules")?;
    let query_block = extract_block(claim, "query")?;

    let fact_atoms = split_top_level(fact_block, ';');
    let rule_texts = split_top_level(rule_block, ';');
    let query_atoms = split_top_level(query_block, ';');
    if fact_atoms.len() + rule_texts.len() > MAX_CLAUSES {
        return Err(format!("clauses>{MAX_CLAUSES}"));
    }
    if query_atoms.len() != 1 {
        return Err(String::from("query_count!=1"));
    }

    let facts = fact_atoms
        .into_iter()
        .map(parse_atom)
        .map(|result| result.and_then(atom_to_ground))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let rules = rule_texts
        .into_iter()
        .map(parse_rule)
        .collect::<Result<Vec<_>, _>>()?;
    let query = atom_to_ground(parse_atom(query_atoms[0])?)?;
    Ok(Program {
        facts,
        rules,
        query,
    })
}

fn extract_block<'a>(text: &'a str, name: &str) -> Result<&'a str, String> {
    let marker = format!("{name} {{");
    let start = text
        .find(&marker)
        .ok_or_else(|| format!("missing_{name}"))?;
    let open = start + marker.len() - 1;
    let mut depth = 0_usize;
    for (offset, character) in text[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(text[open + 1..open + offset].trim());
                }
            }
            _ => {}
        }
    }
    Err(format!("unclosed_{name}"))
}

fn split_top_level(text: &str, delimiter: char) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0_usize;
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            value if value == delimiter && depth == 0 => {
                let item = text[start..index].trim();
                if !item.is_empty() {
                    result.push(item);
                }
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    let item = text[start..].trim();
    if !item.is_empty() {
        result.push(item);
    }
    result
}

fn parse_rule(text: &str) -> Result<Rule, String> {
    let (head, body) = text
        .split_once(":-")
        .ok_or_else(|| format!("rule_missing_separator:{text}"))?;
    let body = split_top_level(body, ',')
        .into_iter()
        .map(parse_atom)
        .collect::<Result<Vec<_>, _>>()?;
    if body.is_empty() {
        return Err(format!("empty_rule_body:{text}"));
    }
    let head = parse_atom(head)?;
    if let Some(variable) = head.terms.iter().find(|term| {
        is_variable(term)
            && !body
                .iter()
                .flat_map(|atom| &atom.terms)
                .any(|body_term| body_term == *term)
    }) {
        return Err(format!("unsafe_head_variable:{variable}"));
    }
    Ok(Rule { head, body })
}

fn parse_atom(text: &str) -> Result<Atom, String> {
    let trimmed = text.trim();
    let open = trimmed
        .find('(')
        .ok_or_else(|| format!("atom_missing_open:{trimmed}"))?;
    if !trimmed.ends_with(')') || !valid_identifier(trimmed[..open].trim()) {
        return Err(format!("invalid_atom:{trimmed}"));
    }
    let terms = split_top_level(&trimmed[open + 1..trimmed.len() - 1], ',')
        .into_iter()
        .map(str::trim)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if terms.is_empty() || terms.len() > MAX_ARITY || terms.iter().any(|term| !valid_term(term)) {
        return Err(format!("invalid_terms:{trimmed}"));
    }
    Ok(Atom {
        predicate: trimmed[..open].trim().to_string(),
        terms,
    })
}

fn valid_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn valid_term(text: &str) -> bool {
    text.strip_prefix('?').map_or_else(
        || valid_identifier(text) || text.chars().all(|character| character.is_ascii_digit()),
        valid_identifier,
    )
}

fn atom_to_ground(atom: Atom) -> Result<GroundAtom, String> {
    if atom.terms.iter().any(|term| is_variable(term)) {
        return Err(format!("nonground_atom:{}", atom.predicate));
    }
    Ok(GroundAtom {
        predicate: atom.predicate,
        terms: atom.terms,
    })
}

fn is_variable(term: &str) -> bool {
    term.starts_with('?')
        || term
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
}

fn evaluate(program: &Program) -> Result<Evaluation, String> {
    let mut facts = program.facts.clone();
    let mut rounds = 0_usize;
    let mut derived = 0_usize;
    loop {
        if rounds == MAX_ROUNDS {
            return Err(format!("rounds={MAX_ROUNDS}"));
        }
        rounds += 1;
        let mut additions = BTreeSet::new();
        for rule in &program.rules {
            for substitution in satisfying_substitutions(&rule.body, &facts)? {
                if let Some(atom) = instantiate(&rule.head, &substitution)
                    && !facts.contains(&atom)
                {
                    additions.insert(atom);
                }
            }
        }
        if additions.is_empty() {
            break;
        }
        derived += additions.len();
        facts.extend(additions);
        if facts.len() > MAX_FACTS {
            return Err(format!("facts>{MAX_FACTS}"));
        }
    }
    Ok(Evaluation {
        facts,
        rounds,
        derived,
    })
}

fn satisfying_substitutions(
    body: &[Atom],
    facts: &BTreeSet<GroundAtom>,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut substitutions = vec![BTreeMap::new()];
    for atom in body {
        let mut next = Vec::new();
        for substitution in &substitutions {
            for fact in facts.iter().filter(|fact| {
                fact.predicate == atom.predicate && fact.terms.len() == atom.terms.len()
            }) {
                if let Some(unified) = unify(atom, fact, substitution) {
                    if next.len() == MAX_SUBSTITUTIONS {
                        return Err(format!("substitutions>{MAX_SUBSTITUTIONS}"));
                    }
                    next.push(unified);
                }
            }
        }
        substitutions = next;
        if substitutions.is_empty() {
            break;
        }
    }
    Ok(substitutions)
}

fn unify(
    atom: &Atom,
    fact: &GroundAtom,
    existing: &BTreeMap<String, String>,
) -> Option<BTreeMap<String, String>> {
    let mut substitution = existing.clone();
    for (term, value) in atom.terms.iter().zip(&fact.terms) {
        if is_variable(term) {
            if let Some(bound) = substitution.get(term) {
                if bound != value {
                    return None;
                }
            } else {
                substitution.insert(term.clone(), value.clone());
            }
        } else if term != value {
            return None;
        }
    }
    Some(substitution)
}

fn instantiate(atom: &Atom, substitution: &BTreeMap<String, String>) -> Option<GroundAtom> {
    let terms = atom
        .terms
        .iter()
        .map(|term| {
            if is_variable(term) {
                substitution.get(term).cloned()
            } else {
                Some(term.clone())
            }
        })
        .collect::<Option<Vec<_>>>()?;
    Some(GroundAtom {
        predicate: atom.predicate.clone(),
        terms,
    })
}
