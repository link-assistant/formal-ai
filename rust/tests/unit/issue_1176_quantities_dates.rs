use formal_ai::{FormalAiEngine, SymbolicAnswer};

// Issue #1176: quantities, dates and statistics answered by exact local
// computation instead of the generic fallback.
//
// The calendar-offset probes run against the already-wired
// `calendar_reasoning` dispatch entry. The statistics, unit-conversion and
// word-problem probes assert the post-integration state: they pass once
// `handle_statistics`, `handle_unit_conversion` and `handle_word_problem`
// are wired into the specialized-handler table, with statistics and unit
// conversion ordered after the coding handlers (so a "rust range" prompt
// stays with coding) and word problems before the generic fallback.

fn cites_response_evidence(response: &SymbolicAnswer, link: &str) -> bool {
    response.evidence_links.iter().any(|found| found == link)
}

// "100 days after Monday" must shift by the stated 100 days, not by one:
// 100 = 14 weeks + 2 days, and Monday + 2 days is Wednesday.
#[test]
fn stated_day_offset_shifts_by_the_stated_count() {
    let response = FormalAiEngine.answer("100 days after Monday");

    assert_eq!(response.intent, "calendar_weekday_relation");
    assert!(
        cites_response_evidence(&response, "response:calendar_weekday_relation"),
        "the offset answer should cite response:calendar_weekday_relation",
    );
    assert!(
        response.answer.contains("Wednesday"),
        "100 days after Monday is Wednesday, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("14 weeks + 2 days"),
        "the answer should derive the offset as weeks plus days, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "100 days after Monday is Wednesday. 100 days = 14 weeks + 2 days, and Monday + 2 days = Wednesday in the seven-day calendar cycle."
    );
}

// A bare "the day after X" states no offset and keeps its original ±1
// reading and prose (issue #1176 changes only stated-offset prompts).
#[test]
fn bare_day_after_keeps_the_plus_one_reading() {
    let response = FormalAiEngine.answer("the day after Monday");

    assert_eq!(response.intent, "calendar_weekday_relation");
    assert!(
        response.answer.contains("The day after Monday is Tuesday"),
        "the bare day-after reading should be unchanged, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "The day after Monday is Tuesday. I move Monday by +1 in the seven-day calendar cycle."
    );
}

// "30 days before Friday" shifts backwards by the stated amount:
// 30 = 4 weeks + 2 days, and Friday - 2 days is Wednesday.
#[test]
fn stated_offset_before_shifts_backward() {
    let response = FormalAiEngine.answer("30 days before Friday");

    assert_eq!(response.intent, "calendar_weekday_relation");
    assert!(
        response.answer.contains("Wednesday"),
        "30 days before Friday is Wednesday, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("4 weeks + 2 days"),
        "the answer should split the stated offset, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "30 days before Friday is Wednesday. 30 days = 4 weeks + 2 days, and Friday - 2 days = Wednesday in the seven-day calendar cycle."
    );
}

// A whole number of weeks cannot move the weekday, and the answer says so
// instead of pretending a shift of zero days happened.
#[test]
fn whole_week_offset_leaves_the_weekday_unchanged() {
    let response = FormalAiEngine.answer("2 weeks after Monday");

    assert_eq!(response.intent, "calendar_weekday_relation");
    assert!(
        response.answer.contains("Monday"),
        "2 weeks after Monday is still Monday, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("14 days exactly"),
        "the answer should state the exact week count, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "2 weeks after Monday is Monday. 2 weeks = 14 days exactly, so the weekday is unchanged in the seven-day calendar cycle."
    );
}

// CJK prompts state the unit without spaces ("100天"); the numeral token
// scan must stop at the day character and read the offset anyway.
#[test]
fn cjk_stated_offset_reads_the_day_unit_character() {
    let response = FormalAiEngine.answer("星期一之后100天是星期几？");

    assert_eq!(response.intent, "calendar_weekday_relation");
    assert!(
        response.answer.contains("星期三"),
        "星期一之后100天是星期三, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("14周 + 2天"),
        "the answer should derive the offset in Chinese, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "星期一之后100天是星期三。100天 = 14周 + 2天；星期一 + 2天 = 星期三（七天日历循环）。"
    );
}

