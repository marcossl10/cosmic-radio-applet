// Library exports for testing
pub mod api;
pub mod audio;
pub mod config;

// Re-export commonly used items for easier testing
pub use api::{search_stations, Station};
pub use audio::AudioManager;
pub use config::Config;
