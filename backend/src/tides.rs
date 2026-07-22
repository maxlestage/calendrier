//! Tides for French beaches and reference ports.
//!
//! Two sources, picked automatically:
//! - **WorldTides** (authoritative SHOM/FES-based extremes) when the
//!   WORLDTIDES_API_KEY config var is set;
//! - otherwise a **zero-config fallback**: the hourly sea-level curve from
//!   the free, keyless Open-Meteo marine API (hydrodynamic tide model),
//!   whose extremes are located by quadratic interpolation around the
//!   hourly samples (≈ minutes of timing error, flagged in the event
//!   description).
//!
//! Tides are location-specific and safety-relevant (foreshore walking,
//! fishing), which is why both paths rely on real tide models rather than a
//! hand-rolled harmonic fit.
//!
//! Because each WorldTides call consumes quota, the seed layer only asks
//! for ports whose stored horizon is running low (see `seed`), and the
//! fetched window is bounded by TIDE_DAYS (default 14 days).

use serde::Deserialize;

use crate::seed::SeedCandidate;

pub const TIDE_COLOR: &str = "#0277bd";

pub struct Port {
    pub key: &'static str,
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
    /// Coastal group, also usable as a selection token:
    /// "ocean" (Atlantique), "mer" (Méditerranée), "manche", "ports"
    pub group: &'static str,
}

