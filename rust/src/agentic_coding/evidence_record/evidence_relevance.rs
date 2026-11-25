use super::*;

/// How a route below this one already produces `target` from the whole request,
/// or `None` when none of them does.
///
/// A composed change plan states the file it was compiled for, so asking it is
/// exact and costs nothing -- and it is the only one of the two that sees a file
/// a shell *redirect* produces, since "Run 'printf learned-output' and write its
/// exact stdout to reports/learned.txt" is delivered by `>` and never by a write
/// call at all.
///
/// Everything else is asked by running it rather than by listing it: plan the
/// request through the routes below, answer each planned call with a bare
/// success, and see whether `target` is among the files they write. That is what
/// reaches the recipes, which declare nothing and are recognised from the
/// artifact they write. A route's own state machine is what decides --
/// `plan_general_change_step` records its plan before writing the file the
/// caller named, so a first step alone would report that nobody owns
/// `policy/retention.md`, and a table of routes that "own an artifact" would
/// have to carry that knowledge secondhand and go stale.
///
/// The probe starts from a conversation holding the request and nothing else, so
/// the answer depends on the request alone and does not drift as the real run
/// accumulates results: a recipe that owns a destination owns it on every turn,
/// and its mid-run plan to read the file back would otherwise look like nobody
/// owning it.
pub(super) fn later_route_delivering(
    task: &str,
    obligation: &DeliveryBinding,
    tool_names: &[&str],
) -> Option<&'static str> {
    // A schema or opening-line declaration constrains bytes that still have to
    // be derived from the residual investigation; it is not itself the file's
    // payload. The broad literal parser can nevertheless recover the words
    // after "with"/"exactly" as content. Do not let that lossy interpretation
    // displace the evidence transaction. A true literal write has no residual
    // finding to derive, so it never reaches this function.
    let derived_delivery = obligation.first_line.is_some() || !obligation.field_lines.is_empty();
    if derived_delivery {
        return None;
    }
    if let Some(plan) = compose_general_change_plan(task)
        && plan.target == obligation.target
    {
        return Some("declined_composed_target");
    }
    // A learning report is named by its artifact and owns it even on the turn
    // it can only ask for the missing memory (issue #686).
    if super::super::learning_report::route(task)
        .is_some_and(|report| obligation.target == report.path)
    {
        return Some("declined_learning_report");
    }
    settled_route_delivers(task, &obligation.target, tool_names).then_some("declined_settled_route")
}

pub(super) fn relevance_score(
    line: &str,
    terms: &[String],
    condition_requested: bool,
    implementation_requested: bool,
    serialized_relationship: Option<&str>,
    source_authority: usize,
) -> (
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
) {
    let words = line
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let matches = |term: &str| {
        words.iter().any(|word| word == term)
            || term
                .split('_')
                .all(|part| words.iter().any(|word| word == part))
    };
    let overlap = terms.iter().filter(|term| matches(term)).count();
    let requested_property = usize::from(terms.last().is_some_and(|term| matches(term)));
    let declaration_shape = if looks_like_declaration(line) {
        1 + usize::from(!looks_like_local_binding(line))
            + usize::from(looks_like_exposed_declaration(line))
    } else {
        0
    };
    let bound_requested = terms
        .iter()
        .any(|term| crate::seed::lexicon().mentions_role(crate::seed::ROLE_CODING_BOUND_CUE, term));
    let semantic_overlap = usize::from(
        bound_requested
            && words.iter().any(|word| {
                crate::seed::lexicon().mentions_role(crate::seed::ROLE_CODING_BOUND_CUE, word)
            }),
    );
    let code_shape = usize::from(line.contains(['{', '}', '(', ')', ';', '=', '<', '>']));
    let direct_serialized_identity =
        usize::from(serialized_relationship.is_some_and(|relationship| {
            words
                .windows(2)
                .any(|pair| pair[0] == relationship && pair[1] == "id")
        }));
    (
        usize::from(implementation_requested && !is_quoted_or_commented_source(source_text(line))),
        direct_serialized_identity,
        requested_property,
        usize::from(condition_requested && looks_like_condition(line)),
        source_authority,
        usize::from(condition_requested && looks_like_instance_condition(line)),
        declaration_shape,
        semantic_overlap,
        overlap,
        code_shape,
    )
}
