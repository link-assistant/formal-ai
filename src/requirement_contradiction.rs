//! Contradiction detection over retained conversational requirements.
//!
//! Issue #661 (R384) asks the solver to notice when a newly formalized
//! requirement clashes with one the user already stated and to surface the
//! clash rather than silently letting the later message win. A requirement
//! here is a *standing directive* about how the assistant should behave —
//! "always answer in Russian", "never answer in Russian" — as opposed to a
//! one-off question. Two directives contradict when they make opposite demands
//! (one obligates, one prohibits) on the same subject.
//!
//! The detector is deliberately small and script-aware: it strips a
//! multilingual set of polarity markers ("always"/"never", "всегда"/"никогда",
//! "हमेशा"/"कभी", "总是"/"永远不要") from the prompt, and whatever remains is the
//! normalized *subject*. When the current subject matches a retained subject
//! but their polarities disagree, we append a `requirement_contradiction`
//! event and return a warning that names both statements, their weights, and a
//! resolution reusing the append-only retraction protocol
//! (`policy:add_only_history`).

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::Language;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::solver_handlers::finalize_simple;

/// Whether a directive obligates a behaviour or prohibits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Polarity {
    Require,
    Forbid,
}

impl Polarity {
    const fn opposes(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Require, Self::Forbid) | (Self::Forbid, Self::Require)
        )
    }
}

/// A parsed standing directive: its polarity plus the normalized subject the
/// polarity applies to (the message with its polarity markers removed).
#[derive(Debug, Clone)]
struct Directive {
    polarity: Polarity,
    subject: String,
}

/// Obligation markers, longest first so multi-word phrases strip before their
/// prefixes. ASCII entries match on word boundaries; non-ASCII entries match as
/// substrings (scriptio-continua languages carry no inter-word spaces).
const REQUIRE_MARKERS: &[&str] = &[
    // English
    "need to",
    "always",
    "please",
    "should",
    "must",
    // Russian
    "\u{043f}\u{043e}\u{0436}\u{0430}\u{043b}\u{0443}\u{0439}\u{0441}\u{0442}\u{0430}", // пожалуйста
    "\u{0432}\u{0441}\u{0435}\u{0433}\u{0434}\u{0430}",                                 // всегда
    "\u{0434}\u{043e}\u{043b}\u{0436}\u{0435}\u{043d}",                                 // должен
    "\u{043d}\u{0443}\u{0436}\u{043d}\u{043e}",                                         // нужно
    // Hindi
    "\u{0939}\u{092e}\u{0947}\u{0936}\u{093e}", // हमेशा
    "\u{0915}\u{0943}\u{092a}\u{092f}\u{093e}", // कृपया
    "\u{091a}\u{093e}\u{0939}\u{093f}\u{090f}", // चाहिए
    // Chinese
    "\u{603b}\u{662f}", // 总是
    "\u{5fc5}\u{987b}", // 必须
    "\u{4e00}\u{5b9a}", // 一定
    "\u{8bf7}",         // 请
];

/// Prohibition markers, longest first for the same reason.
const FORBID_MARKERS: &[&str] = &[
    // English
    "do not",
    "never",
    "avoid",
    "dont",
    "stop",
    "not",
    // Russian
    "\u{043d}\u{0438}\u{043a}\u{043e}\u{0433}\u{0434}\u{0430}", // никогда
    "\u{043d}\u{0435}\u{0442}",                                 // нет
    "\u{043d}\u{0435}",                                         // не
    // Hindi
    "\u{0915}\u{092d}\u{0940} \u{0928}\u{0939}\u{0940}\u{0902}", // कभी नहीं
    "\u{0928}\u{0939}\u{0940}\u{0902}",                          // नहीं
    "\u{0915}\u{092d}\u{0940}",                                  // कभी
    "\u{092e}\u{0924}",                                          // मत
    // Chinese
    "\u{6c38}\u{8fdc}\u{4e0d}\u{8981}", // 永远不要
    "\u{6c38}\u{8fdc}\u{4e0d}",         // 永远不
    "\u{4e0d}\u{8981}",                 // 不要
    "\u{4ece}\u{4e0d}",                 // 从不
    "\u{522b}",                         // 别
    "\u{4e0d}",                         // 不
];

