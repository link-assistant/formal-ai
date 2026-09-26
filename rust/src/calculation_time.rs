//! Clock-time duration extraction for calculator routing.

use crate::calculation::contains_word_operator;
use crate::seed;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClockTimeMention {
    start: usize,
    end: usize,
    text: String,
}

fn clock_time_mentions(value: &str) -> Vec<ClockTimeMention> {
    let mut mentions = Vec::new();
    for (colon, _) in value.match_indices(':') {
        let mut start = colon;
        let mut hour_digits = 0;
        while hour_digits < 2 {
            let Some((previous_start, character)) = value[..start].char_indices().next_back()
            else {
                break;
            };
            if !character.is_ascii_digit() {
                break;
            }
            start = previous_start;
            hour_digits += 1;
        }
        if hour_digits == 0 {
            continue;
        }

        let mut end = colon + ':'.len_utf8();
        let mut minute_digits = 0;
        while minute_digits < 2 {
            let Some(character) = value[end..].chars().next() else {
                break;
            };
            if !character.is_ascii_digit() {
                break;
            }
            end += character.len_utf8();
            minute_digits += 1;
        }
        if minute_digits != 2 {
            continue;
        }

        let before = value[..start].chars().next_back();
        let after = value[end..].chars().next();
        if before.is_some_and(char::is_alphanumeric) || after.is_some_and(char::is_alphanumeric) {
            continue;
        }

        let Ok(hour) = value[start..colon].parse::<u8>() else {
            continue;
        };
        let Ok(minute) = value[colon + ':'.len_utf8()..end].parse::<u8>() else {
            continue;
        };
        if hour > 23 || minute > 59 {
            continue;
        }

        mentions.push(ClockTimeMention {
            start,
            end,
            text: value[start..end].to_owned(),
        });
    }
    mentions
}

fn explicit_time_subtraction_between(
    value: &str,
    left: &ClockTimeMention,
    right: &ClockTimeMention,
) -> bool {
    let between = &value[left.end..right.start];
    between.contains(['-', '−']) || contains_word_operator(between)
}

pub fn elapsed_time_expression(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    if !seed::lexicon().mentions_role(seed::ROLE_TIME_DURATION_CUE, &lower) {
        return None;
    }
    let mentions = clock_time_mentions(prompt);
    let [first, second] = mentions.as_slice() else {
        return None;
    };
    if explicit_time_subtraction_between(prompt, first, second) {
        Some(format!("{} - {}", first.text, second.text))
    } else {
        Some(format!("{} - {}", second.text, first.text))
    }
}
