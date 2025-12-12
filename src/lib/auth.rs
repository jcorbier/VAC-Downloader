/*
 * Copyright (c) 2025 Jeremie Corbier
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the “Software”), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use sha2::{Digest, Sha512};

const SHARE_SECRET: &str = "Y9Q3Ve72nN3PnTXmEtKnS4sggmdsigRMWH9kCDGHpCHyenFKKGhDq5vgBWZ4";
const BASIC_AUTH_USER: &str = "api";
const BASIC_AUTH_PASS: &str = "L4b6P!d9+YuiG8-M";

/// Generates authentication headers for API requests
pub struct AuthGenerator;

impl AuthGenerator {
    /// Generate custom AUTH header for API requests
    ///
    /// # Arguments
    /// * `api_path` - The API path (e.g., "/api/v1/oacis")
    /// * `request_body` - Optional request body for POST/PUT requests
    ///
    /// # Returns
    /// Base64-encoded JSON with SHA-512 hashed token
    pub fn generate_auth_header(api_path: &str, request_body: Option<&str>) -> String {
        // Step 1: Concatenate secret + path
        let combined = format!("{}{}", SHARE_SECRET, api_path);

        // Step 2: Generate SHA-512 hash
        let mut hasher = Sha512::new();
        hasher.update(combined.as_bytes());
        let token_uri = format!("{:x}", hasher.finalize());

        // Step 3: Create JSON object
        let mut auth_obj = json!({
            "tokenUri": token_uri
        });

        // Add tokenParams if request body exists
        if let Some(body) = request_body {
            let mut body_hasher = Sha512::new();
            body_hasher.update(body.as_bytes());
            let token_params = format!("{:x}", body_hasher.finalize());
            auth_obj["tokenParams"] = json!(token_params);
        }

        // Step 4: Base64 encode the JSON
        let json_str = auth_obj.to_string();
        general_purpose::STANDARD.encode(json_str.as_bytes())
    }

    /// Generate Basic Authentication header for PDF downloads
    ///
    /// # Returns
    /// Base64-encoded "api:password" string
    pub fn generate_basic_auth() -> String {
        let credentials = format!("{}:{}", BASIC_AUTH_USER, BASIC_AUTH_PASS);
        format!(
            "Basic {}",
            general_purpose::STANDARD.encode(credentials.as_bytes())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_generation() {
        let auth = AuthGenerator::generate_auth_header("/api/v1/configs", None);
        assert!(!auth.is_empty());

        // Should be valid base64
        let decoded = general_purpose::STANDARD.decode(&auth);
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_basic_auth() {
        let auth = AuthGenerator::generate_basic_auth();
        assert!(auth.starts_with("Basic "));
    }
}
