//! Rendering of source-backed release timelines (issue #892).
//!
//! A timeline answer is never stored as a sentence: it is computed from the
//! checked-in snapshot every time it is asked for. Given the timeline, the
//! answer language, and the day the question is asked, this module
//!
//!   * splits the works into *released* (a publication date on or before that
//!     day) and *announced* (a later date, or no date at all),
//!   * orders the released works by publication date, oldest first,
//!   * and appends the snapshot provenance — which source, taken when — using
//!     the stale wording once the snapshot is older than its freshness window.
//!
//! Everything language-specific comes from the `phrasing` blocks of
//! `data/seed/release-timelines.lino`; this file only fills placeholders.

use crate::seed::{release_timelines, ReleaseTimeline, ReleaseTimelineEntry, ReleaseTimelines};

/// A rendered timeline plus the classification the answer is built from, so
/// callers (and tests) can inspect the split without parsing prose.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedTimeline {
    /// The full answer text.
    pub text: String,
    /// Works released on or before the rendering day, oldest first.
    pub released: Vec<ReleaseTimelineEntry>,
    /// Works with a later date, or no announced date, in snapshot order.
    pub announced: Vec<ReleaseTimelineEntry>,
    /// Whether the snapshot is older than the timeline's freshness window.
    pub stale: bool,
}

/// Render the registered timeline `slug` for `language` as of `today`
/// (an ISO `YYYY-MM-DD` day). Returns `None` when the slug is unknown.
#[must_use]
pub fn render(slug: &str, language: &str, today: &str) -> Option<RenderedTimeline> {
    render_from(release_timelines(), slug, language, today)
}

/// Render a timeline from an explicit registry — the injection point tests use
/// to render fixed snapshots against fixed days.
#[must_use]
pub fn render_from(
    registry: &ReleaseTimelines,
    slug: &str,
    language: &str,
    today: &str,
) -> Option<RenderedTimeline> {
    let timeline = registry.timeline(slug)?;
    let phrasing = registry.phrasing_for(language)?;

    let mut released: Vec<ReleaseTimelineEntry> = timeline
        .entries
        .iter()
        .filter(|entry| is_released(entry, today))
        .cloned()
        .collect();
    // ISO dates sort chronologically as plain strings; the id keeps same-day
    // releases in a stable order.
    released.sort_by(|left, right| {
        (&left.release_date, &left.qid).cmp(&(&right.release_date, &right.qid))
    });
    let announced: Vec<ReleaseTimelineEntry> = timeline
        .entries
        .iter()
        .filter(|entry| !is_released(entry, today))
        .cloned()
        .collect();
    let stale = is_stale(timeline, today);

    let mut sections = Vec::new();
    if !released.is_empty() {
        let items: Vec<String> = released
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                phrasing
                    .released_item
                    .replace("{position}", &(index + 1).to_string())
                    .replace("{title}", entry.title_for(language))
                    .replace("{year}", entry.release_year())
                    .replace("{date}", &entry.release_date)
            })
            .collect();
        sections.push(format!(
            "{} {}{}",
            phrasing
                .released_heading
                .replace("{subject}", timeline.subject_for(language)),
            items.join(&phrasing.item_separator),
            phrasing.section_end
        ));
    }
    if !announced.is_empty() {
        let items: Vec<String> = announced
            .iter()
            .map(|entry| {
                let template = if entry.release_date.is_empty() {
                    &phrasing.undated_item
                } else {
                    &phrasing.announced_item
                };
                template
                    .replace("{title}", entry.title_for(language))
                    .replace("{year}", entry.release_year())
                    .replace("{date}", &entry.release_date)
            })
            .collect();
        sections.push(format!(
            "{} {}{}",
            phrasing
                .announced_heading
                .replace("{subject}", timeline.subject_for(language)),
            items.join(&phrasing.item_separator),
            phrasing.section_end
        ));
    }
    let note = if stale {
        &phrasing.stale_note
    } else {
        &phrasing.provenance_note
    };
    sections.push(
        note.replace("{source}", &timeline.source_label)
            .replace("{retrieved}", &timeline.retrieved_at),
    );

    Some(RenderedTimeline {
        text: sections.join(" "),
        released,
        announced,
        stale,
    })
}

/// A work counts as released once its publication date is on or before `today`.
/// An undated work is announced, never released.
fn is_released(entry: &ReleaseTimelineEntry, today: &str) -> bool {
    !entry.release_date.is_empty() && entry.release_date.as_str() <= today
}

/// A snapshot is stale once more days than its freshness window have passed
/// since it was taken. An unparseable or missing stamp is treated as stale, so
/// a broken provenance record can never claim to be current.
#[must_use]
pub fn is_stale(timeline: &ReleaseTimeline, today: &str) -> bool {
    let (Some(retrieved), Some(now)) = (
        days_from_iso_date(&timeline.retrieved_at),
        days_from_iso_date(today),
    ) else {
        return true;
    };
    now - retrieved > timeline.fresh_for_days
}

/// Days since the Unix epoch for an ISO `YYYY-MM-DD` day.
///
/// Algorithm adapted from days-from-civil (Howard Hinnant, public domain).
fn days_from_iso_date(date: &str) -> Option<i64> {
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}
