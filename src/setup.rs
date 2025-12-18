use crate::download;
use crate::errors::BeError;
use crate::manifest::Manifest; // Import Manifest
use inquire::{Select, Text};
use log::{error, info};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

pub fn setup_system() -> Result<(), BeError> {
    println!("🛠️  Configurando Entorno Brisas en el Sistema...");
    info!("Starting setup_system...");

    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| BeError::Config("No se encontró %LOCALAPPDATA%".into()))?;
    let target_base = PathBuf::from(&local_app_data);
    println!("📂 Destino: {}", target_base.display());

    // LOAD MANIFEST
    let manifest_path = Path::new("tools.json");
    let manifest = if manifest_path.exists() {
        info!("Loading manifest from local file: tools.json");
        println!("📄 Usando manifiesto local: tools.json");
        Manifest::load_from_file(manifest_path).unwrap_or_else(|e| {
            error!("Failed to load local tools.json: {}", e);
            println!("⚠️  Error leyendo tools.json. Usando defaults.");
            Manifest::default()
        })
    } else {
        // En el futuro aquí iría: Manifest::load_from_url(URL).unwrap_or(Manifest::default())
        Manifest::default()
    };
    info!("Manifest loaded with {} tools.", manifest.tools.len());

    let mut found_tools = Vec::new();

    // 1. Check existing
    for tool in &manifest.tools {
        let target_path = target_base.join(&tool.name);
        if target_path.join(&tool.check_file).exists() {
            println!("  ✅ {} ya existe en AppData.", tool.name);
            found_tools.push((tool.name.clone(), target_path));
        }
    }

    if found_tools.len() == manifest.tools.len() {
        println!("✨ Todas las herramientas ya están instaladas.");
    } else {
        println!("⚠️  Faltan herramientas.");

        let options = vec![
            "🔍 Buscar en carpeta local (Pendrive/Descargas)",
            "⬇️  Descargar de Internet (Automático)",
        ];
        let ans = Select::new("¿Cómo deseas obtener las herramientas?", options.clone())
            .prompt()
            .map_err(|_| BeError::Cancelled)?;

        if ans == options[0] {
            // SEARCH LOCAL
            handle_local_search(&manifest, &target_base, &mut found_tools)?;
        } else {
            // DOWNLOAD (Now with Cache & Verification)
            handle_download(&manifest, &target_base, &mut found_tools)?;
        }
    }

    // Register Registry
    register_in_path(&target_base)?;

    Ok(())
}

fn handle_local_search(
    manifest: &Manifest,
    target_base: &Path,
    found_tools: &mut Vec<(String, PathBuf)>,
) -> Result<(), BeError> {
    let source_input = Text::new("Ingresa la ruta de la carpeta origen:")
        .with_default("C:\\Users\\femprobrisas\\Downloads")
        .prompt()
        .map_err(|_| BeError::Cancelled)?;

    let source_path = PathBuf::from(&source_input);
    if !source_path.exists() {
        return Err(BeError::Setup("La ruta origen no existe.".into()));
    }

    for tool in &manifest.tools {
        let target_path = target_base.join(&tool.name);
        if target_path.exists() {
            continue;
        }

        println!("🔍 Buscando {}...", tool.name);
        if let Some(folder) = find_folder_containing(&source_path, &tool.check_file) {
            println!("  📦 Copiando a {}...", target_path.display());
            let options = fs_extra::dir::CopyOptions::new().content_only(true);
            fs::create_dir_all(&target_path)?;

            if let Err(e) = fs_extra::dir::copy(&folder, &target_path, &options) {
                return Err(BeError::Setup(format!(
                    "Error copiando {}: {}",
                    tool.name, e
                )));
            } else {
                found_tools.push((tool.name.clone(), target_path));
            }
        } else {
            eprintln!("❌ No se encontró {} en el origen.", tool.name);
        }
    }
    Ok(())
}

