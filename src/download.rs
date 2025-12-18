use crate::errors::BeError;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use zip::ZipArchive;

pub fn download_file(url: &str, target_path: &Path) -> Result<(), BeError> {
    println!("⬇️  Requerido: {}", url);
    let mut response = reqwest::blocking::get(url)?;

    // Check status
    if !response.status().is_success() {
        return Err(BeError::Reqwest(response.error_for_status().unwrap_err()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut file = File::create(target_path)?;

    // Stream copy with progress
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
    println!(
        "📦 Extrayendo {} en {}...",
        zip_path.display(),
        extract_to.display()
    );

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
