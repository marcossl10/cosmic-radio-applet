//! Custom error types for cosmic-radio-applet
//!
//! These error types are defined for future integration. Currently the application
//! uses simpler error handling, but these types provide a foundation for more
//! robust error handling as the codebase evolves.

#![allow(dead_code)]

use thiserror::Error;

/// Application-wide error type covering all failure cases
#[derive(Error, Debug)]
pub enum AppError {
    /// Configuration-related errors (load, save, parse failures)
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Audio playback errors (mpv spawn, IPC communication)
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    /// API and network errors (HTTP requests, JSON parsing)
    #[error("API error: {0}")]
    Api(#[from] ApiError),

    /// Input validation errors (invalid URLs, malformed data)
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
}

/// Configuration-specific errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    LoadFailed(String),

    #[error("Failed to save configuration: {0}")]
    SaveFailed(String),

    #[error("Failed to parse configuration: {0}")]
    ParseFailed(#[from] serde_json::Error),

    #[error("Configuration file not found: {0}")]
    NotFound(String),

    #[error("IO error accessing configuration: {0}")]
    Io(#[from] std::io::Error),
}

/// Audio playback errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to spawn mpv process: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("MPV IPC communication failed: {0}")]
    IpcFailed(String),

    #[error("Failed to send command to mpv: {0}")]
    CommandFailed(String),

    #[error("MPV process terminated unexpectedly")]
    ProcessTerminated,

    #[error("Invalid audio stream format: {0}")]
    InvalidFormat(String),

    #[error("Audio playback timed out")]
    Timeout,
}

/// API and network errors
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    JsonParseFailed(#[from] serde_json::Error),

    #[error("API returned error: {status} - {message}")]
    Response { status: u16, message: String },

    #[error("Network timeout after {0}s")]
    Timeout(u64),

    #[error("Radio station not found: {0}")]
    StationNotFound(String),

    #[error("Invalid API response format: {0}")]
    InvalidResponse(String),
}

/// Input validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Invalid stream URL: {0}")]
    InvalidStreamUrl(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid station name: {0}")]
    InvalidStationName(String),

    #[error("Station name too long (max {max} characters): {actual}")]
    StationNameTooLong { max: usize, actual: usize },
}

/// Type alias for Results using AppError
pub type Result<T> = std::result::Result<T, AppError>;
