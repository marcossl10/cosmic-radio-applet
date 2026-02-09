use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, warn};
use url::Url;

const MPV_SOCKET_PATH: &str = "/tmp/cosmic-radio-mpv.sock";

pub struct AudioManager {
    process: Arc<Mutex<Option<Child>>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
        }
    }

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

    pub fn play(&self, url: String, volume: u8) {
        // Validate URL before passing to mpv (security)
        if let Err(e) = Self::validate_url(&url) {
            error!("Invalid stream URL: {} - {}", url, e);
            return;
        }

        self.stop(); // Stop current if any

        let child = Command::new("mpv")
            .arg("--no-video")
            .arg(format!("--volume={}", volume))
            .arg("--volume-max=200")
            .arg("--af=lavfi=[dynaudnorm]")
            .arg(format!("--input-ipc-server={}", MPV_SOCKET_PATH))
            .arg(&url)
            .spawn();

        debug!(
            "Spawned mpv for {} with IPC socket at {}",
            url, MPV_SOCKET_PATH
        );

        match child {
            Ok(child) => {
                if let Ok(mut guard) = self.process.lock() {
                    *guard = Some(child);
                }
            }
            Err(e) => {
                error!("Failed to start mpv: {}", e);
            }
        }
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.process.lock() {
            if let Some(mut child) = guard.take() {
                if let Err(e) = child.kill() {
                    warn!("Failed to kill mpv process: {}", e);
                }
                let _ = child.wait();
            }
        }

        // Clean up IPC socket
        let socket_path = Path::new(MPV_SOCKET_PATH);
        if socket_path.exists() {
            if let Err(e) = std::fs::remove_file(socket_path) {
                warn!("Failed to remove mpv socket at {}: {}", MPV_SOCKET_PATH, e);
            } else {
                debug!("Cleaned up mpv socket at {}", MPV_SOCKET_PATH);
            }
        }
    }

    pub fn set_volume(&self, vol: f32) {
        // Clamp volume to 0-100 range
        let volume = vol.clamp(0.0, 100.0);

        // Check if mpv process is running
        if let Ok(guard) = self.process.lock() {
            if guard.is_none() {
                debug!("Cannot set volume: mpv is not running");
                return;
            }
        }

        // Try to connect to IPC socket
        let socket_path = Path::new(MPV_SOCKET_PATH);
        if !socket_path.exists() {
            warn!(
                "Cannot set volume: mpv IPC socket not found at {}",
                MPV_SOCKET_PATH
            );
            return;
        }

        match UnixStream::connect(socket_path) {
            Ok(mut stream) => {
                // Build JSON IPC command: {"command": ["set_property", "volume", VALUE]}
                let command = format!(r#"{{"command": ["set_property", "volume", {}]}}"#, volume);
                let command_with_newline = format!("{}\n", command);

                match stream.write_all(command_with_newline.as_bytes()) {
                    Ok(_) => {
                        debug!("Set mpv volume to {} via IPC", volume);
                    }
                    Err(e) => {
                        error!("Failed to send volume command to mpv IPC: {}", e);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to connect to mpv IPC socket at {}: {}",
                    MPV_SOCKET_PATH, e
                );
            }
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid_http() {
        assert!(AudioManager::validate_url("http://example.com/stream").is_ok());
    }

    #[test]
    fn test_validate_url_valid_https() {
        assert!(AudioManager::validate_url("https://example.com/stream.mp3").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme_file() {
        assert_eq!(
            AudioManager::validate_url("file:///etc/passwd"),
            Err("Only http/https URLs are allowed")
        );
    }

    #[test]
    fn test_validate_url_invalid_scheme_ftp() {
        assert_eq!(
            AudioManager::validate_url("ftp://example.com/file"),
            Err("Only http/https URLs are allowed")
        );
    }

    #[test]
    fn test_validate_url_localhost_blocked() {
        assert_eq!(
            AudioManager::validate_url("http://localhost:8080/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_127_0_0_1_blocked() {
        assert_eq!(
            AudioManager::validate_url("https://127.0.0.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_private_192_168_blocked() {
        assert_eq!(
            AudioManager::validate_url("http://192.168.1.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_private_10_blocked() {
        assert_eq!(
            AudioManager::validate_url("http://10.0.0.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_private_172_16_blocked() {
        assert_eq!(
            AudioManager::validate_url("http://172.16.0.1/stream"),
            Err("Local/private URLs not allowed")
        );
    }

    #[test]
    fn test_validate_url_invalid_format() {
        assert_eq!(
            AudioManager::validate_url("not a url at all"),
            Err("Invalid URL format")
        );
    }

    #[test]
    fn test_validate_url_empty_string() {
        assert_eq!(AudioManager::validate_url(""), Err("Invalid URL format"));
    }

    #[test]
    fn test_validate_url_public_ip() {
        assert!(AudioManager::validate_url("http://8.8.8.8/stream").is_ok());
    }

    #[test]
    fn test_validate_url_with_port() {
        assert!(AudioManager::validate_url("https://example.com:8443/stream").is_ok());
    }

    #[test]
    fn test_validate_url_with_path_and_query() {
        assert!(AudioManager::validate_url("http://radio.example.com/live?quality=high").is_ok());
    }

    #[test]
    fn test_audio_manager_new() {
        let manager = AudioManager::new();
        assert!(manager.process.lock().unwrap().is_none());
    }

    #[test]
    fn test_audio_manager_default() {
        let manager = AudioManager::default();
        assert!(manager.process.lock().unwrap().is_none());
    }
}
