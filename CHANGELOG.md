# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-10-12
### Removed
- Removed the unused `PreflightRejectionReason::MissingAccessControlRequestMethod` variant and the surrounding example branches.

### Documentation
- Synced the English README with the Korean source, expanding explanations and tables to stay aligned.
- Refreshed validation error descriptions to reflect the current configuration checks.

## [0.1.1] - 2025-10-10
### Changed
- Replaced the internal `Headers` map implementation with `std::collections::HashMap`, removing insertion-order guarantees in favor of lower overhead and simpler pooling.
- Swapped the global regex cache and benchmarks to use `std::sync::LazyLock`, eliminating the runtime dependency on `once_cell`.

## [0.1.0] - 2025-10-07
### Added
- Initial release of the `bunner-cors-rs` core library implementing a standards-compliant CORS decision engine.
- Builder-style `CorsOptions` configuration API with validation errors that prevent unsupported combinations.
- Origin matching system supporting exact strings, lists, regular expressions, custom predicates, and callbacks.
- Header utilities covering allowed headers, exposed headers, private network access, and timing allow origin responses.
- Integration-ready examples for Actix Web, Axum, and Hyper alongside thorough unit, property-based, and snapshot tests.

### Documentation
- Comprehensive README (English and Korean) describing configuration options, usage patterns, and integration notes.
- Expanded inline documentation comments across public APIs to improve discoverability for open-source users.
