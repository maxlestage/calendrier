//! Heuristic parser that turns the raw text of *any* timetable
//! ("emploi du temps") — typed, pasted, or produced by OCR from a photo —
//! into a list of draft calendar events the user can review before importing.
//!
//! The client (native app via Apple Vision OCR, or web via Tesseract) does the
//! image → text step; this module does text → events so the logic is shared and
//! can improve without shipping a new app build.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Europe::Paris;

use regex::Regex;
use std::sync::LazyLock;
use serde::Serialize;

/// One candidate event extracted from the schedule text. Times are UTC ISO
/// 8601 (the format the rest of the API stores), derived from Paris-local
/// wall-clock times found in the text.
#[derive(Debug, Serialize, PartialEq)]
pub struct DraftEvent {
    pub title: String,
    pub start: String,
    pub end: String,
    pub all_day: bool,
}

/// A clock time found in the text, in 24h Paris local.
#[derive(Clone, Copy, Debug)]
struct Clock {
    h: u32,
    m: u32,
}

static TIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    // "9h", "9h30", "09 h 30", "9:30", "14:00", "9H30"
    Regex::new(r"(?i)\b(\d{1,2})\s*(?:h|:)\s*(\d{2})?\b").unwrap()
});

static DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    // "12/09", "12/09/2026", "12-09-26", "12.09.2026"
    Regex::new(r"\b(\d{1,2})[/.\-](\d{1,2})(?:[/.\-](\d{2,4}))?\b").unwrap()
});

static MONTH_DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    // "12 septembre", "1er mars 2026"
    Regex::new(r"(?i)\b(\d{1,2})\s*(?:er)?\s+(janvier|février|fevrier|mars|avril|mai|juin|juillet|août|aout|septembre|octobre|novembre|décembre|decembre)\b(?:\s+(\d{4}))?").unwrap()
});

static WEEKDAY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(lundi|mardi|mercredi|jeudi|vendredi|samedi|dimanche|lun|mar|mer|jeu|ven|sam|dim)\b").unwrap()
});

fn weekday_offset(w: &str) -> Option<i64> {
    let w = w.to_lowercase();
    let idx = match w.as_str() {
        s if s.starts_with("lun") => 0,
        s if s.starts_with("mar") => 1,
        s if s.starts_with("mer") => 2,
        s if s.starts_with("jeu") => 3,
        s if s.starts_with("ven") => 4,
        s if s.starts_with("sam") => 5,
        s if s.starts_with("dim") => 6,
        _ => return None,
    };
    Some(idx)
}

fn month_number(m: &str) -> Option<u32> {
    let m = m.to_lowercase();
    Some(match m.as_str() {
        "janvier" => 1,
        "février" | "fevrier" => 2,
        "mars" => 3,
        "avril" => 4,
        "mai" => 5,
        "juin" => 6,
        "juillet" => 7,
        "août" | "aout" => 8,
        "septembre" => 9,
        "octobre" => 10,
        "novembre" => 11,
        "décembre" | "decembre" => 12,
        _ => return None,
    })
}

fn valid_clock(h: u32, m: u32) -> Option<Clock> {
    if h < 24 && m < 60 {
        Some(Clock { h, m })
    } else {
        None
    }
}

/// The Monday (Paris local) of the week containing `today`. Used as the anchor
/// when a schedule gives weekday names but no dates.
pub fn current_paris_monday() -> NaiveDate {
    let today = Utc::now().with_timezone(&Paris).date_naive();
    let back = today.weekday().num_days_from_monday() as i64;
    today - chrono::Duration::days(back)
}

fn paris_to_utc(date: NaiveDate, time: NaiveTime) -> String {
    let naive = NaiveDateTime::new(date, time);
    let dt = Paris
        .from_local_datetime(&naive)
        .single()
        .or_else(|| Paris.from_local_datetime(&naive).earliest())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|| Utc.from_utc_datetime(&naive));
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Parse the schedule `text` into draft events. `base` anchors weekday-only
/// schedules (typically the Monday of the current week).
pub fn parse_schedule(text: &str, base: NaiveDate) -> Vec<DraftEvent> {
    let mut out = Vec::new();
    let mut current: NaiveDate = base;

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        // 1) Establish the day this line refers to (explicit date wins over
        //    weekday name; both update the running context).
        let mut had_day = false;
        if let Some(d) = explicit_date(line, base) {
            current = d;
            had_day = true;
        } else if let Some(cap) = WEEKDAY_RE.captures(line) {
            if let Some(off) = weekday_offset(&cap[1]) {
                current = base + chrono::Duration::days(off);
                had_day = true;
            }
        }

        // 2) Collect clock times in order of appearance.
        let clocks = clocks_in(line);

        // 3) The title is whatever text remains after removing time/date/weekday
        //    tokens and filler words.
        let title = clean_title(line);

        if !clocks.is_empty() {
            if title.is_empty() {
                continue; // a bare time with no label is not an event
            }
            let start_c = clocks[0];
            let end_c = clocks.get(1).copied();
            let start_t = NaiveTime::from_hms_opt(start_c.h, start_c.m, 0).unwrap();
            let end_t = match end_c {
                Some(c) => NaiveTime::from_hms_opt(c.h, c.m, 0).unwrap(),
                None => NaiveTime::from_hms_opt((start_c.h + 1).min(23), start_c.m, 0).unwrap(),
            };
            let (s_date, e_date) = if end_t <= start_t && end_c.is_some() {
                // crosses midnight
                (current, current + chrono::Duration::days(1))
            } else {
                (current, current)
            };
            out.push(DraftEvent {
                title,
                start: paris_to_utc(s_date, start_t),
                end: paris_to_utc(e_date, end_t),
                all_day: false,
            });
        } else if had_day && !title.is_empty() {
            // A titled line tied to a day but without a time → all-day.
            let start_t = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
            let end_t = NaiveTime::from_hms_opt(23, 59, 0).unwrap();
            out.push(DraftEvent {
                title,
                start: paris_to_utc(current, start_t),
                end: paris_to_utc(current, end_t),
                all_day: true,
            });
        }

        if out.len() >= 300 {
            break;
        }
    }

    out
}

