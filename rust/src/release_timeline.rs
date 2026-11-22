//! Rendering of source-backed release timelines (issue #892).
//!
//! A timeline answer is never stored as a sentence: it is computed from the
//! verified publisher capture for the requested series. Given the timeline, the
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

use crate::seed::{
    ReleaseTimeline, ReleaseTimelineEntry, ReleaseTimelinePhrasing, ReleaseTimelines,
    release_timelines,
};

const SUBJECT_SLOT: &str = "{subject}";
const LANGUAGE_SLOT: &str = "{language}";

/// A rendered timeline plus the classification the answer is built from, so
/// callers (and tests) can inspect the split without parsing prose.
#[derive(Debug, Clone, PartialEq, Eq)]
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
/// (an ISO `YYYY-MM-DD` day), using a verified publisher capture.
#[must_use]
pub fn render(slug: &str, language: &str, today: &str) -> Option<RenderedTimeline> {
    let client = crate::fact_live::capture_client(!crate::solver_config::read_network_allowed());
    try_release_timeline(
        slug,
        language,
        today,
        &mut crate::event_log::EventLog::new(),
        &client,
    )
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
    render_timeline(timeline, registry.phrasing_for(language)?, language, today)
}

fn render_timeline(
    timeline: &ReleaseTimeline,
    phrasing: &ReleaseTimelinePhrasing,
    language: &str,
    today: &str,
) -> Option<RenderedTimeline> {
    if timeline.entries.is_empty() {
        return None;
    }

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
                .replace(SUBJECT_SLOT, timeline.subject_for(language)),
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
                .replace(SUBJECT_SLOT, timeline.subject_for(language)),
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

/// Is this snapshot past its freshness window?
///
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
    if date.len() != 10
        || !date.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 4 | 7) {
                byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
    {
        return None;
    }
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    let last_day = match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return None,
    };
    if !(1..=last_day).contains(&day) {
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

const MAXIMUM_WORKS: usize = 100;
const MAXIMUM_CAPTURE_BYTES: usize = 256 * 1024;
const PUBLISHER_REFRESH_SECONDS: u64 = 3_600;
const MAXIMUM_CAPTURE_TIMESTAMP: i64 = 253_402_300_799; // 9999-12-31 23:59:59 UTC

fn timeline_url(timeline: &ReleaseTimeline, query: &str, language: &str) -> Option<url::Url> {
    if wikidata_identifier(&timeline.subject_qid).is_none()
        || !language.bytes().all(|byte| byte.is_ascii_lowercase())
        || !(2..=3).contains(&language.len())
        || !query.contains(SUBJECT_SLOT)
        || !query.contains(LANGUAGE_SLOT)
    {
        return None;
    }
    let mut endpoint = url::Url::parse(&timeline.source_url).ok()?;
    if endpoint.scheme() != "https"
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.fragment().is_some()
        || endpoint.query().is_some()
    {
        return None;
    }
    endpoint
        .query_pairs_mut()
        .append_pair(
            "query",
            &query
                .replace(SUBJECT_SLOT, &timeline.subject_qid)
                .replace(LANGUAGE_SLOT, language),
        )
        .append_pair("format", "json");
    Some(endpoint)
}

fn wikidata_identifier(identifier: &str) -> Option<&str> {
    let digits = identifier.strip_prefix('Q')?;
    (!digits.is_empty() && digits.len() <= 18 && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some(identifier)
}

fn binding_identifier(binding: &serde_json::Value) -> Option<&str> {
    if binding.get("type")?.as_str()? != "uri" {
        return None;
    }
    let value = binding.get("value")?.as_str()?;
    wikidata_identifier(
        value
            .strip_prefix("http://www.wikidata.org/entity/")
            .or_else(|| value.strip_prefix("https://www.wikidata.org/entity/"))?,
    )
}

/// Decode one complete, bounded result set from the declared publisher.
fn captured_entries(
    bytes: &[u8],
    subject: &str,
    language: &str,
) -> Option<Vec<ReleaseTimelineEntry>> {
    if bytes.len() > MAXIMUM_CAPTURE_BYTES {
        return None;
    }
    let document: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    let rows = document.pointer("/results/bindings")?.as_array()?;
    if rows.is_empty() || rows.len() > MAXIMUM_WORKS {
        return None;
    }
    let mut seen = std::collections::BTreeSet::new();
    rows.iter()
        .map(|row| {
            if binding_identifier(row.get("subject")?)? != subject {
                return None;
            }
            let qid = binding_identifier(row.get("film")?)?;
            if !seen.insert(qid) {
                return None;
            }
            let release_date = match row.get("firstRelease") {
                Some(binding) => {
                    if binding.get("type")?.as_str()? != "literal"
                        || binding.get("datatype")?.as_str()?
                            != "http://www.w3.org/2001/XMLSchema#dateTime"
                    {
                        return None;
                    }
                    let timestamp = binding.get("value")?.as_str()?;
                    let (day, _) = timestamp.split_once('T')?;
                    if day.len() != 10 || days_from_iso_date(day).is_none() {
                        return None;
                    }
                    day.to_owned()
                }
                None => String::new(),
            };
            let titles: Vec<_> = [("localTitle", language), ("title", "en")]
                .into_iter()
                .filter_map(|(field, code)| {
                    let binding = row.get(field)?;
                    let locale = binding.get("xml:lang")?.as_str()?;
                    if binding.get("type")?.as_str()? != "literal"
                        || (locale != code && !(field == "title" && locale == "mul"))
                    {
                        return None;
                    }
                    let value = binding.get("value")?.as_str()?.trim();
                    (!value.is_empty() && value.chars().count() <= 300)
                        .then(|| (code.to_owned(), value.to_owned()))
                })
                .collect();
            (!titles.is_empty()).then(|| ReleaseTimelineEntry {
                qid: qid.to_owned(),
                release_date,
                titles,
            })
        })
        .collect()
}

pub(crate) fn try_release_timeline<T: crate::source_fetch::SourceTransport>(
    slug: &str,
    language: &str,
    today: &str,
    log: &mut crate::event_log::EventLog,
    client: &crate::source_fetch::CachedSourceClient<T>,
) -> Option<RenderedTimeline> {
    let registry = release_timelines();
    let timeline = registry.timeline(slug)?.clone();
    if !timeline_source_allowed(log) {
        return None;
    }
    fetch_timeline(timeline, registry, language, today, log, client)
}

fn timeline_source_allowed(log: &mut crate::event_log::EventLog) -> bool {
    let preferences = crate::solver_handler_how_synthesis::service_preferences_from_env(log);
    crate::seed::source_record("wikidata").is_some_and(|record| preferences.allows(&record))
}

fn fetch_timeline<T: crate::source_fetch::SourceTransport>(
    mut timeline: ReleaseTimeline,
    registry: &ReleaseTimelines,
    language: &str,
    today: &str,
    log: &mut crate::event_log::EventLog,
    client: &crate::source_fetch::CachedSourceClient<T>,
) -> Option<RenderedTimeline> {
    let endpoint = timeline_url(&timeline, &registry.query, language)?;
    log.append("release_timeline:request", endpoint.as_str());
    let capture = match client.fetch_with_ttl_seconds(endpoint.as_str(), PUBLISHER_REFRESH_SECONDS)
    {
        Ok(capture) => capture,
        Err(error) => {
            log.append("release_timeline:unavailable", error.to_string());
            return None;
        }
    };
    let Some(entries) = captured_entries(capture.bytes(), &timeline.subject_qid, language) else {
        log.append("release_timeline:rejected", "invalid_publisher_rows");
        return None;
    };
    let timestamp = capture.fetched_at().parse::<i64>().ok()?;
    if !(0..=MAXIMUM_CAPTURE_TIMESTAMP).contains(&timestamp) {
        return None;
    }
    timeline.source_url = endpoint.into();
    timeline.retrieved_at = crate::external_benchmarks::format_date(timestamp.div_euclid(86_400));
    timeline.sha256 = capture.sha256().into();
    timeline.entries = entries;
    let rendered = render_timeline(&timeline, registry.phrasing_for(language)?, language, today)?;
    capture.record(log);
    log.append("release_timeline:hit", &timeline.slug);
    log.append("release_timeline:snapshot", &timeline.retrieved_at);
    log.append("release_timeline:source", &timeline.source_url);
    log.append("release_timeline:sha256", &timeline.sha256);
    log.append(
        "release_timeline:released",
        rendered.released.len().to_string(),
    );
    log.append(
        "release_timeline:announced",
        rendered.announced.len().to_string(),
    );
    if rendered.stale {
        log.append("release_timeline:stale", &timeline.retrieved_at);
    }
    Some(rendered)
}

fn series_subject<'a>(normalized: &'a str, frame: &str) -> Option<&'a str> {
    let (before, after) = frame.split_once(SUBJECT_SLOT)?;
    let subject = normalized.strip_prefix(before)?.strip_suffix(after)?.trim();
    crate::fact_live::bounded_subject(subject)?;
    (subject.chars().count() <= 120).then_some(subject)
}

