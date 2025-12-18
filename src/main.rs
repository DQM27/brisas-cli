mod admin; // New module
mod cli;
mod config;
mod download;
mod errors;
mod logger;
mod manifest;
mod run;
mod setup;

use clap::Parser;
use cli::{Cli, Commands};
use config::{ensure_config, print_config};
use inquire::Select;
use log::{error, info};
use run::run_command;

fn main() {
    // Init Logger ignoring errors (fallback to no logs is fine for CLI ux)
    let _ = logger::init();
    info!("Brisas CLI iniciado.");

    if std::env::args().len() == 1 {
        interactive_menu();
        info!("Menú interactivo cerrado.");
        return;
    }

    let cli = Cli::parse();

    if let Err(e) = execute_command(&cli) {
        error!("Error Fatal: {}", e);
        eprintln!("\n❌ Error Fatal: {}", e);
        std::process::exit(1);
    }
}

fn execute_command(cli: &Cli) -> Result<(), errors::BeError> {
    match &cli.command {
        Commands::Init => {
            println!("🔍 Inicializando entorno...");
            let config = ensure_config()?;
            print_config(&config);
        }
        Commands::Run { args } => {
            if args.is_empty() {
                return Err(errors::BeError::Config(
                    "No se proporcionó ningún comando.".into(),
                ));
            }
            let config = ensure_config()?;
            run_command(&config, &args[0], &args[1..]);
        }
        Commands::Shell => {
            let config = ensure_config()?;
            let shell = "pwsh";
            println!("🚀 Iniciando terminal portable ({})", shell);
            run_command(&config, shell, &[]);
        }
        Commands::Setup => {
            setup::setup_system()?;
        }
        Commands::Clean => {
            setup::clean_system()?;
        }
        Commands::Status => {
            setup::check_status();
        }
        Commands::Help => {
            print_help();
        }
        Commands::ManifestGen => {
            admin::generate_manifest()?;
        }
    }
    Ok(())
}

fn print_help() {
    println!("🌟 MANUAL DE USUARIO BRISAS ENV CLI 🌟");
    println!("--------------------------------------");
    println!("Este programa te ayuda a instalar y gestionar Node, MinGW y PowerShell sin ensuciar tu sistema.");
    println!();
    println!("🛠️  COMANDOS DISPONIBLES:");
    println!(
        "  init              -> Crea la configuración inicial (.dev-env-config) si no existe."
    );
    println!(
        "  setup             -> ⬇️  DESCARGA E INSTALA automáticamente Node.js, GCC y PowerShell."
    );
    println!("                       También añade estas herramientas a tu PATH (temporalmente o en registro).");
    println!("  clean             -> 🧹 DESINSTALADOR COMPLETO. Borra las carpetas descargadas y");
    println!("                       limpia cualquier rastro dejado en el Registro de Windows.");
    println!("  status            -> 🔍 DIAGNÓSTICO. Te dice si falta algo y si las variables de entorno");
    println!("                       están bien configuradas.");
    println!(
        "  shell             -> 🚀 Abre una nueva terminal (PowerShell) con todas las herramientas"
    );
    println!("                       cargadas y listas para usar.");
    println!("  run <cmd>         -> Ejecuta un comando suelto dentro del entorno 'mágico'.");
    println!("                       Ejemplo: 'be run npm start'");
    println!("  help              -> Muestra esta pantalla de ayuda.");
    println!();
    println!(
        "💡 TRUCO: Si ejecutas 'be.exe' (doble click) sin comandos, verás un MENÚ INTERACTIVO."
    );
}

fn interactive_menu() {
    println!("🌟 Brisas Env Manager (CLI) 🌟");

    let options = vec![
        "🚀 Iniciar Shell Portable",
        "🛠️  Instalar / Reparar (Setup)",
        "🔍 Verificar Estado (Status)",
        "❓ Ayuda / Que es esto?",
        "🧹 Desinstalar (Clean)",
        "⚙️  Administración (Manifest Gen)",
        "❌ Salir",
    ];

    loop {
        let ans = Select::new("Selecciona una opción (Usa las flechas):", options.clone()).prompt();
        match ans {
            Ok(choice) => {
                let result = match choice {
                    "🚀 Iniciar Shell Portable" => ensure_config().map(|config| {
                        run_command(&config, "pwsh", &[]);
                    }),
                    "🛠️  Instalar / Reparar (Setup)" => setup::setup_system(),
                    "🔍 Verificar Estado (Status)" => {
                        setup::check_status();
                        Ok(())
                    }
                    "❓ Ayuda / Que es esto?" => {
                        print_help();
                        Ok(())
                    }
                    "🧹 Desinstalar (Clean)" => setup::clean_system(),
                    "⚙️  Administración (Manifest Gen)" => admin::generate_manifest(),
                    "❌ Salir" => break,
                    _ => Ok(()),
                };

                if let Err(e) = result {
                    eprintln!("\n❌ Ocurrió un error: {}", e);
                    error!("Error en menú interactivo: {}", e);
                    println!("Presiona Enter para continuar...");
                    let _ = std::io::stdin().read_line(&mut String::new());
                }
            }
            Err(_) => {
                info!("Menú cancelado o interrumpido.");
                break;
            }
        }
        println!("\n--- Operación finalizada. ---\n");
    }
}
