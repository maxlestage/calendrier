//! Optional cinema releases via TMDB. Only active when the TMDB_API_KEY
//! config var is set (free key at themoviedb.org); without it the static
//! curated list in seed_events.json is the only cinema source.

use serde::Deserialize;

use crate::seed::SeedCandidate;

pub const CINEMA_COLOR: &str = "#8e44ad";

#[derive(Deserialize)]
struct DiscoverResponse {
    #[serde(default)]
    results: Vec<Movie>,
}

#[derive(Deserialize)]
struct Movie {
    title: String,
    #[serde(default)]
    release_date: String,
    #[serde(default)]
    popularity: f64,
}

/// Most popular upcoming theatrical releases in France over the next six
/// months, or an empty list when no key is configured or the API fails.
pub async fn fetch() -> Vec<SeedCandidate> {
    let Ok(key) = std::env::var("TMDB_API_KEY") else {
        return Vec::new();
    };
    let base = std::env::var("TMDB_API_URL")
        .unwrap_or_else(|_| "https://api.themoviedb.org/3".into());
    let today = chrono::Utc::now().date_naive();
    let horizon = today + chrono::Duration::days(180);
    let url = format!(
        "{base}/discover/movie?api_key={key}&language=fr-FR&region=FR&with_release_type=3&sort_by=popularity.desc&release_date.gte={today}&release_date.lte={horizon}"
    );
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            log::warn!("TMDB returned {}", r.status());
            return Vec::new();
        }
        Err(e) => {
            log::warn!("TMDB unreachable: {e}");
            return Vec::new();
        }
    };
    let mut parsed: DiscoverResponse = match resp.json().await {
        Ok(p) => p,
        Err(e) => {
            log::warn!("could not parse TMDB response: {e}");
            return Vec::new();
        }
    };
    parsed
        .results
        .sort_by(|a, b| b.popularity.partial_cmp(&a.popularity).unwrap_or(std::cmp::Ordering::Equal));
    let events: Vec<SeedCandidate> = parsed
        .results
        .into_iter()
        .filter(|m| m.release_date.len() == 10)
        .take(10)
        .map(|m| SeedCandidate {
            date: m.release_date,
            title: format!("🎬 {}", m.title),
            description: Some("Sortie cinéma France".into()),
            color: Some(CINEMA_COLOR.into()),
        })
        .collect();
    if !events.is_empty() {
        log::info!("fetched {} upcoming cinema releases from TMDB", events.len());
    }
    events
}
