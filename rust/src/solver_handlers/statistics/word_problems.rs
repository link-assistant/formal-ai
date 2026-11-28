use super::*;

/// Word-problem entry point for the price×count pattern of issue #1176.
///
/// "buys 4 pens at 3 dollars each and pays with 20; what is the change?" → 20 −
/// 4 × 3 = 8 — and its total sibling ("what is the total?" → 4 × 3 = 12). The
/// markers are seed vocabulary; the arithmetic is shown in the answer.
pub fn handle_word_problem(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !mentions_marker(normalized, MARKER_UNIT_PRICE) {
        if let Some(answer) = one_item_change(prompt, normalized, log) {
            return Some(answer);
        }
        return word_relations::relation_word_problem(prompt, normalized, log);
    }
    let lowered = prompt.to_lowercase();
    let (values, positions) = stated_numbers(&lowered)?;
    let language = detect_language(prompt).slug();

    // Change: a unit price, a count, and a payment. The price is stated
    // nearest before the unit-price marker ("at 3 dollars each"), the payment
    // is the last stated number, and the count is the remaining one.
    if mentions_marker(normalized, MARKER_PAYMENT)
        && mentions_marker(normalized, MARKER_CHANGE)
        && values.len() == 3
    {
        let each_at = marker_position(&lowered, MARKER_UNIT_PRICE).unwrap_or(0);
        let price_index = nearest_before(&positions, each_at).unwrap_or(0);
        let payment_index = values.len() - 1;
        let count_index =
            (0..values.len()).find(|index| *index != price_index && *index != payment_index)?;
        let price = values[price_index];
        let count = values[count_index];
        let payment = values[payment_index];
        let cost = count.mul(price)?;
        let change = payment.sub(cost)?;
        if change.is_negative() {
            // A payment below the cost is not a change question this small
            // pattern can answer; decline instead of guessing.
            return None;
        }
        log.append("word_problem:unit_price", price.render());
        log.append("word_problem:count", count.render());
        log.append("word_problem:payment", payment.render());
        log.append("word_problem:change", change.render());
        let derivation = format!(
            "{} - {} × {} = {}",
            payment.render(),
            count.render(),
            price.render(),
            change.render()
        );
        log.append("word_problem:derivation", derivation.clone());
        let body = localized_response("word_problem_change", language)
            .map(|template| {
                template
                    .replace(concat!("{", "change}"), &change.render())
                    .replace(concat!("{", "derivation}"), &derivation)
            })
            .unwrap_or(derivation);
        return Some(finalize_simple(
            prompt,
            log,
            "word_problem_change",
            "response:word_problem_change",
            &body,
            1.0,
        ));
    }

    // Total: a unit price and a count ("4 pens at 3 dollars each; what is
    // the total?" → 4 × 3 = 12).
    if mentions_marker(normalized, MARKER_TOTAL) && values.len() == 2 {
        let each_at = marker_position(&lowered, MARKER_UNIT_PRICE).unwrap_or(0);
        let price_index = nearest_before(&positions, each_at).unwrap_or(0);
        let count_index = (0..values.len()).find(|index| *index != price_index)?;
        let price = values[price_index];
        let count = values[count_index];
        let total = count.mul(price)?;
        log.append("word_problem:unit_price", price.render());
        log.append("word_problem:count", count.render());
        log.append("word_problem:total", total.render());
        let derivation = format!(
            "{} × {} = {}",
            count.render(),
            price.render(),
            total.render()
        );
        log.append("word_problem:derivation", derivation.clone());
        let body = localized_response("word_problem_total", language)
            .map(|template| {
                template
                    .replace(concat!("{", "total}"), &total.render())
                    .replace(concat!("{", "derivation}"), &derivation)
            })
            .unwrap_or(derivation);
        return Some(finalize_simple(
            prompt,
            log,
            "word_problem_total",
            "response:word_problem_total",
            &body,
            1.0,
        ));
    }

    word_relations::relation_word_problem(prompt, normalized, log)
}
