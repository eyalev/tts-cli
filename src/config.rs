use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_provider: String,
    pub default_language: String,
    pub default_voice: Option<String>,
    pub cache_enabled: bool,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub voice_mapping: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        providers.insert("gcloud".to_string(), ProviderConfig {
            enabled: true,
            api_key: None,
            endpoint: None,
            voice_mapping: HashMap::from([
                ("en-US".to_string(), "en-US-Wavenet-D".to_string()),
                ("es-ES".to_string(), "es-ES-Wavenet-C".to_string()),
                ("fr-FR".to_string(), "fr-FR-Wavenet-D".to_string()),
                ("de-DE".to_string(), "de-DE-Wavenet-D".to_string()),
            ]),
        });
        
        providers.insert("espeak".to_string(), ProviderConfig {
            enabled: true,
            api_key: None,
            endpoint: None,
            voice_mapping: HashMap::new(),
        });
        
        providers.insert("festival".to_string(), ProviderConfig {
            enabled: true,
            api_key: None,
            endpoint: None,
            voice_mapping: HashMap::new(),
        });
        
        providers.insert("say".to_string(), ProviderConfig {
            enabled: true,
            api_key: None,
            endpoint: None,
            voice_mapping: HashMap::new(),
        });
        
        Config {
            default_provider: "gcloud".to_string(),
            default_language: "en-US".to_string(),
            default_voice: None,
            cache_enabled: true,
            providers,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = get_config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save().await?;
            Ok(config)
        }
    }
    
    pub async fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        let config_dir = config_path.parent().unwrap();
        
        fs::create_dir_all(config_dir).await?;
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content).await?;
        
        Ok(())
    }
    
    pub fn get_provider_config(&self, provider: &str) -> Option<&ProviderConfig> {
        self.providers.get(provider)
    }
    
    pub fn get_voice_for_language(&self, provider: &str, language: &str) -> Option<String> {
        self.providers
            .get(provider)?
            .voice_mapping
            .get(language)
            .cloned()
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("tts-cli")
        .join("config.json")
}