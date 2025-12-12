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

use anyhow::Result;
use clap::Parser;
use vac_downloader::VacDownloader;

mod config;
use config::Config;

/// VAC Downloader - Airport (AD) PDF Sync Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the SQLite database file
    #[arg(short, long)]
    db_path: Option<String>,

    /// Directory where PDFs will be downloaded
    #[arg(short = 'o', long)]
    download_dir: Option<String>,

    /// OACI codes to download (if not specified, all entries will be synced)
    #[arg(short = 'c', long = "oaci", value_name = "CODE", value_delimiter = ',')]
    oaci_codes: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🛩️  VAC Downloader - Airport (AD) PDF Sync Tool\n");

    // Load configuration from file (if exists)
    let config = Config::load();

    // Merge config with CLI args (CLI takes precedence)
    // Priority: CLI args > config file > defaults
    let db_path = args
        .db_path
        .or_else(|| config.as_ref().and_then(|c| c.db_path.clone()))
        .unwrap_or_else(|| "vac_cache.db".to_string());

    let download_dir = args
        .download_dir
        .or_else(|| config.as_ref().and_then(|c| c.download_dir.clone()))
        .unwrap_or_else(|| "./downloads".to_string());

    // Show configuration source
    if config.is_some() {
        println!(
            "📝 Loaded configuration from: {}",
            Config::get_config_path_display()
        );
    }
    println!("📂 Database: {}", db_path);
    println!("📥 Download directory: {}", download_dir);

    if !args.oaci_codes.is_empty() {
        println!("🎯 OACI filter: {}", args.oaci_codes.join(", "));
    }
    println!();

    // Create downloader
    let downloader = VacDownloader::new(&db_path, &download_dir)?;

    // Run sync with optional OACI filter
    let oaci_filter = if args.oaci_codes.is_empty() {
        None
    } else {
        Some(args.oaci_codes.as_slice())
    };
    let stats = downloader.sync(oaci_filter)?;

    // Exit with error code if any downloads failed
    if stats.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