// Russian answers inflect the source weekday (genitive after «после») and
// render the derivation through the seed's Russian template.
#[test]
fn russian_stated_offset_answers_in_russian() {
    let response = FormalAiEngine.answer("какой день будет через 100 дней после понедельника?");

    assert_eq!(response.intent, "calendar_weekday_relation");
    assert!(
        response.answer.contains("среда"),
        "100 дней после понедельника — среда, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("14 недель + 2 дня"),
        "the answer should split the offset in Russian, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "Через 100 дней после понедельника — среда. 100 дней = 14 недель + 2 дня; понедельник + 2 дня = среда в семидневном календарном цикле."
    );
}

// The issue's example: mean 108 / 6 = 18 and median (15 + 16) / 2 = 15.5,
// both exact, both with the arithmetic shown.
#[test]
fn mean_and_median_are_computed_exactly() {
    let response = FormalAiEngine
        .answer("Given the values 4, 8, 15, 16, 23, 42, what are the mean and the median?");

    assert_eq!(response.intent, "statistics");
    assert!(
        cites_response_evidence(&response, "response:statistics"),
        "the statistics answer should cite response:statistics",
    );
    assert!(
        response.answer.contains("mean: 18 (108 / 6 = 18)"),
        "the mean should be exact with its derivation, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("median: 15.5 ((15 + 16) / 2 = 15.5)"),
        "the median should state the halving, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "The values are 4, 8, 15, 16, 23, 42 (n = 6).\nmean: 18 (108 / 6 = 18)\nmedian: 15.5 ((15 + 16) / 2 = 15.5)"
    );
}

#[test]
fn mode_and_range_read_from_the_seed_vocabulary() {
    let response =
        FormalAiEngine.answer("For the values 2, 3, 3, 5, what are the mode and the range?");

    assert_eq!(response.intent, "statistics");
    assert!(
        response.answer.contains("mode: 3"),
        "3 is the only repeated value, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("range: 3 (5 - 2 = 3)"),
        "the range should show largest minus smallest, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "The values are 2, 3, 3, 5 (n = 4).\nmode: 3\nrange: 3 (5 - 2 = 3)"
    );
}

// The variance is the exact fraction 5460/36 rendered to seven digits with
// an explicit ≈ marker; the standard deviation is the only value that must
// be marked as an approximation.
#[test]
fn variance_and_standard_deviation_derive_from_exact_fractions() {
    let response = FormalAiEngine
        .answer("What are the variance and the standard deviation of 4, 8, 15, 16, 23, 42?");

    assert_eq!(response.intent, "statistics");
    assert!(
        response
            .answer
            .contains("variance: ≈151.6666667 (910 / 6 ≈ 151.6666667)"),
        "the variance value and derivation must agree, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("standard deviation: ≈12.3153021"),
        "the standard deviation should carry the ≈ marker, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "The values are 4, 8, 15, 16, 23, 42 (n = 6).\nvariance: ≈151.6666667 (910 / 6 ≈ 151.6666667)\nstandard deviation: ≈12.3153021"
    );
}

// Operation names come from the seed lexicon, so the same computation
// labels its results in the prompt's language.
#[test]
fn statistics_labels_follow_the_prompt_language() {
    let response = FormalAiEngine.answer("Каково среднее значение чисел 4, 8, 15, 16, 23, 42?");

    assert_eq!(response.intent, "statistics");
    assert!(
        response.answer.contains("среднее: 18 (108 / 6 = 18)"),
        "the mean line should be labelled in Russian, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "Значения: 4, 8, 15, 16, 23, 42 (n = 6).\nсреднее: 18 (108 / 6 = 18)"
    );
}

// 26.2 × 1.609344 = 42.1648128 exactly; the answer shows the factor it
// multiplied by.
#[test]
fn unit_conversion_multiplies_by_the_seed_factor() {
    let response = FormalAiEngine.answer("How many kilometers are 26.2 miles?");

    assert_eq!(response.intent, "unit_conversion");
    assert!(
        cites_response_evidence(&response, "response:unit_conversion"),
        "the conversion answer should cite response:unit_conversion",
    );
    assert!(
        response.answer.contains("26.2 × 1.609344 = 42.1648128"),
        "the answer should show the multiplication, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "26.2 miles is 42.1648128 kilometers. 26.2 × 1.609344 = 42.1648128, because 1 miles = 1.609344 kilometers."
    );
}

