# Changelog

All notable changes to the Southeast-Nacos project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-03-31

### Added

- Initial release of the Southeast-Nacos library
- Support for retrieving configuration from Alibaba Nacos servers
- Built-in AWS KMS integration for decrypting secure passwords
- Support for encrypted passwords using `ENC(...)` format
- MD5 validation of received configuration
- Type-safe deserialization with serde
- Comprehensive error handling with custom `NacosError` enum
- Environment variable based configuration
- Notes
- Using AWS KMS in the `ap-southeast-1` region by default
- Standard Base64 encoding for KMS encrypted content



### Notes

- Using AWS KMS in the ap-southeast-1 region by default
- Standard Base64 encoding for KMS encrypted content