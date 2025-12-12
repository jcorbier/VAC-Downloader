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

use crate::models::VacEntry;
use rusqlite::{params, Connection, Result};
use std::path::Path;

/// SQLite database for caching VAC versions
pub struct VacDatabase {
    conn: Connection,
}

impl VacDatabase {
    /// Create or open the SQLite database
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vac_cache (
                oaci TEXT NOT NULL,
                vac_type TEXT NOT NULL,
                version TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                city TEXT NOT NULL,
                file_hash TEXT,
                last_updated DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (oaci, vac_type)
            )",
            [],
        )?;

        // Add file_hash column if it doesn't exist (for existing databases)
        let _ = conn.execute("ALTER TABLE vac_cache ADD COLUMN file_hash TEXT", []);

        Ok(VacDatabase { conn })
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM vac_cache", [], |row| row.get(0))?;
        Ok(count == 0)
    }

    /// Get cached version for a specific OACI code and type
    pub fn get_cached_version(&self, oaci: &str, vac_type: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT version FROM vac_cache WHERE oaci = ?1 AND vac_type = ?2",
            params![oaci, vac_type],
            |row| row.get(0),
        );

        match result {
            Ok(version) => Ok(Some(version)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Update or insert a VAC entry in the cache
    pub fn upsert_entry(&self, entry: &VacEntry) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO vac_cache 
             (oaci, vac_type, version, file_name, file_size, city, file_hash, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
            params![
                &entry.oaci,
                &entry.vac_type,
                &entry.version,
                &entry.file_name,
                &entry.file_size,
                &entry.city,
                &entry.file_hash,
            ],
        )?;
        Ok(())
    }

    /// Get cached hash for a specific OACI code and type
    pub fn get_cached_hash(&self, oaci: &str, vac_type: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT file_hash FROM vac_cache WHERE oaci = ?1 AND vac_type = ?2",
            params![oaci, vac_type],
            |row| row.get(0),
        );

        match result {
            Ok(hash) => Ok(hash),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get all cached entries
    pub fn get_all_entries(&self) -> Result<Vec<VacEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT oaci, vac_type, version, file_name, file_size, city, file_hash 
             FROM vac_cache 
             ORDER BY oaci",
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(VacEntry {
                oaci: row.get(0)?,
                vac_type: row.get(1)?,
                version: row.get(2)?,
                file_name: row.get(3)?,
                file_size: row.get(4)?,
                city: row.get(5)?,
                file_hash: row.get(6)?,
            })
        })?;

        entries.collect()
    }

    /// Check if a newer version is available
    pub fn needs_update(&self, entry: &VacEntry) -> Result<bool> {
        match self.get_cached_version(&entry.oaci, &entry.vac_type)? {
            Some(cached_version) => Ok(cached_version != entry.version),
            None => Ok(true), // Not in cache, needs download
        }
    }

    /// Delete an entry from the cache
    /// Returns the file name if the entry existed, None otherwise
    pub fn delete_entry(&self, oaci: &str) -> Result<Option<String>> {
        // First, get the file name before deleting
        let file_name = self.conn.query_row(
            "SELECT file_name FROM vac_cache WHERE oaci = ?1",
            params![oaci],
            |row| row.get(0),
        );

        match file_name {
            Ok(name) => {
                // Entry exists, delete it
                self.conn
                    .execute("DELETE FROM vac_cache WHERE oaci = ?1", params![oaci])?;
                Ok(Some(name))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get statistics about the cache
    pub fn get_stats(&self) -> Result<(i64, String, String)> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM vac_cache", [], |row| row.get(0))?;

        let oldest: String = self
            .conn
            .query_row("SELECT MIN(last_updated) FROM vac_cache", [], |row| {
                row.get(0)
            })
            .unwrap_or_else(|_| "N/A".to_string());

        let newest: String = self
            .conn
            .query_row("SELECT MAX(last_updated) FROM vac_cache", [], |row| {
                row.get(0)
            })
            .unwrap_or_else(|_| "N/A".to_string());

        Ok((count, oldest, newest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = VacDatabase::new(":memory:").unwrap();
        assert!(db.is_empty().unwrap());
    }

    #[test]
    fn test_upsert_and_retrieve() {
        let db = VacDatabase::new(":memory:").unwrap();

        let entry = VacEntry {
            oaci: "LFPG".to_string(),
            city: "Paris".to_string(),
            vac_type: "AD".to_string(),
            version: "1.0".to_string(),
            file_name: "LFPG_AD.pdf".to_string(),
            file_size: 1024,
            file_hash: Some("abc123".to_string()),
        };

        db.upsert_entry(&entry).unwrap();

        let version = db.get_cached_version("LFPG", "AD").unwrap();
        assert_eq!(version, Some("1.0".to_string()));

        assert!(!db.is_empty().unwrap());
    }

    #[test]
    fn test_delete_entry() {
        let db = VacDatabase::new(":memory:").unwrap();

        let entry = VacEntry {
            oaci: "LFPG".to_string(),
            city: "Paris".to_string(),
            vac_type: "AD".to_string(),
            version: "1.0".to_string(),
            file_name: "LFPG_AD.pdf".to_string(),
            file_size: 1024,
            file_hash: Some("abc123".to_string()),
        };

        // Insert entry
        db.upsert_entry(&entry).unwrap();
        assert!(!db.is_empty().unwrap());

        // Delete entry
        let result = db.delete_entry("LFPG").unwrap();
        assert_eq!(result, Some("LFPG_AD.pdf".to_string()));
        assert!(db.is_empty().unwrap());

        // Try to delete non-existent entry
        let result = db.delete_entry("LFPO").unwrap();
        assert_eq!(result, None);
    }
}
