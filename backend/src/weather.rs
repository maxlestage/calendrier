//! Beach weather for the selected tide spots, via the Open-Meteo APIs
//! (free, no key): daily forecast (temperature, wind, UV, rain) from
//! api.open-meteo.com and sea conditions (water temperature, wave height)
//! from marine-api.open-meteo.com.
//!
//! Unlike tides, weather changes constantly, so it is NOT stored as events:
//! `GET /api/beach-weather` serves it live, behind a small in-memory cache
//! (30 min) so repeated app opens don't hammer the API.

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::tides::Port;

/// Forecast horizon in days (Open-Meteo free tier goes to 16).
const FORECAST_DAYS: u32 = 7;
/// Cache time-to-live, seconds.
const CACHE_TTL: i64 = 30 * 60;

// ---------------------------------------------------------------------------
// Response payload (also the cached value)

#[derive(Serialize, Clone)]
pub struct DayWeather {
    /// Paris civil day, "YYYY-MM-DD"
    pub date: String,
    /// WMO weather code (0 clear … 99 thunderstorm), emoji mapping is
    /// done client-side
    pub code: Option<i64>,
    pub tmax: Option<f64>,
    pub tmin: Option<f64>,
    /// Max wind speed, km/h
    pub wind: Option<f64>,
    pub uv: Option<f64>,
    /// Max precipitation probability, %
    pub precip: Option<f64>,
    /// Max wave height, m (marine API)
    pub wave: Option<f64>,
    /// Sea surface temperature around midday, °C (marine API)
    pub water: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct SpotWeather {
    pub key: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub days: Vec<DayWeather>,
}

// ---------------------------------------------------------------------------
// Cache

struct Entry {
    fetched_at: i64,
    /// Joined spot keys the entry was built for — a selection change
    /// invalidates it
    spots_key: String,
    payload: Vec<SpotWeather>,
}

#[derive(Default)]
pub struct WeatherCache {
    inner: RwLock<Option<Entry>>,
}

impl WeatherCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, spots_key: &str, now: i64) -> Option<Vec<SpotWeather>> {
        let guard = self.inner.read().ok()?;
        let entry = guard.as_ref()?;
        if entry.spots_key == spots_key && now - entry.fetched_at < CACHE_TTL {
            Some(entry.payload.clone())
        } else {
            None
        }
    }

    fn put(&self, spots_key: String, now: i64, payload: Vec<SpotWeather>) {
        if let Ok(mut guard) = self.inner.write() {
            *guard = Some(Entry {
                fetched_at: now,
                spots_key,
                payload,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Open-Meteo response shapes

#[derive(Deserialize, Default, Clone)]
struct ForecastDaily {
    #[serde(default)]
    time: Vec<String>,
    #[serde(default)]
    weather_code: Vec<Option<i64>>,
    #[serde(default)]
    temperature_2m_max: Vec<Option<f64>>,
    #[serde(default)]
    temperature_2m_min: Vec<Option<f64>>,
    #[serde(default)]
    wind_speed_10m_max: Vec<Option<f64>>,
    #[serde(default)]
    uv_index_max: Vec<Option<f64>>,
    #[serde(default)]
    precipitation_probability_max: Vec<Option<f64>>,
}

#[derive(Deserialize)]
struct ForecastResponse {
    #[serde(default)]
    daily: Option<ForecastDaily>,
}

#[derive(Deserialize, Default, Clone)]
struct MarineDaily {
    #[serde(default)]
    time: Vec<String>,
    #[serde(default)]
    wave_height_max: Vec<Option<f64>>,
}

#[derive(Deserialize, Default, Clone)]
struct MarineHourly {
    #[serde(default)]
    time: Vec<String>,
    #[serde(default)]
    sea_surface_temperature: Vec<Option<f64>>,
}

#[derive(Deserialize)]
struct MarineResponse {
    #[serde(default)]
    daily: Option<MarineDaily>,
    #[serde(default)]
    hourly: Option<MarineHourly>,
}

/// Open-Meteo returns a single object for one location and an array for a
/// comma-separated batch.
#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    Many(Vec<T>),
    One(T),
}

impl<T> OneOrMany<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::Many(v) => v,
            OneOrMany::One(x) => vec![x],
        }
    }
}

fn opt<T: Copy>(v: &[Option<T>], i: usize) -> Option<T> {
    v.get(i).copied().flatten()
}

