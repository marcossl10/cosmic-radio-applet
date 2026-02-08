use reqwest::Error;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Maximum response body size (1MB) to prevent memory exhaustion attacks
/// This is sufficient for 20 station records with metadata
const MAX_RESPONSE_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Station {
    #[serde(default)]
    pub stationuuid: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub url_resolved: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub favicon: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub language: String,
}

/// Intermediate struct to handle null values from API JSON
#[derive(Deserialize)]
struct ApiStation {
    #[serde(default)]
    stationuuid: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    url_resolved: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    favicon: Option<String>,
    #[serde(default)]
    tags: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

impl From<ApiStation> for Station {
    fn from(api: ApiStation) -> Self {
        Self {
            stationuuid: api.stationuuid.unwrap_or_default(),
            name: api.name.unwrap_or_default(),
            url: api.url.unwrap_or_default(),
            url_resolved: api.url_resolved.unwrap_or_default(),
            homepage: api.homepage.unwrap_or_default(),
            favicon: api.favicon.unwrap_or_default(),
            tags: api.tags.unwrap_or_default(),
            country: api.country.unwrap_or_default(),
            language: api.language.unwrap_or_default(),
        }
    }
}

/// Mirror servers for radio-browser.info API redundancy
const API_SERVERS: &[&str] = &[
    "https://all.api.radio-browser.info",
    "https://de1.api.radio-browser.info",
    "https://fr1.api.radio-browser.info",
    "https://at1.api.radio-browser.info",
    "https://nl1.api.radio-browser.info",
    "https://us1.api.radio-browser.info",
    "https://es1.api.radio-browser.info",
];

/// Search for radio stations by name
pub async fn search_stations(query: String) -> Result<Vec<Station>, Error> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    debug!("Searching stations for '{}'", query);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut last_error: Option<Error> = None;

