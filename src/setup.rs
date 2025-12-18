use crate::download;
use crate::errors::BeError;
use crate::manifest::Manifest;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use log::{error, info};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

// Helper function for directory copy with progress
fn copy_dir_with_progress(src: &Path, dst: &Path) -> Result<(), BeError> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            files.push(entry.path().to_owned());
        }
    }

    let pb = ProgressBar::new(files.len() as u64);
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green}  [{elapsed_precise}] ▕{bar:40.magenta/blue}▏ {pos}/{len} archivos ({eta})")
        .map_err(|e| BeError::Setup(format!("Error configurando barra de progreso: {}", e)))?
        .progress_chars("█░");
    pb.set_style(style);

    for file_path in files {
        let relative_path = file_path.strip_prefix(src).unwrap_or(&file_path);
        let dst_path = dst.join(relative_path);

        if let Some(parent) = dst_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::copy(&file_path, &dst_path)?;
        pb.inc(1);
    }
    pb.finish_with_message("Copia completada");
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

            // Use new helper
            copy_dir_with_progress(&folder, &target_path)?;

            found_tools.push((tool.name.clone(), target_path));
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

        // ensure_downloaded maneja Caché + Verificación SHA256
        let cached_zip = download::ensure_downloaded(&tool.url, &zip_name, tool.sha256.as_deref())?;

        // Extraer
        let temp_extract = std::env::temp_dir().join(format!("{}_extract", tool.name));
        if temp_extract.exists() {
            let _ = fs::remove_dir_all(&temp_extract);
        }

        download::extract_zip(&cached_zip, &temp_extract)?;

        // Mover a destino
        let mut source_to_copy = temp_extract.clone();
        if let Ok(entries) = fs::read_dir(&temp_extract) {
            let items: Vec<_> = entries.filter_map(Result::ok).collect();
            if items.len() == 1 && items[0].path().is_dir() {
                source_to_copy = items[0].path();
            }
        }

        println!("  📦 Instalando en {}...", target_path.display());

        // Use new helper instead of fs_extra
        copy_dir_with_progress(&source_to_copy, &target_path)?;

        println!("  ✨ Instalado correctamente.");
        found_tools.push((tool.name.clone(), target_path));

        // Limpieza (Solo dir temporal, mantén el Caché!)
        let _ = fs::remove_dir_all(&temp_extract);
    }
    Ok(())
}

pub fn setup_system() -> Result<(), BeError> {
    println!("🛠️  Configurando Entorno Brisas en el Sistema...");
    info!("Iniciando setup_system...");

    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| BeError::Config("No se encontró %LOCALAPPDATA%".into()))?;
    let target_base = PathBuf::from(&local_app_data);
    println!("📂 Destino: {}", target_base.display());

    // CARGAR MANIFIESTO
    let manifest_path = Path::new("tools.json");
    let manifest = if manifest_path.exists() {
        info!("Cargando manifiesto desde archivo local: tools.json");
        println!("📄 Usando manifiesto local: tools.json");
        match Manifest::load_from_file(manifest_path) {
            Ok(m) => m,
            Err(e) => {
                error!("Fallo al cargar tools.json local: {}", e);
                println!("⚠️  Error leyendo tools.json. Usando defaults.");
                Manifest::default()
            }
        }
    } else {
        let remote_url = "https://raw.githubusercontent.com/DQM27/brisas-cli/main/tools.json";
        info!("Obteniendo manifiesto remoto desde: {}", remote_url);
        println!("🌐 Buscando manifiesto remoto...");
        match Manifest::load_from_url(remote_url) {
            Ok(m) => m,
            Err(e) => {
                error!("Fallo carga remota: {}. Usando defaults compilados.", e);
                println!(
                    "⚠️  No se pudo cargar config remota (Offline?). Usando defaults internos."
                );
                Manifest::default()
            }
        }
    };
    info!(
        "Manifiesto cargado con {} herramientas.",
        manifest.tools.len()
    );

    let mut found_tools = Vec::new();

    // 1. Verificar existente
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
            handle_local_search(&manifest, &target_base, &mut found_tools)?;
        } else {
            handle_download(&manifest, &target_base, &mut found_tools)?;
        }
    }

    // Actualizar Registro
    register_in_path(&target_base)?;

    Ok(())
}

