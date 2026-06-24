//! Quantifier-free affine real-arithmetic decision procedure.

mod proofs;
mod search;

use std::collections::{BTreeMap, BTreeSet};

use crate::proof_engine::types::{ProofMethod, ProofOutcome};

use self::proofs::{
    linear_entailment_counterexample_proof, linear_entailment_proof, linear_identity_disproof,
    linear_identity_proof, linear_satisfiability_proof, linear_universal_counterexample_proof,
    linear_unsat_proof, linear_vacuous_entailment_proof,
};
use self::search::find_assignment;

const EPSILON: f64 = 1e-9;

pub(super) fn attempt_linear_claim(claim: &str, language: &str) -> Option<ProofOutcome> {
    if let Some(outcome) = attempt_constraint_satisfiability(claim) {
        return Some(outcome);
    }
    if let Some((premises, conclusion)) = split_implication(claim) {
        let premise_atoms = parse_constraint_list(premises)?;
        let conclusion_atom = parse_linear_atom(conclusion)?;
        return decide_linear_entailment(&premise_atoms, &conclusion_atom);
    }
    let atom = parse_linear_atom(claim)?;
    decide_universal_linear_atom(&atom, language)
}

fn split_implication(text: &str) -> Option<(&str, &str)> {
    if let Some(rest) = text.strip_prefix("if ") {
        if let Some(index) = rest.find(" then ") {
            let premise = &rest[..index];
            let conclusion = &rest[index + " then ".len()..];
            return Some((premise.trim(), conclusion.trim()));
        }
    }
    for token in [" implies ", " => ", " -> "] {
        if let Some(index) = text.find(token) {
            let premise = &text[..index];
            let conclusion = &text[index + token.len()..];
            return Some((premise.trim(), conclusion.trim()));
        }
    }
    None
}

