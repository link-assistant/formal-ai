//! Translation pipeline: formalize a source surface into a canonical meaning
//! token, then deformalize that token into a target-language surface.
//!
//! The pipeline is fully offline and deterministic. The same canonical token
//! collapses every recognized surface form across English, Russian, Hindi, and
//! Chinese, which means a single meaning id (produced from the token via
//! [`crate::engine::stable_id`]) is shared by every translation of the same
//! phrase, regardless of the source language. This is the requirement called
//! out by issue #207 — translation must not be limited to a single hardcoded
//! meaning id.
//!
//! The browser worker (`src/web/formal_ai_worker.js`) mirrors the registry
//! and the formatting helpers so the deployed GitHub Pages demo matches the
//! Rust core.

/// One symbolic meaning expressed across the supported languages.
///
/// `token` is the canonical name (e.g. `greeting_how_are_you`) that
/// [`crate::engine::stable_id`] hashes into the public `meaning_*` id.
///
/// `primary` lists the human-readable surface form for each language; the
/// first form is the canonical one used during deformalization. `aliases`
/// lists alternative forms in their normalized (lower-cased, punctuation- and
/// whitespace-stripped) shape, used during formalization to collapse extra
/// variants onto the same canonical token.
#[derive(Debug)]
struct MeaningEntry {
    token: &'static str,
    primary: &'static [(&'static str, &'static str)],
    aliases: &'static [(&'static str, &'static [&'static str])],
}

