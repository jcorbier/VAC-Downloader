/*
 * Example usage of the list_vacs API
 *
 * This demonstrates how to use the new list_vacs() method to query
 * remotely available VACs and check their local availability.
 */

use vac_downloader::VacDownloader;

fn main() -> anyhow::Result<()> {
    // Create downloader instance
    let downloader = VacDownloader::new("vac_cache.db", "./downloads")?;

    println!("=== Example 1: List all remotely available VACs ===\n");

    // Get all remotely available VACs
    let all_vacs = downloader.list_vacs(None)?;

    println!("\n📊 Summary:");
    println!("   Total VACs available remotely: {}", all_vacs.len());

    let local_count = all_vacs.iter().filter(|v| v.available_locally).count();
    println!("   Available locally: {}", local_count);
    println!("   Remote only: {}\n", all_vacs.len() - local_count);

    // Show first 10 entries
    println!("First 10 entries:");
    for vac in all_vacs.iter().take(10) {
        let status = if vac.available_locally {
            "✓ Local"
        } else {
            "✗ Remote only"
        };
        println!(
            "   {} - {} ({}) - {}",
            vac.oaci, vac.city, vac.version, status
        );
    }

    println!("\n=== Example 2: Filter by specific OACI codes ===\n");

    // Filter by specific OACI codes
    let codes = vec!["LFPG".to_string(), "LFPO".to_string(), "LFPB".to_string()];
    let filtered_vacs = downloader.list_vacs(Some(&codes))?;

    println!("\n📊 Filtered results:");
    for vac in &filtered_vacs {
        let status = if vac.available_locally { "✓" } else { "✗" };
        println!(
            "   {} {} - {} ({}) - {} bytes",
            status, vac.oaci, vac.city, vac.version, vac.file_size
        );
    }

    Ok(())
}
