//! Language-neutral formal proofs that can be projected into executable
//! programming-language presentations.
//!
//! The proof representation deliberately contains no presentation language.
//! A proof is constructed once from semantic bounds and can then be rendered by
//! the general code-translation pipeline into any supported target syntax.

use crate::seed::proof_program_templates;

/// One numeric endpoint in an interval proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProofBound {
    value: i64,
    inclusive: bool,
}

impl ProofBound {
    #[must_use]
    pub const fn new(value: i64, inclusive: bool) -> Self {
        Self { value, inclusive }
    }

    const fn lower_operator(self) -> &'static str {
        if self.inclusive {
            ">="
        } else {
            ">"
        }
    }

    const fn upper_operator(self) -> &'static str {
        if self.inclusive {
            "<="
        } else {
            "<"
        }
    }

    const fn first_integer(self) -> i128 {
        let value = self.value as i128;
        if self.inclusive {
            value
        } else {
            value + 1
        }
    }

    const fn last_integer(self) -> i128 {
        let value = self.value as i128;
        if self.inclusive {
            value
        } else {
            value - 1
        }
    }
}

/// A proof that an integer interval is or is not satisfiable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerIntervalProof {
    variable: String,
    lower: ProofBound,
    upper: ProofBound,
    witness: Option<i64>,
}

impl IntegerIntervalProof {
    fn new(variable: &str, lower: ProofBound, upper: ProofBound) -> Option<Self> {
        if variable.is_empty()
            || !variable
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            return None;
        }
        let first = lower.first_integer();
        let last = upper.last_integer();
        let witness = if first <= last {
            i64::try_from(first).ok()
        } else {
            None
        };
        Some(Self {
            variable: variable.to_owned(),
            lower,
            upper,
            witness,
        })
    }

    fn statement(&self) -> String {
        let result = if self.witness.is_some() {
            "satisfiable"
        } else {
            "unsatisfiable"
        };
        self.render_template(&proof_program_templates().statement, result)
    }

    fn slug(&self) -> String {
        format!(
            "proof:integer_interval:{}:{}:{}:{}:{}:{}",
            self.variable,
            self.lower.lower_operator(),
            self.lower.value,
            self.upper.upper_operator(),
            self.upper.value,
            if self.witness.is_some() {
                "satisfiable"
            } else {
                "unsatisfiable"
            }
        )
    }

    fn render_template(&self, template: &str, result: &str) -> String {
        let first = self.lower.first_integer();
        let last = self.upper.last_integer();
        let bindings = [
            ("variable", self.variable.clone()),
            ("lower_operator", self.lower.lower_operator().to_owned()),
            ("lower_value", self.lower.value.to_string()),
            ("upper_operator", self.upper.upper_operator().to_owned()),
            ("upper_value", self.upper.value.to_string()),
            (
                "witness",
                self.witness
                    .map_or_else(String::new, |value| value.to_string()),
            ),
            ("first", first.to_string()),
            ("last", last.to_string()),
            ("result", result.to_owned()),
        ];
        let mut rendered = template.to_owned();
        for (name, value) in bindings {
            rendered = rendered.replace(&format!("{{{name}}}"), &value);
        }
        rendered
    }
}

/// A proof meaning that is independent from both natural-language prose and
/// programming-language syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormalProof {
    IntegerInterval(IntegerIntervalProof),
}

impl FormalProof {
    /// Construct an integer interval proof from semantic bounds.
    #[must_use]
    pub fn integer_interval(
        variable: &str,
        lower_value: i64,
        lower_inclusive: bool,
        upper_value: i64,
        upper_inclusive: bool,
    ) -> Option<Self> {
        IntegerIntervalProof::new(
            variable,
            ProofBound::new(lower_value, lower_inclusive),
            ProofBound::new(upper_value, upper_inclusive),
        )
        .map(Self::IntegerInterval)
    }

    /// Formalize the canonical interval statement emitted by the proof solver.
    #[must_use]
    pub fn from_statement(statement: &str) -> Option<Self> {
        let tokens = statement.split_whitespace().collect::<Vec<_>>();
        if tokens.len() < 9 || tokens[3] != "and" || tokens[0] != tokens[4] {
            return None;
        }
        let lower_inclusive = match tokens[1] {
            ">" => false,
            ">=" => true,
            _ => return None,
        };
        let upper_inclusive = match tokens[5] {
            "<" => false,
            "<=" => true,
            _ => return None,
        };
        let suffix = tokens[7..].join(" ");
        if !matches!(
            suffix.as_str(),
            "is satisfiable" | "is satisfiable over integers" | "is unsatisfiable over integers"
        ) {
            return None;
        }
        let proof = Self::integer_interval(
            tokens[0],
            tokens[2].parse().ok()?,
            lower_inclusive,
            tokens[6].parse().ok()?,
            upper_inclusive,
        )?;
        let expected_satisfiable = !suffix.starts_with("is unsatisfiable");
        (proof.is_satisfiable() == expected_satisfiable).then_some(proof)
    }

    #[must_use]
    pub fn statement(&self) -> String {
        match self {
            Self::IntegerInterval(proof) => proof.statement(),
        }
    }

    #[must_use]
    pub fn slug(&self) -> String {
        match self {
            Self::IntegerInterval(proof) => proof.slug(),
        }
    }

    #[must_use]
    pub const fn is_satisfiable(&self) -> bool {
        match self {
            Self::IntegerInterval(proof) => proof.witness.is_some(),
        }
    }

    /// Project this proof into an executable target-language program.
    #[must_use]
    pub fn render_program(&self, target: &str) -> Option<String> {
        let language = proof_program_templates().language(target)?;
        let proof = match self {
            Self::IntegerInterval(proof) => proof,
        };
        let (template, result) = if proof.witness.is_some() {
            (&language.satisfiable, "satisfiable")
        } else {
            (&language.unsatisfiable, "unsatisfiable")
        };
        Some(proof.render_template(template, result))
    }
}
