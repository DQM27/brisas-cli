use std::io;

#[cfg(windows)]
fn main() -> io::Result<()> {
    let mut res = winres::WindowsResource::new();
    res.set("FileDescription", "Brisas Environment Manager");
    res.set("ProductName", "Brisas CLI");
    res.set("CompanyName", "Brisas Team");
    res.set("LegalCopyright", "Copyright (c) 2025 Brisas Team");
    res.set("OriginalFilename", "be.exe");
    res.compile()?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    // No-op on non-Windows
}