/// Full catalog of French coastal spots by coast — the in-app dropdown
/// (GET /api/tide-spots) exposes all of them and the user picks.
pub static PORTS: &[Port] = &[
    // ── Océan (Atlantique), du nord au sud ─────────────────────────────
    Port { key: "carnac", name: "Carnac", lat: 47.5720, lon: -3.0780, group: "ocean" },
    Port { key: "quiberon", name: "Quiberon", lat: 47.4800, lon: -3.1200, group: "ocean" },
    Port { key: "la-baule", name: "La Baule", lat: 47.2780, lon: -2.3930, group: "ocean" },
    Port { key: "pornic", name: "Pornic", lat: 47.1130, lon: -2.1050, group: "ocean" },
    Port { key: "saint-jean-de-monts", name: "Saint-Jean-de-Monts", lat: 46.7900, lon: -2.0800, group: "ocean" },
    Port { key: "saint-gilles", name: "Saint-Gilles-Croix-de-Vie", lat: 46.6960, lon: -1.9450, group: "ocean" },
    Port { key: "les-sables", name: "Les Sables-d'Olonne", lat: 46.4961, lon: -1.7950, group: "ocean" },
    Port { key: "la-tranche", name: "La Tranche-sur-Mer", lat: 46.3430, lon: -1.4380, group: "ocean" },
    Port { key: "ile-de-re", name: "Île de Ré", lat: 46.2020, lon: -1.3660, group: "ocean" },
    Port { key: "ile-d-oleron", name: "Île d'Oléron", lat: 45.8420, lon: -1.2170, group: "ocean" },
    Port { key: "royan", name: "Royan", lat: 45.6230, lon: -1.0280, group: "ocean" },
    Port { key: "soulac", name: "Soulac-sur-Mer", lat: 45.5140, lon: -1.1250, group: "ocean" },
    Port { key: "montalivet", name: "Montalivet", lat: 45.3760, lon: -1.1520, group: "ocean" },
    Port { key: "lacanau", name: "Lacanau-Océan", lat: 44.9992, lon: -1.2032, group: "ocean" },
    Port { key: "cap-ferret", name: "Cap Ferret", lat: 44.6280, lon: -1.2480, group: "ocean" },
    Port { key: "arcachon", name: "Arcachon", lat: 44.6611, lon: -1.1681, group: "ocean" },
    Port { key: "biscarrosse", name: "Biscarrosse-Plage", lat: 44.4460, lon: -1.2530, group: "ocean" },
    Port { key: "mimizan", name: "Mimizan-Plage", lat: 44.2130, lon: -1.3000, group: "ocean" },
    Port { key: "moliets", name: "Moliets-Plage", lat: 43.8520, lon: -1.3880, group: "ocean" },
    Port { key: "seignosse", name: "Seignosse", lat: 43.6890, lon: -1.4430, group: "ocean" },
    Port { key: "hossegor", name: "Hossegor", lat: 43.6644, lon: -1.4428, group: "ocean" },
    Port { key: "capbreton", name: "Capbreton", lat: 43.6420, lon: -1.4450, group: "ocean" },
    Port { key: "anglet", name: "Anglet", lat: 43.5060, lon: -1.5450, group: "ocean" },
    Port { key: "biarritz", name: "Biarritz", lat: 43.4832, lon: -1.5586, group: "ocean" },
    Port { key: "saint-jean-de-luz", name: "Saint-Jean-de-Luz", lat: 43.3900, lon: -1.6620, group: "ocean" },
    Port { key: "hendaye", name: "Hendaye", lat: 43.3720, lon: -1.7740, group: "ocean" },
    // ── Mer (Méditerranée), d'ouest en est + Corse ─────────────────────
    Port { key: "argeles", name: "Argelès-sur-Mer", lat: 42.5460, lon: 3.0420, group: "mer" },
    Port { key: "collioure", name: "Collioure", lat: 42.5270, lon: 3.0850, group: "mer" },
    Port { key: "canet", name: "Canet-en-Roussillon", lat: 42.7050, lon: 3.0380, group: "mer" },
    Port { key: "leucate", name: "Leucate", lat: 42.9100, lon: 3.0640, group: "mer" },
    Port { key: "gruissan", name: "Gruissan", lat: 43.1080, lon: 3.0870, group: "mer" },
    Port { key: "narbonne-plage", name: "Narbonne-Plage", lat: 43.1660, lon: 3.1730, group: "mer" },
    Port { key: "valras", name: "Valras-Plage", lat: 43.2480, lon: 3.2920, group: "mer" },
    Port { key: "cap-d-agde", name: "Cap d'Agde", lat: 43.2790, lon: 3.5150, group: "mer" },
    Port { key: "sete", name: "Sète", lat: 43.4020, lon: 3.6970, group: "mer" },
    Port { key: "palavas", name: "Palavas-les-Flots", lat: 43.5250, lon: 3.9320, group: "mer" },
    Port { key: "la-grande-motte", name: "La Grande-Motte", lat: 43.5600, lon: 4.0870, group: "mer" },
    Port { key: "le-grau-du-roi", name: "Le Grau-du-Roi", lat: 43.5380, lon: 4.1360, group: "mer" },
    Port { key: "saintes-maries", name: "Saintes-Maries-de-la-Mer", lat: 43.4520, lon: 4.4290, group: "mer" },
    Port { key: "marseille", name: "Marseille", lat: 43.2600, lon: 5.3700, group: "mer" },
    Port { key: "cassis", name: "Cassis", lat: 43.2140, lon: 5.5390, group: "mer" },
    Port { key: "la-ciotat", name: "La Ciotat", lat: 43.1740, lon: 5.6060, group: "mer" },
    Port { key: "bandol", name: "Bandol", lat: 43.1350, lon: 5.7540, group: "mer" },
    Port { key: "hyeres", name: "Hyères", lat: 43.0940, lon: 6.1590, group: "mer" },
    Port { key: "le-lavandou", name: "Le Lavandou", lat: 43.1370, lon: 6.3670, group: "mer" },
    Port { key: "pampelonne", name: "Ramatuelle (Pampelonne)", lat: 43.2280, lon: 6.6630, group: "mer" },
    Port { key: "sainte-maxime", name: "Sainte-Maxime", lat: 43.3090, lon: 6.6390, group: "mer" },
    Port { key: "frejus", name: "Fréjus / Saint-Raphaël", lat: 43.4230, lon: 6.7460, group: "mer" },
    Port { key: "cannes", name: "Cannes", lat: 43.5480, lon: 7.0140, group: "mer" },
    Port { key: "antibes", name: "Antibes (Juan-les-Pins)", lat: 43.5670, lon: 7.1070, group: "mer" },
    Port { key: "nice", name: "Nice", lat: 43.6954, lon: 7.2790, group: "mer" },
    Port { key: "menton", name: "Menton", lat: 43.7740, lon: 7.5000, group: "mer" },
    Port { key: "calvi", name: "Calvi", lat: 42.5680, lon: 8.7570, group: "mer" },
    Port { key: "ajaccio", name: "Ajaccio", lat: 41.9180, lon: 8.7380, group: "mer" },
    Port { key: "porto-vecchio", name: "Porto-Vecchio (Palombaggia)", lat: 41.5580, lon: 9.2870, group: "mer" },
    // ── Manche ─────────────────────────────────────────────────────────
    Port { key: "saint-malo", name: "Saint-Malo", lat: 48.6397, lon: -2.0257, group: "manche" },
    Port { key: "deauville", name: "Deauville", lat: 49.3620, lon: 0.0750, group: "manche" },
    Port { key: "le-touquet", name: "Le Touquet", lat: 50.5240, lon: 1.5850, group: "manche" },
    Port { key: "etretat", name: "Étretat", lat: 49.7080, lon: 0.2050, group: "manche" },
    // ── Ports de référence ─────────────────────────────────────────────
    Port { key: "brest", name: "Brest", lat: 48.3828, lon: -4.4956, group: "ports" },
    Port { key: "la-rochelle", name: "La Rochelle", lat: 46.1558, lon: -1.1517, group: "ports" },
];

