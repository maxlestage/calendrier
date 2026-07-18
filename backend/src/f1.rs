//! F1 calendar fetched at startup from the Jolpica API (Ergast successor,
//! no API key). Falls back to the static seed list when unreachable.

use serde::Deserialize;

use crate::seed::SeedCandidate;

pub const F1_COLOR: &str = "#d93025";

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(rename = "MRData")]
    mr_data: MrData,
}

#[derive(Deserialize)]
struct MrData {
    #[serde(rename = "RaceTable")]
    race_table: RaceTable,
}

#[derive(Deserialize)]
struct RaceTable {
    #[serde(rename = "Races", default)]
    races: Vec<Race>,
}

#[derive(Deserialize)]
struct Race {
    #[serde(rename = "raceName")]
    race_name: String,
    date: String,
    /// Race start, e.g. "13:00:00Z"
    time: Option<String>,
    #[serde(rename = "Circuit")]
    circuit: Circuit,
    #[serde(rename = "Sprint")]
    sprint: Option<Session>,
    #[serde(rename = "Qualifying")]
    qualifying: Option<Session>,
    #[serde(rename = "SprintQualifying")]
    sprint_qualifying: Option<Session>,
}

#[derive(Deserialize)]
struct Session {
    date: String,
    time: Option<String>,
}

/// ("YYYY-MM-DD", "HH:MM:SSZ", hours) → (startISO, endISO)
fn timed(date: &str, time: &str, duration_h: i64) -> Option<(String, String)> {
    let start = format!("{date}T{time}");
    let parsed = chrono::DateTime::parse_from_rfc3339(&start.replace('Z', "+00:00")).ok()?;
    let end = parsed + chrono::Duration::hours(duration_h);
    Some((start, end.format("%Y-%m-%dT%H:%M:%SZ").to_string()))
}

#[derive(Deserialize)]
struct Circuit {
    #[serde(rename = "circuitName")]
    circuit_name: String,
    #[serde(rename = "Location")]
    location: Location,
}

#[derive(Deserialize)]
struct Location {
    locality: String,
}

fn french_name(race_name: &str) -> String {
    let key = race_name.trim_end_matches(" Grand Prix");
    let fr = match key {
        "Australian" => "GP d'Australie",
        "Chinese" => "GP de Chine",
        "Japanese" => "GP du Japon",
        "Bahrain" => "GP de Bahreïn",
        "Saudi Arabian" => "GP d'Arabie saoudite",
        "Miami" => "GP de Miami",
        "Emilia Romagna" => "GP d'Émilie-Romagne",
        "Monaco" => "GP de Monaco",
        "Canadian" => "GP du Canada",
        "Spanish" => "GP d'Espagne",
        "Austrian" => "GP d'Autriche",
        "British" => "GP de Grande-Bretagne",
        "Belgian" => "GP de Belgique",
        "Hungarian" => "GP de Hongrie",
        "Dutch" => "GP des Pays-Bas",
        "Italian" => "GP d'Italie",
        "Azerbaijan" => "GP d'Azerbaïdjan",
        "Singapore" => "GP de Singapour",
        "United States" => "GP des États-Unis",
        "Mexico City" | "Mexican" => "GP du Mexique",
        "São Paulo" | "Brazilian" => "GP de São Paulo",
        "Las Vegas" => "GP de Las Vegas",
        "Qatar" => "GP du Qatar",
        "Abu Dhabi" => "GP d'Abu Dhabi",
        "Madrid" => "GP de Madrid",
        "French" => "GP de France",
        "German" => "GP d'Allemagne",
        "Portuguese" => "GP du Portugal",
        _ => return format!("GP — {race_name}"),
    };
    fr.to_string()
}

/// Race calendar for a season, or None when the API is unreachable or has
/// no data for that year yet.
pub async fn fetch(year: i32) -> Option<Vec<SeedCandidate>> {
    let base =
        std::env::var("F1_API_URL").unwrap_or_else(|_| "https://api.jolpi.ca/ergast/f1".into());
    let url = format!("{base}/{year}/races.json");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            log::warn!("F1 API returned {} for {year}", r.status());
            return None;
        }
        Err(e) => {
            log::warn!("F1 API unreachable ({e}), falling back to embedded calendar");
            return None;
        }
    };
    let parsed: ApiResponse = match resp.json().await {
        Ok(p) => p,
        Err(e) => {
            log::warn!("could not parse F1 API response: {e}");
            return None;
        }
    };
    let races = parsed.mr_data.race_table.races;
    if races.is_empty() {
        return None;
    }
    log::info!("fetched {} F1 races for {year}", races.len());
    let mut events = Vec::new();
    for r in races {
        let name = french_name(&r.race_name);
        // Grand Prix race, with its exact start time when the API has it
        let (start, end) = match r.time.as_deref().and_then(|t| timed(&r.date, t, 2)) {
            Some((s, e)) => (Some(s), Some(e)),
            None => (None, None),
        };
        events.push(SeedCandidate {
            date: r.date.clone(),
            title: format!("🏎️ {name}"),
            description: Some(format!(
                "F1 {year} — {}, {}",
                r.circuit.circuit_name, r.circuit.location.locality
            )),
            color: Some(F1_COLOR.into()),
            start,
            end,
        });
        // Secondary sessions of the weekend, when the API has them
        let sessions: [(Option<Session>, String, String); 3] = [
            (
                r.qualifying,
                format!("🏎️ Qualifs — {name}"),
                format!("F1 {year} — qualifications, {}", r.circuit.circuit_name),
            ),
            (
                r.sprint,
                format!("🏎️ Sprint — {name}"),
                format!("F1 {year} — course sprint, {}", r.circuit.circuit_name),
            ),
            (
                r.sprint_qualifying,
                format!("🏎️ Qualifs sprint — {name}"),
                format!(
                    "F1 {year} — qualifications sprint, {}",
                    r.circuit.circuit_name
                ),
            ),
        ];
        for (session, title, description) in sessions {
            let Some(session) = session else { continue };
            let (start, end) = match session.time.as_deref().and_then(|t| timed(&session.date, t, 1)) {
                Some((s, e)) => (Some(s), Some(e)),
                None => (None, None),
            };
            events.push(SeedCandidate {
                date: session.date,
                title,
                description: Some(description),
                color: Some(F1_COLOR.into()),
                start,
                end,
            });
        }
    }
    Some(events)
}
