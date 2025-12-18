use crate::errors::BeError;
use crate::installer;
use crate::manifest::Manifest;
use crate::ui;
use inquire::MultiSelect;
use log::{error, info};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

pub fn setup_system() -> Result<(), BeError> {
    ui::print_banner();

    // 1. Prepare Environment
    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| BeError::Config("No se encontro %LOCALAPPDATA%".into()))?;
    let target_base = PathBuf::from(&local_app_data);
    ui::print_step(&format!("Ruta Destino: {}", target_base.display()));

    // 2. Load Manifest
    let manifest_path = Path::new("tools.json");
    let manifest = if manifest_path.exists() {
        ui::print_step("Cargando tools.json local...");
        match Manifest::load_from_file(manifest_path) {
            Ok(m) => m,
            Err(e) => {
                ui::print_error(&format!("Error en json local: {}. Usando defaults.", e));
                Manifest::default()
            }
        }
    } else {
        ui::print_step("Cargando tools.json remoto...");
        Manifest::default() // En v2.0 forzamos defaults si no hay local, o implementamos fetch
    };

    // 3. Multi-Select Menu
    ui::print_retro_box(
        "SELECCION DE HERRAMIENTAS",
        &[
            "Marca con ESPACIO las herramientas que deseas instalar.",
            "Presiona ENTER para confirmar.",
        ],
    );

    let tool_names: Vec<&str> = manifest.tools.iter().map(|t| t.name.as_str()).collect();
    // Default selection: Node, MinGW, Git, VSCodium, PowerShell
    let defaults = vec![0, 1, 2, 3, 4]; // Indexes matching manifest order roughly

    let selected_tools = MultiSelect::new("Herramientas a instalar:", tool_names)
        .with_default(&defaults)
        .prompt()
        .map_err(|_| BeError::Cancelled)?;

    if selected_tools.is_empty() {
        ui::print_error("No seleccionaste nada. Saliendo...");
        return Ok(());
    }

    // 4. Install Loop
    let mut installed_tools = Vec::new();
    for tool_name in selected_tools {
        if let Some(tool) = manifest.tools.iter().find(|t| t.name == tool_name) {
            installer::install_tool(tool, &target_base)?;
            installed_tools.push(tool.clone());
        }
    }

    // 5. Register in Path & Shortcuts
    if !installed_tools.is_empty() {
        register_in_path(&target_base, &installed_tools)?;
    }

    ui::print_farewell();
    Ok(())
}

