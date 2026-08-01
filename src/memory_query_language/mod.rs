//! Exact SQL, GraphQL, and learned natural-language access to associative
//! memory (issue #708).
//!
//! Every surface lowers into [`MemoryQueryPlan`], then into the existing
//! link-cli-compatible substitution algebra.  The native executor consumes the
//! same plan, so parser, review trace, and effects cannot disagree.

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write as _;

#[cfg(not(target_arch = "wasm32"))]
use crate::engine::stable_id;
#[cfg(not(target_arch = "wasm32"))]
use crate::links_format::push_lino_node;
#[cfg(not(target_arch = "wasm32"))]
use crate::links_substitution_query::{
    link_substitution_effect, render_link_substitution_query, LinkRewriteProgram,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::memory_program::MemoryProgramLimits;

#[cfg(not(target_arch = "wasm32"))]
mod execution;
mod graphql;
#[cfg(not(target_arch = "wasm32"))]
mod learning;
#[cfg(not(target_arch = "wasm32"))]
mod lowering;
mod sql;
mod syntax;

#[cfg(not(target_arch = "wasm32"))]
pub use execution::{execute_memory_query, MemoryQueryOutcome};
#[cfg(not(target_arch = "wasm32"))]
pub use learning::{
    MemoryQueryCompiler, MemoryQueryLearningApproval, MemoryQueryLearningCandidate,
    MemoryQueryLearningGate, MemoryQueryLearningObservation,
};

#[cfg(not(target_arch = "wasm32"))]
impl MemoryQueryOutcome {
    /// Render the result as lossless-enough Links Notation for solver output,
    /// audit logs, and exact-language parity tests.
    #[must_use]
    pub fn links_notation(&self, query: &CompiledMemoryQuery) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "memory_query_result", None);
        push_lino_node(&mut out, 2, "query", Some(&query.id));
        push_lino_node(&mut out, 2, "dialect", Some(query.dialect.as_str()));
        let _ = writeln!(out, "  matched {}", self.matched_ids.len());
        let _ = writeln!(out, "  changed {}", self.changed);
        push_lino_node(&mut out, 2, "halt", Some(memory_query_halt(&self.halt)));
        for id in &self.matched_ids {
            push_lino_node(&mut out, 2, "matched_id", Some(id));
        }
        for row in &self.rows {
            push_lino_node(&mut out, 2, "row", None);
            for (field, value) in row {
                push_lino_node(&mut out, 4, field, Some(&value.canonical()));
            }
        }
        out.trim_end().to_owned()
    }
}

#[cfg(not(target_arch = "wasm32"))]
const fn memory_query_halt(halt: &crate::memory_program::MemoryProgramHalt) -> &str {
    use crate::memory_program::MemoryProgramHalt;

    match halt {
        MemoryProgramHalt::Complete => "complete",
        MemoryProgramHalt::Fixpoint => "fixpoint",
        MemoryProgramHalt::MatchLimit { .. } => "match_limit",
        MemoryProgramHalt::IterationLimit { .. } => "iteration_limit",
        MemoryProgramHalt::PermissionDenied { .. } => "permission_denied",
        MemoryProgramHalt::ProgramGap { .. } => "program_gap",
    }
}

/// Input syntax used to produce a memory plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryDialect {
    NaturalLanguage,
    SqlAnsi,
    SqlPostgreSql,
    SqlMySql,
    SqlSqlite,
    SqlMsSql,
    SqlBigQuery,
    GraphQl,
}

impl QueryDialect {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NaturalLanguage => "natural_language",
            Self::SqlAnsi => "sql_ansi",
            Self::SqlPostgreSql => "sql_postgresql",
            Self::SqlMySql => "sql_mysql",
            Self::SqlSqlite => "sql_sqlite",
            Self::SqlMsSql => "sql_mssql",
            Self::SqlBigQuery => "sql_bigquery",
            Self::GraphQl => "graphql",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    const fn meta_language_label(self) -> Option<&'static str> {
        match self {
            Self::NaturalLanguage => None,
            Self::GraphQl => Some("GraphQL"),
            Self::SqlAnsi
            | Self::SqlPostgreSql
            | Self::SqlMySql
            | Self::SqlSqlite
            | Self::SqlMsSql
            | Self::SqlBigQuery => Some("sql-ansi"),
        }
    }

    const fn is_sql(self) -> bool {
        matches!(
            self,
            Self::SqlAnsi
                | Self::SqlPostgreSql
                | Self::SqlMySql
                | Self::SqlSqlite
                | Self::SqlMsSql
                | Self::SqlBigQuery
        )
    }
}