fn parse_constraint_list(text: &str) -> Option<Vec<LinearAtom>> {
    let cleaned = text
        .trim()
        .trim_start_matches("constraints ")
        .trim_start_matches("constraint ")
        .trim_start_matches("the constraints ")
        .trim();
    let atoms = cleaned
        .split(" and ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(parse_linear_atom)
        .collect::<Option<Vec<_>>>()?;
    (!atoms.is_empty()).then_some(atoms)
}

fn attempt_constraint_satisfiability(text: &str) -> Option<ProofOutcome> {
    let wants_unsat = text.contains("inconsistent") || text.contains("unsatisfiable");
    let wants_sat = text.contains("satisfiable") || text.contains("consistent");
    if !wants_sat && !wants_unsat {
        return None;
    }
    let mut constraints = text.trim();
    for suffix in [
        " are satisfiable",
        " is satisfiable",
        " satisfiable",
        " are consistent",
        " is consistent",
        " consistent",
        " are unsatisfiable",
        " is unsatisfiable",
        " unsatisfiable",
        " are inconsistent",
        " is inconsistent",
        " inconsistent",
    ] {
        if let Some(rest) = constraints.strip_suffix(suffix) {
            constraints = rest.trim();
            break;
        }
    }
    let atoms = parse_constraint_list(constraints)?;
    let system = build_interval_system(&atoms)?;
    let statement = format!("{constraints} is satisfiable");
    if system.is_satisfiable() {
        let witness = system.witness_assignment()?;
        if wants_unsat {
            return Some(ProofOutcome::Disproven {
                counterexample: format!(
                    "{} satisfies every listed constraint.",
                    format_assignment(&witness)
                ),
                method: ProofMethod::DecisionProcedure,
                partial_proof: Some(linear_satisfiability_proof(
                    &statement, &atoms, &system, &witness, true,
                )),
            });
        }
        return Some(ProofOutcome::Proven {
            proof: linear_satisfiability_proof(&statement, &atoms, &system, &witness, true),
        });
    }
    let contradiction = system
        .interval
        .contradiction
        .clone()
        .unwrap_or_else(|| String::from("the interval constraints have empty intersection"));
    if wants_unsat {
        Some(ProofOutcome::Proven {
            proof: linear_unsat_proof(constraints, &atoms, &system, &contradiction),
        })
    } else {
        Some(ProofOutcome::Disproven {
            counterexample: format!("No assignment exists: {contradiction}."),
            method: ProofMethod::DecisionProcedure,
            partial_proof: Some(linear_unsat_proof(
                constraints,
                &atoms,
                &system,
                &contradiction,
            )),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Comparison {
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}

impl Comparison {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "!=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Ge => ">=",
        }
    }

    const fn negate(self) -> Self {
        match self {
            Self::Eq => Self::Neq,
            Self::Neq => Self::Eq,
            Self::Lt => Self::Ge,
            Self::Gt => Self::Le,
            Self::Le => Self::Gt,
            Self::Ge => Self::Lt,
        }
    }

    const fn flip(self) -> Self {
        match self {
            Self::Eq => Self::Eq,
            Self::Neq => Self::Neq,
            Self::Lt => Self::Gt,
            Self::Gt => Self::Lt,
            Self::Le => Self::Ge,
            Self::Ge => Self::Le,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AffineExpr {
    coefficients: BTreeMap<String, f64>,
    constant: f64,
}

impl AffineExpr {
    const fn constant(value: f64) -> Self {
        Self {
            coefficients: BTreeMap::new(),
            constant: value,
        }
    }

    fn variable(name: String) -> Self {
        let mut coefficients = BTreeMap::new();
        coefficients.insert(name, 1.0);
        Self {
            coefficients,
            constant: 0.0,
        }
    }

    fn add(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.constant += other.constant;
        for (name, value) in &other.coefficients {
            *result.coefficients.entry(name.clone()).or_insert(0.0) += value;
        }
        result.clean();
        result
    }

    fn subtract(&self, other: &Self) -> Self {
        self.add(&other.scale(-1.0))
    }

    fn scale(&self, factor: f64) -> Self {
        let mut result = Self {
            coefficients: self
                .coefficients
                .iter()
                .map(|(name, value)| (name.clone(), value * factor))
                .collect(),
            constant: self.constant * factor,
        };
        result.clean();
        result
    }

    fn multiply(&self, other: &Self) -> Option<Self> {
        if self.is_constant() {
            return Some(other.scale(self.constant));
        }
        if other.is_constant() {
            return Some(self.scale(other.constant));
        }
        None
    }

    fn divide(&self, other: &Self) -> Option<Self> {
        if !other.is_constant() || nearly_zero(other.constant) {
            return None;
        }
        Some(self.scale(1.0 / other.constant))
    }

    fn is_constant(&self) -> bool {
        self.coefficients.is_empty()
    }

    fn is_zero(&self) -> bool {
        self.coefficients.is_empty() && nearly_zero(self.constant)
    }

    fn variables(&self) -> BTreeSet<String> {
        self.coefficients.keys().cloned().collect()
    }

    fn evaluate(&self, assignment: &BTreeMap<String, f64>) -> f64 {
        self.coefficients
            .iter()
            .fold(self.constant, |acc, (name, coefficient)| {
                acc + coefficient * assignment.get(name).copied().unwrap_or(0.0)
            })
    }

    fn clean(&mut self) {
        self.coefficients.retain(|_, value| !nearly_zero(*value));
        if nearly_zero(self.constant) {
            self.constant = 0.0;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LinearAtom {
    original: String,
    expression: AffineExpr,
    comparison: Comparison,
}

impl LinearAtom {
    fn variables(&self) -> BTreeSet<String> {
        self.expression.variables()
    }

    fn evaluate(&self, assignment: &BTreeMap<String, f64>) -> bool {
        compare_zero(self.expression.evaluate(assignment), self.comparison)
    }

    fn negated(&self) -> Self {
        Self {
            original: format!("not ({})", self.original),
            expression: self.expression.clone(),
            comparison: self.comparison.negate(),
        }
    }
}

fn compare_zero(value: f64, comparison: Comparison) -> bool {
    match comparison {
        Comparison::Eq => nearly_zero(value),
        Comparison::Neq => !nearly_zero(value),
        Comparison::Lt => value < -EPSILON,
        Comparison::Gt => value > EPSILON,
        Comparison::Le => value <= EPSILON,
        Comparison::Ge => value >= -EPSILON,
    }
}

fn parse_linear_atom(text: &str) -> Option<LinearAtom> {
    for (token, comparison) in [
        ("!=", Comparison::Neq),
        ("<=", Comparison::Le),
        (">=", Comparison::Ge),
        ("==", Comparison::Eq),
        ("=", Comparison::Eq),
        ("<", Comparison::Lt),
        (">", Comparison::Gt),
    ] {
        if let Some(index) = text.find(token) {
            let (left, after) = text.split_at(index);
            let right = &after[token.len()..];
            let lhs = LinearParser::new(left.trim()).parse()?;
            let rhs = LinearParser::new(right.trim()).parse()?;
            return Some(LinearAtom {
                original: format!("{} {} {}", left.trim(), comparison.symbol(), right.trim()),
                expression: lhs.subtract(&rhs),
                comparison,
            });
        }
    }
    None
}

struct LinearParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> LinearParser<'a> {
    const fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse(mut self) -> Option<AffineExpr> {
        let value = self.parse_expression()?;
        self.skip_whitespace();
        (self.position == self.input.len()).then_some(value)
    }

    fn parse_expression(&mut self) -> Option<AffineExpr> {
        let mut value = self.parse_term()?;
        loop {
            self.skip_whitespace();
            if self.consume('+') {
                value = value.add(&self.parse_term()?);
            } else if self.consume('-') {
                value = value.subtract(&self.parse_term()?);
            } else {
                return Some(value);
            }
        }
    }

    fn parse_term(&mut self) -> Option<AffineExpr> {
        let mut value = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            if self.consume('*') {
                value = value.multiply(&self.parse_factor()?)?;
            } else if self.consume('/') {
                value = value.divide(&self.parse_factor()?)?;
            } else if self.next_starts_factor() {
                value = value.multiply(&self.parse_factor()?)?;
            } else {
                return Some(value);
            }
        }
    }

    fn parse_factor(&mut self) -> Option<AffineExpr> {
        self.skip_whitespace();
        if self.consume('+') {
            return self.parse_factor();
        }
        if self.consume('-') {
            return Some(self.parse_factor()?.scale(-1.0));
        }
        if self.consume('(') {
            let value = self.parse_expression()?;
            self.skip_whitespace();
            return self.consume(')').then_some(value);
        }
        if self
            .peek()
            .is_some_and(|ch| ch.is_ascii_digit() || ch == '.')
        {
            return self.parse_number();
        }
        if self.peek().is_some_and(is_variable_start) {
            return Some(self.parse_variable());
        }
        None
    }

    fn parse_number(&mut self) -> Option<AffineExpr> {
        let start = self.position;
        let mut has_digit = false;
        let mut has_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                has_digit = true;
                self.advance(ch);
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance(ch);
            } else {
                break;
            }
        }
        has_digit.then(|| {
            self.input[start..self.position]
                .parse::<f64>()
                .ok()
                .map(AffineExpr::constant)
        })?
    }

    fn parse_variable(&mut self) -> AffineExpr {
        let start = self.position;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance(ch);
            } else {
                break;
            }
        }
        AffineExpr::variable(self.input[start..self.position].to_owned())
    }

    fn next_starts_factor(&self) -> bool {
        let rest = self.input[self.position..].trim_start();
        rest.chars().next().is_some_and(|ch| {
            ch == '(' || ch.is_ascii_digit() || ch == '.' || is_variable_start(ch)
        })
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance(ch);
            } else {
                break;
            }
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance(expected);
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    const fn advance(&mut self, ch: char) {
        self.position += ch.len_utf8();
    }
}

const fn is_variable_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn decide_universal_linear_atom(atom: &LinearAtom, _language: &str) -> Option<ProofOutcome> {
    let mentions_symbol = atom.original.chars().any(is_variable_start);
    if atom.expression.is_zero() && mentions_symbol {
        return Some(ProofOutcome::Proven {
            proof: linear_identity_proof(atom),
        });
    }
    if atom.variables().is_empty() {
        return None;
    }
    if atom.expression.is_constant() {
        if compare_zero(atom.expression.constant, atom.comparison) {
            return Some(ProofOutcome::Proven {
                proof: linear_identity_proof(atom),
            });
        }
        return Some(ProofOutcome::Disproven {
            counterexample: format!(
                "The affine normal form is the constant {}, so {} is false.",
                format_number(atom.expression.constant),
                atom.original
            ),
            method: ProofMethod::DecisionProcedure,
            partial_proof: Some(linear_identity_disproof(atom)),
        });
    }
    let counterexample = find_assignment(atom, false)?;
    Some(ProofOutcome::Disproven {
        counterexample: format!(
            "{} makes {} false.",
            format_assignment(&counterexample),
            atom.original
        ),
        method: ProofMethod::DecisionProcedure,
        partial_proof: Some(linear_universal_counterexample_proof(atom, &counterexample)),
    })
}

fn decide_linear_entailment(
    premise_atoms: &[LinearAtom],
    conclusion: &LinearAtom,
) -> Option<ProofOutcome> {
    let system = build_interval_system(premise_atoms)?;
    let statement = format!(
        "if {} then {}",
        premise_atoms
            .iter()
            .map(|atom| atom.original.as_str())
            .collect::<Vec<_>>()
            .join(" and "),
        conclusion.original
    );
    if !system.is_satisfiable() {
        return Some(ProofOutcome::Proven {
            proof: linear_vacuous_entailment_proof(&statement, premise_atoms, &system),
        });
    }
    let mut negated_atoms = premise_atoms.to_vec();
    negated_atoms.push(conclusion.negated());
    let counter_system = build_interval_system(&negated_atoms)?;
    if !counter_system.is_satisfiable() {
        return Some(ProofOutcome::Proven {
            proof: linear_entailment_proof(&statement, premise_atoms, conclusion, &system),
        });
    }
    let witness = counter_system.witness_assignment()?;
    Some(ProofOutcome::Disproven {
        counterexample: format!(
            "{} satisfies the premises but makes {} false.",
            format_assignment(&witness),
            conclusion.original
        ),
        method: ProofMethod::DecisionProcedure,
        partial_proof: Some(linear_entailment_counterexample_proof(
            &statement,
            premise_atoms,
            conclusion,
            &system,
            &witness,
        )),
    })
}

#[derive(Debug, Clone, Copy)]
struct Bound {
    value: f64,
    strict: bool,
}

#[derive(Debug, Clone)]
struct Interval {
    lower: Option<Bound>,
    upper: Option<Bound>,
    equality: Option<f64>,
    excluded: Vec<f64>,
    contradiction: Option<String>,
}

impl Interval {
    const fn unconstrained() -> Self {
        Self {
            lower: None,
            upper: None,
            equality: None,
            excluded: Vec::new(),
            contradiction: None,
        }
    }

    fn apply_relation(&mut self, comparison: Comparison, value: f64) {
        match comparison {
            Comparison::Eq => self.apply_equality(value),
            Comparison::Neq => self.excluded.push(value),
            Comparison::Gt => self.apply_lower(Bound {
                value,
                strict: true,
            }),
            Comparison::Ge => self.apply_lower(Bound {
                value,
                strict: false,
            }),
            Comparison::Lt => self.apply_upper(Bound {
                value,
                strict: true,
            }),
            Comparison::Le => self.apply_upper(Bound {
                value,
                strict: false,
            }),
        }
        self.validate();
    }

    fn apply_equality(&mut self, value: f64) {
        if let Some(existing) = self.equality {
            if !nearly_equal(existing, value) {
                self.contradiction = Some(format!(
                    "requires {} and {} at the same time",
                    format_number(existing),
                    format_number(value)
                ));
            }
        } else {
            self.equality = Some(value);
        }
    }

    fn apply_lower(&mut self, candidate: Bound) {
        if self
            .lower
            .is_none_or(|current| stronger_lower(candidate, current))
        {
            self.lower = Some(candidate);
        }
    }

    fn apply_upper(&mut self, candidate: Bound) {
        if self
            .upper
            .is_none_or(|current| stronger_upper(candidate, current))
        {
            self.upper = Some(candidate);
        }
    }

    fn validate(&mut self) {
        if self.contradiction.is_some() {
            return;
        }
        if let (Some(lower), Some(upper)) = (self.lower, self.upper) {
            if lower.value > upper.value + EPSILON
                || (nearly_equal(lower.value, upper.value) && (lower.strict || upper.strict))
            {
                self.contradiction = Some(format!(
                    "requires x {} {} and x {} {}",
                    if lower.strict { ">" } else { ">=" },
                    format_number(lower.value),
                    if upper.strict { "<" } else { "<=" },
                    format_number(upper.value)
                ));
                return;
            }
        }
        if let Some(value) = self.equality {
            if !self.contains(value) {
                self.contradiction = Some(format!(
                    "requires x = {}, but the interval excludes that value",
                    format_number(value)
                ));
            }
        }
    }

    fn contains(&self, value: f64) -> bool {
        if let Some(lower) = self.lower {
            if value < lower.value - EPSILON || (lower.strict && nearly_equal(value, lower.value)) {
                return false;
            }
        }
        if let Some(upper) = self.upper {
            if value > upper.value + EPSILON || (upper.strict && nearly_equal(value, upper.value)) {
                return false;
            }
        }
        if let Some(equal) = self.equality {
            if !nearly_equal(value, equal) {
                return false;
            }
        }
        !self
            .excluded
            .iter()
            .any(|excluded| nearly_equal(value, *excluded))
    }
}

fn stronger_lower(candidate: Bound, current: Bound) -> bool {
    candidate.value > current.value + EPSILON
        || (nearly_equal(candidate.value, current.value) && candidate.strict && !current.strict)
}

fn stronger_upper(candidate: Bound, current: Bound) -> bool {
    candidate.value < current.value - EPSILON
        || (nearly_equal(candidate.value, current.value) && candidate.strict && !current.strict)
}

#[derive(Debug, Clone)]
struct IntervalSystem {
    variable: String,
    interval: Interval,
}

impl IntervalSystem {
    const fn is_satisfiable(&self) -> bool {
        self.interval.contradiction.is_none()
    }

    fn witness_assignment(&self) -> Option<BTreeMap<String, f64>> {
        let value = self.witness_value()?;
        let mut assignment = BTreeMap::new();
        assignment.insert(self.variable.clone(), value);
        Some(assignment)
    }

    fn witness_value(&self) -> Option<f64> {
        if !self.is_satisfiable() {
            return None;
        }
        if let Some(value) = self.interval.equality {
            return self.interval.contains(value).then_some(value);
        }
        if let Some(upper) = self.interval.upper {
            if !upper.strict && self.interval.contains(upper.value) {
                return Some(upper.value);
            }
        }
        if let Some(lower) = self.interval.lower {
            if !lower.strict && self.interval.contains(lower.value) {
                return Some(lower.value);
            }
        }
        let candidate = match (self.interval.lower, self.interval.upper) {
            (Some(lower), Some(upper)) => f64::midpoint(lower.value, upper.value),
            (Some(lower), None) => lower.value + if lower.strict { 1.0 } else { 0.0 },
            (None, Some(upper)) => upper.value - if upper.strict { 1.0 } else { 0.0 },
            (None, None) => 0.0,
        };
        if self.interval.contains(candidate) {
            return Some(candidate);
        }
        [-10.0, -1.0, 0.0, 1.0, 10.0]
            .into_iter()
            .find(|candidate| self.interval.contains(*candidate))
    }

    fn interval_summary(&self) -> String {
        if let Some(value) = self.interval.equality {
            return format!("{} = {}", self.variable, format_number(value));
        }
        let lower = self.interval.lower.map_or_else(
            || String::from("-infinity"),
            |bound| {
                format!(
                    "{} {}",
                    if bound.strict { ">" } else { ">=" },
                    format_number(bound.value)
                )
            },
        );
        let upper = self.interval.upper.map_or_else(
            || String::from("infinity"),
            |bound| {
                format!(
                    "{} {}",
                    if bound.strict { "<" } else { "<=" },
                    format_number(bound.value)
                )
            },
        );
        format!("{}: {lower} and {upper}", self.variable)
    }
}

fn build_interval_system(atoms: &[LinearAtom]) -> Option<IntervalSystem> {
    let mut variable: Option<String> = None;
    let mut interval = Interval::unconstrained();
    for atom in atoms {
        let variables = atom.variables();
        if variables.len() > 1 {
            return None;
        }
        if variables.is_empty() {
            if !atom.evaluate(&BTreeMap::new()) {
                interval.contradiction = Some(format!("{} is false", atom.original));
            }
            continue;
        }
        let name = variables.iter().next()?.clone();
        if let Some(existing) = &variable {
            if existing != &name {
                return None;
            }
        } else {
            variable = Some(name.clone());
        }
        let coefficient = *atom.expression.coefficients.get(&name)?;
        let constant = atom.expression.constant;
        if nearly_zero(coefficient) {
            if !compare_zero(constant, atom.comparison) {
                interval.contradiction = Some(format!("{} is false", atom.original));
            }
            continue;
        }
        let threshold = -constant / coefficient;
        let comparison = if coefficient.is_sign_positive() {
            atom.comparison
        } else {
            atom.comparison.flip()
        };
        interval.apply_relation(comparison, threshold);
    }
    Some(IntervalSystem {
        variable: variable.unwrap_or_else(|| String::from("x")),
        interval,
    })
}

fn format_atoms(atoms: &[LinearAtom]) -> String {
    atoms
        .iter()
        .map(|atom| atom.original.as_str())
        .collect::<Vec<_>>()
        .join(" and ")
}

fn format_assignment(assignment: &BTreeMap<String, f64>) -> String {
    assignment
        .iter()
        .map(|(name, value)| format!("{name} = {}", format_number(*value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_affine(expression: &AffineExpr) -> String {
    let mut parts = Vec::new();
    for (name, coefficient) in &expression.coefficients {
        parts.push(format!("{}*{}", format_number(*coefficient), name));
    }
    if !nearly_zero(expression.constant) || parts.is_empty() {
        parts.push(format_number(expression.constant));
    }
    parts.join(" + ").replace("+ -", "- ")
}

fn format_number(value: f64) -> String {
    if nearly_zero(value) {
        return String::from("0");
    }
    if nearly_zero(value.fract()) {
        return format!("{value:.0}");
    }
    format!("{value:.6}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

fn nearly_zero(value: f64) -> bool {
    value.abs() < EPSILON
}

fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() < EPSILON
}
