use crate::download;
use inquire::{Select, Text};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

pub fn setup_system() {
    println!("🛠️  Configurando Entorno Brisas en el Sistema...");

    let local_app_data = env::var("LOCALAPPDATA").expect("No se encontró %LOCALAPPDATA%");
    let target_base = PathBuf::from(&local_app_data);
    println!("📂 Destino: {}", target_base.display());

    // Tools def: (Name, CheckFile, DownloadURL)
    // URLs can be updated. Using stable versions.
    let tools = vec![
        ("node", "node.exe", "https://nodejs.org/dist/v22.12.0/node-v22.12.0-win-x64.zip"),
        ("mingw64", "bin/gcc.exe", "https://github.com/brechtsanders/winlibs_mingw/releases/download/14.2.0-17.0.6-12.0.0-ucrt-r2/winlibs-x86_64-posix-seh-gcc-14.2.0-llvm-19.1.1-mingw-w64ucrt-12.0.0-r2.zip"),
        ("pwsh", "pwsh.exe", "https://github.com/PowerShell/PowerShell/releases/download/v7.4.6/PowerShell-7.4.6-win-x64.zip") 
    ];

    let mut found_tools = Vec::new();

    // 1. Check existing
    for (name, check_file, _) in &tools {
        // Special handle for extracted folders. Node zip extracts to "node-v22...", MinGW to "mingw64".
        // We want target to be "node", "mingw64", "pwsh".
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
        let ans = Select::new("¿Cómo deseas obtener las herramientas?", options.clone()).prompt();

        match ans {
            Ok(choice) => {
                if choice == options[0] {
                    // SEARCH LOCAL
                    handle_local_search(&tools, &target_base, &mut found_tools);
                } else {
                    // DOWNLOAD
                    handle_download(&tools, &target_base, &mut found_tools);
                }
            }
            Err(_) => println!("Operación cancelada."),
        }
    }

    // Register Registry
    register_in_path(&target_base);
}

fn handle_local_search(
    tools: &[(&str, &str, &str)],
    target_base: &Path,
    found_tools: &mut Vec<(String, PathBuf)>,
) {
    let source_input = Text::new("Ingresa la ruta de la carpeta origen:")
        .with_default("C:\\Users\\femprobrisas\\Downloads")
        .prompt();

    if let Ok(src) = source_input {
        let source_path = PathBuf::from(&src);
        if !source_path.exists() {
            eprintln!("❌ La ruta origen no existe.");
            return;
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
                fs::create_dir_all(&target_path).unwrap();
                if let Err(e) = fs_extra::dir::copy(&folder, &target_path, &options) {
                    eprintln!("❌ Error copiando {}: {}", name, e);
                } else {
                    found_tools.push((name.to_string(), target_path));
                }
            } else {
                eprintln!("❌ No se encontró {} en el origen.", name);
            }
        }
    }
}

