// Twin of numericCalculationResponse in the maintained worker arithmetic handler.
pub(crate) fn numeric_calculation_response<'a>(
    prompt: &str,
    expression: &str,
    formatted: &'a str,
) -> Option<&'a str> {
    static NUMERIC: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"^-?[0-9]+(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?$")
            .expect("valid scalar numeric grammar")
    });
    if !NUMERIC.is_match(formatted) {
        return None;
    }
    let request = prompt.trim();
    let (prefix, clause) = request.rsplit_once(". ")?;
    let clause = clause
        .strip_suffix('.')
        .unwrap_or(clause)
        .trim()
        .to_lowercase();
    let lexicon = crate::seed::lexicon();
    if !lexicon
        .words_for_role("numeric-response-only-clause")
        .contains(&clause)
    {
        return None;
    }
    let operand = expression.trim().to_lowercase();
    let prefix = prefix.trim().to_lowercase();
    let complete_operand = prefix == operand
        || lexicon
            .words_for_role("calculation_request_cue")
            .iter()
            .any(|cue| {
                prefix
                    .strip_prefix(&format!("{cue} "))
                    .is_some_and(|tail| tail.trim() == operand)
            });
    complete_operand.then_some(formatted)
}