/// One field in the shared dynamic-memory record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryField {
    Id,
    Kind,
    Role,
    Intent,
    Tool,
    Inputs,
    Outputs,
    Content,
    SentAt,
    DemoLabel,
    ConversationId,
    ConversationTitle,
    Evidence,
    AccessCount,
    WriteCount,
}

impl MemoryField {
    pub const ALL: [Self; 15] = [
        Self::Id,
        Self::Kind,
        Self::Role,
        Self::Intent,
        Self::Tool,
        Self::Inputs,
        Self::Outputs,
        Self::Content,
        Self::SentAt,
        Self::DemoLabel,
        Self::ConversationId,
        Self::ConversationTitle,
        Self::Evidence,
        Self::AccessCount,
        Self::WriteCount,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Kind => "kind",
            Self::Role => "role",
            Self::Intent => "intent",
            Self::Tool => "tool",
            Self::Inputs => "inputs",
            Self::Outputs => "outputs",
            Self::Content => "content",
            Self::SentAt => "sentAt",
            Self::DemoLabel => "demoLabel",
            Self::ConversationId => "conversationId",
            Self::ConversationTitle => "conversationTitle",
            Self::Evidence => "evidence",
            Self::AccessCount => "accessCount",
            Self::WriteCount => "writeCount",
        }
    }

    fn parse(value: &str) -> Result<Self, MemoryQueryError> {
        let normalized = value
            .chars()
            .filter(|character| *character != '_')
            .flat_map(char::to_lowercase)
            .collect::<String>();
        Self::ALL
            .into_iter()
            .find(|field| {
                field
                    .as_str()
                    .chars()
                    .flat_map(char::to_lowercase)
                    .eq(normalized.chars())
            })
            .ok_or_else(|| MemoryQueryError::new(format!("unknown_memory_field:{value}")))
    }

    const fn is_numeric(self) -> bool {
        matches!(self, Self::AccessCount | Self::WriteCount)
    }
}

/// Scalar and list values used by filters, mutations, and result rows.
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryQueryValue {
    Null,
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<Self>),
}