fn register_in_path(target_base: &Path) -> Result<(), BeError> {
    println!("📝 Actualizando Registro de Usuario (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| BeError::Setup(format!("Error abriendo registro: {}", e)))?;

    let current_path: String = match env_key.get_value("Path") {
        Ok(val) => val,
        Err(e) => {
            println!("⚠️  Advertencia: No se pudo leer el PATH actual: {}", e);
            String::new()
        }
    };
    let mut new_path_parts: Vec<String> = current_path.split(';').map(|s| s.to_string()).collect();
    let mut changed = false;

    // Lógica harcodeada para el registro PATH está bien por ahora,
    // o podríamos añadir `path_suffix` al Manifiesto si queremos desacoplamiento total.
    // Por ahora, manteniéndolo simple ya que las herramientas tienen carpetas bin específicas.
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

    // Crear Acceso Directo al Escritorio
    println!("🖥️  Creando Acceso Directo en el Escritorio...");
    create_desktop_shortcut(target_base)?;

    Ok(())
}

fn create_desktop_shortcut(target_base: &Path) -> Result<(), BeError> {
    let desktop =
        dirs::desktop_dir().ok_or(BeError::Setup("No se encontró el Escritorio".into()))?;
    let link_path = desktop.join("Brisas Shell.lnk");

    // Buscamos pwsh.exe en el sistema (Global) o local
    let pwsh_local = target_base.join("pwsh").join("pwsh.exe");
    let target = if pwsh_local.exists() {
        pwsh_local.to_string_lossy().to_string()
    } else {
        "pwsh.exe".to_string()
    };

    // Comando PS para crear acceso directo
    let script = format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $s = $ws.CreateShortcut('{}'); \
         $s.TargetPath = '{}'; \
         $s.WorkingDirectory = '{}'; \
         $s.Description = 'Brisas Portable Shell'; \
         $s.Save()",
        link_path.display(),
        target,
        dirs::home_dir().unwrap_or(PathBuf::from("C:\\")).display()
    );

    let status = std::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&script)
        .status()
        .map_err(|e| BeError::Setup(format!("Error ejecutando PowerShell para shortcut: {}", e)))?;

    if status.success() {
        println!("  ✅ Acceso directo creado: {}", link_path.display());
    } else {
        println!("  ⚠️  No se pudo crear el acceso directo (probablemente permisos).");
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
    info!("Iniciando clean_system...");

    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| BeError::Config("No se encontró %LOCALAPPDATA%".into()))?;
    let target_base = PathBuf::from(&local_app_data);

    let tools = vec!["node", "mingw64", "pwsh"];

    // 2. Eliminar Archivos (Instalación)
    for tool in &tools {
        let path = target_base.join(tool);
        if path.exists() {
            println!("  🔥 Eliminando carpeta: {}", path.display());
            if let Err(e) = fs::remove_dir_all(&path) {
                error!("Fallo al eliminar directorio {}: {}", path.display(), e);
                eprintln!("❌ Error eliminando {}: {}", tool, e);
            } else {
                info!("Directorio eliminado: {}", path.display());
                println!("    ✨ Eliminado.");
            }
        }
    }

    // 2.1 Eliminar Caché de Descargas
    let cache_dir = std::env::temp_dir().join("BrisasEnv_Cache");
    if cache_dir.exists() {
        println!(
            "  🗑️  Eliminando caché de descargas: {}",
            cache_dir.display()
        );
        if let Err(e) = fs::remove_dir_all(&cache_dir) {
            eprintln!("❌ Error eliminando caché: {}", e);
        } else {
            println!("    ✨ Caché eliminado.");
        }
    }

    // 3. Limpiar Registro
    println!("📝 Limpiando Registro de Usuario (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // Usar open_subkey_with_flags
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| BeError::Setup(format!("Error abriendo registro: {}", e)))?;

    let current_path: String = match env_key.get_value("Path") {
        Ok(val) => val,
        Err(e) => {
            println!("⚠️  Advertencia: No se pudo leer el PATH actual: {}", e);
            String::new()
        }
    };
    let parts: Vec<&str> = current_path.split(';').collect();

    let paths_to_remove = [
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
        info!("Registro limpiado exitosamente.");
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

    // 1. Archivos
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

    // 2. Registro
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
