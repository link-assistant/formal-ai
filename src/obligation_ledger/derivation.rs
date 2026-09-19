//! Reading a request's clauses, and each clause's expectation, out of data.
//!
//! Plan 05 leaves 5, 6 and 7 own this file. The clause splitter is the one
//! `task_obligations` already splits on — the seeded enumeration cues — and the
//! expectation derivation is the rule set of
//! `data/meta/obligation-evidence-contract.lino`, so a new expectation shape is a
//! data edit rather than a `match` arm here.

use crate::agentic_coding::general_planner::GeneralChangePlan;
use crate::engine::stable_id;

use super::ObligationExpectation;

/// The clauses a request enumerates, each with its UTF-8 byte span in the
/// request exactly as the user wrote it.
///
/// The cut points are the seeded enumeration cues — the same role
/// `task_obligations` already splits on — but a cue only cuts where it *opens* a
/// clause: at the start of the request, or just after a sentence ends. A "first"
/// inside "whose first line is exactly `---`" describes a file's contents and
/// opens nothing (issue #1099).
///
/// The sentence terminators include `。`, `！`, `？` and `।` beside the Latin
/// ones, because a Chinese or Hindi request ends its sentences with those and a
/// splitter that does not know them reads five languages as one clause.
#[must_use]
pub fn clauses_with_spans(request: &str) -> Vec<(String, (usize, usize))> {
    let cues = crate::seed::lexicon().words_for_role(crate::seed::ROLE_ENUMERATION_CUE);
    if cues.is_empty() {
        return vec![(request.trim().to_owned(), (0, request.len()))];
    }
    let mut boundaries = vec![0_usize];
    for (index, _) in request.char_indices() {
        if index == 0 || !opens_a_clause(request, index) {
            continue;
        }
        let rest = request[index..].to_lowercase();
        if cues.iter().any(|cue| starts_with_cue(&rest, cue)) {
            boundaries.push(index);
        }
    }
    boundaries.push(request.len());
    boundaries.dedup();
    boundaries
        .windows(2)
        .filter_map(|window| {
            let raw = &request[window[0]..window[1]];
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }
            let start = window[0] + raw.find(trimmed)?;
            Some((trimmed.to_owned(), (start, start + trimmed.len())))
        })
        .collect()
}

/// Whether the byte at `index` begins a clause: everything before it either ends
/// a sentence or is nothing but whitespace.
fn opens_a_clause(request: &str, index: usize) -> bool {
    let before = request[..index].trim_end();
    before.is_empty()
        || before.ends_with(['.', '!', '?', ';', ':', '\n', '。', '！', '？', '।', '॥'])
}

/// Whether `rest` opens with `cue` as a whole word — or, for a script that does
/// not space its words, as its own leading run of characters.
///
/// The word-boundary test is only meaningful where words are spaced. `然后确认`
/// is a cue followed immediately by the verb it introduces, and `确` is
/// alphanumeric, so demanding a non-alphanumeric follower reads Chinese and
/// Japanese as one clause and every enumerated obligation in those languages is
/// silently merged into the first.
fn starts_with_cue(rest: &str, cue: &str) -> bool {
    let Some(after) = rest.strip_prefix(cue) else {
        return false;
    };
    if cue.chars().any(is_unspaced_script) {
        return true;
    }
    after
        .chars()
        .next()
        .is_none_or(|character| !character.is_alphanumeric())
}

/// Whether a character belongs to a script that does not separate its words with
/// spaces: the CJK ideographs, the kana, and the Hangul syllables.
const fn is_unspaced_script(character: char) -> bool {
    matches!(character,
        '\u{3040}'..='\u{30ff}'
            | '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{ac00}'..='\u{d7af}'
            | '\u{f900}'..='\u{faff}')
}

/// Read one clause's expectation out of the clause itself.
///
/// The composer that already plans a standalone request is the authority on what
/// artifact a clause names; when it reads one, the plan's own mode says which
/// shape of observation would settle the clause. When it reads none, a clause
/// that still names an observable action is a symbolic check, and a clause that
/// names neither is `Underivable` — kept, split, and finally reported, never
/// discarded.
#[must_use]
pub fn derive_expectation(clause: &str) -> ObligationExpectation {
    let rules = ExpectationRules::shipped();
    if let Some(plan) = crate::agentic_coding::general_planner::compose_general_change_plan(clause)
    {
        let rule = rules.for_plan_mode(plan.mode.slug());
        return shape_for(&plan, rule);
    }
    if crate::task_decomposition::is_checkable(clause) {
        let rule = rules.for_generated_check();
        if rule.is_none_or(|rule| rule.expectation == "symbolic_check") {
            return ObligationExpectation::SymbolicCheck {
                check_id: format!(
                    "obligation:{}",
                    stable_id("obligation_check", clause.trim())
                ),
            };
        }
    }
    ObligationExpectation::Underivable {
        reason: rules
            .for_absent_artifact()
            .map_or_else(|| String::from("no_artifact_in_clause"), |rule| rule.reason),
    }
}

