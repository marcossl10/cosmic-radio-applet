use crate::api::Station;
use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq, Serialize, Deserialize)]
#[version = 9]
pub struct Config {
    #[serde(default)]
    pub favorites: Vec<Station>,
    #[serde(default)]
    pub volume: u8, // 0-100
}

impl Default for Config {
    fn default() -> Self {
        Self {
            favorites: Vec::new(),
            volume: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_config_default_favorites_empty() {
        let config = Config::default();
        assert!(config.favorites.is_empty());
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
    fn test_config_clone() {
        let station = Station {
            name: "Test".to_string(),
            ..Default::default()
        };
        let config1 = Config {
            favorites: vec![station],
            volume: 60,
        };
        let config2 = config1.clone();

        assert_eq!(config1.favorites.len(), config2.favorites.len());
        assert_eq!(config1.volume, config2.volume);
    }

    #[test]
    fn test_config_equality() {
        let station = Station {
            name: "Same".to_string(),
            ..Default::default()
        };
        let config1 = Config {
            favorites: vec![station.clone()],
            volume: 50,
        };
        let config2 = Config {
            favorites: vec![station],
            volume: 50,
        };
        let config3 = Config {
            favorites: vec![],
            volume: 50,
        };

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
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

    #[test]
    fn test_config_volume_bounds() {
        let config_min = Config {
            favorites: vec![],
            volume: 0,
        };
        let config_max = Config {
            favorites: vec![],
            volume: 100,
        };

        assert_eq!(config_min.volume, 0);
        assert_eq!(config_max.volume, 100);
    }

    #[test]
    fn test_config_multiple_favorites() {
        let station1 = Station {
            name: "Station 1".to_string(),
            ..Default::default()
        };
        let station2 = Station {
            name: "Station 2".to_string(),
            ..Default::default()
        };
        let station3 = Station {
            name: "Station 3".to_string(),
            ..Default::default()
        };

        let config = Config {
            favorites: vec![station1, station2, station3],
            volume: 50,
        };

        assert_eq!(config.favorites.len(), 3);
        assert_eq!(config.favorites[0].name, "Station 1");
        assert_eq!(config.favorites[1].name, "Station 2");
        assert_eq!(config.favorites[2].name, "Station 3");
    }
}
