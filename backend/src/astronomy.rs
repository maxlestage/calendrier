//! Yearly computed astronomy events: solar and lunar eclipses (Meeus ch. 54),
//! equinoxes and solstices (Meeus ch. 27), and major meteor shower peaks.

use chrono::{Datelike, Timelike};
use chrono_tz::Europe::Paris;

use crate::astro::{cos_deg, iso_utc, jde_to_utc, lunations_of_year, phase_args, sin_deg};
use crate::seed::SeedCandidate;

pub const ASTRONOMY_COLOR: &str = "#1a237e";

pub enum EclipseKind {
    SolarTotal,
    SolarAnnular,
    SolarHybrid,
    SolarPartial,
    LunarTotal,
    LunarPartial,
    LunarPenumbral,
}

impl EclipseKind {
    fn title(&self) -> &'static str {
        match self {
            EclipseKind::SolarTotal => "☀️ Éclipse solaire totale",
            EclipseKind::SolarAnnular => "☀️ Éclipse solaire annulaire",
            EclipseKind::SolarHybrid => "☀️ Éclipse solaire hybride",
            EclipseKind::SolarPartial => "☀️ Éclipse solaire partielle",
            EclipseKind::LunarTotal => "🌙 Éclipse totale de Lune",
            EclipseKind::LunarPartial => "🌙 Éclipse partielle de Lune",
            EclipseKind::LunarPenumbral => "🌙 Éclipse de Lune par la pénombre",
        }
    }
}

/// Eclipse at lunation k (solar at new moon, lunar at full moon), if any.
/// Meeus "Astronomical Algorithms" ch. 54.
fn eclipse_at(k: f64, lunar: bool) -> Option<EclipseKind> {
    let a = phase_args(k, lunar);
    // No eclipse when the Moon is too far from a node
    if sin_deg(a.f).abs() > 0.36 {
        return None;
    }
    let f1 = a.f - 0.02665 * sin_deg(a.om);
    let p = 0.2070 * a.e * sin_deg(a.m) + 0.0024 * a.e * sin_deg(2.0 * a.m)
        - 0.0392 * sin_deg(a.mp)
        + 0.0116 * sin_deg(2.0 * a.mp)
        - 0.0073 * a.e * sin_deg(a.mp + a.m)
        + 0.0067 * a.e * sin_deg(a.mp - a.m)
        + 0.0118 * sin_deg(2.0 * f1);
    let q = 5.2207 - 0.0048 * a.e * cos_deg(a.m) + 0.0020 * a.e * cos_deg(2.0 * a.m)
        - 0.3299 * cos_deg(a.mp)
        - 0.0060 * a.e * cos_deg(a.mp + a.m)
        + 0.0041 * a.e * cos_deg(a.mp - a.m);
    let w = cos_deg(f1).abs();
    let gamma = (p * cos_deg(f1) + q * sin_deg(f1)) * (1.0 - 0.0048 * w);
    let u = 0.0059 + 0.0046 * a.e * cos_deg(a.m) - 0.0182 * cos_deg(a.mp)
        + 0.0004 * cos_deg(2.0 * a.mp)
        - 0.0005 * cos_deg(a.m + a.mp);
    let g = gamma.abs();

    if lunar {
        let umbral = (1.0128 - u - g) / 0.5450;
        let penumbral = (1.5573 + u - g) / 0.5450;
        if umbral >= 1.0 {
            Some(EclipseKind::LunarTotal)
        } else if umbral > 0.0 {
            Some(EclipseKind::LunarPartial)
        } else if penumbral > 0.0 {
            Some(EclipseKind::LunarPenumbral)
        } else {
            None
        }
    } else {
        if g > 1.5433 + u {
            return None;
        }
        if g < 0.9972 {
            // Central eclipse
            if u < 0.0 {
                Some(EclipseKind::SolarTotal)
            } else if u > 0.0047 {
                Some(EclipseKind::SolarAnnular)
            } else {
                Some(EclipseKind::SolarHybrid)
            }
        } else {
            Some(EclipseKind::SolarPartial)
        }
    }
}

