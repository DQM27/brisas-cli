use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = ".dev-env-config";

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvConfig {
    #[serde(rename = "NodePath")]
    pub node_path: Option<String>,
    #[serde(rename = "MinGWPath")]
    pub mingw_path: Option<String>,
    #[serde(rename = "LastUpdated")]
    pub last_updated: String,
}

pub fn ensure_config() -> EnvConfig {
    if Path::new(CONFIG_FILE).exists() {
        if let Ok(content) = fs::read_to_string(CONFIG_FILE) {
            if let Ok(config) = serde_json::from_str::<EnvConfig>(&content) {
                return config;
            }
        }
    }

    // Fallback: If installed via setup, look in AppData
    let local = env::var("LOCALAPPDATA").unwrap_or_default();
    let app_data = PathBuf::from(local);
    let node_app = app_data.join("node");
    let mingw_app = app_data.join("mingw64");

    let mut node_path = None;
    let mut mingw_path = None;

    if node_app.join("node.exe").exists() {
        node_path = Some(node_app.to_string_lossy().to_string());
    }
    if mingw_app.join("bin/gcc.exe").exists() {
        mingw_path = Some(mingw_app.to_string_lossy().to_string());
    }

    // If still not found, try old logic
    if node_path.is_none() {
        node_path = find_node(&env::current_dir().unwrap());
    }
    if mingw_path.is_none() {
        mingw_path = find_mingw(&env::current_dir().unwrap());
    }

    let config = EnvConfig {
        node_path,
        mingw_path,
        last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    let _ = fs::write(CONFIG_FILE, json);

    config
}

fn find_node(_base: &Path) -> Option<String> {
    let common = PathBuf::from(r"C:\Users\femprobrisas\node");
    if common.exists() {
        if let Ok(entries) = fs::read_dir(&common) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("node.exe").exists() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

fn find_mingw(base: &Path) -> Option<String> {
    if let Some(parent) = base.parent() {
        let mingw = parent.join("mingw64");
        if mingw.join("bin/gcc.exe").exists() {
            return Some(mingw.to_string_lossy().to_string());
        }
    }
    None
}

pub fn print_config(config: &EnvConfig) {
    if let Some(ref p) = config.node_path {
        println!("✅ Node: {}", p);
    }
    if let Some(ref p) = config.mingw_path {
        println!("✅ MinGW: {}", p);
    }
}
