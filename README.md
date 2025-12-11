# VAC Downloader - Rust Module

A Rust module that fetches VAC (Visual Approach Charts) data from the SOFIA API, filters for airport (AD) entries, caches versions in SQLite, and downloads PDFs only when newer versions are available.

## Features

- ✅ **API Integration**: Fetches OACIS data with Hydra pagination
- ✅ **AD Filtering**: Only processes airport (AD) type entries
- ✅ **Version Caching**: SQLite database tracks downloaded versions
- ✅ **Smart Updates**: Downloads only when newer versions available
- ✅ **Dual Authentication**: Custom AUTH header + Basic Auth for PDFs
- ✅ **First Run Detection**: Downloads all PDFs if database is empty
- ✅ **Progress Reporting**: Detailed sync statistics

## Architecture

```
src/
├── lib.rs          # Module exports
├── main.rs         # CLI executable
├── models.rs       # Data structures (OACIS response, VAC entries)
├── auth.rs         # Authentication (SHA-512 + Basic Auth)
├── database.rs     # SQLite caching and version management
└── downloader.rs   # Main sync logic with API client
```

## Authentication

### Custom AUTH Header
```rust
// Generates: {"tokenUri": "<sha512_hash>"}
let auth = AuthGenerator::generate_auth_header("/api/v1/oacis", None);
```

### Basic Authentication (PDF Downloads)
```rust
// Generates: "Basic YXBpOkw0YjZQIWQ5K1l1aUc4LU0="
let basic = AuthGenerator::generate_basic_auth();
```

## Database Schema

```sql
CREATE TABLE vac_cache (
    oaci TEXT NOT NULL,
    vac_type TEXT NOT NULL,
    version TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    city TEXT NOT NULL,
    last_updated DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (oaci, vac_type)
);
```

## Usage

### As a Library

```rust
use vac_downloader::VacDownloader;

fn main() -> anyhow::Result<()> {
    let downloader = VacDownloader::new("vac_cache.db", "./downloads")?;
    let stats = downloader.sync()?;
    
    println!("Downloaded: {}", stats.downloaded);
    println!("Up to date: {}", stats.up_to_date);
    
    Ok(())
}
```

### As a CLI Tool

```bash
# Build
cargo build --release

# Run
cargo run --release

# Or use the binary directly
./target/release/vac_downloader
```

## Example Output

```
🛩️  VAC Downloader - Airport (AD) PDF Sync Tool

📦 First run detected - database is empty
   Will download ALL AD entries

🌐 Fetching OACIS data from API...
Fetching page 1 from OACIS API...
  Found 156 total AD entries so far
Fetching page 2 from OACIS API...
  Found 312 total AD entries so far
Total AD entries fetched: 312

🔍 Checking for updates...
  Downloading LFPG (LFPG_AD.pdf)...
  ✓ Saved to "./downloads/LFPG_AD.pdf" (1048576 bytes)
  Downloading LFPO (LFPO_AD.pdf)...
  ✓ Saved to "./downloads/LFPO_AD.pdf" (987654 bytes)
  ...

✅ Sync complete!
   Total entries: 312
   Up to date: 0
   Downloaded: 312
   Failed: 0
```

## Dependencies

- `reqwest` - HTTP client with blocking API
- `serde` / `serde_json` - JSON serialization
- `rusqlite` - SQLite database
- `sha2` - SHA-512 hashing
- `base64` - Base64 encoding
- `anyhow` - Error handling
- `tokio` - Async runtime (for reqwest)

## API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /api/v1/oacis` | Fetch VAC metadata (paginated) |
| `GET /api/v1/custom/file-path/{oaci}/{type}` | Download PDF file |

## Error Handling

The module uses `anyhow::Result` for comprehensive error handling:

- Network errors (timeouts, connection failures)
- API errors (non-200 status codes)
- Database errors (SQLite operations)
- File system errors (directory creation, file writes)

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_auth_generation
```

## License

MIT
