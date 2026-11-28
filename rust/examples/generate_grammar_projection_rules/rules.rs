//! The authored rule table for `generate_grammar_projection_rules` —
//! split out of the example root when the table outgrew the file-size
//! gate. Pure data: the root file owns how it serializes.

use super::RuleRow;

mod primary_spine;
mod reverse_spine;

/// Ordered rule groups preserve first-match claim resolution across both directions.
pub const RULE_ROW_GROUPS: &[&[RuleRow]] = &[primary_spine::RULE_ROWS, reverse_spine::RULE_ROWS];
