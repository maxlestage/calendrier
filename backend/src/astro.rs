//! Yearly computed astrology events: zodiac seasons, moon phases (Meeus'
//! algorithm, "Astronomical Algorithms" ch. 49) and French fireworks dates.
//! All civil days are expressed for France (Europe/Paris); stored instants
//! are UTC ISO 8601.

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use chrono_tz::Europe::Paris;
use chrono_tz::Tz;

use crate::seed::SeedCandidate;

pub const ASTRO_COLOR: &str = "#0aa3a3";
pub const FIREWORKS_COLOR: &str = "#f4a300";

/// (start month, start day, symbol, sign name, "de la saison …" article+name)
const SEASONS: [(u32, u32, &str, &str, &str); 12] = [
    (1, 20, "♒", "Verseau", "du Verseau"),
    (2, 19, "♓", "Poissons", "des Poissons"),
    (3, 21, "♈", "Bélier", "du Bélier"),
    (4, 20, "♉", "Taureau", "du Taureau"),
    (5, 21, "♊", "Gémeaux", "des Gémeaux"),
    (6, 21, "♋", "Cancer", "du Cancer"),
    (7, 23, "♌", "Lion", "du Lion"),
    (8, 23, "♍", "Vierge", "de la Vierge"),
    (9, 23, "♎", "Balance", "de la Balance"),
    (10, 23, "♏", "Scorpion", "du Scorpion"),
    (11, 22, "♐", "Sagittaire", "du Sagittaire"),
    (12, 21, "♑", "Capricorne", "du Capricorne"),
];

/// Index in SEASONS of the sun sign on a given (month, day).
fn sun_sign_index(month: u32, day: u32) -> usize {
    let mut idx = 11; // before Jan 20 the sun is still in Capricorne
    for (i, (m, d, ..)) in SEASONS.iter().enumerate() {
        if (month, day) >= (*m, *d) {
            idx = i;
        }
    }
    idx
}

pub(crate) fn sin_deg(x: f64) -> f64 {
    x.to_radians().sin()
}

pub(crate) fn cos_deg(x: f64) -> f64 {
    x.to_radians().cos()
}

