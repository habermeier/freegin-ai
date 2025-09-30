//! Configuration management for the application.
//!
//! Exposes strongly typed structures backed by the `config` crate so the
//! service can load settings from user configuration directories or project
//! overrides alongside environment variables.

use config::{Config, ConfigError, Environment, File};
use dirs::{config_dir, data_dir, home_dir};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The main application configuration structure.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// Server configuration (host, port).
    pub server: ServerConfig,
    /// Database configuration.
    pub database: DatabaseConfig,
    /// Configuration for AI providers.
    pub providers: ProvidersConfig,
}

/// Server-specific configuration.
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// The host address to bind the server to.
    pub host: String,
    /// The port to listen on.
    pub port: u16,
}

/// Database-specific configuration.
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
}

/// Container for all AI provider configurations.
#[derive(Debug, Deserialize)]
pub struct ProvidersConfig {
    /// OpenAI provider settings (optional).
    #[serde(default)]
    pub openai: Option<ProviderDetails>,
    /// Google provider settings (optional).
    #[serde(default)]
    pub google: Option<ProviderDetails>,
    /// Hugging Face provider settings (optional).
    #[serde(default, rename = "hugging_face")]
    pub hugging_face: Option<ProviderDetails>,
    /// Anthropic provider settings (optional).
    #[serde(default)]
    pub anthropic: Option<ProviderDetails>,
    /// Cohere provider settings (optional).
    #[serde(default)]
    pub cohere: Option<ProviderDetails>,
    /// Groq provider settings (optional).
    #[serde(default)]
    pub groq: Option<ProviderDetails>,
    /// DeepSeek provider settings (optional).
    #[serde(default)]
    pub deepseek: Option<ProviderDetails>,
    /// Together AI provider settings (optional).
    #[serde(default)]
    pub together: Option<ProviderDetails>,
}

/// A generic structure for a provider's API key and base URL.
#[derive(Debug, Deserialize)]
pub struct ProviderDetails {
    /// The API key for the provider.
    pub api_key: String,
    /// The base URL for the provider's API.
    pub api_base_url: String,
}

impl AppConfig {
    /// Loads the application configuration.
    ///
    /// Searches the user's configuration directories first, then project-local
    /// overrides, and finally allows environment variables prefixed with
    /// `APP__` to override nested values.
    pub fn load() -> Result<Self, ConfigError> {
        let _dotenv_path = dotenvy::dotenv();

        let mut builder = Config::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8080)?
            .set_default("database.url", default_database_url())?;

        for path in candidate_config_files() {
            builder = builder.add_source(File::from(path));
        }

        let settings = builder
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let mut config: AppConfig = settings.try_deserialize()?;
        if config.database.url.trim().is_empty() {
            config.database.url = default_database_url();
        }

        config.database.url = normalize_database_url(&config.database.url)?;

        Ok(config)
    }
}

fn candidate_config_files() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(dir) = config_dir() {
        let path = dir.join("freegin-ai").join("config.toml");
        if path.exists() {
            paths.push(path);
        }
    }

    if let Some(home) = home_dir() {
        let legacy = home.join(".freegin-ai").join("config.toml");
        if legacy.exists() {
            paths.push(legacy);
        }
    }

    let project_override = Path::new("freegin-ai.toml");
    if project_override.exists() {
        paths.push(project_override.to_path_buf());
    }

    let secrets_path = Path::new(".secrets/app.toml");
    if secrets_path.exists() {
        paths.push(secrets_path.to_path_buf());
    }

    paths
}

fn default_database_url() -> String {
    format!("sqlite://{}", default_database_path().display())
}

fn normalize_database_url(url: &str) -> Result<String, ConfigError> {
    if !url.starts_with("sqlite:") {
        return Ok(url.to_string());
    }

    let remainder = &url["sqlite:".len()..];
    if remainder == ":memory:" || remainder.starts_with("memory") {
        return Ok(url.to_string());
    }

    let default_dir = default_data_dir();
    let mut path = if remainder.is_empty() {
        default_database_path()
    } else if remainder.starts_with("///") {
        PathBuf::from(&remainder[3..])
    } else if remainder.starts_with("//") {
        PathBuf::from(&remainder[2..])
    } else if remainder.starts_with('/') {
        PathBuf::from(remainder)
    } else {
        PathBuf::from(remainder)
    };

    if !path.is_absolute() {
        path = default_dir.join(path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            ConfigError::Message(format!(
                "Failed to create database directory {}: {err}",
                parent.display()
            ))
        })?;
    }

    Ok(format!("sqlite://{}", path.display()))
}

fn default_database_path() -> PathBuf {
    default_data_dir().join("app.db")
}

fn default_data_dir() -> PathBuf {
    data_dir()
        .or_else(home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("freegin-ai")
}
