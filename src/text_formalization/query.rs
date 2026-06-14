//! A declarative conjunctive query over assertions.
//!
//! Implements the protocol's declarative query form (article §9), e.g.
//!
//! ```text
//! SELECT ?shop
//! WHERE  ASSERT.subject = ent:petrov_petr
//!   AND  predicate      = pred:open
//!   AND  ctx.location   = "Москва"
//!   AND  time.year      = 2019
//! ```
//!
//! A [`Query`] is a set of optional equality/threshold constraints; an
//! assertion matches when it satisfies every constraint that is set. Queries
//! can be built programmatically or parsed from the textual form above with
//! [`Query::parse`].

use std::error::Error;
use std::fmt;

use super::knowledge_base::KnowledgeBase;
use super::primitives::Assertion;

/// A declarative conjunctive query: every set field must match.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Query {
    /// The projected variable name from `SELECT ?var` (informational).
    pub projection: Option<String>,
    /// Required subject identifier.
    pub subject: Option<String>,
    /// Required predicate identifier.
    pub predicate: Option<String>,
    /// Required object identifier or literal value.
    pub object: Option<String>,
    /// Required context identifier.
    pub context_id: Option<String>,
    /// Required context `location` property.
    pub context_location: Option<String>,
    /// Required calendar year on the temporal qualifier.
    pub time_year: Option<i32>,
    /// Minimum confidence threshold.
    pub min_confidence: Option<f64>,
    /// Required modality kind.
    pub modality: Option<String>,
}

/// An error encountered while parsing a textual [`Query`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    /// The `WHERE` keyword was missing.
    MissingWhere,
    /// A condition was not of the form `lhs <op> rhs`.
    MalformedCondition(String),
    /// A condition referenced an unknown field on the left-hand side.
    UnknownField(String),
    /// A numeric right-hand side could not be parsed.
    InvalidNumber(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWhere => write!(formatter, "query is missing a WHERE clause"),
            Self::MalformedCondition(condition) => {
                write!(formatter, "malformed condition: {condition}")
            }
            Self::UnknownField(field) => write!(formatter, "unknown query field: {field}"),
            Self::InvalidNumber(value) => write!(formatter, "invalid numeric value: {value}"),
        }
    }
}

impl Error for QueryError {}

impl Query {
    /// Build an empty query that matches every assertion.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constrain the subject identifier.
    #[must_use]
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Constrain the predicate identifier.
    #[must_use]
    pub fn with_predicate(mut self, predicate: impl Into<String>) -> Self {
        self.predicate = Some(predicate.into());
        self
    }

    /// Constrain an object identifier or literal value.
    #[must_use]
    pub fn with_object(mut self, object: impl Into<String>) -> Self {
        self.object = Some(object.into());
        self
    }

    /// Constrain the context `location` property.
    #[must_use]
    pub fn with_context_location(mut self, location: impl Into<String>) -> Self {
        self.context_location = Some(location.into());
        self
    }

    /// Constrain the temporal year.
    #[must_use]
    pub const fn with_time_year(mut self, year: i32) -> Self {
        self.time_year = Some(year);
        self
    }