/// Offline registry covering greetings, polite follow-ups, gratitude,
/// farewells, identity probes, yes/no answers, time-of-day greetings, and
/// well-being checks across English, Russian, Hindi, and Chinese.
///
/// Entries are intentionally small and hand-curated. The online enrichment
/// path (Wikipedia / Wikidata / Wiktionary, documented in
/// `docs/case-studies/issue-207/raw-data/online-research.md`) layers on top
/// of this baseline without changing the public contract.
const MEANING_REGISTRY: &[MeaningEntry] = &[
    MeaningEntry {
        token: "greeting",
        primary: &[
            ("en", "Hello"),
            ("ru", "Привет"),
            ("hi", "नमस्ते"),
            ("zh", "你好"),
        ],
        aliases: &[
            ("en", &["hello", "hi", "hey"]),
            ("ru", &["привет", "здравствуйте", "здравствуй"]),
            ("hi", &["नमस्ते", "नमस्कार"]),
            ("zh", &["你好", "您好"]),
        ],
    },
    MeaningEntry {
        token: "greeting_how_are_you",
        primary: &[
            ("en", "How are you?"),
            ("ru", "Как у тебя дела?"),
            ("hi", "आप कैसे हैं?"),
            ("zh", "你好吗？"),
        ],
        aliases: &[
            ("en", &["howareyou", "hellohowareyou", "hihowareyou"]),
            (
                "ru",
                &[
                    "какдела",
                    "какутебядела",
                    "какувасдела",
                    "какваши дела",
                    "какватидела",
                    "какваши",
                    "приветкакдела",
                    "здравствуйтекаквашидела",
                ],
            ),
            ("hi", &["आपकैसेहैं", "तुमकैसेहो"]),
            ("zh", &["你好吗", "你怎么样"]),
        ],
    },
    MeaningEntry {
        token: "thank_you",
        primary: &[
            ("en", "Thank you"),
            ("ru", "Спасибо"),
            ("hi", "धन्यवाद"),
            ("zh", "谢谢"),
        ],
        aliases: &[
            ("en", &["thanks", "thankyou", "thankyouverymuch"]),
            ("ru", &["спасибо", "благодарю", "большоеспасибо"]),
            ("hi", &["धन्यवाद", "शुक्रिया"]),
            ("zh", &["谢谢", "多谢", "感谢"]),
        ],
    },
    MeaningEntry {
        token: "you_are_welcome",
        primary: &[
            ("en", "You are welcome"),
            ("ru", "Пожалуйста"),
            ("hi", "आपका स्वागत है"),
            ("zh", "不客气"),
        ],
        aliases: &[
            ("en", &["youarewelcome", "yourewelcome", "nottoworry"]),
            ("ru", &["пожалуйста", "незачто"]),
            ("hi", &["आपकास्वागतहै", "कोईबातनहीं"]),
            ("zh", &["不客气", "不用谢"]),
        ],
    },
    MeaningEntry {
        token: "goodbye",
        primary: &[
            ("en", "Goodbye"),
            ("ru", "До свидания"),
            ("hi", "अलविदा"),
            ("zh", "再见"),
        ],
        aliases: &[
            ("en", &["goodbye", "bye", "seeyou", "byebye"]),
            ("ru", &["досвидания", "пока", "прощай"]),
            ("hi", &["अलविदा", "फिरमिलेंगे"]),
            ("zh", &["再见", "拜拜"]),
        ],
    },
    MeaningEntry {
        token: "good_morning",
        primary: &[
            ("en", "Good morning"),
            ("ru", "Доброе утро"),
            ("hi", "सुप्रभात"),
            ("zh", "早上好"),
        ],
        aliases: &[
            ("en", &["goodmorning"]),
            ("ru", &["доброеутро"]),
            ("hi", &["सुप्रभात", "शुभप्रभात"]),
            ("zh", &["早上好", "早安"]),
        ],
    },
    MeaningEntry {
        token: "good_evening",
        primary: &[
            ("en", "Good evening"),
            ("ru", "Добрый вечер"),
            ("hi", "शुभ संध्या"),
            ("zh", "晚上好"),
        ],
        aliases: &[
            ("en", &["goodevening"]),
            ("ru", &["добрыйвечер"]),
            ("hi", &["शुभसंध्या"]),
            ("zh", &["晚上好", "晚安"]),
        ],
    },
    MeaningEntry {
        token: "what_is_your_name",
        primary: &[
            ("en", "What is your name?"),
            ("ru", "Как тебя зовут?"),
            ("hi", "तुम्हारा नाम क्या है?"),
            ("zh", "你叫什么名字？"),
        ],
        aliases: &[
            ("en", &["whatisyourname", "whatsyourname"]),
            ("ru", &["кактебязовут", "каквасзовут"]),
            ("hi", &["तुम्हारानामक्याहै", "आपकानामक्याहै"]),
            ("zh", &["你叫什么名字", "您叫什么名字"]),
        ],
    },
    MeaningEntry {
        token: "i_am_fine",
        primary: &[
            ("en", "I am fine"),
            ("ru", "У меня всё хорошо"),
            ("hi", "मैं ठीक हूँ"),
            ("zh", "我很好"),
        ],
        aliases: &[
            ("en", &["iamfine", "imfine", "imdoingfine", "imdoingwell"]),
            ("ru", &["уменявсёхорошо", "уменявсехорошо", "всёхорошо"]),
            ("hi", &["मैंठीकहूँ", "मैंठीकहूं"]),
            ("zh", &["我很好", "我挺好的"]),
        ],
    },
    MeaningEntry {
        token: "yes",
        primary: &[("en", "Yes"), ("ru", "Да"), ("hi", "हाँ"), ("zh", "是")],
        aliases: &[
            ("en", &["yes", "yeah", "yep", "aye"]),
            ("ru", &["да", "ага", "конечно"]),
            ("hi", &["हाँ", "हां", "जी"]),
            ("zh", &["是", "是的", "对"]),
        ],
    },
    MeaningEntry {
        token: "no",
        primary: &[("en", "No"), ("ru", "Нет"), ("hi", "नहीं"), ("zh", "不")],
        aliases: &[
            ("en", &["no", "nope", "nah"]),
            ("ru", &["нет", "неа"]),
            ("hi", &["नहीं", "ना"]),
            ("zh", &["不", "不是"]),
        ],
    },
];

