use anyhow::Result;
use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn play_notification_sound() -> Result<()> {
    // Try to get sound path from config or use default
    let sound_path = get_sound_path();

    // Check if file exists before trying to play
    if !Path::new(&sound_path).exists() {
        return Err(anyhow::anyhow!("Sound file not found: {}", sound_path));
    }

    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default()?;
    
    // Load the sound file
    let file = File::open(sound_path)?;
    let source = Decoder::new(BufReader::new(file))?;
    
    // Play the sound
    stream_handle.play_raw(source.convert_samples())?;
    
    // The sound plays in a separate thread, so we need to wait a bit to ensure it completes
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    Ok(())
}

fn get_sound_path() -> String {
    // First check if a custom sound is configured
    if let Ok(custom_path) = std::env::var("REMINDME_SOUND") {
        return custom_path;
    }
    
    // Otherwise check common system notification sounds
    let potential_paths = [
        "/usr/share/sounds/freedesktop/stereo/complete.oga",
        "/usr/share/sounds/freedesktop/stereo/bell.oga",
        "/usr/share/sounds/ubuntu/notifications/Blip.ogg",
        "/usr/share/sounds/gnome/default/alerts/glass.ogg",
        "~/.config/remindme/notification.mp3", // For custom user sounds
    ];
    
    for path in potential_paths {
        let expanded_path = shellexpand::tilde(path).to_string();
        if Path::new(&expanded_path).exists() {
            return expanded_path;
        }
    }
    
    // If none found, return a default (which will fail if not found)
    "/usr/share/sounds/freedesktop/stereo/bell.oga".to_string()
}