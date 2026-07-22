//! Tides for French reference ports, via the WorldTides API (authoritative
//! SHOM/FES-based predictions). Only active when the WORLDTIDES_API_KEY
//! config var is set (free key at worldtides.info).
//!
//! Tides are strongly location-specific and safety-relevant (foreshore
//! walking, fishing), so rather than a hand-rolled harmonic model this uses
//! official predictions. Without a key, no tide events are produced.
//!
//! Because each API call consumes quota, the seed layer only asks for ports
//! whose stored horizon is running low (see `seed`), and the fetched window
//! is bounded by TIDE_DAYS (default 14 days).

use serde::Deserialize;

use crate::seed::SeedCandidate;

pub const TIDE_COLOR: &str = "#0277bd";

pub struct Port {
    pub key: &'static str,
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
    /// In the default selection (Atlantic beach spots — « surtout l'océan »)
    pub default_on: bool,
}

/// French coastal spots (Atlantic beaches first) and reference ports.
/// Restrictable via the TIDE_PORTS env var (comma-separated keys,
/// e.g. "biarritz,lacanau" — any key below, defaults ignored then).
pub const PORTS: [Port; 10] = [
    // Plages océanes (sélection par défaut)
    Port { key: "biarritz", name: "Biarritz", lat: 43.4832, lon: -1.5586, default_on: true },
    Port { key: "hossegor", name: "Hossegor", lat: 43.6644, lon: -1.4428, default_on: true },
    Port { key: "lacanau", name: "Lacanau-Océan", lat: 44.9992, lon: -1.2032, default_on: true },
    Port { key: "arcachon", name: "Arcachon", lat: 44.6611, lon: -1.1681, default_on: true },
    Port { key: "les-sables", name: "Les Sables-d'Olonne", lat: 46.4961, lon: -1.7950, default_on: true },
    Port { key: "la-baule", name: "La Baule", lat: 47.2780, lon: -2.3930, default_on: true },
    // Ports de référence (activables via TIDE_PORTS)
    Port { key: "brest", name: "Brest", lat: 48.3828, lon: -4.4956, default_on: false },
    Port { key: "saint-malo", name: "Saint-Malo", lat: 48.6397, lon: -2.0257, default_on: false },
    Port { key: "la-rochelle", name: "La Rochelle", lat: 46.1558, lon: -1.1517, default_on: false },
    Port { key: "nice", name: "Nice", lat: 43.6954, lon: 7.2790, default_on: false },
];

/// Ports enabled for this deployment (Atlantic beaches by default).
pub fn enabled_ports() -> Vec<&'static Port> {
    match std::env::var("TIDE_PORTS") {
        Ok(list) if !list.trim().is_empty() => {
            let keys: Vec<String> = list.split(',').map(|s| s.trim().to_lowercase()).collect();
            PORTS.iter().filter(|p| keys.iter().any(|k| k == p.key)).collect()
        }
        _ => PORTS.iter().filter(|p| p.default_on).collect(),
    }
}

/// Days of tides requested per API call (bounds quota usage).
pub fn horizon_days() -> i64 {
    std::env::var("TIDE_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|d| *d > 0 && *d <= 365)
        .unwrap_or(14)
}

#[derive(Deserialize)]
struct ExtremesResponse {
    #[serde(default)]
    extremes: Vec<Extreme>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct Extreme {
    /// Unix timestamp (seconds, UTC)
    dt: i64,
    height: f64,
    /// "High" or "Low"
    #[serde(rename = "type")]
    kind: String,
}

/// Fetch tides for the given ports. Returns an empty list when no key is set
/// or every request fails.
pub async fn fetch(ports: &[&Port], start_unix: i64) -> Vec<SeedCandidate> {
    let Ok(key) = std::env::var("WORLDTIDES_API_KEY") else {
        return Vec::new();
    };
    if ports.is_empty() {
        return Vec::new();
    }
    let base = std::env::var("WORLDTIDES_API_URL")
        .unwrap_or_else(|_| "https://www.worldtides.info/api/v3".into());
    let length = horizon_days() * 86400;
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for port in ports {
        let url = format!(
            "{base}?extremes&lat={}&lon={}&start={start_unix}&length={length}&key={key}",
            port.lat, port.lon
        );
        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                log::warn!("WorldTides returned {} for {}", r.status(), port.name);
                continue;
            }
            Err(e) => {
                log::warn!("WorldTides unreachable for {}: {e}", port.name);
                continue;
            }
        };
        let parsed: ExtremesResponse = match resp.json().await {
            Ok(p) => p,
            Err(e) => {
                log::warn!("could not parse WorldTides response for {}: {e}", port.name);
                continue;
            }
        };
        if let Some(err) = parsed.error {
            log::warn!("WorldTides error for {}: {err}", port.name);
            continue;
        }
        for ex in parsed.extremes {
            let Some(instant) = chrono::DateTime::from_timestamp(ex.dt, 0) else {
                continue;
            };
            let high = ex.kind.eq_ignore_ascii_case("high");
            let iso = instant.format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let end = (instant + chrono::Duration::minutes(10))
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string();
            out.push(SeedCandidate {
                date: instant.format("%Y-%m-%d").to_string(),
                title: format!(
                    "🌊 {} — {}",
                    port.name,
                    if high { "Pleine mer" } else { "Basse mer" }
                ),
                description: Some(format!(
                    "Marée — hauteur {:.2} m (niveau moyen)",
                    ex.height
                )),
                color: Some(TIDE_COLOR.into()),
                start: Some(iso),
                end: Some(end),
            });
        }
        log::info!("fetched tides for {}", port.name);
    }
    out
}
