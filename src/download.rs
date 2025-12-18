use crate::errors::BeError;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Calculates the SHA256 hash of a file and returns it as a lowercase hex string.
pub fn calculate_hash(path: &Path) -> Result<String, BeError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Downloads a file, utilizing a local cache directory.
/// If `expected_hash` is provided, it verifies the file integrity.
/// Returns the path to the valid file (in cache).
pub fn ensure_downloaded(
    url: &str,
    file_name: &str,
    expected_hash: Option<&str>,
) -> Result<PathBuf, BeError> {
    let cache_dir = std::env::temp_dir().join("BrisasEnv_Cache");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }

    let target_path = cache_dir.join(file_name);

    // 1. Check if exists
    if target_path.exists() {
        info!("File found in cache: {}", target_path.display());
        if let Some(hash) = expected_hash {
            info!("Verifying integrity of cached file...");
            let current_hash = calculate_hash(&target_path)?;
            if current_hash == hash {
                info!("Hash match! Using cached file.");
                return Ok(target_path);
            } else {
                warn!("Hash mismatch for cached file. Deleting and redownloading.");
                warn!("Expected: {}", hash);
                warn!("Actual:   {}", current_hash);
                fs::remove_file(&target_path)?;
            }
        } else {
            // No hash provided, assume cached file is good (or we can't verify it)
            info!("No hash provided for verification. Using cached file.");
            return Ok(target_path);
        }
    }

    // 2. Download
    download_file(url, &target_path)?;

    // 3. Verify after download
    if let Some(hash) = expected_hash {
        info!("Verifying integrity of downloaded file...");
        let current_hash = calculate_hash(&target_path)?;
        if current_hash != hash {
            fs::remove_file(&target_path)?; // Delete bad file
            return Err(BeError::Setup(format!(
                "Integrity check failed for {}. Expected {}, got {}.",
                file_name, hash, current_hash
            )));
        }
        info!("Verification successful.");
    }

    Ok(target_path)
}

pub fn download_file(url: &str, target_path: &Path) -> Result<(), BeError> {
    println!("⬇️  Descargando: {}", url);
    info!("Downloading {} to {}", url, target_path.display());

    let mut response = reqwest::blocking::get(url)?;

    // Check status by converting to error directly if needed
    if let Err(e) = response.error_for_status_ref() {
        return Err(BeError::Reqwest(e));
    }

    let total_size = match response.content_length() {
        Some(len) => len,
        None => 0,
    };
    let pb = ProgressBar::new(total_size);
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .map_err(|e| BeError::Setup(format!("Error configurando barra de progreso: {}", e)))?
        .progress_chars("#>-");
    pb.set_style(style);

    let mut file = File::create(target_path)?;

    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Descarga completada");
    Ok(())
}

pub fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), BeError> {
    info!(
        "Extracting {} to {}",
        zip_path.display(),
        extract_to.display()
    );
    println!("📦 Extrayendo...");

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
