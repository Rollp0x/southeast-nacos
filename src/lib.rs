use serde::de::DeserializeOwned;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_kms as kms;
use base64::Engine;
use crypto::{digest::Digest, md5::Md5};
use kms::primitives::Blob;
use nacos_sdk::api::{
    config::{ConfigService, ConfigServiceBuilder},
    props::ClientProps,
};
use std::{env, fmt, error::Error};

#[derive(Debug)]
pub enum NacosError {
    EnvVarError(String),
    NacosConnectionError(String),
    NacosConfigError(String),
    KmsError(String),
    ConfigParseError(String),
    Base64DecodeError(String),
    Utf8Error(String),
}

impl fmt::Display for NacosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NacosError::EnvVarError(msg) => write!(f, "Environment variable error: {}", msg),
            NacosError::NacosConnectionError(msg) => write!(f, "Nacos connection error: {}", msg),
            NacosError::NacosConfigError(msg) => write!(f, "Nacos config error: {}", msg),
            NacosError::KmsError(msg) => write!(f, "AWS KMS error: {}", msg),
            NacosError::ConfigParseError(msg) => write!(f, "Config parsing error: {}", msg),
            NacosError::Base64DecodeError(msg) => write!(f, "Base64 decoding error: {}", msg),
            NacosError::Utf8Error(msg) => write!(f, "UTF-8 conversion error: {}", msg),
        }
    }
}

impl Error for NacosError {}

/// Get configuration from Nacos
pub async fn from_nacos<T: DeserializeOwned>() -> Result<T, NacosError> {
    // Read Nacos-related environment variables
    let nacos_addr = env::var("NACOS_ADDR")
        .map_err(|_| NacosError::EnvVarError("NACOS_ADDR not set".to_string()))?;
    let nacos_group = env::var("NACOS_GROUP")
        .map_err(|_| NacosError::EnvVarError("NACOS_GROUP not set".to_string()))?;
    let nacos_namespace = env::var("NACOS_NAMESPACE")
        .map_err(|_| NacosError::EnvVarError("NACOS_NAMESPACE not set".to_string()))?;
    let nacos_username = env::var("NACOS_USERNAME")
        .map_err(|_| NacosError::EnvVarError("NACOS_USERNAME not set".to_string()))?;
    let nacos_password = env::var("NACOS_PASSWORD")
        .map_err(|_| NacosError::EnvVarError("NACOS_PASSWORD not set".to_string()))?;
    let nacos_password = decrypt_password(&nacos_password).await?;
    
    let nacos_data_id = env::var("NACOS_DATA_ID")
        .map_err(|_| NacosError::EnvVarError("NACOS_DATA_ID not set".to_string()))?;
    
    // Remove http/https prefix
    let nacos_addr = nacos_addr.trim_start_matches("http://").trim_start_matches("https://").to_string();
    
    // Connect to Nacos to get configuration
    let client_props = ClientProps::new()
        .server_addr(&nacos_addr)
        .namespace(&nacos_namespace)
        .env_first(false)
        .auth_username(&nacos_username)
        .auth_password(&nacos_password);
    
    // nacos client
    let config_services = ConfigServiceBuilder::new(client_props)
        .enable_auth_plugin_http()
        .build()
        .map_err(|e| NacosError::NacosConnectionError(format!("Failed to create ConfigServiceBuilder for nacos: {}: {}", nacos_addr, e)))?;
    
    // Get configuration
    let resp = config_services
        .get_config(nacos_data_id.clone(), nacos_group.clone())
        .await
        .map_err(|e| NacosError::NacosConfigError(format!("Failed to get config from nacos, data_id: {}, group: {}: {}", nacos_data_id, nacos_group, e)))?;
    
    // check config
    let mut hasher = Md5::new();
    let content = resp.content();
    hasher.input_str(content);
    let md5 = hasher.result_str();
    
    if resp.namespace() != &nacos_namespace {
        return Err(NacosError::NacosConfigError("nacos_namespace unmatched".to_string()));
    }
    if resp.data_id() != &nacos_data_id {
        return Err(NacosError::NacosConfigError("nacos_data_id unmatched".to_string()));
    }
    if resp.group() != &nacos_group {
        return Err(NacosError::NacosConfigError("nacos_group unmatched".to_string()));
    }
    if resp.md5() != &md5 {
        return Err(NacosError::NacosConfigError("ConfigResponse md5 unmatched".to_string()));
    }
    
    // Return the configuration file
    serde_json::from_str::<T>(content)
        .map_err(|e| NacosError::ConfigParseError(format!("Failed to parse config from nacos: {}: {}", content, e)))
}

/// Decrypt password if it is encrypted
pub async fn decrypt_password(password: &str) -> Result<String, NacosError> {
    if password.starts_with("ENC(") {
        let key = env::var("KMS_KEY_ID")
            .map_err(|_| NacosError::EnvVarError("KMS_KEY_ID not set".to_string()))?;
        let raw_password = password.trim_start_matches("ENC(").trim_end_matches(')');
        let blob = get_blob(raw_password)?;
        let kms_client = get_kms_client().await;
        decrypt_blob(&kms_client, &key, blob).await
    } else {
        // Return non-encrypted password directly
        Ok(password.to_string())
    }
}
    
/// Get KMS client, the region is fixed
async fn get_kms_client() -> kms::Client {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    kms::Client::new(&config)
}

fn get_blob(raw_password: &str) -> Result<Blob, NacosError> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(raw_password)
        .map_err(|e| NacosError::Base64DecodeError(format!("Failed to decode base64: {}: {}", raw_password, e)))?;
    Ok(Blob::new(raw))
}

async fn decrypt_blob(client: &kms::Client, key: &str, blob: Blob) -> Result<String, NacosError> {
    let resp = client
        .decrypt()
        .key_id(key)
        .ciphertext_blob(blob)
        .send()
        .await
        .map_err(|e| NacosError::KmsError(format!("Failed to decrypt blob from kms: {}", e)))?;
    
    let inner = resp.plaintext
        .ok_or_else(|| NacosError::KmsError("Failed to get plaintext from kms's response".to_string()))?;
    
    let bytes = inner.as_ref();
    String::from_utf8(bytes.to_vec())
        .map_err(|e| NacosError::Utf8Error(format!("Could not convert to UTF-8: {}", e)))
}