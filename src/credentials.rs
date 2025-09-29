//! Secure storage for provider credentials.

use std::{fmt, path::PathBuf, sync::Arc};

use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};
use chrono::Utc;
use rand::RngCore;
use sqlx::Row;
use tokio::fs;

use crate::{
    database::{DbError, DbPool},
    error::AppError,
    providers::Provider,
};

const KEY_FILENAME: &str = "secret.key";
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 24;
const DEFAULT_HF_BASE_URL: &str = "https://api-inference.huggingface.co";

/// Manages provider credentials stored in the database with encryption.
#[derive(Clone)]
pub struct CredentialStore {
    pool: Arc<DbPool>,
    cipher: Arc<XChaCha20Poly1305>,
}

impl fmt::Debug for CredentialStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialStore").finish()
    }
}

impl CredentialStore {
    /// Initialises the store (generates and loads the master key).
    pub async fn new(pool: Arc<DbPool>) -> Result<Self, AppError> {
        let key_path = key_file_path()?;
        let key_bytes = load_or_create_key(&key_path).await?;
        let cipher = XChaCha20Poly1305::new(&key_bytes.into());
        Ok(Self {
            pool,
            cipher: Arc::new(cipher),
        })
    }

    /// Retrieves a decrypted credential for the given provider.
    pub async fn get_token(&self, provider: Provider) -> Result<Option<String>, AppError> {
        let record = sqlx::query_as::<_, (Vec<u8>, Vec<u8>)>(
            r#"SELECT nonce, ciphertext FROM provider_credentials WHERE provider = ?"#,
        )
        .bind(provider.as_str())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let Some(record) = record else {
            return Ok(None);
        };

        let nonce = XNonce::from_slice(&record.0);
        let plaintext = self
            .cipher
            .decrypt(nonce, record.1.as_slice())
            .map_err(|err| AppError::ApiError(format!("Failed to decrypt credential: {err}")))?;

        let token = String::from_utf8(plaintext)
            .map_err(|err| AppError::ApiError(format!("Invalid UTF-8 credential: {err}")))?;

        Ok(Some(token))
    }

    /// Inserts or updates a provider credential.
    pub async fn set_token(&self, provider: Provider, token: &str) -> Result<(), AppError> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, token.as_bytes())
            .map_err(|err| AppError::ApiError(format!("Failed to encrypt credential: {err}")))?;

        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"INSERT INTO provider_credentials (provider, nonce, ciphertext, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?)
               ON CONFLICT(provider) DO UPDATE SET
                   nonce = excluded.nonce,
                   ciphertext = excluded.ciphertext,
                   updated_at = excluded.updated_at"#,
        )
        .bind(provider.as_str())
        .bind(nonce_bytes.to_vec())
        .bind(ciphertext)
        .bind(now.clone())
        .bind(now)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let _ = result.rows_affected();

        Ok(())
    }

    /// Convenience helper for fetching base URLs (with defaults).
    pub fn resolve_base_url<'a>(&self, provider: Provider, configured: Option<&'a str>) -> &'a str {
        match provider {
            Provider::HuggingFace => configured.unwrap_or(DEFAULT_HF_BASE_URL),
            _ => configured.unwrap_or_default(),
        }
    }

    /// Removes a stored credential.
    pub async fn remove_token(&self, provider: Provider) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM provider_credentials WHERE provider = ?")
            .bind(provider.as_str())
            .execute(&*self.pool)
            .await
            .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Returns whether a token exists for the given provider in the store.
    pub async fn has_token(&self, provider: Provider) -> Result<bool, AppError> {
        let exists = sqlx::query("SELECT 1 FROM provider_credentials WHERE provider = ? LIMIT 1")
            .bind(provider.as_str())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        Ok(exists.is_some())
    }

    /// Lists providers that currently have stored credentials.
    pub async fn stored_providers(&self) -> Result<Vec<Provider>, AppError> {
        let rows = sqlx::query("SELECT provider FROM provider_credentials")
            .fetch_all(&*self.pool)
            .await
            .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let mut providers = Vec::new();
        for row in rows {
            let name: String = row.get(0);
            if let Some(provider) = Provider::from_alias(&name) {
                providers.push(provider);
            }
        }

        Ok(providers)
    }
}

fn key_file_path() -> Result<PathBuf, AppError> {
    let config_root = dirs::config_dir()
        .ok_or_else(|| AppError::ConfigError("Unable to determine config directory".into()))?
        .join("freegin-ai");
    Ok(config_root.join(KEY_FILENAME))
}

async fn load_or_create_key(path: &PathBuf) -> Result<[u8; KEY_SIZE], AppError> {
    if let Ok(bytes) = fs::read(path).await {
        if bytes.len() == KEY_SIZE {
            let mut key = [0u8; KEY_SIZE];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| AppError::ConfigError(format!("Failed to create config dir: {err}")))?;
    }

    let mut key = [0u8; KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    fs::write(path, &key)
        .await
        .map_err(|err| AppError::ConfigError(format!("Failed to write key file: {err}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)
            .map_err(|err| AppError::ConfigError(format!("Failed to set key perms: {err}")))?;
    }
    Ok(key)
}
