#!/usr/bin/env python3
"""Issue #891 — regenerate `data/benchmarks/equation-type-corpus.lino`.

The corpus records, for every equation type, the *observed* answer of the
production solver. Rather than transcribing those answers by hand, this script
reads the probe output

    cargo run --example issue_891_equation_probe -- \
        experiments/issue-891-equation-prompts.txt > /tmp/probe.tsv

(one `prompt<TAB>intent<TAB>engine<TAB>answer` row per prompt) and joins it with
the category / equation-type labels below, so the fixture can never drift from
what the engine actually produced. Prompts labelled here but missing from the
probe output — or answered differently than the label expects — abort the run.

    python3 experiments/issue-891-build-corpus.py /tmp/probe.tsv \
        > data/benchmarks/equation-type-corpus.lino

The ratchet test `tests/unit/specification/equation_corpus.rs` then re-runs every
case through the engine, so a regression fails CI even if this script is never
run again.
"""

import sys

# (equation_type, category, language, prompt, verification note)
CASES = [
    # --- linear equations, one operation -------------------------------------
    ("linear_addition_right", "linear_one_operation", "en", "x + 2 = 5", "addition isolated on the left"),
    ("linear_subtraction_right", "linear_one_operation", "en", "x - 3 = 7", "subtraction isolated on the left"),
    ("linear_multiplication_coefficient", "linear_one_operation", "en", "2 * x = 10", "integer coefficient divided out"),
    ("linear_division_by_constant", "linear_one_operation", "en", "x / 4 = 3", "division inverted to multiplication"),
    ("linear_unknown_times_constant", "linear_one_operation", "en", "x * 2 = 123", "non-integer quotient kept exact"),
    ("linear_unknown_on_right", "linear_one_operation", "en", "Solve 5 = x + 1", "unknown on the right-hand side"),
    ("linear_constant_minus_unknown", "linear_one_operation", "en", "Solve 100 - x = 42", "unknown subtracted from a constant"),
    ("linear_negative_coefficient", "linear_one_operation", "en", "Solve -2 * x = 8", "negative coefficient yields a negative root"),
    ("linear_fractional_root", "linear_one_operation", "en", "Solve 4 * x = 6", "root is a fraction"),
    ("linear_decimal_coefficient", "linear_one_operation", "en", "Solve 0.5 * x = 2.5", "decimal coefficient"),
    # --- linear equations, two or more operations ----------------------------
    ("linear_two_step", "linear_multi_operation", "en", "2 * x + 3 = 11", "coefficient plus constant term"),
    ("linear_two_step_named_y", "linear_multi_operation", "en", "10 = y / 3 + 1", "unknown named y, equation reversed"),
    ("linear_unknown_both_sides", "linear_multi_operation", "en", "Solve 3 * x + 2 = x + 10", "unknown on both sides"),
    ("linear_parenthesized_left", "linear_multi_operation", "en", "Solve 2 * (x + 3) = 10", "parenthesized left-hand side"),
    ("linear_parentheses_both_sides", "linear_multi_operation", "en", "Solve 3 * (x - 1) = 2 * (x + 4)", "parentheses on both sides"),
    ("linear_like_terms", "linear_multi_operation", "en", "Solve 2 * x + 3 * x = 25", "like terms collected"),
    ("linear_nested_subtraction", "linear_multi_operation", "en", "Solve x - (2 - x) = 6", "nested subtraction of the unknown"),
    ("linear_fraction_sum", "linear_multi_operation", "en", "Solve x / 2 + x / 3 = 5", "sum of fractional terms"),
    ("linear_rational_constants", "linear_multi_operation", "en", "Solve x + 1 / 2 = 3 / 2", "rational constants on both sides"),
    ("linear_four_terms", "linear_multi_operation", "en", "Solve 7 * x - 4 = 3 * x + 12", "four terms, unknown on both sides"),
    ("linear_zero_right_hand_side", "linear_multi_operation", "en", "Solve x / 5 - 2 = 0", "homogeneous right-hand side"),
    ("linear_reversed_two_step", "linear_multi_operation", "en", "Solve 12 = 4 * x - 8", "two-step equation stated in reverse"),
    # --- placeholder unknowns ------------------------------------------------
    ("placeholder_question_mark_addition", "placeholder_unknown", "en", "?+2=4", "`?` placeholder, no spaces"),
    ("placeholder_asterisk_addition", "placeholder_unknown", "en", "*+2=4", "`*` placeholder, no spaces"),
    ("placeholder_question_mark_two_step", "placeholder_unknown", "en", "Solve 2 * ? + 3 = 11", "`?` placeholder in a two-step equation"),
    ("placeholder_question_mark_division", "placeholder_unknown", "en", "Solve ? / 4 = 3", "`?` placeholder under a division"),
    ("placeholder_question_mark_subtraction", "placeholder_unknown", "en", "Solve ? - 5 = 5", "`?` placeholder in a subtraction"),
    ("placeholder_question_mark_reversed", "placeholder_unknown", "en", "Solve 10 = ? * 5", "`?` placeholder on the right-hand side"),
    ("placeholder_asterisk_two_step", "placeholder_unknown", "en", "Solve 2 * * + 3 = 11", "`*` placeholder beside a multiplication sign"),
    ("placeholder_asterisk_division", "placeholder_unknown", "en", "Solve * / 4 = 3", "`*` placeholder under a division"),
    # --- symbolic / multi-variable answers -----------------------------------
    ("symbolic_two_variables", "symbolic_multi_variable", "en", "Solve 2 * x + 3 * y = 12", "answer stays symbolic in y"),
    ("symbolic_variable_and_question_mark", "symbolic_multi_variable", "en", "Solve x + ? = 4", "placeholder solved in terms of a variable"),
    ("symbolic_variable_and_asterisk", "symbolic_multi_variable", "en", "Solve x + * = 4", "`*` placeholder solved in terms of x"),
    ("symbolic_two_placeholders", "symbolic_multi_variable", "en", "Solve ? + * = 4", "two placeholders, one solved for the other"),
    ("symbolic_three_variables", "symbolic_multi_variable", "en", "Solve x + y + z = 6", "three variables, isolation of the first"),
    ("symbolic_rational_coefficient", "symbolic_multi_variable", "en", "Solve 3 * x - y = 0", "symbolic answer with a rational coefficient"),
    ("symbolic_named_a_b", "symbolic_multi_variable", "en", "Solve a * 2 + b = 10", "unknowns named a and b"),
    # --- polynomial equations ------------------------------------------------
    ("quadratic_pure_square", "polynomial", "en", "Solve x^2 = 4", "pure square, two roots"),
    ("quadratic_two_distinct_roots", "polynomial", "en", "Solve x^2 - 5 * x + 6 = 0", "factorable quadratic, two roots"),
    ("quadratic_double_root", "polynomial", "en", "Solve x^2 - 4 * x + 4 = 0", "double root reported once"),
    ("quadratic_zero_constant_term", "polynomial", "en", "Solve x^2 + 5 * x = 0", "root at zero plus one more"),
    ("quadratic_scaled", "polynomial", "en", "Solve 2 * x^2 - 8 = 0", "leading coefficient other than one"),
    ("quadratic_written_as_product", "polynomial", "en", "Solve x * x = 9", "square written as a product"),
    ("quadratic_single_zero_root", "polynomial", "en", "Solve x^2 = 0", "single root at zero"),
    ("cubic_three_roots_via_factor", "polynomial", "en", "Solve x^3 - x = 0", "cubic with three rational roots"),
    ("cubic_pure_power", "polynomial", "en", "Solve x^3 = 27", "pure cube, one real root"),
    ("quartic_two_real_roots", "polynomial", "en", "Solve x^4 - 1 = 0", "quartic, two real rational roots"),
    ("quintic_repeated_roots", "polynomial", "en", "Solve x^5 - x^3 = 0", "quintic with repeated roots"),
    ("polynomial_placeholder_square", "polynomial", "en", "Solve ? * ? = 4", "`?` placeholder squared"),
    ("polynomial_asterisk_square", "polynomial", "en", "Solve * * * = 4", "`*` placeholder squared"),
    ("cubic_three_distinct_roots", "polynomial", "en", "Solve x^3 - 6 * x^2 + 11 * x - 6 = 0", "cubic with three distinct rational roots"),
    # --- natural-language wrappers, four supported languages -----------------
    ("wrapper_en_solve_the_equation", "natural_language_wrapper", "en", "Solve the equation 2 * x + 3 = 11", "English `solve the equation` cue"),
    ("wrapper_en_solve_equation", "natural_language_wrapper", "en", "Solve equation x - 4 = 6", "English `solve equation` cue"),
    ("wrapper_ru_solve_the_equation", "natural_language_wrapper", "ru", "Реши уравнение 2 * x + 3 = 11", "Russian `реши уравнение` cue"),
    ("wrapper_ru_solve_the_equation_polite", "natural_language_wrapper", "ru", "Решите уравнение x / 3 = 9", "Russian polite `решите уравнение` cue"),
    ("wrapper_ru_solve_bare", "natural_language_wrapper", "ru", "Реши x / 3 = 9", "Russian bare `реши` cue"),
    ("wrapper_ru_how_much_placeholder", "natural_language_wrapper", "ru", "Сколько будет ? + 2 = 4", "Russian `сколько будет` cue with a placeholder"),
    ("wrapper_zh_solve_equation", "natural_language_wrapper", "zh", "解方程 2 * x + 3 = 11", "Chinese `解方程` cue"),
    ("wrapper_zh_solve_for", "natural_language_wrapper", "zh", "求解 x^2 - 5 * x + 6 = 0", "Chinese `求解` cue on a quadratic"),
    ("wrapper_zh_calculate", "natural_language_wrapper", "zh", "计算 x * 4 = 20", "Chinese `计算` cue"),
    ("wrapper_hi_solve_equation_polite", "natural_language_wrapper", "hi", "समीकरण हल करें 2 * x + 3 = 11", "Hindi `समीकरण हल करें` cue"),
    ("wrapper_hi_solve_equation_familiar", "natural_language_wrapper", "hi", "समीकरण हल करो x / 3 = 9", "Hindi familiar `समीकरण हल करो` cue"),
    ("wrapper_hi_solve_bare", "natural_language_wrapper", "hi", "हल करें x + 9 = 15", "Hindi bare `हल करें` cue"),
    ("wrapper_hi_trailing_question", "natural_language_wrapper", "hi", "x + 9 = 15 कितना है?", "Hindi trailing `कितना है` question"),
    ("wrapper_es_solve_the_equation", "natural_language_wrapper", "es", "Resuelve la ecuación x + 2 = 5", "Spanish `resuelve la ecuación` cue"),
    ("wrapper_es_solve_the_equation_infinitive", "natural_language_wrapper", "es", "Resolver la ecuación 3 * x = 12", "Spanish infinitive `resolver la ecuación` cue"),
    ("wrapper_es_solve_bare", "natural_language_wrapper", "es", "Resuelve x^2 - 5 * x + 6 = 0", "Spanish bare `resuelve` cue on a quadratic"),
    ("wrapper_es_calculate", "natural_language_wrapper", "es", "Calcula 2 + 2", "Spanish `calcula` cue"),
    ("wrapper_es_how_much_is", "natural_language_wrapper", "es", "Cuánto es 7 * 6", "Spanish `cuánto es` question opener"),
    # --- evaluation and percent flavours -------------------------------------
    ("evaluation_placeholder_result", "evaluation_and_percent", "en", "2*2+2=?", "placeholder stands for the result, not the unknown"),
    ("evaluation_trailing_question_mark", "evaluation_and_percent", "en", "x*2 = 123 ?", "trailing question mark after an equation"),
    ("percent_of_unknown", "evaluation_and_percent", "en", "Solve 8% of x = 4", "percent-of phrasing with an unknown"),
]

