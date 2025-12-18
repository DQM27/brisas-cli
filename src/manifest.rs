use serde::{Deserialize, Serialize};
use crate::errors::BeError;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub name: String,
    pub version: String,
    pub url: String,
    pub check_file: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub tools: Vec<Tool>,
}

impl Manifest {
    pub fn default() -> Self {
        Manifest {
            tools: vec![
                Tool {
                    name: "node".into(),
                    version: "22.12.0".into(),
                    url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-win-x64.zip".into(),
                    check_file: "node.exe".into(),
                    sha256: None, 
                },
                Tool {
                    name: "mingw64".into(),
                    version: "14.2.0".into(),
                    url: "https://github.com/brechtsanders/winlibs_mingw/releases/download/14.2.0posix-19.1.1-12.0.0-ucrt-r2/winlibs-x86_64-posix-seh-gcc-14.2.0-llvm-19.1.1-mingw-w64ucrt-12.0.0-r2.zip".into(),
                    check_file: "bin/gcc.exe".into(),
                    sha256: None,
                },
                Tool {
                    name: "pwsh".into(),
                    version: "7.4.6".into(),
                    url: "https://github.com/PowerShell/PowerShell/releases/download/v7.4.6/PowerShell-7.4.6-win-x64.zip".into(),
                    check_file: "pwsh.exe".into(),
                    sha256: None,
                },
            ],
        }
    }

    #[allow(dead_code)]
    pub fn load_from_url(url: &str) -> Result<Self, BeError> {
        let resp = reqwest::blocking::get(url)?;
        if let Err(e) = resp.error_for_status_ref() {
            return Err(BeError::Reqwest(e));
        }
        let manifest: Manifest = resp.json()?;
        Ok(manifest)
    }

    pub fn load_from_file(path: &Path) -> Result<Self, BeError> {
        let content = fs::read_to_string(path)?;
        let manifest: Manifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), BeError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| BeError::Config(format!("Error de JSON: {}", e)))?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let json = r#"
        {
            "tools": [
                {
                    "name": "test_tool",
                    "version": "1.0",
                    "url": "http://example.com/tool.zip",
                    "check_file": "bin/tool.exe"
                }
            ]
        }
        "#;

        let manifest: Manifest = serde_json::from_str(json).expect("Deberia parsear correctamente");
        assert_eq!(manifest.tools.len(), 1);
        assert_eq!(manifest.tools[0].name, "test_tool");
        assert_eq!(manifest.tools[0].check_file, "bin/tool.exe");
    }
}