    for server in API_SERVERS {
        let url = format!("{}/json/stations/search", server);
        let params = [("name", query.as_str()), ("limit", "20")];

        match client.get(&url).query(&params).send().await {
            Ok(response) => match response.error_for_status() {
                Ok(valid_response) => {
                    // Check Content-Length header first if available (early rejection)
                    if let Some(content_length) = valid_response.content_length() {
                        if content_length as usize > MAX_RESPONSE_SIZE {
                            warn!(
                                "Response from {} exceeds size limit: {} bytes (max: {})",
                                server, content_length, MAX_RESPONSE_SIZE
                            );
                            continue;
                        }
                    }

                    // Read response body as bytes with size validation
                    match valid_response.bytes().await {
                        Ok(bytes) => {
                            if bytes.len() > MAX_RESPONSE_SIZE {
                                warn!(
                                    "Response body from {} exceeds size limit: {} bytes (max: {})",
                                    server,
                                    bytes.len(),
                                    MAX_RESPONSE_SIZE
                                );
                                continue;
                            }

                            // Deserialize from validated bytes
                            match serde_json::from_slice::<Vec<ApiStation>>(&bytes) {
                                Ok(api_stations) => {
                                    debug!("Found {} stations from {}", api_stations.len(), server);
                                    return Ok(api_stations.into_iter().map(Station::from).collect());
                                }
                                Err(e) => {
                                    warn!("JSON parse error from {}: {}", server, e);
                                    // Continue to next server on parse error
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to read response body from {}: {}", server, e);
                            last_error = Some(e);
                        }
                    }
                }
                Err(e) => {
                    warn!("HTTP error from {}: {}", server, e);
                    last_error = Some(e);
                }
            },
            Err(e) => {
                warn!("Connection error to {}: {}", server, e);
                last_error = Some(e);
            }
        }
    }

    // All servers failed - return the last error or empty result
    match last_error {
        Some(e) => Err(e),
        None => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_station_default() {
        let station = Station::default();
        assert_eq!(station.stationuuid, "");
        assert_eq!(station.name, "");
        assert_eq!(station.url, "");
        assert_eq!(station.url_resolved, "");
        assert_eq!(station.homepage, "");
        assert_eq!(station.favicon, "");
        assert_eq!(station.tags, "");
        assert_eq!(station.country, "");
        assert_eq!(station.language, "");
    }

    #[test]
    fn test_station_deserialize_complete() {
        let json = json!({
            "stationuuid": "abc-123",
            "name": "Test Radio",
            "url": "http://example.com/stream",
            "url_resolved": "http://example.com/resolved",
            "homepage": "http://example.com",
            "favicon": "http://example.com/favicon.png",
            "tags": "rock,pop",
            "country": "USA",
            "language": "English"
        });

        let station: Station = serde_json::from_value(json).unwrap();
        assert_eq!(station.stationuuid, "abc-123");
        assert_eq!(station.name, "Test Radio");
        assert_eq!(station.url, "http://example.com/stream");
        assert_eq!(station.url_resolved, "http://example.com/resolved");
        assert_eq!(station.homepage, "http://example.com");
        assert_eq!(station.favicon, "http://example.com/favicon.png");
        assert_eq!(station.tags, "rock,pop");
        assert_eq!(station.country, "USA");
        assert_eq!(station.language, "English");
    }

    #[test]
    fn test_station_deserialize_with_nulls() {
        let json = json!({
            "stationuuid": null,
            "name": "Test Radio",
            "url": null,
            "url_resolved": "http://example.com/resolved",
            "homepage": null,
            "favicon": null,
            "tags": null,
            "country": null,
            "language": null
        });

        // Station struct uses ApiStation internally for null handling
        let api_station: ApiStation = serde_json::from_value(json).unwrap();
        let station: Station = api_station.into();

        assert_eq!(station.stationuuid, "");
        assert_eq!(station.name, "Test Radio");
        assert_eq!(station.url, "");
        assert_eq!(station.url_resolved, "http://example.com/resolved");
        assert_eq!(station.homepage, "");
        assert_eq!(station.favicon, "");
        assert_eq!(station.tags, "");
        assert_eq!(station.country, "");
        assert_eq!(station.language, "");
    }

    #[test]
    fn test_station_deserialize_missing_fields() {
        let json = json!({
            "name": "Minimal Station"
        });

        let station: Station = serde_json::from_value(json).unwrap();
        assert_eq!(station.name, "Minimal Station");
        assert_eq!(station.stationuuid, "");
        assert_eq!(station.url, "");
        assert_eq!(station.url_resolved, "");
    }

    #[test]
    fn test_station_deserialize_empty_object() {
        let json = json!({});

        let station: Station = serde_json::from_value(json).unwrap();
        assert_eq!(station, Station::default());
    }

    #[test]
    fn test_station_serialize() {
        let station = Station {
            stationuuid: "test-uuid".to_string(),
            name: "Test Station".to_string(),
            url: "http://test.com".to_string(),
            url_resolved: "http://resolved.test.com".to_string(),
            homepage: "http://homepage.com".to_string(),
            favicon: "http://favicon.com".to_string(),
            tags: "test".to_string(),
            country: "TestLand".to_string(),
            language: "TestLang".to_string(),
        };

        let json = serde_json::to_value(&station).unwrap();
        assert_eq!(json["stationuuid"], "test-uuid");
        assert_eq!(json["name"], "Test Station");
        assert_eq!(json["url"], "http://test.com");
    }

    #[test]
    fn test_station_clone() {
        let station1 = Station {
            name: "Clone Test".to_string(),
            ..Default::default()
        };
        let station2 = station1.clone();
        assert_eq!(station1, station2);
    }

    #[test]
    fn test_station_equality() {
        let station1 = Station {
            stationuuid: "same-uuid".to_string(),
            name: "Same Name".to_string(),
            ..Default::default()
        };
        let station2 = Station {
            stationuuid: "same-uuid".to_string(),
            name: "Same Name".to_string(),
            ..Default::default()
        };
        let station3 = Station {
            stationuuid: "different-uuid".to_string(),
            name: "Same Name".to_string(),
            ..Default::default()
        };

        assert_eq!(station1, station2);
        assert_ne!(station1, station3);
    }

    #[test]
    fn test_api_station_to_station_conversion() {
        let api_station = ApiStation {
            stationuuid: Some("uuid".to_string()),
            name: Some("Name".to_string()),
            url: None,
            url_resolved: Some("resolved".to_string()),
            homepage: None,
            favicon: None,
            tags: None,
            country: None,
            language: None,
        };

        let station: Station = api_station.into();
        assert_eq!(station.stationuuid, "uuid");
        assert_eq!(station.name, "Name");
        assert_eq!(station.url, "");
        assert_eq!(station.url_resolved, "resolved");
    }

    #[tokio::test]
    async fn test_search_stations_empty_query() {
        let result = search_stations("".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_search_stations_whitespace_query() {
        let result = search_stations("   ".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