# Equation shapes the production stack does *not* answer today. The ratchet
# asserts each still fails loudly (never a fabricated answer); when upstream
# gains support the assertion fires and the record is promoted into CASES.
LIMITATIONS = [
    (
        "irrational_roots",
        "polynomial",
        "Solve x^2 - 2 = 0",
        "calculation_error",
        "link-calculator returns rational roots only, so an irrational root set is reported as unparseable instead of as sqrt(2).",
        "link-calculator",
    ),
    (
        "complex_roots",
        "polynomial",
        "Solve x^2 + 1 = 0",
        "calculation_error",
        "No complex-root support upstream; an equation with no real root is reported as unparseable.",
        "link-calculator",
    ),
    (
        "degenerate_no_solution",
        "degenerate",
        "Solve 0 * x = 5",
        "calculation_error",
        "A contradiction is reported as unparseable rather than as 'no solution'.",
        "link-calculator",
    ),
    (
        "identity_equation",
        "degenerate",
        "Solve x = x",
        "unknown",
        "An identity (every value satisfies it) carries no calculation signal, so the router declines instead of answering 'any value'.",
        "formal-ai",
    ),
    (
        "malformed_expression",
        "degenerate",
        "Solve x + = 4",
        "calculation_error",
        "A malformed equation is rejected — recorded so the corpus proves malformed input never yields a fabricated answer.",
        "link-calculator",
    ),
    (
        "unit_carrying_unknown",
        "units",
        "Solve x kg = 1000 g",
        "calculation_error",
        "Unit-carrying equations are not converted before solving; the units make the expression unparseable.",
        "link-calculator",
    ),
    (
        "unit_carrying_constant",
        "units",
        "Solve 2 * x = 10 kg",
        "calculation_error",
        "A unit on the constant side is not stripped, so the equation is unparseable.",
        "link-calculator",
    ),
    (
        "named_unknown_if_clause",
        "natural_language_wrapper",
        "What is x if x + 7 = 12?",
        "calculation_error",
        "`x if <equation>` declares the unknown before the equation; the declaration is not stripped, so the remainder `x if x + 7 = 12` is unparseable.",
        "formal-ai",
    ),
    (
        "named_unknown_for_clause",
        "natural_language_wrapper",
        "Calculate x for 6 * x = 42",
        "calculation_error",
        "`x for <equation>` declares the unknown before the equation; the declaration is not stripped.",
        "formal-ai",
    ),
    (
        "named_unknown_colon_clause",
        "natural_language_wrapper",
        "Find x: 5 * x = 45",
        "agent_suggestion",
        "`Find` is a shell command name, so the agent router claims the prompt before the calculator sees it.",
        "formal-ai",
    ),
]

