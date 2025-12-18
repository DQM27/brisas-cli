use console::{style, Term};

pub fn print_banner() {
    let term = Term::stdout();
    let _ = term.clear_screen();

    let banner = r#"
  ╔═════════════════════════════════════════════════════════════╗
  ║                                                             ║
  ║   ____  ____  ___  ____    _    ____     ____ _     ___     ║
  ║  | __ )|  _ \|_ _|( __ )  / \  / ___|   / ___| |   |_ _|    ║
  ║  |  _ \| |_) || | /  __ \/ _ \ \___ \  | |   | |    | |     ║
  ║  | |_) |  _ < | | \ __  / ___ \ ___) | | |___| |___ | |     ║
  ║  |____/|_| \_\___| \___/_/   \_\____/   \____|_____|___|    ║
  ║                                                             ║
  ║            Portable Configuration Setup v2.0                ║
  ║                                                             ║
  ╚═════════════════════════════════════════════════════════════╝
    "#;

    println!("{}", style(banner).cyan().bold());
    println!(
        "{}",
        style("      Sistema de Inicialización de Entorno de Desarrollo").dim()
    );
    println!();
}

pub fn print_step(msg: &str) {
    println!(" {} {}", style(">>").green().bold(), msg);
}

pub fn print_success(msg: &str) {
    println!(" {} {}", style("OK").green().bold(), msg);
}

pub fn print_error(msg: &str) {
    println!(" {} {}", style("ERROR").red().bold(), msg);
}

pub fn print_retro_box(title: &str, content: &[&str]) {
    let width = 60;
    let title_len = title.len();
    let _padding = (width - title_len) / 2;

    println!(" ╔{}╗", "═".repeat(width));
    println!(" ║{:^width$}║", title, width = width);
    println!(" ╠{}╣", "═".repeat(width));
    for line in content {
        println!(" ║ {:<width$}║", line, width = width - 1);
    }
    println!(" ╚{}╝", "═".repeat(width));
}

pub fn print_farewell() {
    println!();
    let msg = r#"
    ¡Gracias por usar Brisas CLI! 
    
    Recuerda: "El código es arte, y tu eres el artista."
    
    Escribe 'be help' para comenzar.
    "#;
    println!("{}", style(msg).cyan().italic());
}