/// Normalize a surface form for registry lookup: lower-case, drop everything
/// that is not a letter or digit, and collapse the result into a single
/// punctuation-free token. Mirrors the normalization performed by
/// `solver_helpers::normalize_meaning`.
fn normalize_alias(surface: &str) -> String {
    surface
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric())
        .collect()
}

/// Look up the canonical meaning token for a given surface form. Returns
/// `Some(token)` when the surface matches a registry alias for the source
/// language, and `None` otherwise. The caller falls back to a content hash
/// when no match is found, which preserves the existing behavior for
/// unrecognized phrases.
pub fn formalize_surface(surface: &str, source: &str) -> Option<&'static str> {
    let normalized = normalize_alias(surface);
    if normalized.is_empty() {
        return None;
    }
    for entry in MEANING_REGISTRY {
        for (lang, aliases) in entry.aliases {
            if *lang != source {
                continue;
            }
            if aliases
                .iter()
                .any(|alias| normalize_alias(alias) == normalized)
            {
                return Some(entry.token);
            }
        }
        for (lang, primary) in entry.primary {
            if *lang != source {
                continue;
            }
            if normalize_alias(primary) == normalized {
                return Some(entry.token);
            }
        }
    }
    None
}

/// Re-render a canonical meaning token into a target-language surface form.
/// Returns `None` when the token has no entry in the registry, or when the
/// requested target language is not registered for that token.
pub fn deformalize_meaning(token: &str, target: &str) -> Option<&'static str> {
    for entry in MEANING_REGISTRY {
        if entry.token != token {
            continue;
        }
        for (lang, primary) in entry.primary {
            if *lang == target {
                return Some(primary);
            }
        }
        return None;
    }
    None
}

/// Return the canonical token for a normalized alias (already lowercased and
/// stripped of non-alphanumeric characters), considering every supported
/// language. Used by `normalize_meaning` so the meaning id stays stable
/// regardless of the language of the input surface.
pub fn canonical_token_for_normalized(normalized: &str) -> Option<&'static str> {
    if normalized.is_empty() {
        return None;
    }
    for entry in MEANING_REGISTRY {
        for (_, aliases) in entry.aliases {
            if aliases
                .iter()
                .any(|alias| normalize_alias(alias) == normalized)
            {
                return Some(entry.token);
            }
        }
        for (_, primary) in entry.primary {
            if normalize_alias(primary) == normalized {
                return Some(entry.token);
            }
        }
    }
    None
}

const TERMINAL_PUNCTUATION: &[char] = &['?', '!', '.', '。', '？', '！', '．'];

/// Copy the source fragment's leading case and terminal punctuation onto the
/// target surface so a lowercase, unpunctuated source like `как у тебя дела`
/// produces a lowercase, unpunctuated target like `how are you`, and a
/// capitalized source like `Как у тебя дела?` stays `How are you?`.
///
/// English / Russian style guides (Chicago Manual of Style 5.10, Garner's
/// Modern English Usage "Capitalization", Розенталь §3) agree that
/// mid-sentence quoted fragments preserve their original capitalization, and
/// terminal punctuation is paired one-to-one between source and target. The
/// registry's primary forms are always capitalized and terminated; this helper
/// adjusts them to match the source.
pub fn match_source_formatting(target: &str, source: &str) -> String {
    let target_trimmed = target.trim();
    if target_trimmed.is_empty() {
        return String::new();
    }
    let source_trimmed = source.trim();

    let source_terminal = source_trimmed
        .chars()
        .next_back()
        .filter(|character| TERMINAL_PUNCTUATION.contains(character));
    let target_no_terminal: String = target_trimmed
        .trim_end_matches(|character: char| TERMINAL_PUNCTUATION.contains(&character))
        .to_owned();
    let with_terminal = match source_terminal {
        Some(character) => format!("{target_no_terminal}{character}"),
        None => target_no_terminal,
    };

    let Some(source_first_letter) = source_trimmed
        .chars()
        .find(|character| character.is_alphabetic())
    else {
        return with_terminal;
    };

    let Some((idx, target_first_letter)) = with_terminal
        .char_indices()
        .find(|(_, character)| character.is_alphabetic())
    else {
        return with_terminal;
    };

    if source_first_letter.is_lowercase() && target_first_letter.is_uppercase() {
        return splice_first_letter(&with_terminal, idx, target_first_letter, true);
    }
    if source_first_letter.is_uppercase() && target_first_letter.is_lowercase() {
        return splice_first_letter(&with_terminal, idx, target_first_letter, false);
    }
    with_terminal
}

