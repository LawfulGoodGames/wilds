use crate::town_dialogue::audio_filename;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[derive(Debug, Default)]
pub struct DialogueAudioState {
    pub active_clip_id: Option<String>,
    process: Option<Child>,
}

impl DialogueAudioState {
    pub fn is_playing(&self) -> bool {
        self.process.is_some() || self.active_clip_id.is_some()
    }
}

pub fn play(id: &str, state: &mut DialogueAudioState) -> std::io::Result<()> {
    stop(state);

    if cfg!(test) {
        state.active_clip_id = Some(id.to_string());
        return Ok(());
    }

    let Some(filename) = audio_filename(id) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("unknown audio cue: {id}"),
        ));
    };

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/audio")
        .join(filename);
    let child = spawn_player(path.as_path())?;
    state.active_clip_id = Some(id.to_string());
    state.process = Some(child);
    Ok(())
}

pub fn stop(state: &mut DialogueAudioState) {
    if let Some(mut child) = state.process.take() {
        match child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
    state.active_clip_id = None;
}

fn spawn_player(path: &Path) -> std::io::Result<Child> {
    #[cfg(target_os = "macos")]
    {
        return Command::new("afplay").arg(path).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        for player in ["aplay", "paplay"] {
            match Command::new(player).arg(path).spawn() {
                Ok(child) => return Ok(child),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => return Err(err),
            }
        }

        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no supported audio player found",
        ));
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "$player = New-Object Media.SoundPlayer '{}'; $player.PlaySync()",
            path.display()
        );
        return Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn();
    }

    #[allow(unreachable_code)]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "audio playback is not supported on this platform",
    ))
}
