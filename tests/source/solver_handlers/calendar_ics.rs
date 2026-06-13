//! Calendar export artifacts for natural-language scheduling (issue #404).
//!
//! A parsed [`ScheduledEvent`] is rendered into two portable, login-free forms:
//! a universal RFC 5545 `.ics` VEVENT document any calendar app can import, and
//! a Google Calendar "render" template URL that pre-fills the event in a browser
//! with no API token or server. The CLI, HTTP server, and browser worker each
//! surface whichever is simplest for the user.

use super::calendar::days_to_date;
use std::time::{SystemTime, UNIX_EPOCH};

/// A fully-parsed scheduling request, ready to be exported to any calendar.
/// Times are wall-clock times in `time_zone` (a floating local time paired with
/// an IANA zone id), which is exactly what both iCalendar `TZID` and the Google
/// Calendar `ctz` parameter expect.
pub(super) struct ScheduledEvent {
    pub(super) title: String,
    pub(super) year: i32,
    pub(super) month: u32,
    pub(super) day: u32,
    pub(super) hour: u32,
    pub(super) minute: u32,
    pub(super) time_zone: &'static str,
    pub(super) duration_minutes: u32,
}

impl ScheduledEvent {
    pub(super) fn iso_date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Wall-clock start as the basic iCalendar date-time form `YYYYMMDDTHHMMSS`.
    fn start_stamp(&self) -> String {
        format!(
            "{:04}{:02}{:02}T{:02}{:02}00",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }

    /// Wall-clock end, derived from the start plus the duration, rolling over
    /// across the hour/day/month/year boundaries as needed.
    fn end_stamp(&self) -> String {
        let (year, month, day, hour, minute) = add_minutes(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.duration_minutes,
        );
        format!(
            "{:04}{:02}{:02}T{:02}{:02}00",
            year, month, day, hour, minute
        )
    }

    /// A stable, content-derived UID so re-importing the same proposal updates
    /// the existing entry instead of duplicating it.
    fn uid(&self) -> String {
        format!("{}-{}@formal-ai", self.start_stamp(), self.time_zone)
    }

    /// Build an RFC 5545 VEVENT calendar document. CRLF line endings keep it
    /// spec-compliant so it imports cleanly into Apple Calendar, Outlook,
    /// Google Calendar, Thunderbird, and any other iCalendar client.
    pub(super) fn to_ics(&self) -> String {
        let lines = [
            "BEGIN:VCALENDAR".to_owned(),
            "VERSION:2.0".to_owned(),
            "PRODID:-//formal-ai//calendar//EN".to_owned(),
            "CALSCALE:GREGORIAN".to_owned(),
            "METHOD:PUBLISH".to_owned(),
            "BEGIN:VEVENT".to_owned(),
            format!("UID:{}", self.uid()),
            format!("DTSTAMP:{}", ics_dtstamp()),
            format!("DTSTART;TZID={}:{}", self.time_zone, self.start_stamp()),
            format!("DTEND;TZID={}:{}", self.time_zone, self.end_stamp()),
            format!("SUMMARY:{}", ics_escape(&self.title)),
            "END:VEVENT".to_owned(),
            "END:VCALENDAR".to_owned(),
        ];
        let mut out = lines.join("\r\n");
        out.push_str("\r\n");
        out
    }

    /// Build a Google Calendar "render" template URL. Opening it pre-fills a new
    /// event in the user's logged-in calendar with no API token or server — the
    /// simplest possible path in a browser environment.
    pub(super) fn to_google_calendar_url(&self) -> String {
        format!(
            "https://calendar.google.com/calendar/render?action=TEMPLATE&text={}&dates={}/{}&ctz={}",
            percent_encode(&self.title),
            self.start_stamp(),
            self.end_stamp(),
            percent_encode(self.time_zone),
        )
    }
}

/// Add `minutes` to a wall-clock date-time, rolling over hour/day/month/year.
fn add_minutes(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    minutes: u32,
) -> (i32, u32, u32, u32, u32) {
    let total = hour * 60 + minute + minutes;
    let mut day_carry = total / (24 * 60);
    let new_minute = total % 60;
    let new_hour = (total / 60) % 24;
    let mut y = year;
    let mut m = month;
    let mut d = day;
    while day_carry > 0 {
        let max_day = days_in_month(y, m);
        if d < max_day {
            d += 1;
        } else {
            d = 1;
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
        day_carry -= 1;
    }
    (y, m, d, new_hour, new_minute)
}

const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Current UTC instant as an iCalendar UTC stamp `YYYYMMDDTHHMMSSZ` for DTSTAMP.
fn ics_dtstamp() -> String {
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return "19700101T000000Z".to_owned();
    };
    let secs = now.as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, m, d) = days_to_date(days);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        y,
        m,
        d,
        sod / 3_600,
        (sod % 3_600) / 60,
        sod % 60,
    )
}

/// Escape a text value for an iCalendar property (RFC 5545 §3.3.11).
fn ics_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

/// Percent-encode a string for use in a URL query value (RFC 3986 unreserved
/// set kept literal; everything else encoded as UTF-8 `%XX`).
fn percent_encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0x0F) as usize] as char);
            }
        }
    }
    out
}
