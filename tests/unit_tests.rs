// Unit tests for cosmic-radio-applet modules
// This file allows running tests without building the full application

#[cfg(test)]
mod audio_tests {
    use url::Url;

    /// Validates that a URL is safe to pass to mpv (http/https only)
    fn validate_url(url: &str) -> Result<(), &'static str> {
        match Url::parse(url) {
            Ok(parsed) => {
                let scheme = parsed.scheme();
                if scheme == "http" || scheme == "https" {
                    // Block localhost and private IP ranges
                    if let Some(host) = parsed.host_str() {
                        if host == "localhost"
                            || host == "127.0.0.1"
                            || host.starts_with("192.168.")
                            || host.starts_with("10.")
                            || host.starts_with("172.16.")
                        {
                            return Err("Local/private URLs not allowed");
                        }
                    }
                    Ok(())
                } else {
                    Err("Only http/https URLs are allowed")
                }
            }
            Err(_) => Err("Invalid URL format"),
        }
    }

    #[test]
    fn test_validate_url_valid_http() {
        assert!(validate_url("http://example.com/stream").is_ok());
    }

    #[test]
    fn test_validate_url_valid_https() {
        assert!(validate_url("https://example.com/stream.mp3").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme_file() {
        assert_eq!(
            validate_url("file:///etc/passwd"),
            Err("Only http/https URLs are allowed")
        );
    }

    #[test]
    fn test_validate_url_invalid_scheme_ftp() {
        assert_eq!(
            validate_url("ftp://example.com/file"),
            Err("Only http/https URLs are allowed")
        );
    }

    #[test]
    fn test_validate_url_localhost_blocked() {
        assert_eq!(
            validate_url("http://localhost:8080/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_127_0_0_1_blocked() {
        assert_eq!(
            validate_url("https://127.0.0.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_private_192_168_blocked() {
        assert_eq!(
            validate_url("http://192.168.1.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_private_10_blocked() {
        assert_eq!(
            validate_url("http://10.0.0.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_private_172_16_blocked() {
        assert_eq!(
            validate_url("http://172.16.0.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_invalid_format() {
        assert_eq!(validate_url("not a url at all"), Err("Invalid URL format"));
    }

    #[test]
    fn test_validate_url_empty_string() {
        assert_eq!(validate_url(""), Err("Invalid URL format"));
    }

    #[test]
    fn test_validate_url_public_ip() {
        assert!(validate_url("http://8.8.8.8/stream").is_ok());
    }

    #[test]
    fn test_validate_url_with_port() {
        assert!(validate_url("https://example.com:8443/stream").is_ok());
    }

    #[test]
    fn test_validate_url_with_path_and_query() {
        assert!(validate_url("http://radio.example.com/live?quality=high").is_ok());
    }
}

#[cfg(test)]
mod api_tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

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

    #[test]
    fn test_station_default() {
        let station = Station::default();
        assert_eq!(station.stationuuid, "");
        assert_eq!(station.name, "");
        assert_eq!(station.url, "");
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
    }

    // Note: Null handling is tested in src/api.rs via ApiStation intermediate struct

    #[test]
    fn test_station_serialize() {
        let station = Station {
            stationuuid: "test-uuid".to_string(),
            name: "Test Station".to_string(),
            url: "http://test.com".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_value(&station).unwrap();
        assert_eq!(json["stationuuid"], "test-uuid");
        assert_eq!(json["name"], "Test Station");
    }
}

#[cfg(test)]
mod config_tests {
    use serde::{Deserialize, Serialize};

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

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Config {
        #[serde(default)]
        pub favorites: Vec<Station>,
        #[serde(default)]
        pub volume: u8,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                favorites: Vec::new(),
                volume: 50,
            }
        }
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.favorites.len(), 0);
        assert_eq!(config.volume, 50);
    }

    #[test]
    fn test_config_default_volume_is_50() {
        let config = Config::default();
        assert_eq!(config.volume, 50);
    }

    #[test]
    fn test_config_with_favorites() {
        let station = Station {
            stationuuid: "test-uuid".to_string(),
            name: "Test Station".to_string(),
            url: "http://test.com".to_string(),
            ..Default::default()
        };

        let config = Config {
            favorites: vec![station.clone()],
            volume: 75,
        };

        assert_eq!(config.favorites.len(), 1);
        assert_eq!(config.favorites[0].name, "Test Station");
        assert_eq!(config.volume, 75);
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let station = Station {
            stationuuid: "uuid".to_string(),
            name: "Station".to_string(),
            url: "http://example.com".to_string(),
            ..Default::default()
        };
        let config = Config {
            favorites: vec![station],
            volume: 80,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }
}