fn handle_download(
    manifest: &Manifest,
    target_base: &Path,
    found_tools: &mut Vec<(String, PathBuf)>,
) -> Result<(), BeError> {
    for tool in &manifest.tools {
        let target_path = target_base.join(&tool.name);
        if target_path.exists() {
            continue;
        }

        println!("☁️  Procesando {}...", tool.name);
        let zip_name = format!("{}.zip", tool.name);

        // ensure_downloaded handles Cache + SHA256 Verification
        let cached_zip = download::ensure_downloaded(&tool.url, &zip_name, tool.sha256.as_deref())?;

        // Extract
        let temp_extract = std::env::temp_dir().join(format!("{}_extract", tool.name));
        if temp_extract.exists() {
            let _ = fs::remove_dir_all(&temp_extract);
        }

        download::extract_zip(&cached_zip, &temp_extract)?;

        // Move to target
        let mut source_to_copy = temp_extract.clone();
        if let Ok(entries) = fs::read_dir(&temp_extract) {
            let items: Vec<_> = entries.filter_map(Result::ok).collect();
            if items.len() == 1 && items[0].path().is_dir() {
                source_to_copy = items[0].path();
            }
        }

        println!("  📦 Instalando en {}...", target_path.display());
        let options = fs_extra::dir::CopyOptions::new().content_only(true);
        fs::create_dir_all(&target_path)?;

        if let Err(e) = fs_extra::dir::copy(&source_to_copy, &target_path, &options) {
            return Err(BeError::Setup(format!(
                "Error moviendo archivos de {}: {}",
                tool.name, e
            )));
        } else {
            println!("  ✨ Instalado correctamente.");
            found_tools.push((tool.name.clone(), target_path));
        }

        // Cleanup (Only extract dir, keep Cache!)
        let _ = fs::remove_dir_all(&temp_extract);
    }
    Ok(())
}

fn register_in_path(target_base: &Path) -> Result<(), BeError> {
    println!("📝 Actualizando Registro de Usuario (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| BeError::Setup(format!("Error abriendo registro: {}", e)))?;

    let current_path: String = env_key.get_value("Path").unwrap_or_default();
    let mut new_path_parts: Vec<String> = current_path.split(';').map(|s| s.to_string()).collect();
    let mut changed = false;

    // Hardcoded logic for PATH registration is OK for now,
    // or we could add `path_suffix` to Manifest if we want total decoupling.
    // For now, keeping it simple as specific tools have specific bin folders.
    let paths_to_add = vec![
        target_base.join("node").to_string_lossy().to_string(),
        target_base
            .join("mingw64")
            .join("bin")
            .to_string_lossy()
            .to_string(),
        target_base.join("pwsh").to_string_lossy().to_string(),
    ];

    for p in paths_to_add {
        if !new_path_parts.contains(&p) {
            new_path_parts.push(p.clone());
            println!("  ➕ Añadiendo al PATH: {}", p);
            changed = true;
        }
    }

    if changed {
        let new_path_str = new_path_parts.join(";");
        env_key
            .set_value("Path", &new_path_str)
            .map_err(|e| BeError::Setup(format!("Error escribiendo registro: {}", e)))?;
        println!("✅ Registro actualizado correctamente.");
        println!("⚠️  Nota: Necesitas reiniciar tus terminales para ver los cambios.");
    } else {
        println!("✨ El PATH ya estaba configurado.");
    }
    Ok(())
}

fn find_folder_containing(base: &Path, file_pattern: &str) -> Option<PathBuf> {
    for entry in walkdir::WalkDir::new(base)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            let candidate = entry.path();
            if candidate.join(file_pattern).exists() {
                return Some(candidate.to_path_buf());
            }
        }
    }
    None
}

