//! Weather for the selected beaches (tide spots) AND French cities, via the
//! Open-Meteo APIs (free, no key): daily forecast (temperature, wind, UV,
//! rain) from api.open-meteo.com and — for sea spots only — sea conditions
//! (water temperature, wave height) from marine-api.open-meteo.com.
//!
//! Unlike tides, weather changes constantly, so it is NOT stored as events:
//! `GET /api/beach-weather` serves it live, behind a small in-memory cache
//! (30 min) so repeated app opens don't hammer the API.

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// A place we can fetch weather for — a beach from the tide catalog or a
/// city from `CITIES`.
#[derive(Clone, Copy)]
pub struct Place {
    pub key: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub lat: f64,
    pub lon: f64,
    /// Marine data (waves, water temperature) only makes sense by the sea
    pub sea: bool,
}

pub struct City {
    pub key: &'static str,
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

/// Main French cities (metropole + Corse) for the in-app weather dropdown.
pub static CITIES: &[City] = &[
    City { key: "paris", name: "Paris", lat: 48.8566, lon: 2.3522 },
    City { key: "marseille-ville", name: "Marseille", lat: 43.2965, lon: 5.3698 },
    City { key: "lyon", name: "Lyon", lat: 45.7640, lon: 4.8357 },
    City { key: "toulouse", name: "Toulouse", lat: 43.6045, lon: 1.4440 },
    City { key: "nice-ville", name: "Nice", lat: 43.7102, lon: 7.2620 },
    City { key: "nantes", name: "Nantes", lat: 47.2184, lon: -1.5536 },
    City { key: "montpellier", name: "Montpellier", lat: 43.6108, lon: 3.8767 },
    City { key: "strasbourg", name: "Strasbourg", lat: 48.5734, lon: 7.7521 },
    City { key: "bordeaux", name: "Bordeaux", lat: 44.8378, lon: -0.5792 },
    City { key: "lille", name: "Lille", lat: 50.6292, lon: 3.0573 },
    City { key: "rennes", name: "Rennes", lat: 48.1173, lon: -1.6778 },
    City { key: "reims", name: "Reims", lat: 49.2583, lon: 4.0317 },
    City { key: "toulon", name: "Toulon", lat: 43.1242, lon: 5.9280 },
    City { key: "saint-etienne", name: "Saint-Étienne", lat: 45.4397, lon: 4.3872 },
    City { key: "le-havre", name: "Le Havre", lat: 49.4944, lon: 0.1079 },
    City { key: "grenoble", name: "Grenoble", lat: 45.1885, lon: 5.7245 },
    City { key: "dijon", name: "Dijon", lat: 47.3220, lon: 5.0415 },
    City { key: "angers", name: "Angers", lat: 47.4784, lon: -0.5632 },
    City { key: "nimes", name: "Nîmes", lat: 43.8367, lon: 4.3601 },
    City { key: "clermont-ferrand", name: "Clermont-Ferrand", lat: 45.7772, lon: 3.0870 },
    City { key: "le-mans", name: "Le Mans", lat: 48.0061, lon: 0.1996 },
    City { key: "aix-en-provence", name: "Aix-en-Provence", lat: 43.5297, lon: 5.4474 },
    City { key: "brest-ville", name: "Brest", lat: 48.3904, lon: -4.4861 },
    City { key: "tours", name: "Tours", lat: 47.3941, lon: 0.6848 },
    City { key: "amiens", name: "Amiens", lat: 49.8942, lon: 2.2957 },
    City { key: "limoges", name: "Limoges", lat: 45.8336, lon: 1.2611 },
    City { key: "annecy", name: "Annecy", lat: 45.8992, lon: 6.1294 },
    City { key: "perpignan", name: "Perpignan", lat: 42.6887, lon: 2.8948 },
    City { key: "besancon", name: "Besançon", lat: 47.2378, lon: 6.0241 },
    City { key: "metz", name: "Metz", lat: 49.1193, lon: 6.1757 },
    City { key: "orleans", name: "Orléans", lat: 47.9029, lon: 1.9093 },
    City { key: "rouen", name: "Rouen", lat: 49.4431, lon: 1.0993 },
    City { key: "mulhouse", name: "Mulhouse", lat: 47.7508, lon: 7.3359 },
    City { key: "caen", name: "Caen", lat: 49.1829, lon: -0.3707 },
    City { key: "nancy", name: "Nancy", lat: 48.6921, lon: 6.1844 },
    City { key: "avignon", name: "Avignon", lat: 43.9493, lon: 4.8055 },
    City { key: "poitiers", name: "Poitiers", lat: 46.5802, lon: 0.3404 },
    City { key: "la-rochelle-ville", name: "La Rochelle", lat: 46.1603, lon: -1.1511 },
    City { key: "pau", name: "Pau", lat: 43.2951, lon: -0.3708 },
    City { key: "bayonne", name: "Bayonne", lat: 43.4929, lon: -1.4748 },
    City { key: "ajaccio-ville", name: "Ajaccio", lat: 41.9192, lon: 8.7386 },
    City { key: "bastia", name: "Bastia", lat: 42.6977, lon: 9.4508 },
    City { key: "chambery", name: "Chambéry", lat: 45.5646, lon: 5.9178 },
    City { key: "vannes", name: "Vannes", lat: 47.6582, lon: -2.7608 },
    City { key: "quimper", name: "Quimper", lat: 47.9960, lon: -4.0972 },
];

pub const CITIES_SETTING: &str = "weather_cities";

pub async fn selected_cities(db: &sea_orm::DatabaseConnection) -> Vec<&'static City> {
    let Some(saved) = crate::settings::get(db, CITIES_SETTING).await else {
        return Vec::new();
    };
    let tokens = crate::tides::parse_tokens(&saved);
    CITIES
        .iter()
        .filter(|c| tokens.iter().any(|t| t == c.key))
        .collect()
}