/// Detect a contradiction between the current prompt and a retained requirement.
///
/// Returns a localized warning answer (and records the clash on `log`) when the
/// current prompt is a standing directive whose subject matches an earlier
/// directive of the opposite polarity; otherwise returns `None` so normal
/// handling proceeds.
pub fn detect_and_report(
    prompt: &str,
    language: Language,
    history: &[ConversationTurn],
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let current = classify(prompt)?;

    // Walk the retained user turns newest-first so the most recent conflicting
    // requirement is the one we surface.
    let prior_turn = history
        .iter()
        .rev()
        .filter(|turn| turn.role == ConversationRole::User)
        .find_map(|turn| {
            let directive = classify(&turn.content)?;
            (directive.subject == current.subject && directive.polarity.opposes(current.polarity))
                .then(|| (turn.content.trim().to_owned(), directive))
        })?;

    let (prior_text, prior) = prior_turn;
    let current_text = prompt.trim().to_owned();

    // The two directives are equally weighted: they are contradictory claims and
    // no symbolic evidence favours one over the other (0.50 + 0.50 = 1).
    let (require_text, forbid_text) = match prior.polarity {
        Polarity::Require => (prior_text.clone(), current_text.clone()),
        Polarity::Forbid => (current_text.clone(), prior_text.clone()),
    };
    log.append(
        "requirement_contradiction",
        format!(
            "subject=\"{}\" weight=0.50 require=\"{require_text}\" forbid=\"{forbid_text}\"",
            current.subject
        ),
    );
    // The resolution reuses the append-only retraction protocol: retracting a
    // requirement adds a superseding event rather than erasing history.
    log.append(
        "policy:add_only_history",
        "retraction_appends_superseding_event".to_owned(),
    );

    let body = warning_reply(language, &prior_text, &current_text, &current.subject);
    Some(finalize_simple(
        prompt,
        log,
        "requirement_contradiction",
        "response:requirement_contradiction",
        &body,
        0.3,
    ))
}

/// Parse a message into a standing directive, or `None` if it carries no clear
/// polarity marker or no subject once markers are removed.
fn classify(text: &str) -> Option<Directive> {
    let normalized = normalize(text);
    if normalized.is_empty() {
        return None;
    }

    let require = REQUIRE_MARKERS
        .iter()
        .any(|marker| marker_present(&normalized, marker));
    let forbid = FORBID_MARKERS
        .iter()
        .any(|marker| marker_present(&normalized, marker));

    // Exactly one polarity must be present. Both (or neither) is not a clean
    // standing directive we can reason about.
    let polarity = match (require, forbid) {
        (true, false) => Polarity::Require,
        (false, true) => Polarity::Forbid,
        _ => return None,
    };

    let mut subject = normalized;
    for marker in REQUIRE_MARKERS.iter().chain(FORBID_MARKERS.iter()) {
        subject = strip_marker(&subject, marker);
    }
    let subject = collapse_whitespace(&subject);
    if subject.is_empty() {
        return None;
    }
    Some(Directive { polarity, subject })
}