fn register_in_path(
    target_base: &Path,
    installed_tools: &[crate::manifest::Tool],
) -> Result<(), BeError> {
    ui::print_step("Actualizando Registro (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| BeError::Setup(format!("Error abriendo registro: {}", e)))?;

    let current_path: String = env_key.get_value("Path").unwrap_or_default();
    let mut new_path_parts: Vec<String> = current_path.split(';').map(|s| s.to_string()).collect();
    let mut changed = false;

    // Define paths based on installed tools
    let mut paths_to_add = Vec::new();

    // Always check what is installed
    if installed_tools.iter().any(|t| t.name == "node") {
        paths_to_add.push(target_base.join("node").to_string_lossy().to_string());
    }
    if installed_tools.iter().any(|t| t.name == "mingw64") {
        paths_to_add.push(
            target_base
                .join("mingw64")
                .join("bin")
                .to_string_lossy()
                .to_string(),
        );
    }
    if installed_tools.iter().any(|t| t.name == "pwsh") {
        paths_to_add.push(target_base.join("pwsh").to_string_lossy().to_string());
    }
    if installed_tools.iter().any(|t| t.name == "git") {
        paths_to_add.push(
            target_base
                .join("git")
                .join("bin")
                .to_string_lossy()
                .to_string(),
        );
        paths_to_add.push(
            target_base
                .join("git")
                .join("cmd")
                .to_string_lossy()
                .to_string(),
        );
    }
    if installed_tools.iter().any(|t| t.name == "vscodium") {
        paths_to_add.push(
            target_base
                .join("vscodium")
                .join("bin")
                .to_string_lossy()
                .to_string(),
        );
    }
    if installed_tools.iter().any(|t| t.name == "rustup") {
        if let Ok(home) = env::var("USERPROFILE") {
            paths_to_add.push(format!("{}\\.cargo\\bin", home));
        }
    }

    for p in paths_to_add {
        if !new_path_parts.contains(&p) {
            new_path_parts.push(p.clone());
            ui::print_step(&format!("Anadiendo al PATH: {}", p));
            changed = true;
        }
    }

    if changed {
        let new_path_str = new_path_parts.join(";");
        env_key
            .set_value("Path", &new_path_str)
            .map_err(|e| BeError::Setup(format!("Error escribiendo registro: {}", e)))?;
        ui::print_success("Registro actualizado.");
    } else {
        ui::print_success("El PATH ya estaba correcto.");
    }

    // Shortcuts
    create_shortcuts(target_base, installed_tools)?;

    Ok(())
}

fn create_shortcuts(
    target_base: &Path,
    installed_tools: &[crate::manifest::Tool],
) -> Result<(), BeError> {
    let desktop = dirs::desktop_dir().ok_or(BeError::Setup("No Desktop".into()))?;

    // START MENU
    let start_menu = dirs::data_dir().map(|d| d.join("Microsoft/Windows/Start Menu/Programs"));

    for tool in installed_tools {
        let (name, target, desc) = match tool.name.as_str() {
            "pwsh" => (
                "PowerShell Portable".to_string(),
                "pwsh.exe",
                "PowerShell 7",
            ),
            "vscodium" => (
                "VSCodium Portable".to_string(),
                "VSCodium.exe",
                "VSCodium Editor",
            ),
            "git" => (
                "Git Bash Portable".to_string(),
                "git-bash.exe",
                "Git Terminal",
            ),
            _ => continue,
        };

        // Determine real target path
        let real_target = if tool.name == "git" {
            target_base.join(&tool.name).join(target) // Git has it in root but sometimes not
        } else {
            target_base.join(&tool.name).join(target)
        };
        // Git Bash special case: it might be in root of git folder

        let link_path = desktop.join(format!("{}.lnk", name));

        create_shortcut_impl(
            target_base,
            &link_path,
            &real_target.to_string_lossy(),
            desc,
        )?;

        // Try start menu
        if let Some(ref start) = start_menu {
            if start.exists() {
                let sm_link = start.join(format!("{}.lnk", name));
                let _ = create_shortcut_impl(
                    target_base,
                    &sm_link,
                    &real_target.to_string_lossy(),
                    desc,
                );
            }
        }
    }
    Ok(())
}

fn create_shortcut_impl(
    _target_base: &Path,
    link_path: &Path,
    target_exe: &str,
    desc: &str,
) -> Result<(), BeError> {
    let script = format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $s = $ws.CreateShortcut('{}'); \
         $s.TargetPath = '{}'; \
         $s.WorkingDirectory = '{}'; \
         $s.Description = '{}'; \
         $s.Save()",
        link_path.display(),
        target_exe,
        dirs::home_dir().unwrap_or(PathBuf::from("C:\\")).display(),
        desc
    );

    let status = std::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&script)
        .status()
        .map_err(|e| BeError::Setup(format!("Error PS Shortcut: {}", e)))?;

    if status.success() {
        ui::print_success(&format!(
            "Acceso directo: {}",
            link_path.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    Ok(())
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
            println!("  Eliminando carpeta: {}", path.display());
            if let Err(e) = fs::remove_dir_all(&path) {
                error!("Fallo al eliminar directorio {}: {}", path.display(), e);
                eprintln!("Error eliminando {}: {}", tool, e);
            } else {
                info!("Directorio eliminado: {}", path.display());
                println!("    Eliminado.");
            }
        }
    }

    // 2.1 Eliminar Cache de Descargas
    let cache_dir = std::env::temp_dir().join("BrisasEnv_Cache");
    if cache_dir.exists() {
        println!("  Borrando cache de descargas: {}", cache_dir.display());
        if let Err(e) = fs::remove_dir_all(&cache_dir) {
            eprintln!("Error eliminando cache: {}", e);
        } else {
            println!("    Cache eliminado.");
        }
    }

    // 3. Limpiar Registro
    println!("Limpiando Registro de Usuario (PATH)...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // Usar open_subkey_with_flags
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| BeError::Setup(format!("Error abriendo registro: {}", e)))?;

    let current_path: String = match env_key.get_value("Path") {
        Ok(val) => val,
        Err(e) => {
            println!("Advertencia: No se pudo leer el PATH actual: {}", e);
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
        println!("Advertencia: El PATH resultante parece muy corto. Abortando actualizacion.");
        return Ok(());
    }

    if new_path_str != current_path {
        env_key
            .set_value("Path", &new_path_str)
            .map_err(|e| BeError::Setup(format!("Error guardando registro: {}", e)))?;
        println!("Registro limpiado correctamente.");
        println!("Nota: Reinicia tus terminales para ver los cambios.");
        info!("Registro limpiado exitosamente.");
    } else {
        println!("El registro ya estaba limpio.");
    }
    Ok(())
}

pub fn check_status() {
    println!("Verificando Estado del Sistema...");

    let tools = vec![
        ("node", "--version"),
        ("gcc", "--version"),
        ("pwsh", "--version"),
        ("git", "--version"),
        ("git-lfs", "--version"),
        ("rustc", "--version"),
        ("cargo", "--version"),
    ];

    println!("\nPrueba de Ejecución (Detecta instalaciones globales y portables):");
    let mut all_ok = true;

    for (cmd, arg) in tools {
        match std::process::Command::new(cmd).arg(arg).output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("Detected")
                    .to_string();
                let v_short = if version.len() > 25 {
                    &version[..25]
                } else {
                    &version
                };
                println!("  [x] {:<10} : Funcionando ({})", cmd, v_short.trim());
            }
            _ => {
                println!("  [ ] {:<10} : No detectado en PATH actual", cmd);
                all_ok = false;
            }
        }
    }

    println!("\nNota: Si acabas de instalar, reinicia tu terminal para recargar el PATH.");

    if all_ok {
        println!("\n¡Sistema Operativo al 100%! 🚀");
    } else {
        println!("\nAlgunas herramientas no responden. Verifica tu instalación.");
    }
}
