use crate::download;
use crate::errors::BeError;
use crate::manifest::{Manifest, Tool};
use inquire::{Confirm, Select, Text};
use log::info;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn generate_manifest() -> Result<(), BeError> {
    println!("🧙‍♂️  Asistente de Generación de Manifiesto (Admin) 🧙‍♂️");
    println!("Este asistente te ayudará a crear/actualizar el archivo 'tools.json'.");

    let manifest_path = Path::new("tools.json");
    let mut manifest = if manifest_path.exists() {
        println!("📂 Se encontró un 'tools.json' existente. Cargando...");
        Manifest::load_from_file(manifest_path)?
    } else {
        println!("✨ Creando nuevo manifiesto basado en los defaults.");
        Manifest::default()
    };

    let mut new_tools = Vec::new();

    for tool in &manifest.tools {
        println!("\n🔧 Herramienta: {}", tool.name);
        println!("   Versión Actual: {}", tool.version);
        println!("   URL Actual: {}", tool.url);

        let actions = vec!["✅ Mantener igual", "✏️  Editar / Actualizar"];
        let choice = Select::new("¿Qué deseas hacer?", actions.clone())
            .prompt()
            .map_err(|_| BeError::Cancelled)?;

        if choice == actions[0] {
            new_tools.push(tool.clone());
        } else {
            // EDIT
            let new_version = Text::new("Nueva Versión:")
                .with_default(&tool.version)
                .prompt()
                .map_err(|_| BeError::Cancelled)?;

            let new_url = Text::new("Nueva URL de Descarga (.zip):")
                .with_default(&tool.url)
                .prompt()
                .map_err(|_| BeError::Cancelled)?;

            // Hashing
            println!("🔄 Calculando Hash SHA256 (Descargando temporalmente)...");

            // Download to temp
            let temp_dir = std::env::temp_dir().join("Brisas_Hash_Calc");
            if !temp_dir.exists() {
                fs::create_dir_all(&temp_dir)?;
            }
            let temp_file = temp_dir.join(format!("{}.tmp", tool.name));

            // Force download (ignore cache for hashing new URL effectively to be sure)
            // But reuse download logic.
            // NOTE: We don't have a verify hash yet, so pass None.
            download::download_file(&new_url, &temp_file)?;

            let hash = download::calculate_hash(&temp_file)?;
            println!("   🔐 Hash calculado: {}", hash);

            // Cleanup
            let _ = fs::remove_file(&temp_file);

            new_tools.push(Tool {
                name: tool.name.clone(),
                version: new_version,
                url: new_url,
                check_file: tool.check_file.clone(),
                sha256: Some(hash),
            });
        }
    }

    manifest.tools = new_tools;
    manifest.save_to_file(manifest_path)?;
    println!("\n💾 'tools.json' guardado correctamente.");

    // GIT PUSH AUTOMATION
    let push = Confirm::new("¿Deseas subir los cambios a GitHub ahora? (Requiere git configurado)")
        .with_default(false)
        .prompt()
        .map_err(|_| BeError::Cancelled)?;

    if push {
        run_git_automation(manifest_path)?;
    }

    Ok(())
}

fn run_git_automation(file_path: &Path) -> Result<(), BeError> {
    println!("🚀 Iniciando secuencia de Git...");

    // 1. Git Add
    println!("   > git add {:?}", file_path);
    let status = Command::new("git")
        .arg("add")
        .arg(file_path)
        .status()
        .map_err(|e| BeError::Setup(format!("Error ejecutando git: {}", e)))?;

    if !status.success() {
        return Err(BeError::Setup("Falló git add".into()));
    }

    // 2. Git Commit
    let msg = format!(
        "Update tools.json: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    );
    println!("   > git commit -m \"{}\"", msg);
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&msg)
        .status()?; // Ignore error if nothing to commit

    // 3. Git Push
    println!("   > git push");
    let status = Command::new("git")
        .arg("push")
        .status()
        .map_err(|e| BeError::Setup(format!("Error ejecutando git push: {}", e)))?;

    if status.success() {
        println!("✨ ¡Subido a GitHub con éxito!");
        info!("Manifest pushed to GitHub.");
    } else {
        println!("⚠️  'git push' falló. Por favor verifica tus credenciales/conexión.");
        return Err(BeError::Setup("Falló git push".into()));
    }

    Ok(())
}
