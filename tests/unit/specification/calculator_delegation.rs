//! Issue #96: delegate calculator-parsable expressions to link-calculator.
//!
//! These tests cover the formal-ai boundary: natural-language prompt wrappers
//! are stripped in the four currently supported languages, calculator-native
//! expressions are handled by the upstream crate, and local arithmetic remains
//! as a fallback for syntax that calculator does not support yet.

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn assert_calculation(prompt: &str, expected_fragments: &[&str]) -> SymbolicAnswer {
    let response = answer(prompt);
    assert_eq!(
        response.intent, "calculation",
        "prompt {prompt:?} should be delegated to calculator, got intent={} answer={}",
        response.intent, response.answer,
    );
    for fragment in expected_fragments {
        assert!(
            response.answer.contains(fragment),
            "prompt {prompt:?} answer should contain {fragment:?}, got {}",
            response.answer,
        );
    }
    response
}

fn assert_calculation_error(prompt: &str, expected_fragments: &[&str]) -> SymbolicAnswer {
    let response = answer(prompt);
    assert_eq!(
        response.intent, "calculation_error",
        "prompt {prompt:?} should report calculator parse failure, got intent={} answer={}",
        response.intent, response.answer,
    );
    for fragment in expected_fragments {
        assert!(
            response.answer.contains(fragment),
            "prompt {prompt:?} answer should contain {fragment:?}, got {}",
            response.answer,
        );
    }
    response
}

