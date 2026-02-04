use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

pub struct AudioManager {
    // Usar Mutex para guardar o processo filho e poder matar depois
    process: Arc<Mutex<Option<Child>>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
        }
    }

    pub fn play(&self, url: String, volume: u8) {
        self.stop(); // Stop current if any
        
        // Spawn mpv --no-video --volume=X --volume-max=200 --af=lavfi=[dynaudnorm] url
        let child = Command::new("mpv")
            .arg("--no-video")
            .arg(format!("--volume={}", volume))
            .arg("--volume-max=200")
            .arg("--af=lavfi=[dynaudnorm]")
            .arg(&url)
            .spawn();
            
        println!("AudioManager: Spawned mpv for {}", url);
            
        if let Ok(child) = child {
            if let Ok(mut guard) = self.process.lock() {
                *guard = Some(child);
            }
        } else {
            eprintln!("AudioManager: Failed to start mpv");
        }
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.process.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
    
    pub fn set_volume(&self, _vol: f32) {
        // Implementar controle de volume via IPC do MPV seria ideal,
        // mas por enquanto deixa sem ou reinicia?
        // MPV supports --volume arg at start.
        // For runtime volume, we need IPC socket. Too complex for now.
    }
}
