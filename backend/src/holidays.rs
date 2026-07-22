//! French public holidays (computed — Easter via the anonymous Gregorian
//! algorithm) and school vacations (official open-data API from the
//! Éducation nationale, per zone A/B/C, free and keyless).

use serde::Deserialize;

use crate::seed::SeedCandidate;

/// Green used by holidays + school vacations (distinct from the user palette).
pub const HOLIDAY_COLOR: &str = "#2e7d32";

pub const ZONE_SETTING: &str = "school_zone";

/// The selected school zone ("a" | "b" | "c"), if any.
pub async fn selected_zone(db: &sea_orm::DatabaseConnection) -> Option<String> {
    let z = crate::settings::get(db, ZONE_SETTING).await?;
    let z = z.trim().to_lowercase();
    if ["a", "b", "c"].contains(&z.as_str()) {
        Some(z)
    } else {
        None
    }
}

/// Easter Sunday (Gregorian) — anonymous algorithm (Meeus ch. 8).
fn easter(year: i32) -> (u32, u32) {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    (month as u32, day as u32)
}

fn holiday(date: chrono::NaiveDate, title: &str) -> SeedCandidate {
    SeedCandidate {
        date: date.format("%Y-%m-%d").to_string(),
        title: title.to_string(),
        description: Some("Jour férié en France".into()),
        color: Some(HOLIDAY_COLOR.into()),
        start: None,
        end: None,
        all_day: None,
    }
}

/// The 11 French public holidays of a year (all-day events).
pub fn public_holidays(year: i32) -> Vec<SeedCandidate> {
    let d = |m: u32, day: u32| chrono::NaiveDate::from_ymd_opt(year, m, day).unwrap();
    let (em, ed) = easter(year);
    let easter_sunday = d(em, ed);
    let days = chrono::Days::new;
    vec![
        holiday(d(1, 1), "🇫🇷 Jour de l'an"),
        holiday(easter_sunday + days(1), "🇫🇷 Lundi de Pâques"),
        holiday(d(5, 1), "🇫🇷 Fête du Travail"),
        holiday(d(5, 8), "🇫🇷 Victoire 1945"),
        holiday(easter_sunday + days(39), "🇫🇷 Ascension"),
        holiday(easter_sunday + days(50), "🇫🇷 Lundi de Pentecôte"),
        holiday(d(7, 14), "🇫🇷 Fête nationale"),
        holiday(d(8, 15), "🇫🇷 Assomption"),
        holiday(d(11, 1), "🇫🇷 Toussaint"),
        holiday(d(11, 11), "🇫🇷 Armistice 1918"),
        holiday(d(12, 25), "🇫🇷 Noël"),
    ]
}

// ---------------------------------------------------------------------------
// School vacations (data.education.gouv.fr, dataset fr-en-calendrier-scolaire)

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(default)]
    records: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    fields: Fields,
}

#[derive(Deserialize)]
struct Fields {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    end_date: Option<String>,
    #[serde(default)]
    population: Option<String>,
}

/// Fetch the vacation periods of one zone for the school years overlapping
/// `year` and `year + 1`, deduplicated (the API repeats each period once per
/// académie of the zone). Periods become all-day events spanning the whole
/// vacation.
pub async fn school_vacations(zone: &str, year: i32) -> Vec<SeedCandidate> {
    let base = std::env::var("SCHOOL_API_URL").unwrap_or_else(|_| {
        "https://data.education.gouv.fr/api/records/1.0/search/".into()
    });
    let zone_label = format!("Zone {}", zone.to_uppercase());
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    // A civil year overlaps two school years; the seed also wants year+1
    for start_year in [year - 1, year, year + 1] {
        let school_year = format!("{start_year}-{}", start_year + 1);
        let url = format!(
            "{base}?dataset=fr-en-calendrier-scolaire&refine.zones={}&refine.annee_scolaire={school_year}&rows=100",
            zone_label.replace(' ', "+")
        );
        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                log::warn!("school calendar API returned {} for {school_year}", r.status());
                continue;
            }
            Err(e) => {
                log::warn!("school calendar API unreachable: {e}");
                continue;
            }
        };
        let parsed: ApiResponse = match resp.json().await {
            Ok(p) => p,
            Err(e) => {
                log::warn!("could not parse school calendar response: {e}");
                continue;
            }
        };
        for rec in parsed.records {
            let f = rec.fields;
            let (Some(desc), Some(start), Some(end)) = (f.description, f.start_date, f.end_date)
            else {
                continue;
            };
            // Teacher-only entries (pré-rentrée) don't belong in a personal
            // calendar; "-" means everyone.
            if matches!(f.population.as_deref(), Some("Enseignants")) {
                continue;
            }
            let dedup = format!("{school_year}|{desc}");
            if !seen.insert(dedup) {
                continue;
            }
            let (Ok(start_dt), Ok(end_dt)) = (
                chrono::DateTime::parse_from_rfc3339(&start),
                chrono::DateTime::parse_from_rfc3339(&end),
            ) else {
                continue;
            };
            let start_utc = start_dt.with_timezone(&chrono::Utc);
            let mut end_utc = end_dt.with_timezone(&chrono::Utc);
            // The API's end_date is the return-to-school midnight; pull it
            // back a few hours so the event's last covered day is the last
            // actual vacation day. Single-day markers (start == end, e.g.
            // « Début des Vacances d'Été ») get a same-day span instead.
            if end_utc <= start_utc + chrono::Duration::hours(4) {
                end_utc = start_utc + chrono::Duration::hours(20);
            } else {
                end_utc -= chrono::Duration::hours(4);
            }
            out.push(SeedCandidate {
                date: start_utc
                    .with_timezone(&chrono_tz::Europe::Paris)
                    .format("%Y-%m-%d")
                    .to_string(),
                title: format!("🎒 {desc} ({zone_label})"),
                description: Some(format!("Vacances scolaires {school_year} — {zone_label}")),
                color: Some(HOLIDAY_COLOR.into()),
                start: Some(start_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                end: Some(end_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                all_day: Some(true),
            });
        }
    }
    log::info!("fetched {} school vacation periods for {zone_label}", out.len());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easter_dates() {
        assert_eq!(easter(2024), (3, 31));
        assert_eq!(easter(2025), (4, 20));
        assert_eq!(easter(2026), (4, 5));
        assert_eq!(easter(2027), (3, 28));
        assert_eq!(easter(2038), (4, 25));
    }

    #[test]
    fn holidays_2026() {
        let hs = public_holidays(2026);
        assert_eq!(hs.len(), 11);
        let find = |t: &str| hs.iter().find(|h| h.title.contains(t)).unwrap().date.clone();
        assert_eq!(find("Pâques"), "2026-04-06");
        assert_eq!(find("Ascension"), "2026-05-14");
        assert_eq!(find("Pentecôte"), "2026-05-25");
    }
}
