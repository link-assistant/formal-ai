use std::collections::BTreeMap;
use std::fmt;

use super::{AlgorithmCandidate, ArgumentPattern};

/// Named automated evidence required for promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmGate {
    pub suite: String,
    pub passed: usize,
    pub failed: usize,
}

impl AlgorithmGate {
    #[must_use]
    pub fn passed(suite: impl Into<String>, passed: usize) -> Self {
        Self {
            suite: suite.into(),
            passed,
            failed: 0,
        }
    }

    #[must_use]
    pub fn failed(suite: impl Into<String>, passed: usize, failed: usize) -> Self {
        Self {
            suite: suite.into(),
            passed,
            failed,
        }
    }

    #[must_use]
    pub const fn is_green(&self) -> bool {
        self.passed > 0 && self.failed == 0
    }
}

/// Explicit named human decision. Discovery never manufactures this value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmApproval {
    pub reviewer: String,
    pub granted: bool,
}

impl AlgorithmApproval {
    #[must_use]
    pub fn granted(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: true,
        }
    }

    #[must_use]
    pub fn declined(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlgorithmDiscoveryError {
    NotValidated,
    GateFailed(String),
    ApprovalRequired,
    UnnamedReviewer,
    MissingBinding(String),
    Host(String),
    InvalidArtifact(String),
}

impl fmt::Display for AlgorithmDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotValidated => {
                formatter.write_str("candidate has not passed held-out validation")
            }
            Self::GateFailed(suite) => write!(formatter, "automated gate {suite} is not green"),
            Self::ApprovalRequired => formatter.write_str("named human approval is required"),
            Self::UnnamedReviewer => formatter.write_str("approval reviewer must be named"),
            Self::MissingBinding(name) => write!(formatter, "missing parameter binding: {name}"),
            Self::Host(error) => write!(formatter, "algorithm host failed: {error}"),
            Self::InvalidArtifact(error) => {
                write!(formatter, "invalid algorithm artifact: {error}")
            }
        }
    }
}

impl std::error::Error for AlgorithmDiscoveryError {}

/// Side-effect boundary for approved algorithms. A proposal cannot reach this
/// trait without passing [`AlgorithmCandidate::promote`].
pub trait AlgorithmHost {
    fn perform(
        &mut self,
        operation: &str,
        arguments: &BTreeMap<String, String>,
        input: &str,
    ) -> Result<String, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedAlgorithm {
    pub(super) candidate: AlgorithmCandidate,
    pub(super) gate: AlgorithmGate,
    pub(super) approval: AlgorithmApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmExecution {
    pub algorithm_id: String,
    pub trigger: String,
    pub reviewer: String,
    pub outcomes: Vec<String>,
}

impl ApprovedAlgorithm {
    /// The immutable proposal that cleared promotion.
    #[must_use]
    pub const fn candidate(&self) -> &AlgorithmCandidate {
        &self.candidate
    }

    /// The immutable automated gate captured at promotion time.
    #[must_use]
    pub const fn gate(&self) -> &AlgorithmGate {
        &self.gate
    }

    /// The immutable named approval captured at promotion time.
    #[must_use]
    pub const fn approval(&self) -> &AlgorithmApproval {
        &self.approval
    }

    pub fn execute<H: AlgorithmHost>(
        &self,
        trigger: &str,
        bindings: &BTreeMap<String, String>,
        host: &mut H,
    ) -> Result<AlgorithmExecution, AlgorithmDiscoveryError> {
        let mut input = trigger.to_owned();
        let mut outcomes = Vec::with_capacity(self.candidate.steps.len());
        for step in &self.candidate.steps {
            let mut arguments = BTreeMap::new();
            for (name, pattern) in &step.arguments {
                let value = match pattern {
                    ArgumentPattern::Constant(value) => value.clone(),
                    ArgumentPattern::Parameter(parameter) => {
                        bindings.get(parameter).cloned().ok_or_else(|| {
                            AlgorithmDiscoveryError::MissingBinding(parameter.clone())
                        })?
                    }
                };
                arguments.insert(name.clone(), value);
            }
            input = host
                .perform(&step.operation, &arguments, &input)
                .map_err(AlgorithmDiscoveryError::Host)?;
            outcomes.push(input.clone());
        }
        Ok(AlgorithmExecution {
            algorithm_id: self.candidate.id.clone(),
            trigger: trigger.to_owned(),
            reviewer: self.approval.reviewer.clone(),
            outcomes,
        })
    }
}