/// Resolve selection tokens (spot keys or whole group names) to ports.
pub fn ports_for_tokens(tokens: &[String]) -> Vec<&'static Port> {
    PORTS
        .iter()
        .filter(|p| tokens.iter().any(|t| t == p.key || t == p.group))
        .collect()
}

pub fn parse_tokens(csv: &str) -> Vec<String> {
    csv.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub const SPOTS_SETTING: &str = "tide_spots";

/// Selection tokens: the in-app choice (DB setting) wins; the TIDE_PORTS
/// env var is only a fallback for deployments that never used the UI;
/// otherwise empty (no tides until the user picks spots).
pub async fn selected_tokens(db: &sea_orm::DatabaseConnection) -> Vec<String> {
    if let Some(saved) = crate::settings::get(db, SPOTS_SETTING).await {
        return parse_tokens(&saved);
    }
    match std::env::var("TIDE_PORTS") {
        Ok(list) => parse_tokens(&list),
        Err(_) => Vec::new(),
    }
}

pub async fn selected_ports(db: &sea_orm::DatabaseConnection) -> Vec<&'static Port> {
    ports_for_tokens(&selected_tokens(db).await)
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

/// Fetch tides for the given ports: WorldTides when a key is configured,
/// otherwise the keyless Open-Meteo fallback. Empty when every request fails.
pub async fn fetch(ports: &[&Port], start_unix: i64) -> Vec<SeedCandidate> {
    if ports.is_empty() {
        return Vec::new();
    }
    match std::env::var("WORLDTIDES_API_KEY") {
        Ok(key) => fetch_worldtides(ports, start_unix, &key).await,
        Err(_) => fetch_openmeteo(ports, start_unix).await,
    }
}

async fn fetch_worldtides(ports: &[&Port], start_unix: i64, key: &str) -> Vec<SeedCandidate> {
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
            out.push(tide_candidate(port.name, instant, high, ex.height, ""));
        }
        log::info!("fetched tides for {}", port.name);
    }
    out
}

fn tide_candidate(
    port_name: &str,
    instant: chrono::DateTime<chrono::Utc>,
    high: bool,
    height: f64,
    source_note: &str,
) -> SeedCandidate {
    let iso = instant.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let end = (instant + chrono::Duration::minutes(10))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    SeedCandidate {
        date: instant.format("%Y-%m-%d").to_string(),
        title: format!(
            "🌊 {} — {}",
            port_name,
            if high { "Pleine mer" } else { "Basse mer" }
        ),
        description: Some(format!(
            "Marée — hauteur {height:.2} m (niveau moyen){source_note}"
        )),
        color: Some(TIDE_COLOR.into()),
        start: Some(iso),
        end: Some(end),
    }
}