pub fn clean_system() -> Result<(), BeError> {
    println!("🧹 Limpiando Entorno Brisas del Sistema...");
    info!("Starting clean_system...");

    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| BeError::Config("No se encontró %LOCALAPPDATA%".into()))?;
    let target_base = PathBuf::from(&local_app_data);

    let tools = vec!["node", "mingw64", "pwsh"];

    // 2. Remove Files
    for tool in &tools {
        let path = target_base.join(tool);
        if path.exists() {
            println!("  🔥 Eliminando carpeta: {}", path.display());
            if let Err(e) = fs::remove_dir_all(&path) {
                error!("Failed to remove directory {}: {}", path.display(), e);
                eprintln!("❌ Error eliminando {}: {}", tool, e);
            } else {
                info!("Removed directory: {}", path.display());
                println!("    ✨ Eliminado.");
            }
        }
    }

    // 3. Clean Registry
    println!("📝 Limpiando Registro de Usuario (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // Use open_subkey_with_flags
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| BeError::Setup(format!("Error abriendo registro: {}", e)))?;

    let current_path: String = env_key.get_value("Path").unwrap_or_default();
    let parts: Vec<&str> = current_path.split(';').collect();

    let paths_to_remove = vec![
        target_base.join("node").to_string_lossy().to_string(),
        target_base
            .join("mingw64")
            .join("bin")
            .to_string_lossy()
            .to_string(),
        target_base.join("pwsh").to_string_lossy().to_string(),
    ];

    let new_parts: Vec<&str> = parts
        .into_iter()
        .filter(|part| {
            !part.is_empty() && !paths_to_remove.iter().any(|remove| part.contains(remove))
        })
        .collect();

    let new_path_str = new_parts.join(";");

    if new_path_str.len() < 5 && !current_path.is_empty() {
        println!("⚠️  Advertencia: El PATH resultante parece muy corto. Abortando actualización.");
        return Ok(());
    }

    if new_path_str != current_path {
        env_key
            .set_value("Path", &new_path_str)
            .map_err(|e| BeError::Setup(format!("Error guardando registro: {}", e)))?;
        println!("✅ Registro limpiado correctamente.");
        println!("⚠️  Reinicia tus terminales para ver los cambios.");
        info!("Registry cleaned successfully.");
    } else {
        println!("✨ El registro ya estaba limpio.");
    }
    Ok(())
}

pub fn check_status() {
    println!("🔍 Verificando Estado del Sistema...");

    let local_app_data = match env::var("LOCALAPPDATA") {
        Ok(val) => val,
        Err(_) => {
            println!("❌ No se encontró %LOCALAPPDATA%.");
            return;
        }
    };
    if local_app_data.is_empty() {
        println!("❌ %LOCALAPPDATA% está vacío.");
        return;
    }

    let target_base = PathBuf::from(&local_app_data);
    let tools = vec!["node", "mingw64", "pwsh"];
    let mut missing = false;

    // 1. Files
    println!("📂 Archivos (AppData\\Local):");
    for tool in &tools {
        let path = target_base.join(tool);
        if path.exists() {
            println!("  ✅ {}: Instalado", tool);
        } else {
            println!("  ❌ {}: No encontrado", tool);
            missing = true;
        }
    }

    // 2. Registry
    println!("📝 Registro (User PATH):");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(env_key) = hkcu.open_subkey_with_flags("Environment", KEY_READ) {
        let current_path: String = match env_key.get_value("Path") {
            Ok(val) => val,
            Err(e) => {
                println!("❌ Error leyendo valor 'Path' del registro: {}", e);
                return;
            }
        };

        for tool in &tools {
            let expected = target_base.join(tool);
            let needle = if *tool == "mingw64" {
                expected.join("bin").to_string_lossy().to_string()
            } else {
                expected.to_string_lossy().to_string()
            };

            if current_path.contains(&needle) {
                println!("  ✅ {}: En PATH", tool);
            } else {
                println!("  ❌ {}: Falta en PATH", tool);
                missing = true;
            }
        }
    } else {
        println!("❌ Error leyendo Registro.");
    }

    if !missing {
        println!("\n✨ Todo parece estar CORRECTO. El entorno debería funcionar.");
    } else {
        println!("\n⚠️  Hay inconsistencias. Recomendado: Selecciona '🛠️  Instalar / Reparar' en el menú.");
    }
}
