use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let local_app_data = match env::var("LOCALAPPDATA") {
        Ok(val) => val,
        Err(_) => ".".to_string(),
    };
    let log_dir = PathBuf::from(&local_app_data).join("BrisasEnv");

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    let log_file = log_dir.join("be.log");

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create(log_file)?,
    )])?;

    log::info!("Logger initialized successfully.");
    Ok(())
}
