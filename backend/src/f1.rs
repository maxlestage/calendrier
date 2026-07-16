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
    #[serde(rename = "Circuit")]
    circuit: Circuit,
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
    Some(
        races
            .into_iter()
            .map(|r| SeedCandidate {
                date: r.date,
                title: format!("🏎️ {}", french_name(&r.race_name)),
                description: Some(format!(
                    "F1 {year} — {}, {}",
                    r.circuit.circuit_name, r.circuit.location.locality
                )),
                color: Some(F1_COLOR.into()),
            })
            .collect(),
    )
}