/// An explicit date on the line (numeric `12/09[/2026]` or `12 septembre`),
/// resolving a missing year to the one nearest `base`.
fn explicit_date(line: &str, base: NaiveDate) -> Option<NaiveDate> {
    if let Some(cap) = MONTH_DATE_RE.captures(line) {
        let day: u32 = cap[1].parse().ok()?;
        let month = month_number(&cap[2])?;
        let year = cap
            .get(3)
            .and_then(|y| y.as_str().parse::<i32>().ok())
            .unwrap_or(base.year());
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    if let Some(cap) = DATE_RE.captures(line) {
        let day: u32 = cap[1].parse().ok()?;
        let month: u32 = cap[2].parse().ok()?;
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        let year = cap
            .get(3)
            .and_then(|y| y.as_str().parse::<i32>().ok())
            .map(|y| if y < 100 { 2000 + y } else { y })
            .unwrap_or(base.year());
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    None
}

/// All valid clock times on the line, left to right.
fn clocks_in(line: &str) -> Vec<Clock> {
    let mut out = Vec::new();
    for cap in TIME_RE.captures_iter(line) {
        let h: u32 = match cap[1].parse() {
            Ok(h) => h,
            Err(_) => continue,
        };
        let m: u32 = cap.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(0);
        if let Some(c) = valid_clock(h, m) {
            out.push(c);
        }
    }
    out
}

/// Remove time / date / weekday tokens and French filler words, then tidy the
/// leftover punctuation so the remainder reads like an event title.
fn clean_title(line: &str) -> String {
    let mut s = line.to_string();
    s = MONTH_DATE_RE.replace_all(&s, " ").into_owned();
    s = DATE_RE.replace_all(&s, " ").into_owned();
    s = TIME_RE.replace_all(&s, " ").into_owned();
    s = WEEKDAY_RE.replace_all(&s, " ").into_owned();

    // Drop connective words left dangling by the removals.
    static FILLER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b(de|du|au|à|a|le|la|les|et|h)\b").unwrap());
    s = FILLER.replace_all(&s, " ").into_owned();

    // Collapse separators/whitespace and trim leading/trailing noise.
    static SEP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s\-–—:/.,;|]+").unwrap());
    s = SEP.replace_all(&s, " ").into_owned();
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, 7).unwrap() // a Monday
    }

    #[test]
    fn weekday_and_time_range() {
        let evs = parse_schedule("Lundi 9h-10h30 Mathématiques", base());
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].title, "Mathématiques");
        assert!(!evs[0].all_day);
        // 9h Paris in September (CEST, +02) → 07:00 UTC
        assert_eq!(evs[0].start, "2026-09-07T07:00:00Z");
        assert_eq!(evs[0].end, "2026-09-07T08:30:00Z");
    }

    #[test]
    fn context_day_then_times() {
        let text = "Mardi\n10:00 - 11:00 Français\n14h Sport";
        let evs = parse_schedule(text, base());
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0].title, "Français");
        assert_eq!(evs[0].start, "2026-09-08T08:00:00Z");
        assert_eq!(evs[1].title, "Sport");
        // single time → +1h
        assert_eq!(evs[1].start, "2026-09-08T12:00:00Z");
        assert_eq!(evs[1].end, "2026-09-08T13:00:00Z");
    }

    #[test]
    fn explicit_numeric_date() {
        let evs = parse_schedule("12/09/2026 9h Rendez-vous médecin", base());
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].title, "Rendez vous médecin");
        assert_eq!(evs[0].start, "2026-09-12T07:00:00Z");
    }

    #[test]
    fn month_name_all_day() {
        let evs = parse_schedule("15 octobre Réunion parents", base());
        assert_eq!(evs.len(), 1);
        assert!(evs[0].all_day);
        assert_eq!(evs[0].title, "Réunion parents");
    }

    #[test]
    fn ignores_bare_times_and_blank_lines() {
        let evs = parse_schedule("\n9h-10h\n   \nlundi", base());
        assert!(evs.is_empty());
    }
}