pub(crate) fn try_open_release_timeline<T: crate::source_fetch::SourceTransport>(
    normalized: &str,
    language: &str,
    today: &str,
    log: &mut crate::event_log::EventLog,
    client: &crate::source_fetch::CachedSourceClient<T>,
) -> Option<RenderedTimeline> {
    let registry = release_timelines();
    let series = registry.open_series.as_ref()?;
    let request = series
        .requests
        .iter()
        .find(|entry| entry.language == language)?;
    let subject = request
        .frames
        .iter()
        .find_map(|frame| series_subject(normalized, frame))?;
    if !timeline_source_allowed(log) {
        return None;
    }
    let query = request.search.replace(SUBJECT_SLOT, subject);
    let (mut candidates, mut captures) =
        crate::fact_live::search_item_candidates(client, &query, language).ok()?;
    if candidates.is_empty() {
        let (matches, evidence) =
            crate::fact_live::search_item_candidates(client, subject, language).ok()?;
        candidates = matches;
        captures.extend(evidence);
    }
    let [matched] = candidates.as_slice() else {
        log.append("release_timeline:rejected", "unresolved_series");
        return None;
    };
    let mut timeline = series.timeline.clone();
    timeline.subject_qid.clone_from(&matched.identifier);
    timeline.subjects = vec![(
        language.to_owned(),
        request.subject.replace(SUBJECT_SLOT, &matched.label),
    )];
    let rendered = fetch_timeline(timeline, registry, language, today, log, client)?;
    for capture in &captures {
        capture.record(log);
    }
    log.append("wikidata", matched.identifier.as_str());
    Some(rendered)
}