/// UTC instant → "YYYY-MM-DDTHH:MM:SSZ"
pub(crate) fn iso_utc(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Paris local time → UTC ISO string (DST handled by chrono-tz).
pub(crate) fn paris_to_utc_iso(year: i32, month: u32, day: u32, hour: u32, min: u32) -> String {
    let local = Paris
        .with_ymd_and_hms(year, month, day, hour, min, 0)
        .single()
        .or_else(|| Paris.with_ymd_and_hms(year, month, day, hour, min, 0).earliest())
        .expect("valid Paris local time");
    iso_utc(local.with_timezone(&Utc))
}

pub(crate) fn jde_to_utc(jde: f64) -> DateTime<Utc> {
    let unix = (jde - 2440587.5) * 86400.0;
    Utc.timestamp_opt(unix as i64, 0).unwrap()
}

/// Fundamental arguments of lunation k (Meeus ch. 49): corrected JDE of the
/// phase plus the angles needed by the eclipse computation (ch. 54).
/// Integer k = new moon, k + 0.5 = full moon, k = 0 near 2000-01-06.
pub(crate) struct PhaseArgs {
    pub jde: f64,
    pub t: f64,
    pub e: f64,
    /// Sun mean anomaly (deg)
    pub m: f64,
    /// Moon mean anomaly (deg)
    pub mp: f64,
    /// Argument of latitude (deg)
    pub f: f64,
    /// Longitude of ascending node (deg)
    pub om: f64,
}

pub(crate) fn phase_args(k: f64, full: bool) -> PhaseArgs {
    let t = k / 1236.85;
    let mean_jde = 2451550.09766
        + 29.530588861 * k
        + t * t * (0.00015437 + t * (-0.000000150 + t * 0.00000000073));
    let e = 1.0 - t * (0.002516 + t * 0.0000074);
    let m = 2.5534 + 29.10535670 * k - t * t * (0.0000014 + t * 0.00000011);
    let mp = 201.5643 + 385.81693528 * k + t * t * (0.0107582 + t * (0.00001238 - t * 0.000000058));
    let f = 160.7108 + 390.67050284 * k - t * t * (0.0016118 + t * (0.00000227 - t * 0.000000011));
    let om = 124.7746 - 1.56375588 * k + t * t * (0.0020672 + t * 0.00000215);

    let (c1, c2, c3, c4, c5, c6, c7) = if full {
        (-0.40614, 0.17302, 0.01614, 0.01043, 0.00734, -0.00515, 0.00209)
    } else {
        (-0.40720, 0.17241, 0.01608, 0.01039, 0.00739, -0.00514, 0.00208)
    };
    let corr = c1 * sin_deg(mp)
        + c2 * e * sin_deg(m)
        + c3 * sin_deg(2.0 * mp)
        + c4 * sin_deg(2.0 * f)
        + c5 * e * sin_deg(mp - m)
        + c6 * e * sin_deg(mp + m)
        + c7 * e * e * sin_deg(2.0 * m)
        - 0.00111 * sin_deg(mp - 2.0 * f)
        - 0.00057 * sin_deg(mp + 2.0 * f)
        + 0.00056 * e * sin_deg(2.0 * mp + m)
        - 0.00042 * sin_deg(3.0 * mp)
        + 0.00042 * e * sin_deg(m + 2.0 * f)
        + 0.00038 * e * sin_deg(m - 2.0 * f)
        - 0.00024 * e * sin_deg(2.0 * mp - m)
        - 0.00017 * sin_deg(om)
        - 0.00007 * sin_deg(mp + 2.0 * m);
    PhaseArgs {
        jde: mean_jde + corr,
        t,
        e,
        m,
        mp,
        f,
        om,
    }
}

/// Lunation numbers whose phases may fall within `year` (with margin).
pub(crate) fn lunations_of_year(year: i32) -> impl Iterator<Item = f64> {
    let k0 = ((year as f64) - 2000.0) * 12.3685;
    let start = k0.floor() as i64 - 2;
    (start..start + 18).map(|k| k as f64)
}

/// All new (false) and full (true) moons whose Paris date falls in `year`.
pub fn phases_of_year(year: i32) -> Vec<(DateTime<Tz>, bool)> {
    let mut out = Vec::new();
    for k in lunations_of_year(year) {
        for (offset, full) in [(0.0, false), (0.5, true)] {
            let paris = jde_to_utc(phase_args(k + offset, full).jde).with_timezone(&Paris);
            if paris.year() == year {
                out.push((paris, full));
            }
        }
    }
    out.sort_by_key(|(dt, _)| *dt);
    out
}

pub fn seasons(year: i32) -> Vec<SeedCandidate> {
    SEASONS
        .iter()
        .map(|(m, d, symbol, name, article)| SeedCandidate {
            all_day: None,
            date: format!("{year}-{m:02}-{d:02}"),
            title: format!("{symbol} Début de la saison {article}"),
            description: Some(format!("Astrologie — le Soleil entre en {name}")),
            color: Some(ASTRO_COLOR.into()),
            start: None,
            end: None,
        })
        .collect()
}

pub fn fireworks(year: i32) -> Vec<SeedCandidate> {
    let mk = |month: u32, day: u32, h1: u32, m1: u32, h2: u32, m2: u32, title: &str, desc: &str| {
        SeedCandidate {
            all_day: None,
            date: format!("{year}-{month:02}-{day:02}"),
            title: title.into(),
            description: Some(desc.into()),
            color: Some(FIREWORKS_COLOR.into()),
            start: Some(paris_to_utc_iso(year, month, day, h1, m1)),
            end: Some(paris_to_utc_iso(year, month, day, h2, m2)),
        }
    };
    vec![
        mk(
            7, 14, 22, 30, 23, 45,
            "🎆 Feu d'artifice du 14 Juillet",
            "Fête nationale — vers 22h30-23h dans la plupart des villes (souvent aussi le 13 au soir)",
        ),
        mk(
            8, 15, 22, 0, 23, 30,
            "🎆 Feux d'artifice du 15 août",
            "Assomption — feux dans de nombreuses communes en soirée",
        ),
        mk(
            12, 31, 23, 0, 23, 59,
            "🎆 Feux d'artifice de la Saint-Sylvestre",
            "Nouvel An — à minuit",
        ),
    ]
}

pub fn moon_phases(year: i32) -> Vec<SeedCandidate> {
    phases_of_year(year)
        .into_iter()
        .map(|(dt, full)| {
            let sun = sun_sign_index(dt.month(), dt.day());
            // At full moon the Moon sits opposite the Sun, hence the
            // opposite zodiac sign; at new moon they are conjunct.
            let sign = if full { (sun + 6) % 12 } else { sun };
            let (.., name, _) = SEASONS[sign];
            let utc = dt.with_timezone(&Utc);
            SeedCandidate {
                all_day: None,
                date: format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day()),
                title: if full {
                    format!("🌕 Pleine lune en {name}")
                } else {
                    format!("🌑 Nouvelle lune en {name}")
                },
                description: Some(format!(
                    "Astrologie — instant exact : {:02}h{:02} (heure de Paris)",
                    dt.hour(),
                    dt.minute()
                )),
                color: Some(ASTRO_COLOR.into()),
                start: Some(iso_utc(utc)),
                end: Some(iso_utc(utc + chrono::Duration::hours(1))),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference dates for H2 2026, Paris time: full moons from
    /// pleine-lune.org / timeanddate, new moons anchored on the 2026 solar
    /// eclipses (a solar eclipse happens exactly at new moon — NASA gives
    /// 2026-08-12 17:37 UTC for the total eclipse over Spain).
    /// (month, day, hour, minute, is_full)
    const REFERENCE_2026_H2: [(u32, u32, u32, u32, bool); 12] = [
        (7, 14, 11, 44, false),
        (7, 29, 16, 35, true),
        (8, 12, 19, 37, false),
        (8, 28, 6, 18, true),
        (9, 11, 5, 27, false),
        (9, 26, 18, 49, true),
        (10, 10, 17, 50, false),
        (10, 26, 5, 11, true),
        (11, 9, 8, 2, false),
        (11, 24, 15, 53, true),
        (12, 9, 1, 52, false),
        (12, 24, 2, 28, true),
    ];

    #[test]
    fn moon_phases_2026_match_reference() {
        let phases = phases_of_year(2026);
        for (month, day, hour, minute, full) in REFERENCE_2026_H2 {
            let found = phases.iter().find(|(dt, f)| {
                *f == full && dt.month() == month && dt.day() == day
            });
            let (dt, _) = found.unwrap_or_else(|| {
                panic!("no {} found on 2026-{month:02}-{day:02}", if full { "full moon" } else { "new moon" })
            });
            let computed = dt.hour() as i64 * 60 + dt.minute() as i64;
            let expected = hour as i64 * 60 + minute as i64;
            assert!(
                (computed - expected).abs() <= 20,
                "2026-{month:02}-{day:02}: computed {}:{:02}, expected {hour}:{minute:02}",
                dt.hour(),
                dt.minute()
            );
        }
    }

    #[test]
    fn moon_signs_match_reference() {
        let events = moon_phases(2026);
        let find = |date: &str| {
            events
                .iter()
                .find(|e| e.date == date)
                .unwrap_or_else(|| panic!("no event on {date}"))
                .title
                .clone()
        };
        assert_eq!(find("2026-07-14"), "🌑 Nouvelle lune en Cancer");
        assert_eq!(find("2026-07-29"), "🌕 Pleine lune en Verseau");
        assert_eq!(find("2026-08-12"), "🌑 Nouvelle lune en Lion");
        assert_eq!(find("2026-09-26"), "🌕 Pleine lune en Bélier");
        assert_eq!(find("2026-11-24"), "🌕 Pleine lune en Gémeaux");
        assert_eq!(find("2026-12-24"), "🌕 Pleine lune en Cancer");
    }

    #[test]
    fn season_titles_match_static_seed() {
        let events = seasons(2026);
        assert!(events
            .iter()
            .any(|e| e.date == "2026-07-23" && e.title == "♌ Début de la saison du Lion"));
        assert!(events
            .iter()
            .any(|e| e.date == "2026-08-23" && e.title == "♍ Début de la saison de la Vierge"));
    }

    #[test]
    fn fireworks_are_timed_in_paris_evening() {
        let events = fireworks(2026);
        let july = events.iter().find(|e| e.date == "2026-07-14").unwrap();
        // 22:30 Paris in July (UTC+2) = 20:30 UTC
        assert_eq!(july.start.as_deref(), Some("2026-07-14T20:30:00Z"));
        let nye = events.iter().find(|e| e.date == "2026-12-31").unwrap();
        // 23:00 Paris in December (UTC+1) = 22:00 UTC
        assert_eq!(nye.start.as_deref(), Some("2026-12-31T22:00:00Z"));
    }
}
