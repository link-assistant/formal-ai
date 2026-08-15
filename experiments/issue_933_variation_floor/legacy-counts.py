#!/usr/bin/env python3
"""Count distinct wordings per case per language in the pre-issue-#933 corpus.

`tests/unit/multilingual_variations.rs` is the corpus issue #123 was talking
about: hand-written prompt arrays, one or more per conversational case. This
script applies the same normalization the new CI floor applies (case,
punctuation, symbols and whitespace folded away) so the "before" numbers are
measured with the same ruler as the "after" numbers, and prints every group
that sits below five.

Usage: python3 experiments/issue_933_variation_floor/legacy-counts.py
"""

import pathlib
import re
import sys
import unicodedata

SOURCE = pathlib.Path("tests/unit/multilingual_variations.rs")
FLOOR = 5

# The case each test function contributes prompts to, keyed by the intent the
# function asserts rather than by its name, so several per-language functions
# collapse into one case the way the new corpus groups them.
CASE_OF_FUNCTION = {
    "greeting_english_variations_match": "greeting",
    "greeting_russian_variations_match": "greeting",
    "greeting_hindi_variations_match": "greeting",
    "greeting_chinese_variations_match": "greeting",
    "wellbeing_how_are_you_variations_match_across_languages": "wellbeing",
    "assistant_free_time_variations_match_across_languages": "assistant_free_time",
    "farewell_english_variations_match": "farewell",
    "farewell_russian_variations_match": "farewell",
    "farewell_hindi_variations_match": "farewell",
    "farewell_chinese_variations_match": "farewell",
    "identity_english_variations_match": "identity",
    "identity_russian_variations_match": "identity",
    "identity_hindi_variations_match": "identity",
    "identity_chinese_variations_match": "identity",
    "arithmetic_english_word_variations_match": "calculation",
    "arithmetic_russian_word_variations_match": "calculation",
    "arithmetic_hindi_word_variations_match": "calculation",
    "arithmetic_chinese_word_variations_match": "calculation",
    "arithmetic_symbolic_variations_match": "calculation",
}

LANGUAGES = ("en", "ru", "hi", "zh")


def normalize(prompt):
    """Fold case, punctuation, symbols and spacing -- the CI floor's rule."""
    folded = unicodedata.normalize("NFKC", prompt).lower()
    return "".join(
        character
        for character in folded
        if not unicodedata.category(character).startswith(("P", "S", "Z"))
        and not character.isspace()
    )


def language_of(prompt):
    """The writing system a prompt is in, which is what its language is here."""
    for character in prompt:
        code = ord(character)
        if 0x0400 <= code <= 0x04FF:
            return "ru"
        if 0x0900 <= code <= 0x097F:
            return "hi"
        if 0x4E00 <= code <= 0x9FFF:
            return "zh"
    return "en"


def prompts_by_function(text):
    """Every string literal inside each `fn ...() { ... }` block."""
    groups = {}
    current = None
    for line in text.splitlines():
        match = re.match(r"fn (\w+)\(\)", line)
        if match:
            current = match.group(1)
            groups.setdefault(current, [])
            continue
        if current is None:
            continue
        for literal in re.findall(r'"((?:[^"\\]|\\.)*)"', line):
            groups[current].append(literal)
    return groups


def main():
    text = SOURCE.read_text(encoding="utf-8")
    counts = {(case, language): set() for case in set(CASE_OF_FUNCTION.values()) for language in LANGUAGES}

    for function, prompts in prompts_by_function(text).items():
        case = CASE_OF_FUNCTION.get(function)
        if case is None:
            continue
        for prompt in prompts:
            # Arithmetic functions list (prompt, expected answer) pairs; the
            # expected answers are digits, which normalize to themselves.
            if case == "calculation" and prompt.strip().isdigit():
                continue
            normalized = normalize(prompt)
            if not normalized:
                continue
            language = language_of(prompt)
            if language in LANGUAGES:
                counts[(case, language)].add(normalized)

    cases = sorted({case for case in CASE_OF_FUNCTION.values()})
    width = max(len(case) for case in cases)
    print(f"{SOURCE}: distinct wordings per case per language (floor: {FLOOR})")
    print(f"  {'case'.ljust(width)}  " + " ".join(language.rjust(3) for language in LANGUAGES))
    for case in cases:
        cells = " ".join(str(len(counts[(case, language)])).rjust(3) for language in LANGUAGES)
        print(f"  {case.ljust(width)}  {cells}")

    below = [
        (case, language, len(counts[(case, language)]))
        for case in cases
        for language in LANGUAGES
        if len(counts[(case, language)]) < FLOOR
    ]
    print()
    for case, language, count in below:
        print(f"- case {case} has {count} {language} variation(s); the floor is {FLOOR}")
    print(f"\n{len(below)} of {len(cases) * len(LANGUAGES)} groups are below the floor.")
    return 1 if below else 0


if __name__ == "__main__":
    sys.exit(main())