SUITE_HEADER = """benchmark_suite_issue_891_equation_type_corpus
  record_type "benchmark_suite"
  id "issue_891_equation_type_corpus"
  title "Equation-type corpus with a verified-type ratchet"
  purpose "Machine-readable corpus for issue #891 (parent #710, requirement from issue #406): at least fifty distinct equation types, each run through the production solver and each carrying the exact answer the engine produced. Categories span one-step and multi-step linear equations, `?`/`*` placeholder unknowns, symbolic multi-variable isolation, polynomial equations up to degree five, natural-language wrappers in all four supported languages, and evaluation/percent flavours. Every case is self-authored in this repository, so no upstream text is redistributed."
  runner "cargo test --test unit issue_891_equation_corpus -- --nocapture"
  report_mode "Pass/fail counts and the distinct verified-type count are reported; a failing case is a regression because every expected answer was observed from the production solver."
  minimum_pass_count "{minimum_pass_count}"
  minimum_verified_types "50"
  ratchet_policy "CI asserts passed >= minimum_pass_count and distinct verified equation types >= minimum_verified_types, so the corpus can only grow. Raise the floor when the pass count rises; never lower it."
  held_out_policy "Cases are generated by probing the production solver, not by memorizing phrasings: each equation type appears once, and the wrapper category repeats the same equations under different cues and languages so a pass cannot come from one memorized surface."
  limitation_policy "benchmark_limitation records name equation shapes the stack does not answer today. The ratchet asserts each still fails loudly (a non-calculation intent, never a fabricated answer); when upstream gains support the assertion fires so the record is promoted into a verified case."
  topic "calculation / equation_solving"
  imported_at "2026-08-04"
  updated_at "2026-08-04"
benchmark_source_equation_types
  record_type "benchmark_source"
  id "equation_types"
  domain "equation_solving"
  title "Self-authored equation-type corpus (issue #891)"
  license "CC-BY-4.0"
  license_url "https://creativecommons.org/licenses/by/4.0/"
  source_url "https://github.com/link-assistant/formal-ai"
  source_ref "issue-891"
  note "Equation shapes drawn from the school-algebra categories issue #406 asks for: linear, placeholder, symbolic and polynomial equations plus natural-language wrappers."
  usage_note "Authored inside this repository; expected answers are the observed output of the production solver, captured with `cargo run --example issue_891_equation_probe`."
"""