// The reverse direction divides by the same factor and marks the
// non-terminating quotient with ≈.
#[test]
fn unit_conversion_divides_when_the_factor_inverts() {
    let response = FormalAiEngine.answer("10 kilometers in miles");

    assert_eq!(response.intent, "unit_conversion");
    assert!(
        response.answer.contains("10 ÷ 1.609344 ≈ 6.2137119"),
        "the reverse conversion should divide by the factor, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "10 kilometers is ≈6.2137119 miles. 10 ÷ 1.609344 ≈ 6.2137119, because 1 miles = 1.609344 kilometers."
    );
}

// Temperature conversion is a formula, not a factor: each exact step is
// shown (25 × 9/5 + 32 = 77).
#[test]
fn temperature_formula_shows_every_step() {
    let response = FormalAiEngine.answer("convert 25 celsius to fahrenheit");

    assert_eq!(response.intent, "unit_conversion");
    assert!(
        response.answer.contains("25 × 9/5 + 32 = 77"),
        "the formula steps should be visible, got: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "25 celsius is 77 fahrenheit. 25 × 9/5 + 32 = 77."
    );
}

// Unknown units are declined rather than guessed.
#[test]
fn unknown_units_are_declined() {
    let response = FormalAiEngine.answer("How many meters are 5 furlongs?");

    assert_ne!(
        response.intent, "unit_conversion",
        "furlongs are not in the seed, so the prompt must not be claimed: {}",
        response.answer
    );
}

// A compound measurement is ambiguous (which of the three numbers converts?)
// and is declined rather than guessed.
#[test]
fn ambiguous_unit_lists_are_declined() {
    let response = FormalAiEngine.answer("5 feet 9 inches in cm");

    assert_ne!(
        response.intent, "unit_conversion",
        "three units state no single conversion, got: {}",
        response.answer
    );
}

// The price×count−payment pattern: 20 - 4 × 3 = 8, with the arithmetic
// shown in the answer.
#[test]
fn change_word_problem_shows_the_arithmetic() {
    let response = FormalAiEngine.answer(
        "I buy 4 pens at 3 dollars each and pay with a 20 dollar note. How much change do I get?",
    );

    assert_eq!(response.intent, "word_problem_change");
    assert!(
        cites_response_evidence(&response, "response:word_problem_change"),
        "the word-problem answer should cite response:word_problem_change",
    );
    assert!(
        response.answer.contains("The change is 8: 20 - 4 × 3 = 8"),
        "the answer should state and derive the change, got: {}",
        response.answer
    );
    assert_eq!(response.answer, "The change is 8: 20 - 4 × 3 = 8.");
}

#[test]
fn total_word_problem_multiplies_price_by_count() {
    let response = FormalAiEngine.answer("I buy 4 pens at 3 dollars each. What is the total?");

    assert_eq!(response.intent, "word_problem_total");
    assert!(
        response.answer.contains("The total is 12: 4 × 3 = 12"),
        "the answer should state and derive the total, got: {}",
        response.answer
    );
    assert_eq!(response.answer, "The total is 12: 4 × 3 = 12.");
}

// The markers are seed data in every language, so a Spanish purchase reads
// the same pattern; the derivation itself is language-neutral.
#[test]
fn spanish_change_word_problem_reads_the_seed_markers() {
    let response = FormalAiEngine
        .answer("Compra 4 bolígrafos a 3 dólares cada uno y paga con 20. ¿Cuánto cambio recibe?");

    assert_eq!(
        response.intent, "word_problem_change",
        "the Spanish markers (cada, paga con, cambio) should claim the prompt: {}",
        response.answer
    );
    assert!(
        response.answer.contains("20 - 4 × 3 = 8"),
        "the derivation is language-neutral, got: {}",
        response.answer
    );
    assert_eq!(response.answer, "El cambio es 8: 20 - 4 × 3 = 8.");
}

// Questions without a statistics operation or a unit conversion must not be
// claimed by the new handlers.
#[test]
fn non_quantity_questions_are_not_claimed() {
    for prompt in [
        "What is the deeper meaning of joy?",
        "What is 2 + 2?",
        "Who wrote The Master and Margarita?",
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_ne!(
            response.intent, "statistics",
            "prompt {prompt:?} is not a statistics question, got: {}",
            response.answer
        );
        assert_ne!(
            response.intent, "unit_conversion",
            "prompt {prompt:?} is not a unit conversion, got: {}",
            response.answer
        );
        assert_ne!(
            response.intent, "word_problem_change",
            "prompt {prompt:?} is not a word problem, got: {}",
            response.answer
        );
    }
}

