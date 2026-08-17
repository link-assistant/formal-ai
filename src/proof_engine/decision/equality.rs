//! Bounded equality saturation over symbolic S-expressions.

use egg::{rewrite, RecExpr, Rewrite, Runner, SymbolLang};

use crate::proof_engine::types::{Proof, ProofMethod, ProofOutcome, ProofStep, StepKind};

use super::render_proof_text;

const ITERATION_LIMIT: usize = 12;
const NODE_LIMIT: usize = 20_000;

#[must_use]
pub fn has_symbolic_equality(claim: &str) -> bool {
    split_equality(claim).is_some() && super::has_prefix_equality(claim)
}

pub fn attempt_equality_claim(claim: &str, language: &str) -> Option<ProofOutcome> {
    let (left_text, right_text) = split_equality(claim)?;
    let left: RecExpr<SymbolLang> = left_text.parse().ok()?;
    let right: RecExpr<SymbolLang> = right_text.parse().ok()?;
    let runner = Runner::<SymbolLang, ()>::default()
        .with_iter_limit(ITERATION_LIMIT)
        .with_node_limit(NODE_LIMIT)
        .with_expr(&left)
        .with_expr(&right)
        .run(&rewrite_system());
    let equivalent = runner.egraph.find(runner.roots[0]) == runner.egraph.find(runner.roots[1]);
    if !equivalent {
        return None;
    }

    let certificate = format!(
        "egraph(iterations={},classes={},nodes={})",
        runner.iterations.len(),
        runner.egraph.number_of_classes(),
        runner.egraph.total_size(),
    );
    Some(ProofOutcome::Proven {
        proof: Proof {
            statement: claim.to_string(),
            steps: vec![
                ProofStep {
                    kind: StepKind::Hypothesis,
                    text: format!("egraph_input({left_text},{right_text})"),
                },
                ProofStep {
                    kind: StepKind::Inference,
                    text: certificate,
                },
            ],
            conclusion: render_proof_text(
                "proof_equality_conclusion",
                language,
                &[("left", left_text), ("right", right_text)],
            ),
            method: ProofMethod::DecisionProcedure,
        },
    })
}

fn split_equality(claim: &str) -> Option<(&str, &str)> {
    let mut depth = 0_usize;
    for (index, character) in claim.char_indices() {
        match character {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            '=' if depth == 0 => {
                let left = claim[..index].trim();
                let right = claim[index + character.len_utf8()..].trim();
                if left.is_empty()
                    || right.is_empty()
                    || left.ends_with(['!', '<', '>'])
                    || right.starts_with('=')
                {
                    return None;
                }
                return Some((left, right));
            }
            _ => {}
        }
    }
    None
}

fn rewrite_system() -> Vec<Rewrite<SymbolLang, ()>> {
    vec![
        rewrite!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rewrite!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rewrite!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
        rewrite!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        rewrite!("zero-add"; "(+ ?a 0)" => "?a"),
        rewrite!("zero-mul"; "(* ?a 0)" => "0"),
        rewrite!("one-mul";  "(* ?a 1)" => "?a"),
        rewrite!("cancel-sub"; "(- ?a ?a)" => "0"),
        rewrite!("distribute"; "(* ?a (+ ?b ?c))" => "(+ (* ?a ?b) (* ?a ?c))"),
        rewrite!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
        rewrite!("pow1"; "(pow ?x 1)" => "?x"),
        rewrite!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
        rewrite!("d-add"; "(d ?x (+ ?a ?b))" => "(+ (d ?x ?a) (d ?x ?b))"),
        rewrite!("d-mul"; "(d ?x (* ?a ?b))" => "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))"),
        rewrite!("d-sin"; "(d ?x (sin ?x))" => "(cos ?x)"),
        rewrite!("d-cos"; "(d ?x (cos ?x))" => "(* -1 (sin ?x))"),
        rewrite!("i-one"; "(i 1 ?x)" => "?x"),
        rewrite!("i-cos"; "(i (cos ?x) ?x)" => "(sin ?x)"),
        rewrite!("i-sin"; "(i (sin ?x) ?x)" => "(* -1 (cos ?x))"),
        rewrite!("i-sum"; "(i (+ ?f ?g) ?x)" => "(+ (i ?f ?x) (i ?g ?x))"),
        rewrite!("i-dif"; "(i (- ?f ?g) ?x)" => "(- (i ?f ?x) (i ?g ?x))"),
        rewrite!("i-parts"; "(i (* ?a ?b) ?x)" => "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))"),
    ]
}