fn splice_first_letter(source: &str, idx: usize, first_letter: char, to_lowercase: bool) -> String {
    let mut result = String::with_capacity(source.len());
    result.push_str(&source[..idx]);
    if to_lowercase {
        for character in first_letter.to_lowercase() {
            result.push(character);
        }
    } else {
        for character in first_letter.to_uppercase() {
            result.push(character);
        }
    }
    result.push_str(&source[idx + first_letter.len_utf8()..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formalize_matches_russian_alias() {
        assert_eq!(
            formalize_surface("как у тебя дела?", "ru"),
            Some("greeting_how_are_you"),
        );
    }

    #[test]
    fn formalize_matches_capitalized_russian_alias() {
        assert_eq!(
            formalize_surface("Как у тебя дела?", "ru"),
            Some("greeting_how_are_you"),
        );
    }

    #[test]
    fn formalize_matches_english_alias() {
        assert_eq!(formalize_surface("Hello", "en"), Some("greeting"));
        assert_eq!(formalize_surface("thanks", "en"), Some("thank_you"));
        assert_eq!(formalize_surface("yes", "en"), Some("yes"));
    }

    #[test]
    fn formalize_returns_none_for_unknown_surface() {
        assert_eq!(formalize_surface("xyzzy", "en"), None);
    }

    #[test]
    fn deformalize_returns_primary_form() {
        assert_eq!(
            deformalize_meaning("greeting_how_are_you", "en"),
            Some("How are you?"),
        );
        assert_eq!(
            deformalize_meaning("greeting_how_are_you", "ru"),
            Some("Как у тебя дела?"),
        );
        assert_eq!(deformalize_meaning("greeting", "hi"), Some("नमस्ते"));
        assert_eq!(deformalize_meaning("greeting", "zh"), Some("你好"));
    }

    #[test]
    fn deformalize_returns_none_for_unknown_token() {
        assert_eq!(deformalize_meaning("unknown_meaning", "en"), None);
    }

    #[test]
    fn match_source_formatting_preserves_lowercase_and_question_mark() {
        let result = match_source_formatting("How are you?", "как у тебя дела?");
        assert_eq!(result, "how are you?");
    }

    #[test]
    fn match_source_formatting_preserves_uppercase_and_question_mark() {
        let result = match_source_formatting("How are you?", "Как у тебя дела?");
        assert_eq!(result, "How are you?");
    }

    #[test]
    fn match_source_formatting_drops_terminal_when_source_has_none() {
        let result = match_source_formatting("How are you?", "как дела");
        assert_eq!(result, "how are you");
    }

    #[test]
    fn match_source_formatting_keeps_target_when_source_has_no_letters() {
        // The source has no letters, so the target keeps its own leading
        // case. Terminal punctuation still mirrors the source.
        let result = match_source_formatting("Hello", "...");
        assert_eq!(result, "Hello.");
    }

    #[test]
    fn match_source_formatting_handles_empty_source() {
        let result = match_source_formatting("Hello", "");
        assert_eq!(result, "Hello");
    }

    #[test]
    fn canonical_token_for_normalized_finds_token_regardless_of_language() {
        assert_eq!(canonical_token_for_normalized("hello"), Some("greeting"));
        assert_eq!(canonical_token_for_normalized("привет"), Some("greeting"));
        assert_eq!(
            canonical_token_for_normalized("какутебядела"),
            Some("greeting_how_are_you"),
        );
        assert_eq!(canonical_token_for_normalized("你好"), Some("greeting"));
    }
}