/// Build the expectation a matched rule names out of the plan it matched.
///
/// Hashing the observed bytes is only honest when the request states what they
/// must be; when it does not, the rule's `otherwise` shape applies and the
/// observable claim is the command's exit status, never a digest nothing
/// declared (plan 05 risk 4).
fn shape_for(plan: &GeneralChangePlan, rule: Option<ExpectationRule>) -> ObligationExpectation {
    let Some(rule) = rule else {
        return ObligationExpectation::Underivable {
            reason: format!("no_rule_for_plan_mode_{}", plan.mode.slug()),
        };
    };
    let declared = expected_digest(&plan.content);
    let shape = match (&declared, rule.otherwise.as_deref()) {
        (None, Some(otherwise)) => otherwise,
        _ => rule.expectation.as_str(),
    };
    match shape {
        "file_bytes" => ObligationExpectation::FileBytes {
            path: plan.target.clone(),
            sha256: declared,
        },
        "output_hash" => declared.map_or_else(
            || ObligationExpectation::CommandExit {
                command: plan.verification_command.clone(),
                expected_exit: 0,
            },
            |sha256| ObligationExpectation::OutputHash {
                command: plan.verification_command.clone(),
                sha256,
            },
        ),
        "command_exit" => ObligationExpectation::CommandExit {
            command: plan.verification_command.clone(),
            expected_exit: 0,
        },
        "symbolic_check" => ObligationExpectation::SymbolicCheck {
            check_id: format!("obligation:{}", stable_id("obligation_check", &plan.goal)),
        },
        _ => ObligationExpectation::Underivable {
            reason: rule.reason,
        },
    }
}

/// One `rule` row of `data/meta/obligation-evidence-contract.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectationRule {
    /// The rule's name, as the document writes it.
    pub name: String,
    /// The condition the rule fires on.
    pub when: String,
    /// The general-plan mode the condition narrows to, when it names one.
    pub mode: String,
    /// The expectation shape the rule names.
    pub expectation: String,
    /// The shape to use when the request declared no bytes to hash.
    pub otherwise: Option<String>,
    /// The reason an underivable rule records.
    pub reason: String,
}

/// The derivation rules, read from the contract document rather than written as
/// a `match` here, so a new expectation shape is a data edit (plan 05 leaf 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectationRules {
    /// The rule rows, in document order.
    pub rules: Vec<ExpectationRule>,
}

/// The contract document, embedded so the derivation needs no filesystem.
const CONTRACT_LINO: &str = include_str!("../../data/meta/obligation-evidence-contract.lino");

/// The `when` keys the contract's rule rows are addressed by. Each is one token,
/// because a rule condition is a key the document and this reader agree on, not
/// a sentence anybody reads.
const WHEN_PLAN_MODE: &str = "general_plan_mode";
/// The condition naming a clause a reader can already tell is done.
const WHEN_GENERATED_CHECK: &str = "generated_check";
/// The condition naming a clause with neither a plan nor an observable action.
const WHEN_NO_ARTIFACT: &str = "no_artifact";

impl ExpectationRules {
    /// Parse the checked-in contract.
    #[must_use]
    pub fn shipped() -> Self {
        Self::from_lino(CONTRACT_LINO)
    }

    /// Parse a contract document's `rule` rows.
    #[must_use]
    pub fn from_lino(text: &str) -> Self {
        let mut rules: Vec<ExpectationRule> = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix("rule ") {
                rules.push(ExpectationRule {
                    name: unquoted(name.trim()),
                    when: String::new(),
                    mode: String::new(),
                    expectation: String::new(),
                    otherwise: None,
                    reason: String::new(),
                });
                continue;
            }
            let Some(rule) = rules.last_mut() else {
                continue;
            };
            if let Some(value) = field(trimmed, "when") {
                rule.when = value;
            } else if let Some(value) = field(trimmed, "mode") {
                rule.mode = value;
            } else if let Some(value) = field(trimmed, "expectation") {
                rule.expectation = value;
            } else if let Some(value) = field(trimmed, "otherwise") {
                rule.otherwise = Some(value);
            } else if let Some(value) = field(trimmed, "reason") {
                rule.reason = value;
            }
        }
        Self { rules }
    }

    /// The rule whose `when` names this general-plan mode.
    #[must_use]
    pub fn for_plan_mode(&self, mode: &str) -> Option<ExpectationRule> {
        self.rules
            .iter()
            .find(|rule| rule.when == WHEN_PLAN_MODE && rule.mode == mode)
            .cloned()
    }

    /// The rule that governs a clause carrying a generated check.
    #[must_use]
    pub fn for_generated_check(&self) -> Option<ExpectationRule> {
        self.matching(WHEN_GENERATED_CHECK)
    }

    /// The rule that governs a clause naming no artifact at all.
    #[must_use]
    pub fn for_absent_artifact(&self) -> Option<ExpectationRule> {
        self.matching(WHEN_NO_ARTIFACT)
    }

    fn matching(&self, when: &str) -> Option<ExpectationRule> {
        self.rules.iter().find(|rule| rule.when == when).cloned()
    }
}

/// Read `key "value"` from a contract line when the key matches exactly.
fn field(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some(unquoted(rest.trim()))
}

/// Strip one pair of surrounding quotes, if the value carries them.
fn unquoted(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}

/// The digest a read-back of the requested bytes must produce.
///
/// Two normalizations, and both are about the difference between the prose and
/// the file. The sentence that states the bytes ends with its own full stop —
/// "containing Gemfile.lock." names twelve bytes, not thirteen — and the file a
/// write produces ends with exactly one newline, which is what the verification
/// command `cat <path>` observes. Content the request never stated yields no
/// digest at all rather than a guessed one.
#[must_use]
pub fn expected_digest(content: &str) -> Option<String> {
    let stated = content.trim_end();
    let stated = stated
        .strip_suffix(['.', '。', '।', '!', '？', '?'])
        .unwrap_or(stated)
        .trim_end();
    if stated.is_empty() {
        return None;
    }
    let normalized = if stated.ends_with('\n') {
        stated.to_owned()
    } else {
        format!("{stated}\n")
    };
    Some(crate::source_fetch::sha256_hex(normalized.as_bytes()))
}
