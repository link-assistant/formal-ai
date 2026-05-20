//! Calendar and weekday relation reasoning.
//!
//! This handler keeps date-like questions inside the symbolic solver when the
//! answer can be derived from a stable calendar relation instead of an external
//! clock or lookup.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::solver_handlers::finalize_simple;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    const fn index(self) -> usize {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }

    const fn from_index(index: usize) -> Self {
        match index % 7 {
            0 => Self::Monday,
            1 => Self::Tuesday,
            2 => Self::Wednesday,
            3 => Self::Thursday,
            4 => Self::Friday,
            5 => Self::Saturday,
            _ => Self::Sunday,
        }
    }

    const fn shifted(self, operation: WeekdayOperation) -> Self {
        match operation {
            WeekdayOperation::Next => Self::from_index(self.index() + 1),
            WeekdayOperation::Previous => Self::from_index(self.index() + 6),
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Monday => "monday",
            Self::Tuesday => "tuesday",
            Self::Wednesday => "wednesday",
            Self::Thursday => "thursday",
            Self::Friday => "friday",
            Self::Saturday => "saturday",
            Self::Sunday => "sunday",
        }
    }

    const fn en(self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }

    const fn ru(self) -> &'static str {
        match self {
            Self::Monday => "понедельник",
            Self::Tuesday => "вторник",
            Self::Wednesday => "среда",
            Self::Thursday => "четверг",
            Self::Friday => "пятница",
            Self::Saturday => "суббота",
            Self::Sunday => "воскресенье",
        }
    }

    const fn ru_genitive(self) -> &'static str {
        match self {
            Self::Monday => "понедельника",
            Self::Tuesday => "вторника",
            Self::Wednesday => "среды",
            Self::Thursday => "четверга",
            Self::Friday => "пятницы",
            Self::Saturday => "субботы",
            Self::Sunday => "воскресенья",
        }
    }

    const fn ru_instrumental(self) -> &'static str {
        match self {
            Self::Monday => "понедельником",
            Self::Tuesday => "вторником",
            Self::Wednesday => "средой",
            Self::Thursday => "четвергом",
            Self::Friday => "пятницей",
            Self::Saturday => "субботой",
            Self::Sunday => "воскресеньем",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeekdayOperation {
    Next,
    Previous,
}

impl WeekdayOperation {
    const fn delta(self) -> &'static str {
        match self {
            Self::Next => "+1",
            Self::Previous => "-1",
        }
    }

    const fn event_kind(self) -> &'static str {
        match self {
            Self::Next => "calendar:operation:next",
            Self::Previous => "calendar:operation:previous",
        }
    }
}

const WEEKDAY_ALIASES: &[(&str, Weekday)] = &[
    ("monday", Weekday::Monday),
    ("mon", Weekday::Monday),
    ("понедельника", Weekday::Monday),
    ("понедельником", Weekday::Monday),
    ("понедельнику", Weekday::Monday),
    ("понедельнике", Weekday::Monday),
    ("понедельник", Weekday::Monday),
    ("tuesday", Weekday::Tuesday),
    ("tue", Weekday::Tuesday),
    ("tues", Weekday::Tuesday),
    ("вторника", Weekday::Tuesday),
    ("вторником", Weekday::Tuesday),
    ("вторнику", Weekday::Tuesday),
    ("вторнике", Weekday::Tuesday),
    ("вторник", Weekday::Tuesday),
    ("wednesday", Weekday::Wednesday),
    ("wed", Weekday::Wednesday),
    ("средой", Weekday::Wednesday),
    ("среде", Weekday::Wednesday),
    ("среду", Weekday::Wednesday),
    ("среды", Weekday::Wednesday),
    ("среда", Weekday::Wednesday),
    ("thursday", Weekday::Thursday),
    ("thu", Weekday::Thursday),
    ("thur", Weekday::Thursday),
    ("thurs", Weekday::Thursday),
    ("четверга", Weekday::Thursday),
    ("четвергом", Weekday::Thursday),
    ("четвергу", Weekday::Thursday),
    ("четверге", Weekday::Thursday),
    ("четверг", Weekday::Thursday),
    ("friday", Weekday::Friday),
    ("fri", Weekday::Friday),
    ("пятницей", Weekday::Friday),
    ("пятнице", Weekday::Friday),
    ("пятницу", Weekday::Friday),
    ("пятницы", Weekday::Friday),
    ("пятница", Weekday::Friday),
    ("saturday", Weekday::Saturday),
    ("sat", Weekday::Saturday),
    ("субботой", Weekday::Saturday),
    ("субботе", Weekday::Saturday),
    ("субботу", Weekday::Saturday),
    ("субботы", Weekday::Saturday),
    ("суббота", Weekday::Saturday),
    ("sunday", Weekday::Sunday),
    ("sun", Weekday::Sunday),
    ("воскресеньем", Weekday::Sunday),
    ("воскресенью", Weekday::Sunday),
    ("воскресенья", Weekday::Sunday),
    ("воскресенье", Weekday::Sunday),
];

