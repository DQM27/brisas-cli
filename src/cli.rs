use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "be")]
#[command(about = "Gestor de Entorno Brisas - Herramientas de Desarrollo Portables", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inicializar o verificar el entorno
    Init,
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
}