/// Lowercase, drop apostrophes (so "don't" -> "dont"), and turn every other
/// ASCII punctuation mark into a space so attached punctuation never fuses to a
/// subject token.
fn normalize(text: &str) -> String {
    let lowered = text.to_lowercase();
    let mut normalized = String::with_capacity(lowered.len());
    for character in lowered.chars() {
        if character == '\'' || character == '\u{2019}' {
            continue;
        }
        if character.is_ascii_punctuation() {
            normalized.push(' ');
        } else {
            normalized.push(character);
        }
    }
    collapse_whitespace(&normalized)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `marker` occurs in `haystack`. ASCII markers require word boundaries
/// so "not" never matches inside "nothing"; non-ASCII markers match as
/// substrings.
fn marker_present(haystack: &str, marker: &str) -> bool {
    if marker.is_ascii() {
        boundary_positions(haystack, marker).next().is_some()
    } else {
        haystack.contains(marker)
    }
}

/// Remove every (boundary-valid, for ASCII) occurrence of `marker`, leaving a
/// space in its place.
fn strip_marker(haystack: &str, marker: &str) -> String {
    if marker.is_ascii() {
        if boundary_positions(haystack, marker).next().is_none() {
            return haystack.to_owned();
        }
        let mut result = String::with_capacity(haystack.len());
        let mut cursor = 0;
        for start in boundary_positions(haystack, marker) {
            result.push_str(&haystack[cursor..start]);
            result.push(' ');
            cursor = start + marker.len();
        }
        result.push_str(&haystack[cursor..]);
        result
    } else {
        haystack.replace(marker, " ")
    }
}

/// Byte offsets of `marker` in `haystack` where both edges sit on an ASCII
/// word boundary. Collected up front so callers can iterate without borrow
/// juggling.
fn boundary_positions(haystack: &str, marker: &str) -> std::vec::IntoIter<usize> {
    let mut positions = Vec::new();
    if marker.is_empty() {
        return positions.into_iter();
    }
    let bytes = haystack.as_bytes();
    let mut search_from = 0;
    while let Some(offset) = haystack[search_from..].find(marker) {
        let start = search_from + offset;
        let end = start + marker.len();
        let before_ok = start == 0 || !bytes[start - 1].is_ascii_alphanumeric();
        let after_ok = end >= bytes.len() || !bytes[end].is_ascii_alphanumeric();
        if before_ok && after_ok {
            positions.push(start);
        }
        search_from = start + marker.len();
        if search_from >= haystack.len() {
            break;
        }
    }
    positions.into_iter()
}

/// Compose the localized contradiction warning. Every variant names both
/// statements, their equal weights, and a concrete resolution.
fn warning_reply(language: Language, prior: &str, current: &str, subject: &str) -> String {
    match language {
        Language::Russian => format!(
            "Предупреждение: два ваших требования противоречат друг другу.\n\
             - Утверждение 1 (вес 0.50): «{prior}»\n\
             - Утверждение 2 (вес 0.50): «{current}»\n\
             Они предъявляют противоположные требования к одному и тому же предмету \
             ({subject}), поэтому оба не могут выполняться одновременно.\n\
             Предлагаемое решение: оставьте одно требование и отзовите другое, отправив \
             замещающее требование — сеть требований работает только на добавление, поэтому \
             отзыв записывается новым событием и не стирает историю. Либо разделите смыслы, \
             либо ограничьте каждое требование своим контекстом, чтобы они не сталкивались."
        ),
        Language::Hindi => format!(
            "चेतावनी: आपकी दो आवश्यकताएँ एक-दूसरे का खंडन करती हैं।\n\
             - कथन 1 (भार 0.50): «{prior}»\n\
             - कथन 2 (भार 0.50): «{current}»\n\
             ये एक ही विषय ({subject}) पर विपरीत माँग करती हैं, इसलिए दोनों एक साथ लागू नहीं हो सकतीं।\n\
             प्रस्तावित समाधान: एक को रखें और दूसरी को एक अधिभावी आवश्यकता भेजकर वापस लें — आवश्यकताओं \
             का नेटवर्क केवल जोड़ने वाला है, इसलिए वापसी एक नई घटना के रूप में दर्ज होती है और इतिहास \
             मिटाती नहीं। अथवा अर्थों को अलग करें या प्रत्येक आवश्यकता को अलग संदर्भ में सीमित करें।"
        ),
        Language::Chinese => format!(
            "警告：您的两条要求相互矛盾。\n\
             - 陈述 1（权重 0.50）：「{prior}」\n\
             - 陈述 2（权重 0.50）：「{current}」\n\
             它们对同一主题（{subject}）提出相反的要求，因此二者不能同时成立。\n\
             建议的解决方案：保留其中一条，并通过发送一条取代性要求来撤回另一条——要求网络是仅追加的，\
             所以撤回会记录为一条新事件而不会抹去历史。或者拆分含义，或将每条要求限定在不同的上下文中，\
             使它们不再冲突。"
        ),
        Language::English | Language::Unknown => format!(
            "Warning: two of your requirements contradict each other.\n\
             - Statement 1 (weight 0.50): \"{prior}\"\n\
             - Statement 2 (weight 0.50): \"{current}\"\n\
             They make opposite demands on the same subject ({subject}), so both cannot hold at \
             once.\n\
             Proposed resolution: keep one and retract the other by sending a superseding \
             requirement — the requirement network is append-only, so the retraction is recorded \
             as a new event without erasing history. Alternatively, split the meanings or scope \
             each requirement to a different context so they no longer collide."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_polarity_same_subject_is_a_contradiction() {
        let always = classify("always answer in Russian").expect("directive");
        let never = classify("never answer in Russian").expect("directive");
        assert_eq!(always.subject, never.subject);
        assert!(always.polarity.opposes(never.polarity));
    }

    #[test]
    fn non_directive_prompt_is_not_classified() {
        assert!(classify("what is the capital of France").is_none());
    }

    #[test]
    fn boundary_prevents_substring_false_positive() {
        // "nothing" contains "not" but is not a prohibition marker hit.
        assert!(!marker_present("nothing lasts", "not"));
        assert!(marker_present("do not shout", "not"));
    }
}
