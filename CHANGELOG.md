# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-12-12

### Added
- Partial VAC PDF downloads by allowing specific OACI codes to be provided via `--oaci` flag
- List remote VACs API to show remotely available VACs and their local availability status
- Remove cache entry API to delete entries from cache database along with their PDF files
- PDF hash verification feature to ensure data integrity and detect corrupted/deleted files
- Configuration file support for default settings (database path and download directory)

### Changed
- Refactored project structure to separate CLI and library components

## [0.2.0] - 2025-12-12

### Added
- Support for custom database path via `--db-path` argument
- Support for custom download directory via `--download-dir` argument

## [0.1.0] - 2025-12-11

### Added
- Initial release of VAC Downloader

[Unreleased]: https://github.com/jcorbier/VAC-Downloader/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/jcorbier/VAC-Downloader/releases/tag/v0.3.0
[0.2.0]: https://github.com/jcorbier/VAC-Downloader/releases/tag/v0.2.0
[0.1.0]: https://github.com/jcorbier/VAC-Downloader/releases/tag/v0.1.0
