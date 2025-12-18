use crate::errors::BeError;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvConfig {
    #[serde(rename = "NodePath")]
    pub node_path: Option<String>,
    #[serde(rename = "MinGWPath")]
    pub mingw_path: Option<String>,
    #[serde(rename = "LastUpdated")]
    pub last_updated: String,
}

pub fn get_env_config() -> Result<EnvConfig, BeError> {
    // Look in AppData (Standard installation)
    let local = env::var("LOCALAPPDATA")
        .map_err(|_| BeError::Config("No se encontro %LOCALAPPDATA%".into()))?;

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

    // Return config struct (LastUpdated is dummy/current)
    Ok(EnvConfig {
        node_path,
        mingw_path,
        last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}
