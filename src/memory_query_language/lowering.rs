use super::{ComparisonOperator, FilterExpression, MemoryQueryOperation, MemoryQueryPlan};
use crate::links_substitution_query::{
    LinkPattern, LinkRewriteProgram, LinkRewriteRule, Slot, link_substitution_effect,
};
use crate::memory_program::MemoryProgramLimits;
use crate::substitution::CrudEvent;

pub(super) fn lower_to_link_program(
    plan: &MemoryQueryPlan,
    limits: MemoryProgramLimits,
) -> LinkRewriteProgram {
    let rules = match plan.operation {
        MemoryQueryOperation::Select => read_rules(plan),
        MemoryQueryOperation::Insert => create_rules(plan),
        MemoryQueryOperation::Update => update_rules(plan),
        MemoryQueryOperation::Delete => delete_rules(plan),
    };
    LinkRewriteProgram::new(
        rules,
        limits.max_matches.saturating_mul(limits.max_iterations),
    )
}

/// Reject a plan/program pair whose executable link effect or step bound has
/// drifted from the typed semantics. This is checked both after lowering and
/// immediately before execution so the link program is an enforced contract.
pub(super) fn validate_link_program(
    plan: &MemoryQueryPlan,
    program: &LinkRewriteProgram,
    limits: MemoryProgramLimits,
) -> Result<(), &'static str> {
    let expected = match plan.operation {
        MemoryQueryOperation::Select => CrudEvent::Read,
        MemoryQueryOperation::Insert => CrudEvent::Create,
        MemoryQueryOperation::Update => CrudEvent::Update,
        MemoryQueryOperation::Delete => CrudEvent::Delete,
    };
    if program.rules.is_empty()
        || !program
            .rules
            .iter()
            .all(|rule| link_substitution_effect(rule) == expected)
    {
        return Err("link_effect_drift");
    }
    let expected_steps = limits.max_matches.saturating_mul(limits.max_iterations);
    if program.max_steps != expected_steps || expected_steps == 0 {
        return Err("link_bound_drift");
    }
    Ok(())
}

fn read_rules(plan: &MemoryQueryPlan) -> Vec<LinkRewriteRule> {
    let patterns = equality_patterns(plan.filter.as_ref());
    if patterns.is_empty() {
        let pattern = any_link();
        vec![LinkRewriteRule {
            pattern: Some(pattern.clone()),
            replacement: Some(pattern),
        }]
    } else {
        patterns
            .into_iter()
            .map(|pattern| LinkRewriteRule {
                pattern: Some(pattern.clone()),
                replacement: Some(pattern),
            })
            .collect()
    }
}

fn create_rules(plan: &MemoryQueryPlan) -> Vec<LinkRewriteRule> {
    plan.assignments
        .iter()
        .map(|(field, value)| LinkRewriteRule {
            pattern: None,
            replacement: Some(LinkPattern {
                index: None,
                source: Slot::Value(format!("field:{}", field.as_str())),
                target: Slot::Value(format!("value:{}", value.display_text())),
            }),
        })
        .collect()
}

fn update_rules(plan: &MemoryQueryPlan) -> Vec<LinkRewriteRule> {
    plan.assignments
        .iter()
        .map(|(field, value)| LinkRewriteRule {
            pattern: Some(LinkPattern {
                index: Some(Slot::Variable(String::from("i"))),
                source: Slot::Value(format!("field:{}", field.as_str())),
                target: Slot::Variable(String::from("old")),
            }),
            replacement: Some(LinkPattern {
                index: Some(Slot::Variable(String::from("i"))),
                source: Slot::Value(format!("field:{}", field.as_str())),
                target: Slot::Value(format!("value:{}", value.display_text())),
            }),
        })
        .collect()
}

fn delete_rules(plan: &MemoryQueryPlan) -> Vec<LinkRewriteRule> {
    let mut patterns = equality_patterns(plan.filter.as_ref());
    if patterns.is_empty() {
        patterns.push(any_link());
    }
    patterns
        .into_iter()
        .map(|pattern| LinkRewriteRule {
            pattern: Some(pattern),
            replacement: None,
        })
        .collect()
}

fn equality_patterns(filter: Option<&FilterExpression>) -> Vec<LinkPattern> {
    let Some(filter) = filter else {
        return Vec::new();
    };
    match filter {
        FilterExpression::Compare {
            field,
            operator: ComparisonOperator::Equal,
            value,
        } => vec![LinkPattern {
            index: Some(Slot::Variable(String::from("i"))),
            source: Slot::Value(format!("field:{}", field.as_str())),
            target: Slot::Value(format!("value:{}", value.display_text())),
        }],
        FilterExpression::And(expressions) | FilterExpression::Or(expressions) => expressions
            .iter()
            .flat_map(|expression| equality_patterns(Some(expression)))
            .collect(),
        FilterExpression::Compare { .. } | FilterExpression::Not(_) => Vec::new(),
    }
}

fn any_link() -> LinkPattern {
    LinkPattern {
        index: Some(Slot::Variable(String::from("i"))),
        source: Slot::Variable(String::from("source")),
        target: Slot::Variable(String::from("target")),
    }
}