impl MemoryQueryValue {
    pub(crate) fn canonical(&self) -> String {
        match self {
            Self::Null => String::from("null"),
            Self::Text(value) => format!("text:{value:?}"),
            Self::Integer(value) => format!("integer:{value}"),
            Self::Float(value) => format!("float:{value:.12}"),
            Self::Boolean(value) => format!("boolean:{value}"),
            Self::List(values) => format!(
                "list:[{}]",
                values
                    .iter()
                    .map(Self::canonical)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }

    pub(crate) fn display_text(&self) -> String {
        match self {
            Self::Null => String::new(),
            Self::Text(value) => value.clone(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::List(values) => values
                .iter()
                .map(Self::display_text)
                .collect::<Vec<_>>()
                .join("|"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Contains,
    Like,
    IsNull,
    IsNotNull,
}

impl ComparisonOperator {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Equal => "eq",
            Self::NotEqual => "ne",
            Self::LessThan => "lt",
            Self::LessThanOrEqual => "le",
            Self::GreaterThan => "gt",
            Self::GreaterThanOrEqual => "ge",
            Self::Contains => "contains",
            Self::Like => "like",
            Self::IsNull => "is_null",
            Self::IsNotNull => "is_not_null",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpression {
    Compare {
        field: MemoryField,
        operator: ComparisonOperator,
        value: MemoryQueryValue,
    },
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

impl FilterExpression {
    fn canonical(&self) -> String {
        match self {
            Self::Compare {
                field,
                operator,
                value,
            } => format!(
                "compare:{}:{}:{}",
                field.as_str(),
                operator.as_str(),
                value.canonical()
            ),
            Self::And(expressions) => format!(
                "and({})",
                expressions
                    .iter()
                    .map(Self::canonical)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Or(expressions) => format!(
                "or({})",
                expressions
                    .iter()
                    .map(Self::canonical)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Not(expression) => format!("not({})", expression.canonical()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Average,
    Minimum,
    Maximum,
    PopulationVariance,
    PopulationStandardDeviation,
}

impl AggregateFunction {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Sum => "sum",
            Self::Average => "average",
            Self::Minimum => "minimum",
            Self::Maximum => "maximum",
            Self::PopulationVariance => "population_variance",
            Self::PopulationStandardDeviation => "population_standard_deviation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateExpression {
    pub function: AggregateFunction,
    pub field: Option<MemoryField>,
    pub alias: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortExpression {
    pub field: MemoryField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryQueryOperation {
    Select,
    Insert,
    Update,
    Delete,
}

impl MemoryQueryOperation {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}

/// Language-neutral executable semantics.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryPlan {
    pub operation: MemoryQueryOperation,
    pub projection: Vec<MemoryField>,
    pub filter: Option<FilterExpression>,
    pub assignments: BTreeMap<MemoryField, MemoryQueryValue>,
    pub aggregates: Vec<AggregateExpression>,
    pub group_by: Vec<MemoryField>,
    pub order_by: Vec<SortExpression>,
    pub limit: Option<usize>,
    pub offset: usize,
}

impl MemoryQueryPlan {
    const fn empty(operation: MemoryQueryOperation) -> Self {
        Self {
            operation,
            projection: Vec::new(),
            filter: None,
            assignments: BTreeMap::new(),
            aggregates: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: 0,
        }
    }

    pub(crate) fn canonical(&self, max_matches: usize, max_iterations: usize) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "operation={}", self.operation.as_str());
        let _ = writeln!(out, "max_matches={max_matches}");
        let _ = writeln!(out, "max_iterations={max_iterations}");
        let _ = writeln!(
            out,
            "projection={}",
            self.projection
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
        if let Some(filter) = &self.filter {
            let _ = writeln!(out, "filter={}", filter.canonical());
        }
        for (field, value) in &self.assignments {
            let _ = writeln!(out, "set:{}={}", field.as_str(), value.canonical());
        }
        for aggregate in &self.aggregates {
            let _ = writeln!(
                out,
                "aggregate:{}:{}:{}",
                aggregate.function.as_str(),
                aggregate.field.map_or("*", MemoryField::as_str),
                aggregate.alias
            );
        }
        let _ = writeln!(
            out,
            "group_by={}",
            self.group_by
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
        for order in &self.order_by {
            let direction = match order.direction {
                SortDirection::Ascending => "asc",
                SortDirection::Descending => "desc",
            };
            let _ = writeln!(out, "order_by={}:{direction}", order.field.as_str());
        }
        let _ = writeln!(out, "limit={:?}\noffset={}", self.limit, self.offset);
        out
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserEvidence {
    pub engine: String,
    pub grammar: String,
    pub full_match: bool,
    pub text_preserved: bool,
    pub syntax_link_count: usize,
}

/// Fully validated, reviewable, executable query.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledMemoryQuery {
    pub id: String,
    pub dialect: QueryDialect,
    pub parser: ParserEvidence,
    pub plan: MemoryQueryPlan,
    pub link_program: LinkRewriteProgram,
    pub limits: MemoryProgramLimits,
    learned_from: Option<String>,
    canonical_semantics: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl CompiledMemoryQuery {
    #[must_use]
    pub fn canonical_semantics(&self) -> &str {
        &self.canonical_semantics
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "memory_query", None);
        push_lino_node(&mut out, 2, "id", Some(&self.id));
        push_lino_node(&mut out, 2, "dialect", Some(self.dialect.as_str()));
        push_lino_node(&mut out, 2, "parser_engine", Some(&self.parser.engine));
        push_lino_node(&mut out, 2, "grammar", Some(&self.parser.grammar));
        let _ = writeln!(out, "  full_match {}", self.parser.full_match);
        let _ = writeln!(out, "  text_preserved {}", self.parser.text_preserved);
        push_lino_node(&mut out, 2, "operation", Some(self.plan.operation.as_str()));
        if let Some(learned_from) = &self.learned_from {
            push_lino_node(&mut out, 2, "learned_from", Some(learned_from));
        }
        let rendered = render_link_substitution_query(&self.link_program);
        push_lino_node(&mut out, 2, "link_cli_substitution", Some(&rendered));
        for rule in &self.link_program.rules {
            let effect = link_substitution_effect(rule);
            let _ = writeln!(out, "  effect {}", effect.as_str());
        }
        out.trim_end().to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryQueryError {
    pub message: String,
}

impl MemoryQueryError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for MemoryQueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for MemoryQueryError {}

/// Parse one exact query into the shared language-neutral plan.
///
/// This function is compiled unchanged into both the native crate and the
/// browser's no-std WASM worker, preventing parser and semantic drift.
pub(crate) fn parse_memory_query_plan(
    source: &str,
    dialect: QueryDialect,
) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let plan = if dialect.is_sql() {
        sql::parse_sql(source)?
    } else if dialect == QueryDialect::GraphQl {
        graphql::parse_graphql(source)?
    } else {
        return Err(MemoryQueryError::new("natural_language_requires_template"));
    };
    validate_plan(&plan)?;
    Ok(plan)
}

/// Parse, validate, and lower one exact query surface.
#[cfg(not(target_arch = "wasm32"))]
pub fn compile_memory_query(
    source: &str,
    dialect: QueryDialect,
    limits: MemoryProgramLimits,
) -> Result<CompiledMemoryQuery, MemoryQueryError> {
    compile_exact_memory_query(source, dialect, limits, None)
}

#[cfg(not(target_arch = "wasm32"))]
fn compile_exact_memory_query(
    source: &str,
    dialect: QueryDialect,
    limits: MemoryProgramLimits,
    learned_from: Option<String>,
) -> Result<CompiledMemoryQuery, MemoryQueryError> {
    if limits.max_matches == 0 || limits.max_iterations == 0 {
        return Err(MemoryQueryError::new("memory_query_bounds_zero"));
    }
    let parser = validate_exact_syntax(source, dialect)?;
    let plan = parse_memory_query_plan(source, dialect)?;
    let canonical_semantics = plan.canonical(limits.max_matches, limits.max_iterations);
    let id = stable_id("memory_query", &canonical_semantics);
    let link_program = lowering::lower_to_link_program(&plan, limits);
    lowering::validate_link_program(&plan, &link_program, limits).map_err(MemoryQueryError::new)?;
    Ok(CompiledMemoryQuery {
        id,
        dialect,
        parser,
        plan,
        link_program,
        limits,
        learned_from,
        canonical_semantics,
    })
}

fn validate_plan(plan: &MemoryQueryPlan) -> Result<(), MemoryQueryError> {
    for (field, value) in &plan.assignments {
        validate_query_value(value)?;
        if field.is_numeric() && !matches!(value, MemoryQueryValue::Integer(number) if *number >= 0)
        {
            return Err(MemoryQueryError::new(format!(
                "invalid_memory_counter:{}",
                field.as_str()
            )));
        }
    }
    if let Some(filter) = &plan.filter {
        validate_filter_values(filter)?;
    }
    for aggregate in &plan.aggregates {
        if aggregate.function != AggregateFunction::Count
            && !aggregate.field.is_some_and(MemoryField::is_numeric)
        {
            return Err(MemoryQueryError::new(format!(
                "non_numeric_aggregate:{}",
                aggregate.function.as_str()
            )));
        }
    }
    Ok(())
}

fn validate_filter_values(filter: &FilterExpression) -> Result<(), MemoryQueryError> {
    match filter {
        FilterExpression::Compare { value, .. } => validate_query_value(value),
        FilterExpression::And(expressions) | FilterExpression::Or(expressions) => {
            for expression in expressions {
                validate_filter_values(expression)?;
            }
            Ok(())
        }
        FilterExpression::Not(expression) => validate_filter_values(expression),
    }
}

fn validate_query_value(value: &MemoryQueryValue) -> Result<(), MemoryQueryError> {
    match value {
        MemoryQueryValue::Float(number) if !number.is_finite() => {
            Err(MemoryQueryError::new("memory_query_number_non_finite"))
        }
        MemoryQueryValue::List(values) => {
            for value in values {
                validate_query_value(value)?;
            }
            Ok(())
        }
        MemoryQueryValue::Null
        | MemoryQueryValue::Text(_)
        | MemoryQueryValue::Integer(_)
        | MemoryQueryValue::Float(_)
        | MemoryQueryValue::Boolean(_) => Ok(()),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "meta-language"))]
fn validate_exact_syntax(
    source: &str,
    dialect: QueryDialect,
) -> Result<ParserEvidence, MemoryQueryError> {
    use meta_language::{LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};

    let label = dialect
        .meta_language_label()
        .ok_or_else(|| MemoryQueryError::new("exact_grammar_unavailable"))?;
    let network = LinkNetwork::parse(source, label, ParseConfiguration::default());
    let verification = network.verify_full_match(None);
    let syntax_link_count = network
        .projected_links(NetworkProjection::ConcreteSyntax)
        .filter(|link| link.metadata().link_type() == Some(LinkType::Syntax))
        .count();
    let evidence = ParserEvidence {
        engine: String::from("meta_language"),
        grammar: label.to_owned(),
        full_match: verification.is_clean(),
        text_preserved: network.reconstruct_text() == source,
        syntax_link_count,
    };
    if !evidence.full_match || !evidence.text_preserved || evidence.syntax_link_count == 0 {
        return Err(MemoryQueryError::new(format!(
            "exact_parser_rejected:{label}"
        )));
    }
    Ok(evidence)
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "meta-language")))]
fn validate_exact_syntax(
    _source: &str,
    dialect: QueryDialect,
) -> Result<ParserEvidence, MemoryQueryError> {
    let grammar = dialect
        .meta_language_label()
        .ok_or_else(|| MemoryQueryError::new("exact_grammar_unavailable"))?;
    Ok(ParserEvidence {
        engine: String::from("built_in_exact_parser"),
        grammar: grammar.to_owned(),
        full_match: true,
        text_preserved: true,
        syntax_link_count: 0,
    })
}