    /// Constrain the minimum confidence.
    #[must_use]
    pub const fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = Some(confidence);
        self
    }

    /// Whether an assertion satisfies every set constraint.
    #[must_use]
    pub fn matches(&self, assertion: &Assertion) -> bool {
        self.subject_matches(assertion)
            && self.predicate_matches(assertion)
            && self.object_matches(assertion)
            && self.context_matches(assertion)
            && self.time_matches(assertion)
            && self.confidence_matches(assertion)
            && self.modality_matches(assertion)
    }

    fn subject_matches(&self, assertion: &Assertion) -> bool {
        self.subject.as_ref().map_or(true, |wanted| {
            assertion.subject_id() == Some(wanted.as_str())
        })
    }

    fn predicate_matches(&self, assertion: &Assertion) -> bool {
        self.predicate
            .as_ref()
            .map_or(true, |wanted| assertion.predicate_id() == wanted)
    }

    fn object_matches(&self, assertion: &Assertion) -> bool {
        let Some(wanted) = self.object.as_ref() else {
            return true;
        };
        assertion
            .object
            .iter()
            .any(|term| term.reference_id() == Some(wanted.as_str()) || &term.node_id() == wanted)
    }

    fn context_matches(&self, assertion: &Assertion) -> bool {
        let id_ok = self.context_id.as_ref().map_or(true, |wanted| {
            assertion
                .context
                .as_ref()
                .is_some_and(|context| &context.id == wanted)
        });
        let location_ok = self.context_location.as_ref().map_or(true, |wanted| {
            assertion
                .context
                .as_ref()
                .and_then(super::primitives::Context::location)
                == Some(wanted.as_str())
        });
        id_ok && location_ok
    }

    fn time_matches(&self, assertion: &Assertion) -> bool {
        self.time_year.map_or(true, |wanted| {
            assertion
                .time
                .as_ref()
                .and_then(super::primitives::Temporal::calendar_year)
                == Some(wanted)
        })
    }

    fn confidence_matches(&self, assertion: &Assertion) -> bool {
        self.min_confidence
            .map_or(true, |wanted| assertion.confidence() >= wanted)
    }

    fn modality_matches(&self, assertion: &Assertion) -> bool {
        self.modality
            .as_ref()
            .map_or(true, |wanted| &assertion.modal.kind == wanted)
    }

    /// Parse a query from the protocol's textual form.
    pub fn parse(input: &str) -> Result<Self, QueryError> {
        let mut query = Self::new();
        let upper = input.to_uppercase();
        let where_at = upper.find("WHERE").ok_or(QueryError::MissingWhere)?;

        let head = &input[..where_at];
        if let Some(select_at) = head.to_uppercase().find("SELECT") {
            let after = &head[select_at + "SELECT".len()..];
            if let Some(token) = after.split_whitespace().next() {
                query.projection = Some(token.trim_start_matches('?').to_string());
            }
        }

        let body = &input[where_at + "WHERE".len()..];
        for raw in split_conjunction(body) {
            let condition = raw.trim();
            if condition.is_empty() {
                continue;
            }
            query.apply_condition(condition)?;
        }
        Ok(query)
    }

    fn apply_condition(&mut self, condition: &str) -> Result<(), QueryError> {
        let (field, operator, value) = split_condition(condition)
            .ok_or_else(|| QueryError::MalformedCondition(condition.to_string()))?;
        let field_key = field.to_lowercase();
        let value = unquote(value);

        match field_key.as_str() {
            "assert.subject" | "subject" | "subj" => self.subject = Some(value.to_string()),
            "assert.predicate" | "predicate" | "pred" => self.predicate = Some(value.to_string()),
            "assert.object" | "object" | "obj" => self.object = Some(value.to_string()),
            "ctx.location" | "context.location" => {
                self.context_location = Some(value.to_string());
            }
            "ctx.id" | "context.id" | "ctx" | "context" => {
                self.context_id = Some(value.to_string());
            }
            "time.year" => {
                let year = value
                    .parse()
                    .map_err(|_ignored| QueryError::InvalidNumber(value.to_string()))?;
                self.time_year = Some(year);
            }
            "confidence" | "conf" | "modal.confidence" => {
                let confidence = value
                    .parse()
                    .map_err(|_ignored| QueryError::InvalidNumber(value.to_string()))?;
                self.min_confidence = Some(confidence);
            }
            "modality" | "modal" | "modal.type" => self.modality = Some(value.to_string()),
            _other => return Err(QueryError::UnknownField(field.to_string())),
        }
        let _ = operator;
        Ok(())
    }
}

/// Split a `WHERE` body on the `AND` keyword (case-insensitive, whole word).
fn split_conjunction(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    for token in body.split_whitespace() {
        if token.eq_ignore_ascii_case("AND") {
            parts.push(std::mem::take(&mut current));
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(token);
        }
    }
    parts.push(current);
    parts
}

/// Split a single condition into `(field, operator, value)`.
fn split_condition(condition: &str) -> Option<(&str, &str, &str)> {
    for operator in [">=", "<=", "=", ">", "<"] {
        if let Some(index) = condition.find(operator) {
            let field = condition[..index].trim();
            let value = condition[index + operator.len()..].trim();
            if !field.is_empty() && !value.is_empty() {
                return Some((field, operator, value));
            }
        }
    }
    None
}

/// Strip a single pair of surrounding ASCII or guillemet quotes.
fn unquote(value: &str) -> &str {
    let trimmed = value.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        return &trimmed[1..trimmed.len() - 1];
    }
    if let Some(inner) = trimmed
        .strip_prefix('«')
        .and_then(|rest| rest.strip_suffix('»'))
    {
        return inner;
    }
    trimmed
}

impl KnowledgeBase {
    /// Return every assertion that matches the query, in document order.
    #[must_use]
    pub fn query<'kb>(&'kb self, query: &Query) -> Vec<&'kb Assertion> {
        self.assertions
            .iter()
            .filter(|assertion| query.matches(assertion))
            .collect()
    }

    /// Run a textual query and return the matching assertions.
    pub fn query_text(&self, input: &str) -> Result<Vec<&Assertion>, QueryError> {
        let query = Query::parse(input)?;
        Ok(self.query(&query))
    }

    /// Project the object node identifiers of every matching assertion.
    ///
    /// This realizes the `SELECT ?var` projection: for a query selecting the
    /// object position it returns the bound object identifiers.
    #[must_use]
    pub fn query_objects(&self, query: &Query) -> Vec<String> {
        self.query(query)
            .into_iter()
            .flat_map(|assertion| {
                assertion
                    .object
                    .iter()
                    .map(super::primitives::Term::node_id)
            })
            .collect()
    }
}
