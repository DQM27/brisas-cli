use thiserror::Error;

#[derive(Error, Debug)]
pub enum BeError {
    #[error("Error de IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error de Red: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Error de ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Error de Configuración: {0}")]
    Config(String),

    #[error("Error de Setup: {0}")]
    Setup(String),

    #[error("Operación cancelada por el usuario.")]
    Cancelled,
}
