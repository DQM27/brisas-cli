use crate::download;
use crate::errors::BeError;
use inquire::{Select, Text};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

pub fn setup_system() -> Result<(), BeError> {
    println!("🛠️  Configurando Entorno Brisas en el Sistema...");

    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| BeError::Config("No se encontró %LOCALAPPDATA%".into()))?;
    let target_base = PathBuf::from(&local_app_data);
    println!("📂 Destino: {}", target_base.display());

    // Tools def: (Name, CheckFile, DownloadURL)
    let tools = vec![
        ("node", "node.exe", "https://nodejs.org/dist/v22.12.0/node-v22.12.0-win-x64.zip"),
        ("mingw64", "bin/gcc.exe", "https://github.com/brechtsanders/winlibs_mingw/releases/download/14.2.0-17.0.6-12.0.0-ucrt-r2/winlibs-x86_64-posix-seh-gcc-14.2.0-llvm-19.1.1-mingw-w64ucrt-12.0.0-r2.zip"),
        ("pwsh", "pwsh.exe", "https://github.com/PowerShell/PowerShell/releases/download/v7.4.6/PowerShell-7.4.6-win-x64.zip") 
    ];

    let mut found_tools = Vec::new();

    // 1. Check existing
    for (name, check_file, _) in &tools {
        let target_path = target_base.join(name);
        if target_path.join(check_file).exists() {
            println!("  ✅ {} ya existe en AppData.", name);
            found_tools.push((name.to_string(), target_path));
        }
    }

    if found_tools.len() == tools.len() {
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
            handle_local_search(&tools, &target_base, &mut found_tools)?;
        } else {
            // DOWNLOAD
            handle_download(&tools, &target_base, &mut found_tools)?;
        }
    }

    // Register Registry
    register_in_path(&target_base)?;

    Ok(())
}

fn handle_local_search(
    tools: &[(&str, &str, &str)],
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

    for (name, check_file, _) in tools {
        let target_path = target_base.join(name);
        if target_path.exists() {
            continue;
        }

        println!("🔍 Buscando {}...", name);
        if let Some(folder) = find_folder_containing(&source_path, check_file) {
            println!("  📦 Copiando a {}...", target_path.display());
            let options = fs_extra::dir::CopyOptions::new().content_only(true);
            fs::create_dir_all(&target_path)?;

            if let Err(e) = fs_extra::dir::copy(&folder, &target_path, &options) {
                // fs_extra error is distinct, we map it manually or just stringify
                return Err(BeError::Setup(format!("Error copiando {}: {}", name, e)));
            } else {
                found_tools.push((name.to_string(), target_path));
            }
        } else {
            eprintln!("❌ No se encontró {} en el origen.", name);
        }
    }
    Ok(())
}

fn handle_download(
    tools: &[(&str, &str, &str)],
    target_base: &Path,
    found_tools: &mut Vec<(String, PathBuf)>,
) -> Result<(), BeError> {
    for (name, _, url) in tools {
        let target_path = target_base.join(name);
        if target_path.exists() {
            continue;
        }

        println!("☁️  Procesando {}...", name);
        let zip_name = format!("{}.zip", name);
        let temp_zip = std::env::temp_dir().join(&zip_name);

        // Download
        download::download_file(url, &temp_zip)?;

        // Extract
        let temp_extract = std::env::temp_dir().join(format!("{}_extract", name));
        if temp_extract.exists() {
            let _ = fs::remove_dir_all(&temp_extract);
        }

        download::extract_zip(&temp_zip, &temp_extract)?;

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
                name, e
            )));
        } else {
            println!("  ✨ Instalado correctamente.");
            found_tools.push((name.to_string(), target_path));
        }

        // Cleanup
        let _ = fs::remove_file(&temp_zip);
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
                eprintln!("❌ Error eliminando {}: {}", tool, e); // Warn but continue
            } else {
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
    } else {
        println!("✨ El registro ya estaba limpio.");
    }
    Ok(())
}

pub fn check_status() {
    // This is safe to keep as no-result or wrap it if we want strictness,
    // but check_status generally just prints. We can leave it or wrap it.
    // Let's wrap it for consistency in calling convention if needed,
    // but main.rs calls it directly. Let's leave it void as it doesn't "fail" critically.
    println!("🔍 Verificando Estado del Sistema...");

    let local_app_data = env::var("LOCALAPPDATA").unwrap_or_default();
    if local_app_data.is_empty() {
        println!("❌ No se pudo leer %LOCALAPPDATA%");
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
        let current_path: String = env_key.get_value("Path").unwrap_or_default();

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
