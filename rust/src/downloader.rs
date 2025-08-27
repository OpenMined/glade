use anyhow::Context;
use chrono::Local;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use std::fs;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::Result;

pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3600))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    pub async fn download_file(&self, url: &str, target_path: &Path) -> Result<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            return Err(
                anyhow::anyhow!("HTTP request failed with status: {}", response.status()).into(),
            );
        }

        let total_size = response.content_length().unwrap_or(0);

        let pb = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "    [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                    )
                    .expect("Failed to set progress bar template")
                    .progress_chars("#>-"),
            );
            Some(pb)
        } else {
            println!("    Downloading (size unknown)...");
            None
        };

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).context("Failed to create target directory")?;
        }

        let mut file = File::create(target_path)
            .await
            .context("Failed to create target file")?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write chunk to file")?;

            downloaded += chunk.len() as u64;
            if let Some(ref pb) = pb {
                pb.set_position(downloaded);
            }
        }

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        Ok(())
    }

    pub async fn download_text(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            return Err(
                anyhow::anyhow!("HTTP request failed with status: {}", response.status()).into(),
            );
        }

        response.text().await.map_err(Into::into)
    }
}

pub fn parse_md5_file(md5_content: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = md5_content.trim().split_whitespace().collect();

    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid MD5 file format").into());
    }

    let md5_hash = parts[0].to_string();
    let path = parts[1].to_string();

    if let Some(filename) = path.split('/').last() {
        if filename.contains("_") {
            let date_parts: Vec<&str> = filename.split('_').collect();
            for part in date_parts {
                if part.len() >= 8 && part.chars().take(8).all(|c| c.is_numeric()) {
                    let date = &part[0..8];
                    return Ok((md5_hash, date.to_string()));
                }
            }
        }
    }

    let date = Local::now().format("%Y%m%d").to_string();
    Ok((md5_hash, date))
}

pub fn calculate_md5(path: &Path) -> Result<String> {
    use std::io::Read;
    
    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open file for MD5: {}", path.display()))?;
    
    let mut context = md5::Context::new();
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)
            .with_context(|| format!("Failed to read file for MD5: {}", path.display()))?;
        
        if bytes_read == 0 {
            break;
        }
        
        context.consume(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", context.compute()))
}

pub fn verify_md5(path: &Path, expected_md5: &str) -> Result<bool> {
    let actual = calculate_md5(path)?;
    Ok(actual == expected_md5)
}

pub fn create_symlink(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_file(dst).context("Failed to remove existing symlink")?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst).context("Failed to create symlink")?;
    }

    #[cfg(not(unix))]
    {
        return Err(anyhow::anyhow!("Symlinks not supported on this platform").into());
    }

    Ok(())
}
