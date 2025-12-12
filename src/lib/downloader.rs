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

use crate::{AuthGenerator, OacisResponse, VacDatabase, VacEntry};
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const API_BASE_URL: &str = "https://bo-prod-sofia-vac.sia-france.fr";
const OACIS_ENDPOINT: &str = "/api/v1/oacis";
const FILE_ENDPOINT: &str = "/api/v1/custom/file-path";
const CACHE_TTL_SECONDS: u64 = 600; // 10 minutes

/// Cached OACIS data with timestamp
struct CachedOacisData {
    entries: Vec<VacEntry>,
    fetched_at: Instant,
}

/// Main VAC downloader with caching and version management
pub struct VacDownloader {
    client: Client,
    database: VacDatabase,
    download_dir: PathBuf,
    oacis_cache: RefCell<Option<CachedOacisData>>,
}

impl VacDownloader {
    /// Create a new VAC downloader
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file
    /// * `download_dir` - Directory to save downloaded PDFs
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(db_path: P, download_dir: Q) -> Result<Self> {
        let database = VacDatabase::new(db_path).context("Failed to initialize database")?;

        let download_dir = download_dir.as_ref().to_path_buf();
        fs::create_dir_all(&download_dir).context("Failed to create download directory")?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(VacDownloader {
            client,
            database,
            download_dir,
            oacis_cache: RefCell::new(None),
        })
    }

    /// Calculate SHA-256 hash of a file
    fn calculate_file_hash(path: &Path) -> Result<String> {
        let mut file =
            fs::File::open(path).context(format!("Failed to open file for hashing: {:?}", path))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .context("Failed to read file for hashing")?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Fetch all OACIS entries from the API (with pagination and caching)
    fn fetch_oacis_data(&self) -> Result<Vec<VacEntry>> {
        // Check if we have valid cached data
        {
            let cache = self.oacis_cache.borrow();
            if let Some(cached) = cache.as_ref() {
                let age = cached.fetched_at.elapsed();
                if age < Duration::from_secs(CACHE_TTL_SECONDS) {
                    let remaining = Duration::from_secs(CACHE_TTL_SECONDS) - age;
                    println!(
                        "📦 Using cached OACIS data ({} entries, cache expires in {}s)",
                        cached.entries.len(),
                        remaining.as_secs()
                    );
                    return Ok(cached.entries.clone());
                } else {
                    println!(
                        "⏰ Cache expired (age: {}s), fetching fresh data",
                        age.as_secs()
                    );
                }
            }
        }

        // Cache miss or expired, fetch fresh data
        let mut all_entries = Vec::new();
        let mut page = 1;

        loop {
            let api_path = format!("{}?page={}", OACIS_ENDPOINT, page);
            let url = format!("{}{}", API_BASE_URL, api_path);
            let auth_header = AuthGenerator::generate_auth_header(&api_path, None);

            println!("Fetching page {} from OACIS API...", page);

            let response = self
                .client
                .get(&url)
                .header("AUTH", auth_header)
                .header("Content-Type", "application/json")
                .send()
                .context(format!("Failed to fetch OACIS page {}", page))?;

            if !response.status().is_success() {
                anyhow::bail!("API returned error status: {}", response.status());
            }

            let oacis_response: OacisResponse =
                response.json().context("Failed to parse OACIS response")?;

            // Extract AD entries from this page
            for entry in &oacis_response.members {
                let vac_entries = VacEntry::from_oacis_entry(entry);
                all_entries.extend(vac_entries);
            }

            println!("  Found {} total AD entries so far", all_entries.len());

            // Check if we've fetched all pages
            let items_per_page = oacis_response.members.len() as i32;
            if items_per_page == 0 || all_entries.len() >= oacis_response.total_items as usize {
                break;
            }

            page += 1;
        }

        println!("Total AD entries fetched: {}", all_entries.len());

        // Update cache
        *self.oacis_cache.borrow_mut() = Some(CachedOacisData {
            entries: all_entries.clone(),
            fetched_at: Instant::now(),
        });
        println!("💾 Cached OACIS data (TTL: {}s)", CACHE_TTL_SECONDS);

        Ok(all_entries)
    }

    /// Download a PDF file for a VAC entry and return the file hash
    fn download_pdf(&self, entry: &VacEntry) -> Result<(PathBuf, String)> {
        let api_path = format!("{}/{}/{}", FILE_ENDPOINT, entry.oaci, entry.vac_type);
        let url = format!("{}{}", API_BASE_URL, api_path);

        // Generate both auth headers
        let auth_header = AuthGenerator::generate_auth_header(&api_path, None);
        let basic_auth = AuthGenerator::generate_basic_auth();

        println!("  Downloading {} ({})...", entry.oaci, entry.file_name);

        let response = self
            .client
            .get(&url)
            .header("AUTH", auth_header)
            .header("Authorization", basic_auth)
            .send()
            .context(format!("Failed to download PDF for {}", entry.oaci))?;

        if !response.status().is_success() {
            anyhow::bail!("PDF download failed with status: {}", response.status());
        }

        let bytes = response.bytes().context("Failed to read PDF bytes")?;

        // Calculate hash of downloaded bytes
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());

        // Save to file
        let file_path = self.download_dir.join(&entry.file_name);
        fs::write(&file_path, bytes).context(format!("Failed to write PDF to {:?}", file_path))?;

        println!("  ✓ Saved to {:?} ({} bytes)", file_path, entry.file_size);

        Ok((file_path, hash))
    }

