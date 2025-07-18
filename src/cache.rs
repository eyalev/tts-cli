use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;

pub fn generate_cache_key(text: &str, provider: &str, language: &str, voice: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(provider.as_bytes());
    hasher.update(language.as_bytes());
    
    if let Some(v) = voice {
        hasher.update(v.as_bytes());
    }
    
    let result = hasher.finalize();
    hex::encode(result)
}

pub async fn get_cached_audio(cache_key: &str) -> Result<Option<Vec<u8>>> {
    let cache_path = get_cache_path(cache_key);
    
    if cache_path.exists() {
        let data = fs::read(&cache_path).await?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

pub async fn cache_audio(cache_key: &str, audio_data: &[u8]) -> Result<()> {
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir).await?;
    
    let cache_path = get_cache_path(cache_key);
    fs::write(&cache_path, audio_data).await?;
    
    Ok(())
}

pub async fn clear_text_cache(text: &str, provider: &str, language: &str, voice: Option<&str>) -> Result<()> {
    let cache_key = generate_cache_key(text, provider, language, voice);
    let cache_path = get_cache_path(&cache_key);
    
    if cache_path.exists() {
        fs::remove_file(&cache_path).await?;
    }
    
    Ok(())
}

pub async fn clear_all_cache() -> Result<()> {
    let cache_dir = get_cache_dir();
    
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).await?;
    }
    
    Ok(())
}

pub async fn show_cache_stats() -> Result<()> {
    let cache_dir = get_cache_dir();
    
    if !cache_dir.exists() {
        println!("No cache directory found");
        return Ok(());
    }
    
    let mut total_size = 0u64;
    let mut file_count = 0u32;
    
    let mut entries = fs::read_dir(&cache_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if let Ok(metadata) = entry.metadata().await {
            if metadata.is_file() {
                total_size += metadata.len();
                file_count += 1;
            }
        }
    }
    
    let size_mb = total_size as f64 / (1024.0 * 1024.0);
    
    println!("Cache Statistics:");
    println!("  Files: {}", file_count);
    println!("  Total size: {:.2} MB", size_mb);
    println!("  Cache directory: {}", cache_dir.display());
    
    Ok(())
}

fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("tts-cli")
}

fn get_cache_path(cache_key: &str) -> PathBuf {
    get_cache_dir().join(format!("{}.audio", cache_key))
}