/*
 * Copyright (c) 2025 Jeremie Corbier
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Configuration structure for VAC Downloader
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    /// Path to the SQLite database file
    pub db_path: Option<String>,

    /// Directory where PDFs will be downloaded
    pub download_dir: Option<String>,
}

impl Config {
    /// Load configuration from the platform-specific config file
    ///
    /// Returns None if the config file doesn't exist or can't be read.
    /// Returns Some(Config) if the file exists and is valid TOML.
    pub fn load() -> Option<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return None;
        }

        let contents = fs::read_to_string(&config_path).ok()?;
        toml::from_str(&contents).ok()
    }

    /// Get the platform-specific configuration file path
    ///
    /// - Linux: ~/.config/vac-downloader/config.toml
    /// - macOS: ~/Library/Application Support/vac-downloader/config.toml
    /// - Windows: %APPDATA%\vac-downloader\config.toml
    fn get_config_path() -> Option<PathBuf> {
        let config_dir = dirs::config_dir()?;
        Some(config_dir.join("vac-downloader").join("config.toml"))
    }

    /// Get the configuration file path as a string for display purposes
    pub fn get_config_path_display() -> String {
        Self::get_config_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unable to determine config path".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_exists() {
        // Just verify we can get a config path
        let path = Config::get_config_path();
        assert!(path.is_some());
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.db_path.is_none());
        assert!(config.download_dir.is_none());
    }
}