/// Fetch beach weather for the given spots, one batched call per API.
/// The marine call is best-effort: if it fails, weather is served without
/// water temperature / wave height.
pub async fn fetch(ports: &[&Port]) -> Vec<SpotWeather> {
    if ports.is_empty() {
        return Vec::new();
    }
    let lat: Vec<String> = ports.iter().map(|p| p.lat.to_string()).collect();
    let lon: Vec<String> = ports.iter().map(|p| p.lon.to_string()).collect();
    let (lat, lon) = (lat.join(","), lon.join(","));

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let forecast_base = std::env::var("WEATHER_API_URL")
        .unwrap_or_else(|_| "https://api.open-meteo.com/v1/forecast".into());
    let forecast_url = format!(
        "{forecast_base}?latitude={lat}&longitude={lon}\
         &daily=weather_code,temperature_2m_max,temperature_2m_min,\
         wind_speed_10m_max,uv_index_max,precipitation_probability_max\
         &timezone=Europe%2FParis&forecast_days={FORECAST_DAYS}"
    );
    let forecasts: Vec<ForecastResponse> = match fetch_json::<OneOrMany<ForecastResponse>>(
        &client,
        &forecast_url,
        "Open-Meteo forecast",
    )
    .await
    {
        Some(r) => r.into_vec(),
        None => return Vec::new(),
    };

    let marine_base = std::env::var("MARINE_API_URL")
        .unwrap_or_else(|_| "https://marine-api.open-meteo.com/v1/marine".into());
    let marine_url = format!(
        "{marine_base}?latitude={lat}&longitude={lon}\
         &daily=wave_height_max&hourly=sea_surface_temperature\
         &timezone=Europe%2FParis&forecast_days={FORECAST_DAYS}"
    );
    let marines: Vec<MarineResponse> =
        match fetch_json::<OneOrMany<MarineResponse>>(&client, &marine_url, "Open-Meteo marine")
            .await
        {
            Some(r) => r.into_vec(),
            None => Vec::new(),
        };

    ports
        .iter()
        .enumerate()
        .map(|(i, port)| {
            let daily = forecasts
                .get(i)
                .and_then(|f| f.daily.as_ref())
                .cloned_or_default();
            let marine_daily = marines
                .get(i)
                .and_then(|m| m.daily.as_ref())
                .cloned_or_default();
            let marine_hourly = marines
                .get(i)
                .and_then(|m| m.hourly.as_ref())
                .cloned_or_default();

            let days = daily
                .time
                .iter()
                .enumerate()
                .map(|(d, date)| {
                    let wave = marine_daily
                        .time
                        .iter()
                        .position(|t| t == date)
                        .and_then(|j| opt(&marine_daily.wave_height_max, j));
                    // Water temperature: the hourly value closest to midday
                    let noon = format!("{date}T12:00");
                    let water = marine_hourly
                        .time
                        .iter()
                        .position(|t| t == &noon)
                        .and_then(|j| opt(&marine_hourly.sea_surface_temperature, j));
                    DayWeather {
                        date: date.clone(),
                        code: opt(&daily.weather_code, d),
                        tmax: opt(&daily.temperature_2m_max, d),
                        tmin: opt(&daily.temperature_2m_min, d),
                        wind: opt(&daily.wind_speed_10m_max, d),
                        uv: opt(&daily.uv_index_max, d),
                        precip: opt(&daily.precipitation_probability_max, d),
                        wave,
                        water,
                    }
                })
                .collect();
            SpotWeather {
                key: port.key,
                name: port.name,
                group: port.group,
                days,
            }
        })
        .collect()
}

/// Cached wrapper used by the handler.
pub async fn for_ports(cache: &WeatherCache, ports: &[&Port]) -> Vec<SpotWeather> {
    let spots_key: String = ports
        .iter()
        .map(|p| p.key)
        .collect::<Vec<_>>()
        .join(",");
    let now = chrono::Utc::now().timestamp();
    if let Some(hit) = cache.get(&spots_key, now) {
        return hit;
    }
    let fresh = fetch(ports).await;
    if !fresh.is_empty() {
        cache.put(spots_key, now, fresh.clone());
    }
    fresh
}

async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    what: &str,
) -> Option<T> {
    let resp = match client.get(url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            log::warn!("{what} returned {}", r.status());
            return None;
        }
        Err(e) => {
            log::warn!("{what} unreachable: {e}");
            return None;
        }
    };
    match resp.json::<T>().await {
        Ok(v) => Some(v),
        Err(e) => {
            log::warn!("could not parse {what} response: {e}");
            None
        }
    }
}

// Small helper: Option<&T> → T (cloned) or T::default()
trait ClonedOrDefault<T> {
    fn cloned_or_default(self) -> T;
}

impl<T: Clone + Default> ClonedOrDefault<T> for Option<&T> {
    fn cloned_or_default(self) -> T {
        self.cloned().unwrap_or_default()
    }
}