const NEXT_MARKERS: &[&str] = &[
    "after",
    "comes after",
    "day after",
    "next day",
    "following day",
    "following weekday",
    "follows",
    "после",
    "наступает после",
    "следующий день",
    "следующая",
    "следом за",
];

const PREVIOUS_MARKERS: &[&str] = &[
    "before",
    "comes before",
    "day before",
    "previous day",
    "previous weekday",
    "precedes",
    "перед",
    "предыдущий день",
    "предыдущая",
    "предшествует",
];

pub fn try_calendar_reasoning(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !mentions_weekday_context(normalized) {
        return None;
    }
    let operation = detect_operation(normalized)?;
    let source = detect_weekday(normalized)?;
    let result = source.shifted(operation);

    log.append(
        "calendar:cycle",
        "monday,tuesday,wednesday,thursday,friday,saturday,sunday",
    );
    log.append("calendar:subject_weekday", source.slug());
    log.append(operation.event_kind(), source.slug());
    log.append("calendar:result_weekday", result.slug());

    let language = detect_language(prompt).slug();
    let body = render_answer(language, operation, source, result);
    Some(finalize_simple(
        prompt,
        log,
        "calendar_weekday_relation",
        "response:calendar_weekday_relation",
        &body,
        1.0,
    ))
}

fn mentions_weekday_context(normalized: &str) -> bool {
    ["day", "weekday", "week day", "день", "дня", "дни", "дней"]
        .iter()
        .any(|marker| contains_term(normalized, marker))
        || normalized.contains("недел")
}

fn detect_operation(normalized: &str) -> Option<WeekdayOperation> {
    let has_next = NEXT_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker));
    let has_previous = PREVIOUS_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker));
    match (has_next, has_previous) {
        (true, false) => Some(WeekdayOperation::Next),
        (false, true) => Some(WeekdayOperation::Previous),
        _ => None,
    }
}

fn detect_weekday(normalized: &str) -> Option<Weekday> {
    WEEKDAY_ALIASES
        .iter()
        .find_map(|(alias, weekday)| contains_term(normalized, alias).then_some(*weekday))
}

fn contains_term(haystack: &str, needle: &str) -> bool {
    haystack.match_indices(needle).any(|(start, _)| {
        let before = haystack[..start].chars().next_back();
        let after = haystack[start + needle.len()..].chars().next();
        before.map_or(true, |character| !is_word_character(character))
            && after.map_or(true, |character| !is_word_character(character))
    })
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn render_answer(
    language: &str,
    operation: WeekdayOperation,
    source: Weekday,
    result: Weekday,
) -> String {
    match language {
        "ru" => match operation {
            WeekdayOperation::Next => format!(
                "После {} наступает {}. Я сдвинул {} на {} в семидневном календарном цикле.",
                source.ru_genitive(),
                result.ru(),
                source.ru(),
                operation.delta(),
            ),
            WeekdayOperation::Previous => format!(
                "Перед {} идёт {}. Я сдвинул {} на {} в семидневном календарном цикле.",
                source.ru_instrumental(),
                result.ru(),
                source.ru(),
                operation.delta(),
            ),
        },
        _ => match operation {
            WeekdayOperation::Next => format!(
                "The day after {} is {}. I move {} by {} in the seven-day calendar cycle.",
                source.en(),
                result.en(),
                source.en(),
                operation.delta(),
            ),
            WeekdayOperation::Previous => format!(
                "The day before {} is {}. I move {} by {} in the seven-day calendar cycle.",
                source.en(),
                result.en(),
                source.en(),
                operation.delta(),
            ),
        },
    }
}