/// All eclipses whose Paris date falls in `year`.
pub fn eclipses(year: i32) -> Vec<SeedCandidate> {
    let mut out = Vec::new();
    for k in lunations_of_year(year) {
        for (offset, lunar) in [(0.0, false), (0.5, true)] {
            let Some(kind) = eclipse_at(k + offset, lunar) else {
                continue;
            };
            let utc = jde_to_utc(phase_args(k + offset, lunar).jde);
            let paris = utc.with_timezone(&Paris);
            if paris.year() != year {
                continue;
            }
            out.push(SeedCandidate {
                date: format!("{}-{:02}-{:02}", paris.year(), paris.month(), paris.day()),
                title: kind.title().to_string(),
                description: Some(format!(
                    "Astronomie — maximum vers {:02}h{:02} (heure de Paris)",
                    paris.hour(),
                    paris.minute()
                )),
                color: Some(ASTRONOMY_COLOR.into()),
                start: Some(iso_utc(utc - chrono::Duration::hours(1))),
                end: Some(iso_utc(utc + chrono::Duration::hours(1))),
            });
        }
    }
    out
}

/// Equinox/solstice instants (Meeus ch. 27, years 1000–3000).
/// Returns (month hint, JDE) for the four events of the year.
fn solstice_equinox_jde(year: i32) -> [(u32, f64); 4] {
    let y = (year as f64 - 2000.0) / 1000.0;
    let mean = [
        (3, 2451623.80984 + 365242.37404 * y + 0.05169 * y * y - 0.00411 * y * y * y - 0.00057 * y * y * y * y),
        (6, 2451716.56767 + 365241.62603 * y + 0.00325 * y * y + 0.00888 * y * y * y - 0.00030 * y * y * y * y),
        (9, 2451810.21715 + 365242.01767 * y - 0.11575 * y * y + 0.00337 * y * y * y + 0.00078 * y * y * y * y),
        (12, 2451900.05952 + 365242.74049 * y - 0.06223 * y * y - 0.00823 * y * y * y + 0.00032 * y * y * y * y),
    ];
    // Periodic terms (A, B, C): S = Σ A·cos(B + C·T)
    const TERMS: [(f64, f64, f64); 24] = [
        (485.0, 324.96, 1934.136),
        (203.0, 337.23, 32964.467),
        (199.0, 342.08, 20.186),
        (182.0, 27.85, 445267.112),
        (156.0, 73.14, 45036.886),
        (136.0, 171.52, 22518.443),
        (77.0, 222.54, 65928.934),
        (74.0, 296.72, 3034.906),
        (70.0, 243.58, 9037.513),
        (58.0, 119.81, 33718.147),
        (52.0, 297.17, 150.678),
        (50.0, 21.02, 2281.226),
        (45.0, 247.54, 29929.562),
        (44.0, 325.15, 31555.956),
        (29.0, 60.93, 4443.417),
        (18.0, 155.12, 67555.328),
        (17.0, 288.79, 4562.452),
        (16.0, 198.04, 62894.029),
        (14.0, 199.76, 31436.921),
        (12.0, 95.39, 14577.848),
        (12.0, 287.11, 31931.756),
        (12.0, 320.81, 34777.259),
        (9.0, 227.73, 1222.114),
        (8.0, 15.45, 16859.074),
    ];
    mean.map(|(month, jde0)| {
        let t = (jde0 - 2451545.0) / 36525.0;
        let w = 35999.373 * t - 2.47;
        let dl = 1.0 + 0.0334 * cos_deg(w) + 0.0007 * cos_deg(2.0 * w);
        let s: f64 = TERMS.iter().map(|(a, b, c)| a * cos_deg(b + c * t)).sum();
        (month, jde0 + 0.00001 * s / dl)
    })
}

pub fn solstices_equinoxes(year: i32) -> Vec<SeedCandidate> {
    solstice_equinox_jde(year)
        .into_iter()
        .map(|(month, jde)| {
            let utc = jde_to_utc(jde);
            let paris = utc.with_timezone(&Paris);
            let title = match month {
                3 => "🌍 Équinoxe de printemps",
                6 => "🌍 Solstice d'été",
                9 => "🌍 Équinoxe d'automne",
                _ => "🌍 Solstice d'hiver",
            };
            SeedCandidate {
                date: format!("{}-{:02}-{:02}", paris.year(), paris.month(), paris.day()),
                title: title.to_string(),
                description: Some(format!(
                    "Astronomie — instant exact : {:02}h{:02} (heure de Paris)",
                    paris.hour(),
                    paris.minute()
                )),
                color: Some(ASTRONOMY_COLOR.into()),
                start: Some(iso_utc(utc)),
                end: Some(iso_utc(utc + chrono::Duration::hours(1))),
            }
        })
        .collect()
}

