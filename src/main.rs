mod cli;
mod config;
mod download;
mod run;
mod setup;

use clap::Parser;
use cli::{Cli, Commands};
use config::{ensure_config, print_config};
use run::run_command;
use setup::setup_system;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            println!("🔍 Inicializando entorno...");
            let config = ensure_config();
            print_config(&config);
        }
        Commands::Run { args } => {
            if args.is_empty() {
                eprintln!("Error: No se proporcionó ningún comando.");
                std::process::exit(1);
            }
            let config = ensure_config();
            run_command(&config, &args[0], &args[1..]);
        }
        Commands::Shell => {
            let config = ensure_config();
            let shell = "pwsh";
            println!("🚀 Iniciando terminal portable ({})", shell);
            run_command(&config, shell, &[]);
        }
        Commands::Setup => {
            setup_system();
        }
        Commands::Clean => {
            setup::clean_system();
        }
    }
}