def emit_field(name, value):
    return f'  {name} "{value}"\n'


def main():
    if len(sys.argv) != 2:
        raise SystemExit(f"usage: {sys.argv[0]} <probe.tsv>")
    observed = {}
    with open(sys.argv[1], encoding="utf-8") as handle:
        for line in handle:
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 4:
                continue
            observed[parts[0]] = (parts[1], parts[2], parts[3])

    out = []
    verified = []
    for equation_type, category, language, prompt, verification in CASES:
        if prompt not in observed:
            raise SystemExit(f"prompt not present in probe output: {prompt}")
        intent, engine, answer = observed[prompt]
        if intent != "calculation":
            raise SystemExit(f"prompt did not solve ({intent}): {prompt}")
        verified.append((equation_type, category, language, prompt, verification, engine, answer))

    out.append(SUITE_HEADER.format(minimum_pass_count=len(verified)))
    for equation_type, category, language, prompt, verification, engine, answer in verified:
        out.append(f"benchmark_case_{equation_type}\n")
        out.append(emit_field("record_type", "benchmark_case"))
        out.append(emit_field("id", equation_type))
        out.append(emit_field("source", "equation_types"))
        out.append(emit_field("equation_type", equation_type))
        out.append(emit_field("category", category))
        out.append(emit_field("language", language))
        out.append(emit_field("prompt", prompt))
        out.append(emit_field("expected_intent", "calculation"))
        out.append(emit_field("expected_engine", engine))
        out.append(emit_field("expected_answer", answer))
        out.append(emit_field("verification", verification))

    for limitation_id, category, prompt, intent, limitation, upstream in LIMITATIONS:
        if prompt in observed and observed[prompt][0] != intent:
            raise SystemExit(
                f"limitation {limitation_id} no longer fails as recorded: {observed[prompt]}"
            )
        out.append(f"benchmark_limitation_{limitation_id}\n")
        out.append(emit_field("record_type", "benchmark_limitation"))
        out.append(emit_field("id", limitation_id))
        out.append(emit_field("source", "equation_types"))
        out.append(emit_field("category", category))
        out.append(emit_field("prompt", prompt))
        out.append(emit_field("observed_intent", intent))
        out.append(emit_field("upstream", upstream))
        out.append(emit_field("limitation", limitation))

    sys.stdout.write("".join(out))


if __name__ == "__main__":
    main()
