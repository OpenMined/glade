use anyhow::Context;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::load_config;
use crate::downloader::{create_symlink, parse_md5_file, verify_md5, Downloader};
use crate::Result;

pub struct DatabaseManager {
    base_dir: PathBuf,
    downloader: Downloader,
}

impl DatabaseManager {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".glade")
            .join("databases");

        fs::create_dir_all(&base_dir).context("Failed to create base directory")?;

        Ok(Self {
            base_dir,
            downloader: Downloader::new()?,
        })
    }

    pub async fn download_database(&self, db_name: &str, genome_version: &str) -> Result<()> {
        let config = load_config()?;

        let db_config = config
            .get(db_name)
            .ok_or_else(|| anyhow::anyhow!("Database '{}' not found in configuration", db_name))?;

        let version_config = db_config.get(genome_version).ok_or_else(|| {
            anyhow::anyhow!(
                "Genome version '{}' not found for database '{}'",
                genome_version,
                db_name
            )
        })?;

        println!(
            "Downloading {} database for genome version {}",
            db_name, genome_version
        );
        println!("{}", "=".repeat(60));

        let md5_content = self
            .downloader
            .download_text(&version_config.md5)
            .await
            .context("Failed to download MD5 file")?;

        let (expected_md5, date) = parse_md5_file(&md5_content)?;

        let db_dir = self.base_dir.join(db_name).join(genome_version);
        let dated_dir = db_dir.join(&date);
        fs::create_dir_all(&dated_dir).context("Failed to create database directory")?;

        let files = vec![
            ("VCF", &version_config.vcf, "clinvar.vcf.gz"),
            ("TBI", &version_config.tbi, "clinvar.vcf.gz.tbi"),
            ("MD5", &version_config.md5, "clinvar.vcf.gz.md5"),
        ];

        for (desc, url, filename) in files {
            let target_path = dated_dir.join(filename);
            let symlink_path = db_dir.join(filename);

            if target_path.exists() {
                println!("  ✓ {} already exists", desc);

                if filename == "clinvar.vcf.gz" {
                    print!("    Verifying MD5 checksum... ");
                    std::io::stdout().flush().unwrap();

                    match verify_md5(&target_path, &expected_md5) {
                        Ok(true) => println!("✓ Valid"),
                        Ok(false) => {
                            println!("✗ Invalid checksum!");
                            println!("    Expected: {}", expected_md5);
                            fs::remove_file(&target_path)?;
                            self.download_and_verify(url, &target_path, desc, Some(&expected_md5))
                                .await?;
                        }
                        Err(e) => {
                            println!("⚠ Could not verify: {}", e);
                        }
                    }
                }
            } else {
                self.download_and_verify(
                    url,
                    &target_path,
                    desc,
                    if filename == "clinvar.vcf.gz" {
                        Some(&expected_md5)
                    } else {
                        None
                    },
                )
                .await?;
            }

            if !symlink_path.exists() || symlink_path.is_symlink() {
                create_symlink(&target_path, &symlink_path)
                    .context(format!("Failed to create symlink for {}", desc))?;
                println!("    ✓ Updated symlink: {}", symlink_path.display());
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("✓ Download complete!");
        println!("  Database: {}/{}", db_name, genome_version);
        println!("  Location: {}", db_dir.display());
        println!("  Date: {}", date);
        println!("{}", "=".repeat(60));

        Ok(())
    }

    async fn download_and_verify(
        &self,
        url: &str,
        target_path: &Path,
        desc: &str,
        expected_md5: Option<&str>,
    ) -> Result<()> {
        println!("  ↓ Downloading {}...", desc);
        self.downloader
            .download_file(url, target_path)
            .await
            .with_context(|| format!("Failed to download {}", desc))?;
        println!("    ✓ Download complete");

        if let Some(md5) = expected_md5 {
            print!("    Verifying MD5 checksum... ");
            std::io::stdout().flush().unwrap();

            match verify_md5(target_path, md5) {
                Ok(true) => println!("✓ Valid"),
                Ok(false) => {
                    println!("✗ Invalid checksum!");
                    fs::remove_file(target_path)?;
                    return Err(anyhow::anyhow!("Downloaded file has invalid checksum").into());
                }
                Err(e) => {
                    println!("⚠ Could not verify: {}", e);
                }
            }
        }

        Ok(())
    }

    pub async fn download_all_databases(&self) -> Result<()> {
        let config = load_config()?;

        for (db_name, versions) in config.iter() {
            for genome_version in versions.keys() {
                self.download_database(db_name, genome_version).await?;
            }
        }

        Ok(())
    }

    pub fn list_databases(&self) -> Result<()> {
        let config = load_config()?;

        println!("Available databases:");
        println!("{}", "=".repeat(60));

        for (db_name, versions) in config.iter() {
            println!("\nDatabase: {}", db_name);
            for (genome_version, files) in versions.iter() {
                println!("  Genome Version: {}", genome_version);
                println!("    VCF: {}", files.vcf);
                println!("    TBI: {}", files.tbi);
                println!("    MD5: {}", files.md5);

                let db_dir = self.base_dir.join(db_name).join(genome_version);
                if db_dir.exists() {
                    println!("    Status: ✓ Downloaded to {}", db_dir.display());
                } else {
                    println!("    Status: Not downloaded");
                }
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("Use 'glade database download --database <NAME> --genome-version <VERSION>' to download");
        println!("Use 'glade database download --all' to download all databases");

        Ok(())
    }
}
