use crate::errors::BeError;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Calcula el hash SHA256 de un archivo y lo devuelve como string hex minúscula.
pub fn calculate_hash(path: &Path) -> Result<String, BeError> {
    let file = File::open(path)?;
    let total_size = file.metadata()?.len();

    let pb = ProgressBar::new(total_size);
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green}  [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({binary_bytes_per_sec})")
        .map_err(|e| BeError::Setup(format!("Error configurando barra de progreso: {}", e)))?
        .progress_chars("█░");
    pb.set_style(style);

    let mut reader = pb.wrap_read(file);
    let mut hasher = Sha256::new();
    io::copy(&mut reader, &mut hasher)?;

    pb.finish_and_clear(); // Limpiar barra al terminar para no ensuciar
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

// ... ensure_downloaded and download_file remain ...

pub fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), BeError> {
    info!(
        "Extrayendo {} a {}",
        zip_path.display(),
        extract_to.display()
    );
    println!("Extrayendo...");

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let len = archive.len();

    let pb = ProgressBar::new(len as u64);
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green}  [{elapsed_precise}] [{bar:40.yellow/blue}] {pos}/{len} archivos ({eta})")
        .map_err(|e| BeError::Setup(format!("Error configurando barra de progreso: {}", e)))?
        .progress_chars("█░");
    pb.set_style(style);

    for i in 0..len {
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
        pb.inc(1);
    }
    pb.finish_with_message("Extracción completada");
    Ok(())
}

/// Descarga un archivo, utilizando un directorio de caché local.
/// Si `expected_hash` es proporcionado, verifica la integridad del archivo.
/// Devuelve la ruta al archivo válido (en caché).
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

    // 1. Verificar si existe
    if target_path.exists() {
        info!("Archivo encontrado en caché: {}", target_path.display());
        if let Some(hash) = expected_hash {
            info!("Verificando integridad del archivo en caché...");
            let current_hash = calculate_hash(&target_path)?;
            if current_hash == hash {
                info!("¡Hash correcto! Usando archivo en caché.");
                return Ok(target_path);
            } else {
                warn!("Hash incorrecto en caché. Eliminando y re-descargando.");
                warn!("Esperado: {}", hash);
                warn!("Obtenido: {}", current_hash);
                fs::remove_file(&target_path)?;
            }
        } else {
            // Sin hash proporcionado, asumir que el caché está bien
            info!("Sin hash para verificar. Usando archivo en caché.");
            return Ok(target_path);
        }
    }

    // 2. Descargar
    download_file(url, &target_path)?;

    // 3. Verificar después de descargar
    if let Some(hash) = expected_hash {
        info!("Verificando integridad del archivo descargado...");
        let current_hash = calculate_hash(&target_path)?;
        if current_hash != hash {
            fs::remove_file(&target_path)?; // Eliminar archivo malo
            return Err(BeError::Setup(format!(
                "Falló la verificación de integridad para {}. Esperado {}, obtenido {}.",
                file_name, hash, current_hash
            )));
        }
        info!("Verificación exitosa.");
    }

    Ok(target_path)
}

pub fn download_file(url: &str, target_path: &Path) -> Result<(), BeError> {
    println!("Descargando: {}", url);
    info!("Descargando {} a {}", url, target_path.display());

    let mut response = reqwest::blocking::get(url)?;

    // Verificar estado convirtiendo a error directamente si es necesario
    if let Err(e) = response.error_for_status_ref() {
        return Err(BeError::Reqwest(e));
    }

    let total_size = match response.content_length() {
        Some(len) => len,
        None => 0,
    };
    let pb = ProgressBar::new(total_size);
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green}  [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({binary_bytes_per_sec}, ETA {eta})")
        .map_err(|e| BeError::Setup(format!("Error configurando barra de progreso: {}", e)))?
        .progress_chars("█░");
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

/// Verifica si un nombre de archivo dado existe dentro del archivo zip.
/// Devuelve Ok(true) si se encuentra, Ok(false) si no.
pub fn verify_zip_contains_file(zip_path: &Path, file_name: &str) -> Result<bool, BeError> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        // Buscamos coincidencia exacta o coincidencia final (ej. "bin/gcc.exe" coincide con "mingw64/bin/gcc.exe")
        if file.name() == file_name || file.name().ends_with(file_name) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_calculate_hash() {
        let dir = std::env::temp_dir().join("be_test_hash");
        if !dir.exists() {
            let _ = std::fs::create_dir(&dir);
        }
        let file_path = dir.join("test.txt");
        let mut file =
            std::fs::File::create(&file_path).expect("Fallo al crear archivo temporal de prueba");
        file.write_all(b"hello world")
            .expect("Fallo al escribir en archivo temporal");

        let hash = calculate_hash(&file_path).expect("Deberia calcular el hash");

        // sha256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        // limpieza
        let _ = std::fs::remove_file(file_path);
        let _ = std::fs::remove_dir(dir);
    }
}
