# Southeast-Nacos

A Rust library for retrieving configuration from Alibaba Nacos with AWS KMS integration for secure password handling.

## Features

- Fetch configuration from Nacos server with robust error handling
- Automatic decryption of passwords encrypted with AWS KMS
- Strong type conversion with serde for type-safe configuration
- MD5 validation of received configuration
- Comprehensive error types for better debugging



## Installation

Add this to your Cargo.toml:

```bash
[dependencies]
southeast-nacos = "0.1.0"
```



## Usage

```Rust
use southeast_nacos::from_nacos;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MyConfig {
    database_url: String,
    api_key: String,
    timeout_seconds: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set required environment variables before using
    std::env::set_var("NACOS_ADDR", "nacos-server:8848");
    std::env::set_var("NACOS_GROUP", "DEFAULT_GROUP");
    std::env::set_var("NACOS_NAMESPACE", "public");
    std::env::set_var("NACOS_USERNAME", "nacos");
    std::env::set_var("NACOS_PASSWORD", "nacos");
    std::env::set_var("NACOS_DATA_ID", "my-application");
    
    // Fetch and deserialize configuration
    let config: MyConfig = from_nacos().await?;
    
    println!("Database URL: {}", config.database_url);
    println!("API Key: {}", config.api_key);
    println!("Timeout: {} seconds", config.timeout_seconds);
    
    Ok(())
}
```



## Required Environment Variables

The library requires the following environment variables to be set:

| Variable        | **Description**                                             |
| --------------- | ----------------------------------------------------------- |
| NACOS_ADDR      | Nacos server address (e.g., "nacos-server:8848")            |
| NACOS_GROUP     | Nacos configuration group (e.g., "DEFAULT_GROUP")           |
| NACOS_NAMESPACE | Nacos namespace (e.g., "SAS")                               |
| NACOS_USERNAME  | Username for Nacos authentication                           |
| NACOS_PASSWORD  | Password for Nacos authentication (can be encrypted)        |
| NACOS_DATA_ID   | Data ID for the configuration to retrieve                   |
| KMS_KEY_ID      | AWS KMS key ID (only required if using encrypted passwords) |



## Password Encryption

For enhanced security, passwords can be encrypted using AWS KMS. To use an encrypted password, format it as:

```bash
ENC(base64-encoded-encrypted-content)
```

The library will automatically detect this format and decrypt the password using the AWS KMS key specified in the `KMS_KEY_ID` environment variable.



## AWS KMS Integration

This library uses the AWS KMS service in the `ap-southeast-1` region by default. The encrypted content should be base64-encoded using standard encoding. If you encounter issues with decoding, you might need to modify the `get_blob` function to use `URL_SAFE` encoding instead.



## Error Handling

The library provides detailed error types through the NacosError enum, which helps diagnose issues with:

- Missing environment variables
- Nacos connection problems
- Configuration retrieval errors
- KMS decryption issues
- JSON parsing errors
- Base64 decoding failures
- UTF-8 conversion issues



## License

MIT