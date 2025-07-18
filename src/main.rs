use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cache;
mod providers;
mod config;

#[derive(Parser)]
#[command(name = "tts-cli")]
#[command(about = "A command-line text-to-speech tool with multiple providers and caching")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Synthesize text to speech
    Speak {
        /// Text to synthesize
        text: String,
        /// TTS provider to use
        #[arg(short, long, default_value = "gcloud")]
        provider: String,
        /// Voice to use
        #[arg(short, long)]
        voice: Option<String>,
        /// Language code (e.g., en-US, es-ES)
        #[arg(short, long, default_value = "en-US")]
        language: String,
        /// Output file path (optional, will play audio directly if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Save to temp file instead of playing audio directly
        #[arg(long)]
        no_play: bool,
        /// Disable cache
        #[arg(long)]
        no_cache: bool,
        /// Clear cache for this text
        #[arg(long)]
        clear_cache: bool,
    },
    /// List available providers
    Providers,
    /// Clear all cache
    ClearCache,
    /// Show cache statistics
    CacheStats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Speak {
            text,
            provider,
            voice,
            language,
            output,
            no_play,
            no_cache,
            clear_cache,
        } => {
            if clear_cache {
                cache::clear_text_cache(&text, &provider, &language, voice.as_deref()).await?;
                println!("Cache cleared for the specified text");
                return Ok(());
            }

            let audio_data = if no_cache {
                synthesize_with_fallback(&text, &provider, &language, voice.as_deref()).await?
            } else {
                let cache_key = cache::generate_cache_key(&text, &provider, &language, voice.as_deref());
                
                if let Some(cached_data) = cache::get_cached_audio(&cache_key).await? {
                    println!("Using cached audio");
                    cached_data
                } else {
                    let audio_data = synthesize_with_fallback(&text, &provider, &language, voice.as_deref()).await?;
                    cache::cache_audio(&cache_key, &audio_data).await?;
                    println!("Audio cached for future use");
                    audio_data
                }
            };

            if let Some(output_path) = output {
                std::fs::write(&output_path, audio_data)?;
                println!("Audio saved to: {}", output_path.display());
            } else if no_play {
                // User explicitly requested to save to file instead of playing
                let temp_file = std::env::temp_dir().join("tts_output.wav");
                std::fs::write(&temp_file, &audio_data)?;
                println!("Audio saved to: {}", temp_file.display());
                println!("You can play it with: aplay {} or mpv {}", temp_file.display(), temp_file.display());
            } else {
                // Default behavior: play audio directly
                match try_play_audio_with_timeout(&audio_data) {
                    Ok(_) => {
                        println!("Audio playback completed");
                    }
                    Err(e) => {
                        println!("Audio playback failed: {}", e);
                        let temp_file = std::env::temp_dir().join("tts_output.wav");
                        std::fs::write(&temp_file, &audio_data)?;
                        println!("Audio saved to: {}", temp_file.display());
                        println!("You can play it with: aplay {} or mpv {}", temp_file.display(), temp_file.display());
                        println!("Use --no-play flag to save to file by default");
                    }
                }
            }
        }
        Commands::Providers => {
            providers::list_providers();
        }
        Commands::ClearCache => {
            cache::clear_all_cache().await?;
            println!("All cache cleared");
        }
        Commands::CacheStats => {
            cache::show_cache_stats().await?;
        }
    }

    Ok(())
}

async fn synthesize_with_fallback(
    text: &str,
    preferred_provider: &str,
    language: &str,
    voice: Option<&str>,
) -> Result<Vec<u8>> {
    // Try the preferred provider first
    match providers::synthesize_text(text, preferred_provider, language, voice).await {
        Ok(audio_data) => {
            println!("Using {} provider", preferred_provider);
            return Ok(audio_data);
        }
        Err(e) => {
            println!("Warning: {} provider failed: {}", preferred_provider, e);
        }
    }

    // Get available providers and try them in order
    let available_providers = providers::get_available_providers();
    let fallback_order = ["espeak", "festival", "say", "gcloud"];
    
    for provider_name in &fallback_order {
        if provider_name == &preferred_provider {
            continue; // Already tried
        }
        
        if let Some(provider) = available_providers.iter().find(|p| p.name == *provider_name && p.available) {
            println!("Trying fallback provider: {}", provider.name);
            match providers::synthesize_text(text, &provider.name, language, voice).await {
                Ok(audio_data) => {
                    println!("Successfully used {} provider", provider.name);
                    return Ok(audio_data);
                }
                Err(e) => {
                    println!("Warning: {} provider failed: {}", provider.name, e);
                }
            }
        }
    }

    Err(anyhow!("All TTS providers failed. Please install at least one: espeak, festival, or Google Cloud SDK"))
}

fn try_play_audio_with_timeout(audio_data: &[u8]) -> Result<()> {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();
    let audio_data = audio_data.to_vec();

    // Spawn audio playback in a separate thread
    thread::spawn(move || {
        let result = play_audio_blocking(&audio_data);
        let _ = tx.send(result);
    });

    // Wait for completion with timeout
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            Err(anyhow!("Audio playback timed out after 10 seconds - this may indicate an issue with the audio system"))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(anyhow!("Audio playback thread disconnected unexpectedly"))
        }
    }
}

fn play_audio_blocking(audio_data: &[u8]) -> Result<()> {
    use std::process::Command;
    
    // Save audio to a temporary file
    let temp_file = std::env::temp_dir().join("tts_playback.wav");
    std::fs::write(&temp_file, audio_data)?;
    
    // Try different audio players in order of preference
    let players = ["aplay", "paplay", "mpv", "ffplay", "play"];
    
    for player in &players {
        if Command::new(player).arg("--help").output().is_ok() || 
           Command::new("which").arg(player).output().map_or(false, |o| o.status.success()) {
            
            let output = Command::new(player)
                .arg(&temp_file)
                .output();
                
            // Clean up temp file
            let _ = std::fs::remove_file(&temp_file);
            
            match output {
                Ok(output) if output.status.success() => {
                    return Ok(());
                }
                Ok(output) => {
                    return Err(anyhow!("Audio player {} failed: {}", player, String::from_utf8_lossy(&output.stderr)));
                }
                Err(e) => {
                    // Try next player
                    continue;
                }
            }
        }
    }
    
    // Clean up temp file if we get here
    let _ = std::fs::remove_file(&temp_file);
    
    Err(anyhow!("No working audio player found. Please install one of: {}", players.join(", ")))
}