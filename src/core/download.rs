//! Download manager with progress tracking

use anyhow::{Context, Result};
use log::info;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Represents the state of a download
#[derive(Clone, Debug)]
pub struct DownloadState {
    pub downloaded: u64,
    pub total: u64,
    pub speed: f64, // bytes per second
    #[allow(dead_code)]
    pub is_paused: bool,
    #[allow(dead_code)]
    pub is_cancelled: bool,
}

/// Fetch the latest Arch Linux ISO information
pub async fn fetch_arch_iso_info() -> Result<(String, String)> {
    info!("Fetching Arch Linux ISO information...");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client")?;

    // Use LeaseWeb mirror (same as bash script)
    let base_url = "https://ftp.energotel.sk/pub/linux/arch/iso/latest/";
    let html = client
        .get(base_url)
        .send()
        .await
        .context("Failed to fetch ISO listing")?
        .text()
        .await
        .context("Failed to read response body")?;

    // Find ISO filename using regex pattern matching
    // Pattern: archlinux-YYYY.MM.DD-x86_64.iso
    let re = Regex::new(r"archlinux-\d{4}\.\d{2}\.\d{2}-x86_64\.iso")?;

    let iso_name = re
        .find(&html)
        .map(|m| m.as_str().to_string())
        .context("Could not detect ISO filename in mirror listing")?;

    // Construct download URL
    let download_url = format!("{}{}", base_url, iso_name);

    info!("Found ISO: {} at {}", iso_name, download_url);
    Ok((iso_name, download_url))
}

/// Download a file with progress tracking
pub async fn download_file<F>(
    url: String,
    dest_path: String,
    progress_callback: F,
    pause_flag: Arc<AtomicBool>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<()>
where
    F: Fn(DownloadState) + Send + 'static,
{
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    info!("Starting download from {} to {}", url, dest_path);

    // Use connection timeout only - no overall timeout since we're streaming
    // We handle cancellation manually via the cancel_flag
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to download server")?;
    let total_size = response.content_length().unwrap_or(0);

    info!(
        "Download size: {} bytes ({:.2} MB)",
        total_size,
        total_size as f64 / 1024.0 / 1024.0
    );

    let mut file = tokio::fs::File::create(&dest_path)
        .await
        .context("Failed to create destination file")?;
    let mut stream = response.bytes_stream();

    let mut downloaded: u64 = 0;
    let mut last_update = Instant::now();
    let mut last_downloaded = 0u64;
    let update_interval = Duration::from_millis(100);

    // For speed averaging to reduce fluctuations
    let mut speed_samples: Vec<f64> = Vec::with_capacity(20);
    let max_samples = 20; // Average over last 20 samples (2 seconds)

    while let Some(chunk_result) = stream.next().await {
        // Check if cancelled
        if cancel_flag.load(Ordering::Relaxed) {
            info!("Download cancelled");
            drop(file);
            let _ = tokio::fs::remove_file(&dest_path).await;
            anyhow::bail!("Download cancelled");
        }

        // Handle pause
        while pause_flag.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if cancel_flag.load(Ordering::Relaxed) {
                info!("Download cancelled while paused");
                drop(file);
                let _ = tokio::fs::remove_file(&dest_path).await;
                anyhow::bail!("Download cancelled");
            }
        }

        let chunk = chunk_result.context("Error receiving data chunk")?;

        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        // Update progress
        let now = Instant::now();
        if now.duration_since(last_update) >= update_interval {
            let elapsed = now.duration_since(last_update).as_secs_f64();
            let bytes_since_update = downloaded - last_downloaded;
            let instant_speed = bytes_since_update as f64 / elapsed;

            // Add to speed samples for averaging
            speed_samples.push(instant_speed);
            if speed_samples.len() > max_samples {
                speed_samples.remove(0);
            }

            // Calculate average speed
            let avg_speed = if !speed_samples.is_empty() {
                speed_samples.iter().sum::<f64>() / speed_samples.len() as f64
            } else {
                instant_speed
            };

            let state = DownloadState {
                downloaded,
                total: total_size,
                speed: avg_speed,
                is_paused: pause_flag.load(Ordering::Relaxed),
                is_cancelled: cancel_flag.load(Ordering::Relaxed),
            };

            progress_callback(state);

            last_update = now;
            last_downloaded = downloaded;
        }
    }

    file.flush().await?;
    drop(file);

    // Final update
    let state = DownloadState {
        downloaded,
        total: total_size,
        speed: 0.0,
        is_paused: false,
        is_cancelled: false,
    };
    progress_callback(state);

    info!("Download completed: {}", dest_path);
    Ok(())
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format speed to human readable string
pub fn format_speed(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec as u64))
}

/// Format time remaining
pub fn format_time_remaining(seconds: u64) -> String {
    // Don't show "Calculating..." - show "Less than 1s" for very short times
    if seconds == 0 {
        return "Less than 1s".to_string();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
