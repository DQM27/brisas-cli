use crate::download;
use crate::errors::BeError;
use crate::manifest::Tool;
use crate::ui;
use std::fs;
use std::path::Path;

pub fn install_tool(tool: &Tool, target_base: &Path) -> Result<(), BeError> {
    ui::print_step(&format!("Instalando {}...", tool.name));

    // 1. Download
    let zip_name = format!("{}.zip", tool.name);
    let cached_file = download::ensure_downloaded(&tool.url, &zip_name, tool.sha256.as_deref())?;

    // 2. Prepare Target
    let target_path = target_base.join(&tool.name);
    if target_path.exists() {
        return Ok(()); // Already installed
    }

    // 3. Extract logic based on tool type
    match tool.name.as_str() {
        // Rustup is an EXE installer, not a ZIP
        "rustup" => install_rust(&cached_file, target_base)?,
        // Git Portable is a self-extracting EXE (7z)
        "git" => install_git_portable(&cached_file, &target_path)?,
        // VSCodium is a ZIP
        "vscodium" => install_vscodium(&cached_file, &target_path)?,
        // Standard ZIPs
        _ => install_generic_zip(&cached_file, &target_path)?,
    }

    ui::print_success(&format!("{} instalado.", tool.name));
    Ok(())
}

fn install_generic_zip(source: &Path, target: &Path) -> Result<(), BeError> {
    let temp_extract =
        std::env::temp_dir().join(format!("brisas_extract_{}", uuid::Uuid::new_v4()));
    if temp_extract.exists() {
        let _ = fs::remove_dir_all(&temp_extract);
    }

    download::extract_zip(source, &temp_extract)?;

    let mut final_source = temp_extract.clone();
    // Verify if it contains a single folder wrapper
    if let Ok(entries) = fs::read_dir(&temp_extract) {
        let items: Vec<_> = entries.filter_map(Result::ok).collect();
        if items.len() == 1 && items[0].path().is_dir() {
            final_source = items[0].path();
        }
    }

    copy_dir_with_progress(&final_source, target)?;
    let _ = fs::remove_dir_all(&temp_extract);
    Ok(())
}

fn install_git_portable(source: &Path, target: &Path) -> Result<(), BeError> {
    // Git Portable is an EXE but acts like a self-extracting archive.
    // However, 7z.exe args are tricky.
    // Simpler approach: It's actually a 7z archive with an SFX header.
    // We can try to unzip it if our zip lib supports it, likely NOT.
    // Better: Run it with `-y -gm2 -nr -o"Target"` (7-zip args)

    ui::print_step("Descomprimiendo Git Portable...");
    if !target.exists() {
        fs::create_dir_all(target)?;
    }

    let status = std::process::Command::new(source)
        .arg("-y")
        .arg(format!("-o{}", target.display()))
        .status()
        .map_err(|e| BeError::Setup(format!("Fallo descomprimiendo Git: {}", e)))?;

    if !status.success() {
        return Err(BeError::Setup(
            "Git Portable fallo al descomprimirse.".into(),
        ));
    }

    Ok(())
}

fn install_vscodium(source: &Path, target: &Path) -> Result<(), BeError> {
    install_generic_zip(source, target)?;

    // Make Portable
    ui::print_step("Haciendo VSCodium Portable...");
    let data_dir = target.join("data");
    if !data_dir.exists() {
        fs::create_dir(&data_dir)?;
    }
    Ok(())
}

fn install_rust(source: &Path, _target_base: &Path) -> Result<(), BeError> {
    ui::print_step("Ejecutando Instalador de Rust (rustup-init)...");

    // rustup-init.exe -y --default-host x86_64-pc-windows-gnu --default-toolchain stable --no-modify-path
    let status = std::process::Command::new(source)
        .arg("-y")
        .arg("--default-host")
        .arg("x86_64-pc-windows-gnu")
        .arg("--default-toolchain")
        .arg("stable")
        .arg("--no-modify-path")
        .status()
        .map_err(|e| BeError::Setup(format!("Fallo ejecutando rustup-init: {}", e)))?;

    if !status.success() {
        return Err(BeError::Setup(
            "rustup-init fallo (posiblemente falta internet).".into(),
        ));
    }

    Ok(())
}

// Helper (Reused)
fn copy_dir_with_progress(src: &Path, dst: &Path) -> Result<(), BeError> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    // (Simplified copy logic for brevity, ideally reuse the main one or move to common utils)
    // For now, implementing simple recursive copy
    let options = fs_extra::dir::CopyOptions::new()
        .overwrite(true)
        .content_only(true);
    fs_extra::dir::copy(src, dst, &options)
        .map_err(|e| BeError::Setup(format!("Error copiando archivos: {}", e)))?;
    Ok(())
}