// ---------------------------------------------------------------------------
// Keyless fallback: extremes from Open-Meteo's hourly sea-level curve

#[derive(Deserialize, Default)]
struct SeaLevelHourly {
    #[serde(default)]
    time: Vec<String>,
    #[serde(default)]
    sea_level_height_msl: Vec<Option<f64>>,
}

#[derive(Deserialize)]
struct SeaLevelResponse {
    #[serde(default)]
    hourly: Option<SeaLevelHourly>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    Many(Vec<T>),
    One(T),
}

/// Tides without any API key: fetch the hourly sea-level series (Open-Meteo
/// marine, free) for all ports in one batched call, then locate each local
/// extreme and refine its time/height with a parabola through the three
/// surrounding samples (timing error of a few minutes on a smooth ~12h25
/// tidal curve).
async fn fetch_openmeteo(ports: &[&Port], start_unix: i64) -> Vec<SeedCandidate> {
    let base = std::env::var("MARINE_API_URL")
        .unwrap_or_else(|_| "https://marine-api.open-meteo.com/v1/marine".into());
    let days = horizon_days().clamp(1, 16);
    let lat: Vec<String> = ports.iter().map(|p| p.lat.to_string()).collect();
    let lon: Vec<String> = ports.iter().map(|p| p.lon.to_string()).collect();
    let url = format!(
        "{base}?latitude={}&longitude={}&hourly=sea_level_height_msl\
         &timezone=UTC&forecast_days={days}",
        lat.join(","),
        lon.join(",")
    );
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            log::warn!("Open-Meteo marine returned {} for tides", r.status());
            return Vec::new();
        }
        Err(e) => {
            log::warn!("Open-Meteo marine unreachable for tides: {e}");
            return Vec::new();
        }
    };
    let parsed: OneOrMany<SeaLevelResponse> = match resp.json().await {
        Ok(p) => p,
        Err(e) => {
            log::warn!("could not parse Open-Meteo sea level response: {e}");
            return Vec::new();
        }
    };
    let responses = match parsed {
        OneOrMany::Many(v) => v,
        OneOrMany::One(x) => vec![x],
    };

    let mut out = Vec::new();
    for (port, resp) in ports.iter().zip(responses.iter()) {
        let Some(hourly) = &resp.hourly else { continue };
        let times = &hourly.time;
        let levels = &hourly.sea_level_height_msl;
        let n = times.len().min(levels.len());
        let mut found = 0;
        for i in 1..n.saturating_sub(1) {
            let (Some(prev), Some(cur), Some(next)) = (levels[i - 1], levels[i], levels[i + 1])
            else {
                continue;
            };
            // Local extreme (strict on one side so a flat pair counts once)
            let is_max = cur >= prev && cur > next;
            let is_min = cur <= prev && cur < next;
            if !is_max && !is_min {
                continue;
            }
            let Ok(naive) =
                chrono::NaiveDateTime::parse_from_str(&times[i], "%Y-%m-%dT%H:%M")
            else {
                continue;
            };
            // Parabola through (-1h, 0, +1h): vertex gives the refined
            // extreme time and height
            let a2 = prev + next - 2.0 * cur; // = 2a
            let b = (next - prev) / 2.0;
            let (dt_hours, height) = if a2.abs() > 1e-9 {
                let dt = (-b / a2).clamp(-1.0, 1.0);
                (dt, cur - b * b / (2.0 * a2))
            } else {
                (0.0, cur)
            };
            let instant = naive.and_utc()
                + chrono::Duration::seconds((dt_hours * 3600.0).round() as i64);
            if instant.timestamp() < start_unix {
                continue;
            }
            out.push(tide_candidate(
                port.name,
                instant,
                is_max,
                height,
                " · prévision Open-Meteo",
            ));
            found += 1;
        }
        log::info!("computed {found} tide extremes for {} (Open-Meteo)", port.name);
    }
    out
}
