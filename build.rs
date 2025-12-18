use std::io;

#[cfg(windows)]
fn main() -> io::Result<()> {
    let mut res = winres::WindowsResource::new();
    res.set("FileDescription", "Gestor de Entorno Brisas");
    res.set("ProductName", "Brisas CLI");
    res.set("CompanyName", "Equipo Brisas");
    res.set("LegalCopyright", "Copyright (c) 2025 Equipo Brisas");
    res.set("OriginalFilename", "be.exe");
    res.compile()?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    // No-op on non-Windows
}
