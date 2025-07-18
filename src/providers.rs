use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsProvider {
    pub name: String,
    pub description: String,
    pub available: bool,
}

pub fn list_providers() {
    let providers = get_available_providers();
    println!("Available TTS providers:");
    for provider in providers {
        let status = if provider.available { "✓" } else { "✗" };
        println!("  {} {} - {}", status, provider.name, provider.description);
    }
}

pub fn get_available_providers() -> Vec<TtsProvider> {
    vec![
        TtsProvider {
            name: "gcloud".to_string(),
            description: "Google Cloud Text-to-Speech API".to_string(),
            available: check_gcloud_availability(),
        },
        TtsProvider {
            name: "espeak".to_string(),
            description: "eSpeak TTS engine".to_string(),
            available: check_espeak_availability(),
        },
        TtsProvider {
            name: "festival".to_string(),
            description: "Festival TTS engine".to_string(),
            available: check_festival_availability(),
        },
        TtsProvider {
            name: "say".to_string(),
            description: "macOS built-in TTS".to_string(),
            available: check_say_availability(),
        },
    ]
}

pub async fn synthesize_text(
    text: &str,
    provider: &str,
    language: &str,
    voice: Option<&str>,
) -> Result<Vec<u8>> {
    match provider {
        "gcloud" => synthesize_gcloud(text, language, voice).await,
        "espeak" => synthesize_espeak(text, language, voice).await,
        "festival" => synthesize_festival(text, language, voice).await,
        "say" => synthesize_say(text, language, voice).await,
        _ => Err(anyhow!("Unknown provider: {}", provider)),
    }
}

async fn synthesize_gcloud(text: &str, language: &str, voice: Option<&str>) -> Result<Vec<u8>> {
    use serde_json::json;
    
    let voice_name = voice.unwrap_or_else(|| match language {
        "en-US" => "en-US-Wavenet-D",
        "es-ES" => "es-ES-Wavenet-C",
        "fr-FR" => "fr-FR-Wavenet-D",
        "de-DE" => "de-DE-Wavenet-D",
        _ => "en-US-Wavenet-D",
    });

    let request_body = json!({
        "input": {
            "text": text
        },
        "voice": {
            "languageCode": language,
            "name": voice_name,
            "ssmlGender": "NEUTRAL"
        },
        "audioConfig": {
            "audioEncoding": "MP3",
            "sampleRateHertz": 22050,
            "speakingRate": 1.0,
            "pitch": 0.0,
            "volumeGainDb": 0.0
        }
    });

    let client = reqwest::Client::new();
    let token = get_gcloud_token().await?;
    
    let response = client
        .post("https://texttospeech.googleapis.com/v1/text:synthesize")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Google Cloud TTS API error: {}", error_text));
    }

    let response_json: serde_json::Value = response.json().await?;
    let audio_content = response_json["audioContent"]
        .as_str()
        .ok_or_else(|| anyhow!("No audioContent in response"))?;
    
    let audio_bytes = base64::engine::general_purpose::STANDARD.decode(audio_content)?;
    Ok(audio_bytes)
}

async fn get_gcloud_token() -> Result<String> {
    let output = Command::new("gcloud")
        .args(&["auth", "print-access-token"])
        .output()
        .map_err(|e| anyhow!("gcloud command not found. Please install Google Cloud SDK and run 'gcloud auth application-default login': {}", e))?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get gcloud access token. Please run 'gcloud auth application-default login': {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

async fn synthesize_espeak(text: &str, language: &str, _voice: Option<&str>) -> Result<Vec<u8>> {
    let lang_code = match language {
        "en-US" | "en" => "en",
        "es-ES" | "es" => "es",
        "fr-FR" | "fr" => "fr",
        "de-DE" | "de" => "de",
        _ => "en",
    };

    let output = Command::new("espeak")
        .arg("-v")
        .arg(lang_code)
        .arg("--stdout")
        .arg(text)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("espeak command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(output.stdout)
}

async fn synthesize_festival(text: &str, _language: &str, _voice: Option<&str>) -> Result<Vec<u8>> {
    let temp_file = std::env::temp_dir().join("tts_temp.wav");
    
    let output = Command::new("festival")
        .arg("--tts")
        .arg("--otype")
        .arg("wav")
        .arg("--output")
        .arg(&temp_file)
        .arg(text)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("festival command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let audio_data = std::fs::read(&temp_file)?;
    let _ = std::fs::remove_file(&temp_file);
    
    Ok(audio_data)
}

async fn synthesize_say(text: &str, _language: &str, voice: Option<&str>) -> Result<Vec<u8>> {
    let temp_file = std::env::temp_dir().join("tts_temp.aiff");
    
    let mut cmd = Command::new("say");
    cmd.arg("-o").arg(&temp_file);
    
    if let Some(v) = voice {
        cmd.arg("-v").arg(v);
    }
    
    cmd.arg(text);
    
    let output = cmd.output()?;

    if !output.status.success() {
        return Err(anyhow!("say command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let audio_data = std::fs::read(&temp_file)?;
    let _ = std::fs::remove_file(&temp_file);
    
    Ok(audio_data)
}

fn check_gcloud_availability() -> bool {
    std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok()
        || std::env::var("GCLOUD_PROJECT").is_ok()
        || Command::new("gcloud").arg("--version").output().is_ok()
}

fn check_espeak_availability() -> bool {
    Command::new("espeak").arg("--version").output().is_ok()
}

fn check_festival_availability() -> bool {
    Command::new("festival").arg("--version").output().is_ok()
}

fn check_say_availability() -> bool {
    Command::new("say").arg("--version").output().is_ok()
}