    /// Main sync operation: fetch, filter, cache, and download
    ///
    /// # Arguments
    /// * `oaci_filter` - Optional list of OACI codes to filter downloads. If None, all entries are processed.
    pub fn sync(&self, oaci_filter: Option<&[String]>) -> Result<SyncStats> {
        let mut stats = SyncStats::default();

        // Check if database is empty
        let is_first_run = self
            .database
            .is_empty()
            .context("Failed to check database status")?;

        if is_first_run {
            println!("📦 First run detected - database is empty");
            println!("   Will download ALL AD entries\n");
        } else {
            let (count, oldest, newest) = self.database.get_stats()?;
            println!("📊 Database contains {} cached entries", count);
            println!("   Oldest: {}", oldest);
            println!("   Newest: {}\n", newest);
        }

        // Fetch all OACIS data
        println!("🌐 Fetching OACIS data from API...");
        let mut entries = self.fetch_oacis_data()?;

        // Filter by OACI codes if specified
        if let Some(codes) = oaci_filter {
            let original_count = entries.len();
            let codes_upper: Vec<String> = codes.iter().map(|c| c.to_uppercase()).collect();
            entries.retain(|entry| codes_upper.contains(&entry.oaci.to_uppercase()));

            println!("\n🔍 Filtering by OACI codes: {}", codes_upper.join(", "));
            println!(
                "   Matched {} out of {} total entries",
                entries.len(),
                original_count
            );

            if entries.is_empty() {
                println!("\n⚠️  No entries found matching the specified OACI codes");
                return Ok(stats);
            }
        }

        stats.total_entries = entries.len();

        println!("\n🔍 Checking for updates...");

        // Process each entry
        for mut entry in entries {
            let needs_version_update = if is_first_run {
                true
            } else {
                self.database
                    .needs_update(&entry)
                    .context(format!("Failed to check update status for {}", entry.oaci))?
            };

            let mut needs_download = needs_version_update;

            // If no version update needed, verify file integrity
            if !needs_version_update && !is_first_run {
                let file_path = self.download_dir.join(&entry.file_name);

                if file_path.exists() {
                    // File exists, verify hash
                    match Self::calculate_file_hash(&file_path) {
                        Ok(current_hash) => {
                            if let Ok(Some(cached_hash)) =
                                self.database.get_cached_hash(&entry.oaci, &entry.vac_type)
                            {
                                if current_hash != cached_hash {
                                    println!("  ⚠️  Hash mismatch for {} - file corrupted, redownloading", entry.oaci);
                                    needs_download = true;
                                    stats.redownloaded_corrupted += 1;
                                } else {
                                    stats.verified += 1;
                                }
                            } else {
                                // No hash in database, calculate and store it
                                entry.file_hash = Some(current_hash);
                                let _ = self.database.upsert_entry(&entry);
                                stats.verified += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("  ✗ Failed to calculate hash for {}: {}", entry.oaci, e);
                            stats.verified += 1; // Count as verified even if hash calc failed
                        }
                    }
                } else {
                    // File missing, redownload
                    println!("  ⚠️  File missing for {} - redownloading", entry.oaci);
                    needs_download = true;
                    stats.redownloaded_corrupted += 1;
                }
            }

            if needs_download {
                stats.to_download += 1;

                // Download the PDF
                match self.download_pdf(&entry) {
                    Ok((_path, hash)) => {
                        // Update entry with hash
                        entry.file_hash = Some(hash);

                        // Update cache
                        self.database
                            .upsert_entry(&entry)
                            .context(format!("Failed to update cache for {}", entry.oaci))?;
                        stats.downloaded += 1;
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed to download {}: {}", entry.oaci, e);
                        stats.failed += 1;
                    }
                }
            } else if !needs_version_update {
                stats.up_to_date += 1;
            }
        }

        println!("\n✅ Sync complete!");
        println!("   Total entries: {}", stats.total_entries);
        println!("   Up to date: {}", stats.up_to_date);
        println!("   Verified: {}", stats.verified);
        println!("   Downloaded: {}", stats.downloaded);
        println!(
            "   Redownloaded (corrupted/missing): {}",
            stats.redownloaded_corrupted
        );
        println!("   Failed: {}", stats.failed);

        Ok(stats)
    }

    /// Get a list of all remotely available VACs with local availability status
    ///
    /// # Arguments
    /// * `oaci_filter` - Optional list of OACI codes to filter results. If None, all entries are returned.
    ///
    /// # Returns
    /// A vector of VacEntry containing remote VAC information and local availability
    pub fn list_vacs(&self, oaci_filter: Option<&[String]>) -> Result<Vec<VacEntry>> {
        println!("🌐 Fetching OACIS data from API...");
        let mut entries = self.fetch_oacis_data()?;

        // Filter by OACI codes if specified
        if let Some(codes) = oaci_filter {
            let original_count = entries.len();
            let codes_upper: Vec<String> = codes.iter().map(|c| c.to_uppercase()).collect();
            entries.retain(|entry| codes_upper.contains(&entry.oaci.to_uppercase()));

            println!("\n🔍 Filtering by OACI codes: {}", codes_upper.join(", "));
            println!(
                "   Matched {} out of {} total entries",
                entries.len(),
                original_count
            );

            if entries.is_empty() {
                println!("\n⚠️  No entries found matching the specified OACI codes");
                return Ok(entries);
            }
        }

        println!("\n🔍 Checking local availability...");

        // Check local availability for each entry
        for entry in &mut entries {
            entry.available_locally = self.database.has_entry(&entry.oaci).unwrap_or(false);
        }

        let local_count = entries.iter().filter(|e| e.available_locally).count();
        println!(
            "   {} out of {} entries are available locally",
            local_count,
            entries.len()
        );

        Ok(entries)
    }

    /// Delete a VAC entry from the cache and remove the PDF file
    ///
    /// # Arguments
    /// * `oaci` - OACI code of the entry to delete
    pub fn delete(&self, oaci: &str) -> Result<DeleteResult> {
        let mut result = DeleteResult {
            oaci: oaci.to_string(),
            database_deleted: false,
            file_deleted: false,
            file_name: None,
        };

        // Delete from database
        match self.database.delete_entry(oaci) {
            Ok(Some(file_name)) => {
                result.database_deleted = true;
                result.file_name = Some(file_name.clone());

                // Delete the PDF file
                let file_path = self.download_dir.join(&file_name);
                if file_path.exists() {
                    match fs::remove_file(&file_path) {
                        Ok(_) => {
                            result.file_deleted = true;
                            println!("✓ Deleted {} from database and filesystem", oaci);
                        }
                        Err(e) => {
                            eprintln!(
                                "✗ Deleted {} from database but failed to delete file: {}",
                                oaci, e
                            );
                        }
                    }
                } else {
                    println!(
                        "✓ Deleted {} from database (file was already missing)",
                        oaci
                    );
                }
            }
            Ok(None) => {
                println!("⚠️  Entry {} (AD) not found in database", oaci);
            }
            Err(e) => {
                anyhow::bail!("Failed to delete entry from database: {}", e);
            }
        }

        Ok(result)
    }
}

/// Statistics from a sync operation
#[derive(Debug, Default)]
pub struct SyncStats {
    pub total_entries: usize,
    pub to_download: usize,
    pub downloaded: usize,
    pub failed: usize,
    pub up_to_date: usize,
    pub verified: usize,
    pub redownloaded_corrupted: usize,
}

/// Result from a delete operation
#[derive(Debug)]
pub struct DeleteResult {
    pub oaci: String,
    pub database_deleted: bool,
    pub file_deleted: bool,
    pub file_name: Option<String>,
}