// R1176-3: the percentile by linear interpolation between closest ranks (the
// C = 1 variant the seed meaning cites). The rank 90 is a parameter, not a
// data point, so it leaves the echoed list.
#[test]
fn percentile_interpolates_between_closest_ranks() {
    let response = FormalAiEngine.answer("What is the 90th percentile of 1, 2, 3, 4, 5?");

    assert_eq!(response.intent, "statistics");
    assert_eq!(
        response.answer,
        "The values are 1, 2, 3, 4, 5 (n = 5).\npercentile: 4.6 (p = 90; rank = 1 + (5 - 1) × 90 / 100 = 4.6; 4 + 0.6 × (5 - 4) = 4.6)"
    );
}

#[test]
fn percentile_joins_other_operations_and_languages() {
    let response =
        FormalAiEngine.answer("What are the mean and the 25th percentile of 4, 8, 15, 16, 23, 42?");
    assert_eq!(response.intent, "statistics");
    assert!(
        response.answer.contains("mean: 18 (108 / 6 = 18)")
            && response.answer.contains(
                "percentile: 9.75 (p = 25; rank = 1 + (6 - 1) × 25 / 100 = 2.25; 8 + 0.25 × (15 - 8) = 9.75)"
            ),
        "got: {}",
        response.answer
    );

    let russian = FormalAiEngine.answer("Каков 90-й перцентиль чисел 1, 2, 3, 4, 5?");
    assert!(
        russian
            .answer
            .contains("перцентиль: 4.6 (p = 90; ранг = 1 + (5 - 1) × 90 / 100 = 4.6;"),
        "got: {}",
        russian.answer
    );
    let spanish = FormalAiEngine.answer("¿Cuál es el percentil 90 de 1, 2, 3, 4, 5?");
    assert!(
        spanish
            .answer
            .contains("percentil: 4.6 (p = 90; posición ="),
        "got: {}",
        spanish.answer
    );
}

#[test]
fn percentile_without_a_valid_rank_is_declined() {
    for prompt in [
        "What is the percentile of 1, 2, 3?",
        "What is the 150th percentile of 1, 2, 3?",
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_ne!(
            response.intent, "statistics",
            "prompt {prompt:?} states no usable rank, got: {}",
            response.answer
        );
    }
}

// R1176-4: a month offset from a month name is exact modulo-12 arithmetic.
#[test]
fn month_offset_from_a_month_name_is_modulo_twelve() {
    let response = FormalAiEngine.answer("2 months after January");
    assert_eq!(response.intent, "calendar_month_relation");
    assert_eq!(
        response.answer,
        "2 months after January is March. January is month 1; 1 + 2 = 3, and 3 ≡ 3 (mod 12), which is March in the twelve-month calendar cycle."
    );

    let forward = FormalAiEngine.answer("What month is 5 months after November?");
    assert!(
        forward
            .answer
            .contains("11 + 5 = 16, and 16 ≡ 4 (mod 12), which is April"),
        "got: {}",
        forward.answer
    );
    let backward = FormalAiEngine.answer("3 months before February");
    assert!(
        backward
            .answer
            .starts_with("3 months before February is November.")
            && backward.answer.contains("2 - 3 = -1, and -1 ≡ 11 (mod 12)"),
        "got: {}",
        backward.answer
    );
    let russian = FormalAiEngine.answer("через 2 месяца после января");
    assert!(
        russian
            .answer
            .starts_with("Через 2 месяца после месяца «январь» наступает март."),
        "got: {}",
        russian.answer
    );
    let chinese = FormalAiEngine.answer("三月之后2个月是几月？");
    assert!(
        chinese.answer.starts_with("三月之后2个月是五月。"),
        "got: {}",
        chinese.answer
    );
}

// R1176-4: months over weekdays are no whole number of days, so the honest
// answer is a clarification that shows the day span and names why.
#[test]
fn month_offset_from_a_weekday_asks_for_the_starting_date() {
    let response = FormalAiEngine.answer("3 months after Monday");
    assert_eq!(response.intent, "calendar_month_offset_clarification");
    assert_eq!(
        response.answer,
        "3 months after Monday has no single weekday answer: 3 months span 89 to 92 days depending on the starting date, because calendar months run 28 to 31 days, so the weekday it lands on is not fixed. Tell me the starting date, or state the offset in days or weeks."
    );

    let chinese = FormalAiEngine.answer("星期一之后3个月是星期几？");
    assert_eq!(chinese.intent, "calendar_month_offset_clarification");
    assert!(
        chinese.answer.contains("89到92天"),
        "got: {}",
        chinese.answer
    );
}
