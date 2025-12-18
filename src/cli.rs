use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "be")]
#[command(about = "Gestor de Entorno Brisas - Herramientas de Desarrollo Portables", long_about = None)]
#[command(disable_help_subcommand = true)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Ejecutar un comando en el entorno portable
    Run {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Iniciar una terminal con el entorno portable
    Shell,
    /// Instalar herramientas en el sistema (AppData\Local)
    Setup,
    /// Desinstalar herramientas y limpiar registro
    Clean,
    /// Verificar estado de la instalación
    Status,
    /// Ver lista de comandos y ayuda
    Help,
    /// (Admin) Generar/Actualizar el manifiesto tools.json
    ManifestGen,
}
