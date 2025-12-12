# VAC Downloader - Rust Module

A Rust module that fetches French VAC (Visual Approach Charts) data from the SOFIA API, filters for airport (AD) entries, caches versions in SQLite, and downloads PDFs only when newer versions are available.

## Features

- ✅ **API Integration**: Fetches VAC data from SOFIA API
- ✅ **AD Filtering**: Only processes airport (AD) type entries
- ✅ **Partial Downloads**: Filter by OACI codes to download specific airports
- ✅ **Version Caching**: SQLite database tracks downloaded versions
- ✅ **Smart Updates**: Downloads only when newer versions available
- ✅ **Progress Reporting**: Detailed sync statistics

## Code Structure

```
src/
├── cli/
│   ├── main.rs       # CLI executable entry point
│   └── config.rs     # Configuration file handling
└── lib/
    ├── lib.rs        # Library module exports
    ├── models.rs     # Data structures (OACIS response, VAC entries)
    ├── auth.rs       # Authentication (SHA-512 + Basic Auth)
    ├── database.rs   # SQLite caching and version management
    └── downloader.rs # Main sync logic with API client
```

## Usage

### As a Library

```rust
use vac_downloader::VacDownloader;

fn main() -> anyhow::Result<()> {
    let downloader = VacDownloader::new("vac_cache.db", "./downloads")?;

    // Download all entries
    let stats = downloader.sync(None)?;

    // Or download specific OACI codes
    let oaci_codes = vec!["LFPG".to_string(), "LFPO".to_string()];
    let stats = downloader.sync(Some(&oaci_codes))?;

    println!("Downloaded: {}", stats.downloaded);
    println!("Up to date: {}", stats.up_to_date);

    Ok(())
}
```

### As a CLI Tool

```bash
# Build
cargo build --release

# Run with default settings (db: vac_cache.db, downloads: ./downloads)
cargo run --release

# Specify custom database path
cargo run --release -- --db-path /path/to/custom.db

# Specify custom download directory
cargo run --release -- --download-dir /path/to/downloads

# Specify both
cargo run --release -- -d /path/to/custom.db -o /path/to/downloads

# Download specific airports by OACI code
cargo run --release -- --oaci LFPG
cargo run --release -- --oaci LFPG,LFPO,LFPB

# Combine with custom paths
cargo run --release -- -d custom.db -o ./pdfs --oaci LFPG

# View help
cargo run --release -- --help

# Or use the binary directly
./target/release/vac_downloader --db-path custom.db --download-dir ./pdfs
```

#### Command-Line Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--db-path` | `-d` | `vac_cache.db` | Path to the SQLite database file |
| `--download-dir` | `-o` | `./downloads` | Directory where PDFs will be downloaded |
| `--oaci` | `-c` | - | OACI codes to download (can specify multiple, separated by commas) |
| `--help` | `-h` | - | Print help information |
| `--version` | `-V` | - | Print version information |

#### Configuration File

You can create a configuration file to set default values for the database path and download directory. Command-line arguments will override these settings.

The configuration file is located at:

- **Linux**: `~/.config/vac-downloader/config.toml`
- **macOS**: `~/Library/Application Support/vac-downloader/config.toml`
- **Windows**: `%APPDATA%\vac-downloader\config.toml`

It has the following format:

```toml
# Path to the SQLite database file
db_path = "/var/lib/vac/cache.db"

# Directory where PDFs will be downloaded
download_dir = "/var/lib/vac/pdfs"
```

See [config.toml.example](config.toml.example) for a complete example with documentation.

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
- `clap` - Command-line argument parsing
- `toml` - TOML configuration file parsing
- `dirs` - Cross-platform config directory detection

## Architecture

### SOFIA API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /api/v1/oacis` | Fetch VAC metadata (paginated) |
| `GET /api/v1/custom/file-path/{oaci}/{type}` | Download PDF file |

#### Authentication

##### Custom AUTH Header
```rust
// Generates: {"tokenUri": "<sha512_hash>"}
let auth = AuthGenerator::generate_auth_header("/api/v1/oacis", None);
```

##### Basic Authentication (PDF Downloads)
```rust
// Generates: "Basic YXBpOkw0YjZQIWQ5K1l1aUc4LU0="
let basic = AuthGenerator::generate_basic_auth();
```

### Cache Database Schema

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

### Error Handling

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