#[test]
fn calculator_handles_english_variations() {
    for (prompt, expected) in [
        ("What is 8% of $50?", &["4", "USD"][..]),
        ("Please calculate sqrt(16)", &["4"][..]),
        ("Compute 300000 ms in seconds", &["300"][..]),
        ("Evaluate 741 KB as MB", &["0.741", "MB"][..]),
        ("How much is 10 tons to kg?", &["10000", "kg"][..]),
        ("Solve 2^3", &["8"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

// Issue #386 (38-C4): the currency-conversion exemption in `has_calculation_signal`
// is driven by the `quantity_conversion` meaning (role `quantity_conversion_cue`)
// rather than a hardcoded to/into/convert/exchange list. A prompt that pairs a
// currency symbol with letters is otherwise treated as prose and rejected; a
// quantity_conversion_cue exempts it, because a conversion is itself a calculation.
#[test]
fn calculator_currency_conversion_is_exempt_from_prose_rejection() {
    // "to" is a quantity_conversion_cue (matched whole-token), so the currency
    // conversion is recognised and delegated to the calculator.
    assert_calculation("$100 to euros", &["EUR"]);

    // With no conversion cue the same currency-plus-letters prompt is prose, so
    // the guard rejects it and it is not delegated to the calculator.
    let bare = answer("$100 euros");
    assert_ne!(
        bare.intent, "calculation",
        "a currency-plus-letters prompt with no quantity_conversion_cue should not be \
         treated as a calculation, got answer={}",
        bare.answer,
    );
}

#[test]
fn calculator_explains_fuzzy_calculate_typo() {
    assert_calculation(
        "Calcualte 2+5050",
        &[
            "Interpreted \"Calcualte\" as \"calculate\".",
            "2+5050 = 5052",
        ],
    );
}

#[test]
fn calculator_fuzzy_prefix_is_not_limited_to_one_spelling() {
    assert_calculation(
        "Calcuate 2+5050",
        &[
            "Interpreted \"Calcuate\" as \"calculate\".",
            "2+5050 = 5052",
        ],
    );
}

#[test]
fn calculator_error_keeps_fuzzy_interpretation() {
    assert_calculation_error(
        "Calcualte 2+",
        &[
            "Interpreted \"Calcualte\" as \"calculate\".",
            "could not evaluate",
        ],
    );
}

#[test]
fn calculator_handles_compact_question_equals_suffix() {
    for prompt in ["2*2+2=?", "2*2+2 = ?"] {
        assert_calculation(prompt, &["2*2+2 = 6"]);
    }
}

#[test]
fn calculator_handles_russian_variations() {
    for (prompt, expected) in [
        ("Сколько будет 2 + 2?", &["4"][..]),
        ("Сколько будет два плюс два?", &["4"][..]),
        ("Посчитай 1000 рублей в долларах", &["USD"][..]),
        ("Вычисли 1 тонна в кг", &["1000", "kg"][..]),
        ("Рассчитай 17 февраля 2027 - 6 месяцев", &["2026-08"][..]),
        ("Сколько будет 300000 ms in seconds?", &["300"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_handles_embedded_request_variations() {
    for (prompt, expected) in [
        ("хочу понять сколько будет 2+2", &["2+2 = 4"][..]),
        ("подскажи, сколько будет 7 - 4", &["7 - 4 = 3"][..]),
        (
            "мне нужно узнать сколько будет два плюс два",
            &["два плюс два = 4"][..],
        ),
        ("I want to know what is 2+2", &["2+2 = 4"][..]),
        ("Tell me what is 3 * 3", &["3 * 3 = 9"][..]),
        (
            "Before we continue, please calculate sqrt(16)",
            &["sqrt(16) = 4"][..],
        ),
        ("First say hi, then compute 10 / 2", &["10 / 2 = 5"][..]),
        ("我想知道计算 2 + 2", &["2 + 2 = 4"][..]),
        ("请帮我算一下 6 * 7", &["6 * 7 = 42"][..]),
        ("मुझे बताओ गणना करें 8 / 2", &["8 / 2 = 4"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_explains_usd_rate_basis_prompts() {
    for (language, prompt) in [
        (
            "en",
            "what dollar exchange rate do you use for calculations?",
        ),
        ("ru", "какой курс долора у тебя при расчетах?"),
        ("ru", "какой курс доллара у тебя при расчётах?"),
        ("hi", "गणना में आप डॉलर का कौन सा विनिमय दर उपयोग करते हैं?"),
        ("zh", "你计算时使用什么美元汇率?"),
    ] {
        let response = assert_calculation(prompt, &["link-calculator", "1 USD in RUB = 89.5 RUB"]);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "calculation:engine:link-calculator"),
            "{language} rate-basis answers should be delegated to link-calculator: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn calculator_handles_chinese_variations() {
    for (prompt, expected) in [
        ("计算 2 + 2", &["4"][..]),
        ("1000 克 换成 公斤 是多少?", &["1", "kg"][..]),
        ("请计算 1000 美元 换成 欧元", &["EUR"][..]),
        ("17 二月 2027 - 6 个月 等于多少", &["2026-08"][..]),
        ("1 一月 2027 + 7 天 等于多少", &["2027-01-08"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_handles_hindi_variations() {
    for (prompt, expected) in [
        ("2 + 2 कितना है?", &["4"][..]),
        ("गणना करें 1000 डॉलर में यूरो", &["EUR"][..]),
        ("1000 ग्राम में किलोग्राम कितना है?", &["1", "kg"][..]),
        ("17 फरवरी 2027 - 6 महीने की गणना करें", &["2026-08"][..]),
        ("1 जनवरी 2027 + 7 दिन कितना है?", &["2027-01-08"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_delegation_is_visible_in_evidence() {
    let response = assert_calculation("What is 8% of $50?", &["4", "USD"]);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "calculation:engine:link-calculator"),
        "calculator-backed answers should record the delegated engine: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("calculation:lino:")),
        "calculator-backed answers should preserve the upstream LINO: {:?}",
        response.evidence_links,
    );
}

#[test]
fn local_arithmetic_fallback_keeps_word_operators() {
    let word_response = assert_calculation("What is 10 plus 20 times 3?", &["70"]);
    assert!(
        word_response
            .evidence_links
            .iter()
            .any(|link| link == "calculation:engine:formal-ai-fallback"),
        "word-operator fallback should be observable: {:?}",
        word_response.evidence_links,
    );
}

#[test]
fn calculator_handles_binary_modulo_after_upstream_fix() {
    let modulo_response = assert_calculation("Compute 100 - 25 % 7", &["96"]);
    assert!(
        modulo_response
            .evidence_links
            .iter()
            .any(|link| link == "calculation:engine:link-calculator"),
        "binary modulo should be delegated after link-calculator v0.17.0: {:?}",
        modulo_response.evidence_links,
    );
}

#[test]
fn simple_variable_equations_are_solved_after_calculator_delegation() {
    for (prompt, expected) in [
        ("x*2 = 123", &["x = 61.5"][..]),
        ("Solve x * 2 = 123", &["x = 61.5"][..]),
        ("2 * x + 3 = 11", &["x = 4"][..]),
        ("10 = y / 3 + 1", &["y = 27"][..]),
    ] {
        let response = assert_calculation(prompt, expected);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "calculation:engine:link-calculator"),
            "simple equations should be delegated after link-calculator v0.17.1: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn placeholder_unknown_equations_are_solved_instead_of_reported_unknown() {
    for (prompt, expected) in [
        ("?+2=4", &["?+2=4 => ? = 2"][..]),
        ("*+2=4", &["*+2=4 => * = 2"][..]),
        ("Solve ? + 2 = 4", &["? = 2"][..]),
        ("Solve * + 2 = 4", &["* = 2"][..]),
        ("Solve 2 * ? + 3 = 11", &["? = 4"][..]),
        ("Solve 2 * * + 3 = 11", &["* = 4"][..]),
    ] {
        let response = assert_calculation(prompt, expected);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "calculation:engine:link-calculator"),
            "placeholder equations should be delegated after link-calculator v0.18.2: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn calculator_handles_upstream_equation_categories_after_placeholder_fix() {
    for (prompt, expected) in [
        ("Solve 2 * x + 3 * y = 12", &["x = 6 - 1.5*y"][..]),
        ("Solve x + ? = 4", &["? = 4 - x"][..]),
        ("Solve x + * = 4", &["* = 4 - x"][..]),
        ("Solve ? + * = 4", &["? = 4 - *"][..]),
        ("Solve x^2 = 4", &["x = -2 or x = 2"][..]),
        ("Solve x^2 - 5 * x + 6 = 0", &["x = 2 or x = 3"][..]),
        ("Solve x^3 - x = 0", &["x = -1 or x = 0 or x = 1"][..]),
        ("Solve ? * ? = 4", &["? = -2 or ? = 2"][..]),
        ("Solve * * * = 4", &["* = -2 or * = 2"][..]),
    ] {
        let response = assert_calculation(prompt, expected);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "calculation:engine:link-calculator"),
            "upstream equation category should be delegated to link-calculator: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("calculation:lino:")),
            "upstream equation category should preserve LINO evidence: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn calculator_extraction_does_not_steal_named_entities_with_digits() {
    for prompt in [
        "What is Python 3?",
        "What is GPT-5?",
        "What is Python 3 in programming?",
    ] {
        let response = answer(prompt);
        assert_ne!(
            response.intent, "calculation",
            "named entity prompt {prompt:?} should not be treated as a calculation: {}",
            response.answer,
        );
        assert_ne!(
            response.intent, "calculation_error",
            "named entity prompt {prompt:?} should fall through instead of becoming a calculator error: {}",
            response.answer,
        );
    }
}

#[test]
fn fibonacci_word_problem_reduces_to_calculator_expression() {
    // Issue #334 step 2: the website demo asked to "calculate the 10th
    // Fibonacci number and multiply it by 8% of 500. Show me the code and the
    // final result." This reduces to `55 * 8% of 500` (F(10) = 55, 8% of 500 =
    // 40, 55 * 40 = 2200) once the Fibonacci reference is resolved, the
    // spelled-out operator is rewritten, and the trailing instruction sentence
    // is dropped.
    assert_calculation(
        "calculate the 10th Fibonacci number and multiply it by 8% of 500. Show me the code and the final result.",
        &["2200"],
    );
    // The cardinal-word spelling and the bare "multiplied by" connector resolve
    // the same way (F(5) = 5, 5 * 10 = 50).
    assert_calculation("the fifth Fibonacci number multiplied by 10", &["50"]);
}

#[test]
fn box_relation_word_problem_resolves_total_with_reasoning() {
    // Issue #338: the prompt is an arithmetic word problem expressed as object
    // relations, not as a calculator expression. It should reduce the given
    // facts (B = 10, A = 2 * B, C = A + 5) before summing the boxes.
    assert_calculation(
        "I have 3 boxes. Box A has twice as many apples as Box B. Box C has 5 more apples than Box A. If Box B has 10 apples, how many apples are there in total? Show your reasoning step by step.",
        &[
            "Step 1: Box B = 10 apples.",
            "Step 2: Box A = 2 * 10 = 20 apples.",
            "Step 3: Box C = 20 + 5 = 25 apples.",
            "20 + 10 + 25 = 55",
            "there are 55 apples in total",
        ],
    );
}

#[test]
fn compound_interest_prompt_returns_formula_steps_and_eur_conversion() {
    let response = assert_calculation(
        "If I invest $1000 at 8% annual interest compounded monthly for 5 years, how much will I have? Show the formula, calculate step by step, and then convert the final amount to EUR using current exchange rates from the web.",
        &[
            "A = P(1 + r/n)^(n*t)",
            "P = 1000 USD",
            "r = 0.08",
            "n = 12",
            "t = 5",
            "Final amount: 1489.85 USD",
            "EUR",
        ],
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("calculation:compound_interest")),
        "compound-interest answers should expose calculation evidence: {:?}",
        response.evidence_links,
    );
}

#[test]
fn final_amount_conversion_uses_prior_compound_interest_answer() {
    let solver = UniversalSolver::default();
    let first_prompt =
        "If I invest $1000 at 8% annual interest compounded monthly for 5 years, how much will I have? Show the formula, calculate step by step, and";
    let first = solver.solve(first_prompt);
    assert_eq!(first.intent, "calculation");
    assert!(first.answer.contains("Final amount: 1489.85 USD"));

    let response = solver.solve_with_history(
        "convert the final amount to EUR using current exchange rates from the web.",
        &[
            ConversationTurn::user(first_prompt),
            ConversationTurn::assistant(first.answer),
        ],
    );
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("1489.85 USD"));
    assert!(response.answer.contains("EUR"));
    let lower_answer = response.answer.to_lowercase();
    assert!(
        !lower_answer.contains("i didn't understand")
            && !lower_answer.contains("i'm not sure how to respond"),
        "history-backed final amount conversion should not fall through: {}",
        response.answer,
    );
}

#[test]
fn program_request_is_not_misread_as_unit_incompatibility() {
    // Issue #334: "Write a program that computes the 10th Fibonacci number"
    // used to answer with a length/mass unit-incompatibility refusal because a
    // plain substring match found "mb" inside "nu*mb*er" and "gram" inside
    // "pro*gram*". A standalone-word match keeps coding prompts away from the
    // unit handler.
    for prompt in [
        "Write a program that computes the 10th Fibonacci number",
        "Show me a program in Python",
        "How many grams are in a number?",
    ] {
        let response = answer(prompt);
        assert_ne!(
            response.intent, "unit_incompatibility",
            "prompt {prompt:?} must not be misread as a unit conversion: {}",
            response.answer,
        );
    }
}

#[test]
fn bare_dot_calculation_candidates_do_not_crash_the_process() {
    // Issue #334: these prompts reduce to calculator candidates that contain a
    // "bare" period (`2. 3`). `link-calculator` (<= 0.17.2) attempts a multi-
    // gigabyte allocation and aborts the whole process on such input, so the
    // dialog "Write a Python function ... Then calculate the 10th Fibonacci
    // number and multiply it by 8% of 500 ..." used to crash with SIGKILL.
    // After the guard the engine returns a normal answer for every prompt — the
    // test reaching its assertions at all proves the process did not abort.
    for prompt in [
        "What is 2+2. What is 3+3.",
        "calculate 8% of 500. Show me the code and the final result.",
        "What is 2. 3",
        "Then calculate the 10th Fibonacci number and multiply it by 8% of 500. Show me the code and the final result.",
    ] {
        let response = answer(prompt);
        assert!(
            !response.answer.trim().is_empty(),
            "prompt {prompt:?} should return a graceful answer instead of crashing"
        );
        assert!(
            !response.answer.contains("4831838208")
                && !response.answer.to_lowercase().contains("memory allocation"),
            "prompt {prompt:?} must not surface an allocation failure: {}",
            response.answer,
        );
    }
}