/// Everything weather-worthy the user selected: beaches (with marine data)
/// then cities.
pub async fn selected_places(db: &sea_orm::DatabaseConnection) -> Vec<Place> {
    let mut places: Vec<Place> = crate::tides::selected_ports(db)
        .await
        .into_iter()
        .map(|p| Place {
            key: p.key,
            name: p.name,
            group: p.group,
            lat: p.lat,
            lon: p.lon,
            sea: true,
        })
        .collect();
    places.extend(selected_cities(db).await.into_iter().map(|c| Place {
        key: c.key,
        name: c.name,
        group: "ville",
        lat: c.lat,
        lon: c.lon,
        sea: false,
    }));
    places
}

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

/// Fetch weather for the given places, one batched call per API. The
/// marine call only covers sea places (waves/water temperature are
/// meaningless inland — and an inland point could poison the whole batch)
/// and is best-effort: if it fails, weather is served without those fields.
pub async fn fetch(places: &[Place]) -> Vec<SpotWeather> {
    if places.is_empty() {
        return Vec::new();
    }
    let lat: Vec<String> = places.iter().map(|p| p.lat.to_string()).collect();
    let lon: Vec<String> = places.iter().map(|p| p.lon.to_string()).collect();
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

    // Position of each place in the (sea-only) marine batch
    let sea_positions: Vec<Option<usize>> = {
        let mut next = 0usize;
        places
            .iter()
            .map(|p| {
                if p.sea {
                    let pos = next;
                    next += 1;
                    Some(pos)
                } else {
                    None
                }
            })
            .collect()
    };
    let sea_places: Vec<&Place> = places.iter().filter(|p| p.sea).collect();
    let marines: Vec<MarineResponse> = if sea_places.is_empty() {
        Vec::new()
    } else {
        let mlat: Vec<String> = sea_places.iter().map(|p| p.lat.to_string()).collect();
        let mlon: Vec<String> = sea_places.iter().map(|p| p.lon.to_string()).collect();
        let marine_base = std::env::var("MARINE_API_URL")
            .unwrap_or_else(|_| "https://marine-api.open-meteo.com/v1/marine".into());
        let marine_url = format!(
            "{marine_base}?latitude={}&longitude={}\
             &daily=wave_height_max&hourly=sea_surface_temperature\
             &timezone=Europe%2FParis&forecast_days={FORECAST_DAYS}",
            mlat.join(","),
            mlon.join(",")
        );
        match fetch_json::<OneOrMany<MarineResponse>>(&client, &marine_url, "Open-Meteo marine")
            .await
        {
            Some(r) => r.into_vec(),
            None => Vec::new(),
        }
    };

    places
        .iter()
        .enumerate()
        .map(|(i, port)| {
            let daily = forecasts
                .get(i)
                .and_then(|f| f.daily.as_ref())
                .cloned_or_default();
            let marine = sea_positions[i].and_then(|pos| marines.get(pos));
            let marine_daily = marine
                .and_then(|m| m.daily.as_ref())
                .cloned_or_default();
            let marine_hourly = marine
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
pub async fn for_places(cache: &WeatherCache, places: &[Place]) -> Vec<SpotWeather> {
    let spots_key: String = places
        .iter()
        .map(|p| p.key)
        .collect::<Vec<_>>()
        .join(",");
    let now = chrono::Utc::now().timestamp();
    if let Some(hit) = cache.get(&spots_key, now) {
        return hit;
    }
    let fresh = fetch(places).await;
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
