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

/// VAC Downloader - Airport (AD) PDF Sync Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the SQLite database file
    #[arg(short, long, default_value = "vac_cache.db")]
    db_path: String,

    /// Directory where PDFs will be downloaded
    #[arg(short = 'o', long, default_value = "./downloads")]
    download_dir: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🛩️  VAC Downloader - Airport (AD) PDF Sync Tool\n");

    // Create downloader
    let downloader = VacDownloader::new(&args.db_path, &args.download_dir)?;

    // Run sync
    let stats = downloader.sync()?;

    // Exit with error code if any downloads failed
    if stats.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
