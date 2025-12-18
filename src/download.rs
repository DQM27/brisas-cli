use std::fs::{self, File};
use std::io::{self, Cursor};
use std::path::Path;
use zip::ZipArchive;

pub fn download_file(url: &str, target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("⬇️  Descargando: {}", url);
    let response = reqwest::blocking::get(url)?;

    // Check status
    if !response.status().is_success() {
        return Err(format!("Error descargando: {}", response.status()).into());
    }

    let mut content = Cursor::new(response.bytes()?);
    let mut file = File::create(target_path)?;
    std::io::copy(&mut content, &mut file)?;

    Ok(())
}

pub fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