/// Major meteor shower peaks (conventional dates, all-day events).
pub fn meteor_showers(year: i32) -> Vec<SeedCandidate> {
    const SHOWERS: [(u32, u32, &str, &str); 9] = [
        (1, 3, "☄️ Quadrantides (pic)", "jusqu'à ~120 météores/h, nuit du 3 au 4 janvier"),
        (4, 22, "☄️ Lyrides (pic)", "~18 météores/h, nuit du 22 au 23 avril"),
        (5, 6, "☄️ Êta Aquarides (pic)", "~50 météores/h, débris de la comète de Halley"),
        (7, 30, "☄️ Delta Aquarides (pic)", "~25 météores/h, nuit du 30 au 31 juillet"),
        (8, 12, "☄️ Perséides (pic)", "jusqu'à ~100 météores/h, nuit du 12 au 13 août — le grand rendez-vous de l'été"),
        (10, 21, "☄️ Orionides (pic)", "~20 météores/h, débris de la comète de Halley"),
        (11, 17, "☄️ Léonides (pic)", "~15 météores/h, nuit du 17 au 18 novembre"),
        (12, 14, "☄️ Géminides (pic)", "jusqu'à ~150 météores/h, le plus actif de l'année"),
        (12, 22, "☄️ Ursides (pic)", "~10 météores/h, nuit du 22 au 23 décembre"),
    ];
    SHOWERS
        .iter()
        .map(|(m, d, title, desc)| SeedCandidate {
            date: format!("{year}-{m:02}-{d:02}"),
            title: title.to_string(),
            description: Some(format!("Astronomie — {desc}")),
            color: Some(ASTRONOMY_COLOR.into()),
            start: None,
            end: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eclipses_2026_match_nasa_catalog() {
        let events = eclipses(2026);
        let mut summary: Vec<(String, String)> = events
            .iter()
            .map(|e| (e.date.clone(), e.title.clone()))
            .collect();
        summary.sort();
        assert_eq!(
            summary,
            vec![
                ("2026-02-17".into(), "☀️ Éclipse solaire annulaire".into()),
                ("2026-03-03".into(), "🌙 Éclipse totale de Lune".into()),
                ("2026-08-12".into(), "☀️ Éclipse solaire totale".into()),
                ("2026-08-28".into(), "🌙 Éclipse partielle de Lune".into()),
            ]
        );
    }

    #[test]
    fn total_solar_eclipse_2027_found() {
        // The famous long total eclipse over Luxor on 2027-08-02
        let events = eclipses(2027);
        assert!(events
            .iter()
            .any(|e| e.date == "2027-08-02" && e.title == "☀️ Éclipse solaire totale"));
    }

    #[test]
    fn solstices_equinoxes_2026_match_ephemerides() {
        // Reference UTC instants (timeanddate / IMCCE):
        // (month, day, hour, minute)
        let reference = [(3u32, 20u32, 14i64, 46i64), (6, 21, 8, 25), (9, 23, 0, 5), (12, 21, 20, 50)];
        let events = solstices_equinoxes(2026);
        assert_eq!(events.len(), 4);
        for ((month, day, hour, minute), ev) in reference.iter().zip(&events) {
            let start = ev.start.as_deref().expect("timed event");
            let dt = chrono::DateTime::parse_from_rfc3339(&start.replace('Z', "+00:00")).unwrap();
            assert_eq!((dt.month(), dt.day()), (*month, *day), "{}", ev.title);
            let computed = dt.hour() as i64 * 60 + dt.minute() as i64;
            let expected = hour * 60 + minute;
            assert!(
                (computed - expected).abs() <= 30,
                "{}: computed {}:{:02} UTC, expected {}:{:02} UTC",
                ev.title,
                dt.hour(),
                dt.minute(),
                hour,
                minute
            );
        }
    }
}
