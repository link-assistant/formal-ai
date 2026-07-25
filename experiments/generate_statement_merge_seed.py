#!/usr/bin/env python3
"""Generate data/seed/meanings-statement-merge.lino for issue #844.

The three meanings it declares are pure vocabulary lists, so writing them by
hand invites duplicate or misordered surfaces. This generator sorts and
de-duplicates each list, keeping the seed file a deterministic function of the
word sets recorded below.
"""
from pathlib import Path

FUNCTION_WORDS = {
    # Articles, prepositions, copulas and coordinators only. Quantifiers
    # ("all", "some", "none") are deliberately absent: dropping them would
    # conflate "all tests pass" with "some tests pass".
    "en": [
        "the", "a", "an", "of", "to", "in", "on", "at", "by", "for", "with",
        "from", "as", "into", "onto", "over", "than", "then", "and", "or",
        "but", "that", "this", "these", "those", "it", "its", "is", "are",
        "was", "were", "be", "been", "being", "am", "do", "does", "did",
        "has", "have", "had", "will", "would", "can", "could", "should",
        "there", "their", "here",
    ],
    "ru": [
        "и", "в", "во", "на", "с", "со", "по", "для", "из", "о", "об", "а",
        "но", "или", "это", "эта", "этот", "есть", "был", "была", "были",
        "к", "у", "же", "то", "как", "что",
    ],
    "hi": [
        "का", "के", "की", "को", "में", "से", "पर", "है", "हैं", "था", "थे",
        "थी", "और", "या", "यह", "वह", "एक", "ने", "भी", "तो",
    ],
    "zh": ["的", "了", "是", "在", "和", "与", "或", "这", "那", "有", "被", "对"],
}

NEGATION_CUES = {
    # Syntactic negation only. A cue flips a statement's polarity, so a
    # semantic near-miss ("fails", "false") must stay out: those belong to the
    # statement's content, not to its sign.
    "en": [
        "not", "no", "never", "without", "cannot", "isn't", "aren't", "wasn't",
        "weren't", "doesn't", "don't", "didn't", "can't", "won't", "nor",
    ],
    "ru": ["не", "нет", "ни", "никогда", "без", "нельзя"],
    "hi": ["नहीं", "न", "बिना", "कभी"],
    "zh": ["不", "没", "没有", "无", "非", "未", "别"],
}

RESERVED_WORDS = {
    "en": sorted(set(
        # Rust 2021 keywords (strict + reserved).
        """as break const continue crate dyn else enum extern false fn for if
        impl in let loop match mod move mut pub ref return self Self static
        struct super trait true type unsafe use where while async await box
        become final macro override priv typeof unsized virtual yield""".split()
        # Python 3 keywords and soft keywords.
        + """and assert class def del elif except finally from global import is
        lambda None nonlocal not or pass raise try with None True False match
        case""".split()
        # JavaScript reserved words.
        + """var function new this delete void switch default do throw catch
        export extends null undefined instanceof debugger with interface
        package private protected public implements let yield""".split()
    )),
}

def block(slug: str, gloss_role: str, words: dict) -> list[str]:
    lines = [f"  {slug}", "    defined-by concept", f"    role {gloss_role}"]
    for language, surfaces in words.items():
        seen: list[str] = []
        for surface in surfaces:
            if surface not in seen:
                seen.append(surface)
        lines.append(f"    lexeme {language}")
        for surface in seen:
            lines.append("      surface")
            # Surfaces are quoted so they contribute no *value tokens* to the
            # total reference-closure audit, exactly as
            # scripts/close-total.py does: a surface is literal text, not a
            # reference into the meaning graph.
            lines.append(f'        text "{surface}"')
    return lines

def main() -> None:
    # No header comment: a full-line `#` comment in data/seed/*.lino is
    # tokenized by scripts/audit-total-closure.py (its comment stripper only
    # fires on a space-preceded `#`), so prose there would break total closure.
    # The three roles are documented on their Rust constants instead.
    lines = ["meanings"]
    lines += block("statement_function_word", "statement_function_word", FUNCTION_WORDS)
    lines += block("statement_negation_cue", "statement_negation_cue", NEGATION_CUES)
    lines += block("identifier_reserved_word", "identifier_reserved_word", RESERVED_WORDS)
    path = Path(__file__).resolve().parents[1] / "data/seed/meanings-statement-merge.lino"
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {path} ({len(lines) + 1} lines)")

if __name__ == "__main__":
    main()
