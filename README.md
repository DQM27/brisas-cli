# Brisas Env CLI (be)

Your portable, robust, and self-updating development environment manager.

## 🚀 Features

*   **Zero Dependencies**: Just download `be.exe`.
*   **Auto-Setup**: Installs Node.js, MinGW (GCC), and PowerShell automatically.
*   **Portable Shell**: `be shell` launches a terminal with all tools in PATH.
*   **Admin Tools**: Built-in manifest generator and auto-updater.
*   **Verify Integrity**: All downloads are SHA256 verified.

## 🛠️ Installation

1.  Download `be.exe` from [Releases](https://github.com/DQM27/brisas-cli/releases).
2.  Put it anywhere (e.g., `C:\Brisas`).
3.  Run it!

## 📖 Usage

### Interactive Mode
Double-click `be.exe` to see the menu.

### Command Line
```powershell
be setup        # Install tools
be shell        # Open portable terminal
be run npm start # Run a command
be status       # Check health
be clean        # Uninstall everything
```

### Admin (Update Tools)
```powershell
be manifest-gen
```
1. Paste new URLs.
2. Auto-hashes and updates `tools.json`.
3. Pushes to GitHub automatically.

## 📦 Build from Source

```powershell
cargo build --release
```