fn handle_download(
    tools: &[(&str, &str, &str)],
    target_base: &Path,
    found_tools: &mut Vec<(String, PathBuf)>,
) {
    for (name, _, url) in tools {
        let target_path = target_base.join(name);
        if target_path.exists() {
            continue;
        }

        println!("☁️  Procesando {}...", name);
        let zip_name = format!("{}.zip", name);
        let temp_zip = std::env::temp_dir().join(&zip_name);

        // Download
        if let Err(e) = download::download_file(url, &temp_zip) {
            eprintln!("❌ Error descargando {}: {}", name, e);
            continue;
        }

        // Extract
        // Caution: Node zip extracts to "node-vXX-win-x64", MinGW to "mingw64", Pwsh files are at root of zip.
        // We need to handle this structure.

        let temp_extract = std::env::temp_dir().join(format!("{}_extract", name));
        if temp_extract.exists() {
            let _ = fs::remove_dir_all(&temp_extract);
        }

        if let Err(e) = download::extract_zip(&temp_zip, &temp_extract) {
            eprintln!("❌ Error extrayendo {}: {}", name, e);
            continue;
        }

        // Move to target
        // Logic: Find the "real" root inside extract.
        // For PWSH, items are in root. For Node, in subdir.

        let mut source_to_copy = temp_extract.clone();

        // Peek inside to see if there is a single folder
        if let Ok(entries) = fs::read_dir(&temp_extract) {
            let items: Vec<_> = entries.filter_map(Result::ok).collect();
            if items.len() == 1 && items[0].path().is_dir() {
                // It's likely a nested root (standard for Node/MinGW zips)
                source_to_copy = items[0].path();
            }
        }

        println!("  📦 Instalando en {}...", target_path.display());
        let options = fs_extra::dir::CopyOptions::new().content_only(true);
        fs::create_dir_all(&target_path).unwrap();

        if let Err(e) = fs_extra::dir::copy(&source_to_copy, &target_path, &options) {
            eprintln!("❌ Falla al mover archivos: {}", e);
        } else {
            println!("  ✨ Instalado correctamente.");
            found_tools.push((name.to_string(), target_path));
        }

        // Cleanup
        let _ = fs::remove_file(&temp_zip);
        let _ = fs::remove_dir_all(&temp_extract);
    }
}

fn register_in_path(target_base: &Path) {
    println!("📝 Actualizando Registro de Usuario (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .unwrap();

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
        } else {
            println!("  ℹ️  Ya está en PATH: {}", p);
        }
    }

    if changed {
        let new_path_str = new_path_parts.join(";");
        match env_key.set_value("Path", &new_path_str) {
            Ok(_) => println!("✅ Registro actualizado correctamente."),
            Err(e) => eprintln!("❌ Error actualizando registro: {}", e),
        }
        println!("⚠️  Nota: Necesitas reiniciar tus terminales para ver los cambios.");
    } else {
        println!("✨ El PATH ya estaba configurado.");
    }
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

pub fn clean_system() {
    println!("🧹 Limpiando Entorno Brisas del Sistema...");

    // 1. Define paths
    let local_app_data = env::var("LOCALAPPDATA").expect("No se encontró %LOCALAPPDATA%");
    let target_base = PathBuf::from(&local_app_data);

    let tools = vec!["node", "mingw64", "pwsh"];

    // 2. Remove Files
    for tool in &tools {
        let path = target_base.join(tool);
        if path.exists() {
            println!("  🔥 Eliminando carpeta: {}", path.display());
            if let Err(e) = fs::remove_dir_all(&path) {
                eprintln!("❌ Error eliminando {}: {}", tool, e);
            } else {
                println!("    ✨ Eliminado.");
            }
        }
    }

    // 3. Clean Registry
    println!("📝 Limpiando Registro de Usuario (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // Use open_subkey_with_flags or open_subkey (read/write implied if not specified differently in some versions but safer with flags)
    let env_key = match hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("❌ No se pudo abrir el registro: {}", e);
            return;
        }
    };

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

    // Filter keep parts
    let new_parts: Vec<&str> = parts
        .into_iter()
        .filter(|part| {
            !part.is_empty() && !paths_to_remove.iter().any(|remove| part.contains(remove))
        })
        .collect();

    let new_path_str = new_parts.join(";");

    // Safety check: Don't allow empty path unless it was empty/broken
    if new_path_str.len() < 5 && !current_path.is_empty() {
        println!("⚠️  Advertencia: El PATH resultante parece muy corto. Abortando actualización de registro por seguridad.");
        return;
    }

    if new_path_str != current_path {
        match env_key.set_value("Path", &new_path_str) {
            Ok(_) => println!("✅ Registro limpiado correctamente."),
            Err(e) => eprintln!("❌ Error actualizando registro: {}", e),
        }
        println!("⚠️  Reinicia tus terminales para ver los cambios.");
    } else {
        println!("✨ El registro ya estaba limpio.");
    }
}
