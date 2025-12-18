# ============================================
# ENTORNO DE DESARROLLO COMPLETO (PORTABLE)
# Node.js + MinGW64 + Rust
# ============================================

param(
    [switch]$Verify,
    [switch]$Reconfigure,
    [switch]$Clean
)

# Suprimir errores temporalmente
$ErrorActionPreference = "SilentlyContinue"

# ============================================
# 1. RUTAS Y CONFIGURACIÓN
# ============================================
$script:ProjectDir = $PSScriptRoot
$script:ConfigFile = Join-Path $ProjectDir ".dev-env-config"

# ============================================
# 2. FUNCIONES DE BÚSQUEDA
# ============================================

function Find-NodeInstallation {
    param([string]$StartPath = $ProjectDir)
    
    # Buscar en ubicaciones comunes
    $searchPaths = @(
        "C:\Users\femprobrisas\node",
        (Join-Path $env:USERPROFILE "node"),
        (Join-Path $StartPath "node"),
        (Join-Path (Split-Path $StartPath -Parent) "node")
    )
    
    $patterns = @("node-v*-win-x64", "node", "nodejs")
    
    foreach ($basePath in $searchPaths) {
        if (Test-Path "$basePath\node.exe") {
            return $basePath
        }
        
        foreach ($pattern in $patterns) {
            $matches = Get-ChildItem -Path (Split-Path $basePath -Parent) -Directory -Filter $pattern -ErrorAction SilentlyContinue
            foreach ($match in $matches) {
                if (Test-Path "$($match.FullName)\node.exe") {
                    return $match.FullName
                }
            }
        }
    }
    
    return $null
}

function Find-MinGW64Installation {
    param([string]$StartPath = $ProjectDir)
    
    # Buscar en proyecto
    $mingwDir = Join-Path $StartPath "mingw64"
    if (Test-Path "$mingwDir\bin\gcc.exe") {
        return $mingwDir
    }
    
    # Buscar recursivamente (máximo 3 niveles)
    $found = Get-ChildItem -Path $StartPath -Recurse -Filter "gcc.exe" -Depth 3 -ErrorAction SilentlyContinue |
             Select-Object -First 1
    
    if ($found) {
        return $found.Directory.Parent.FullName
    }
    
    return $null
}

# ============================================
# 3. GUARDAR/CARGAR CONFIGURACIÓN
# ============================================

function Save-Configuration {
    param(
        [string]$NodePath,
        [string]$MinGWPath
    )
    
    $config = @{
        NodePath = $NodePath
        MinGWPath = $MinGWPath
        LastUpdated = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    }
    
    $config | ConvertTo-Json | Out-File $ConfigFile -Encoding UTF8
}

# ============================================
# 4. LIMPIAR CONFIGURACIÓN
# ============================================

if ($Clean) {
    if (Test-Path $ConfigFile) {
        Remove-Item $ConfigFile -Force
        Write-Host "[OK] Configuración limpiada" -ForegroundColor Green
    } else {
        Write-Host "[INFO] No hay configuración guardada" -ForegroundColor Gray
    }
    exit 0
}

# ============================================
# 5. BUSCAR HERRAMIENTAS
# ============================================

# ============================================
# 5. CARGAR O BUSCAR HERRAMIENTAS
# ============================================

$loadedFromConfig = $false

# 1. Intentar Cargar Configuración Existente
if (Test-Path $ConfigFile) {
    try {
        $config = Get-Content $ConfigFile | ConvertFrom-Json
        
        $validNode = $false
        if ($config.NodePath -and (Test-Path "$($config.NodePath)\node.exe")) {
            $global:NodeDir = $config.NodePath
            $validNode = $true
        }

        $validMinGW = $false
        if ($config.MinGWPath -and (Test-Path "$($config.MinGWPath)\bin\gcc.exe")) {
            $global:MinGWDir = $config.MinGWPath
            $validMinGW = $true
        }

        if ($validNode -or $validMinGW) {
            Write-Host "📂 Configuración cargada desde .dev-env-config" -ForegroundColor Cyan
            $loadedFromConfig = $true
        }
    } catch {
        Write-Host "[WARN] Error leyendo configuración. Se buscará de nuevo." -ForegroundColor Yellow
    }
}

# 2. Si no se cargó (o faltan herramientas), Buscar
if (-not $global:NodeDir) {
    Write-Host "🔍 Buscando Node.js..." -ForegroundColor Cyan
    $global:NodeDir = Find-NodeInstallation
    if (-not $global:NodeDir) {
        Write-Host "[WARN] Node.js no encontrado" -ForegroundColor Yellow
    } else {
        Write-Host "  ✅ Node.js: $global:NodeDir" -ForegroundColor Green
    }
}

if (-not $global:MinGWDir) {
    Write-Host "🔍 Buscando MinGW64..." -ForegroundColor Cyan
    $global:MinGWDir = Find-MinGW64Installation
    if (-not $global:MinGWDir) {
        Write-Host "[WARN] MinGW64 no encontrado" -ForegroundColor Yellow
    } else {
        Write-Host "  ✅ MinGW64: $global:MinGWDir" -ForegroundColor Green
    }
}

# Verificar Rust
$cargoPath = "$env:USERPROFILE\.cargo\bin"
$global:RustInstalled = Test-Path "$cargoPath\cargo.exe"
if ($global:RustInstalled) {
    Write-Host "  ✅ Rust: $cargoPath" -ForegroundColor Green
}

# ============================================
# 6. CONFIGURAR NODE.JS
# ============================================

if ($global:NodeDir) {
    # Agregar al PATH
    $pathParts = $env:PATH -split ';'
    if ($global:NodeDir -notin $pathParts) {
        $env:PATH = "$global:NodeDir;$env:PATH"
    }
    
    # Variables de entorno
    $env:NODE_PATH = "$global:NodeDir\node_modules"
    
    # Funciones globales
    function global:npm {
        & "$global:NodeDir\node.exe" "$global:NodeDir\node_modules\npm\bin\npm-cli.js" @args
    }
    
    function global:npx {
        & "$global:NodeDir\node.exe" "$global:NodeDir\node_modules\npm\bin\npx-cli.js" @args
    }
}

# ============================================
# 7. CONFIGURAR MINGW64
# ============================================

if ($global:MinGWDir) {
    $mingwBin = Join-Path $global:MinGWDir "bin"
    
    # Agregar al PATH
    $pathParts = $env:PATH -split ';'
    if ($mingwBin -notin $pathParts) {
        $env:PATH = "$mingwBin;$env:PATH"
    }
    
    # Variables para compilación C/C++
    $env:CC  = Join-Path $mingwBin "gcc.exe"
    $env:CXX = Join-Path $mingwBin "g++.exe"
    $env:AR  = Join-Path $mingwBin "ar.exe"
    $env:C_INCLUDE_PATH     = Join-Path $global:MinGWDir "include"
    $env:CPLUS_INCLUDE_PATH = Join-Path $global:MinGWDir "include"
    $env:LIBRARY_PATH       = Join-Path $global:MinGWDir "lib"
}

# ============================================
# 8. CONFIGURAR RUST
# ============================================

if ($global:RustInstalled) {
    $pathParts = $env:PATH -split ';'
    if ($cargoPath -notin $pathParts) {
        $env:PATH = "$cargoPath;$env:PATH"
    }
}

# ============================================
# 9. GUARDAR CONFIGURACIÓN (SOLO SI CAMBIA)
# ============================================

# ============================================
# 9. GUARDAR CONFIGURACIÓN (SOLO SI ES NUEVA)
# ============================================

# Si NO cargamos desde config, significa que buscamos y encontramos algo -> Guardar.
if (-not $loadedFromConfig -and ($global:NodeDir -or $global:MinGWDir)) {
    Save-Configuration -NodePath $global:NodeDir -MinGWPath $global:MinGWDir
}

# ============================================
# 10. FUNCIONES HELPER
# ============================================

function global:Test-DevEnv {
    Write-Host "`n🔍 Verificación del entorno de desarrollo" -ForegroundColor Cyan
    Write-Host "═════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
    
    $tools = @(
        @{ Name = "node";   Command = "node --version" },
        @{ Name = "npm";    Command = "npm --version" },
        @{ Name = "pnpm";   Command = "pnpm --version" },
        @{ Name = "gcc";    Command = "gcc --version" },
        @{ Name = "g++";    Command = "g++ --version" },
        @{ Name = "rustc";  Command = "rustc --version" },
        @{ Name = "cargo";  Command = "cargo --version" }
    )
    
    foreach ($tool in $tools) {
        try {
            $version = Invoke-Expression "$($tool.Command) 2>&1" | Select-Object -First 1
            Write-Host "  ✅ " -NoNewline -ForegroundColor Green
            Write-Host "$($tool.Name.PadRight(10))" -NoNewline -ForegroundColor White
            Write-Host "$version" -ForegroundColor Gray
        } catch {
            Write-Host "  ❌ " -NoNewline -ForegroundColor Red
            Write-Host "$($tool.Name.PadRight(10))" -NoNewline -ForegroundColor White
            Write-Host "NO DISPONIBLE" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
    Write-Host "📁 Rutas configuradas:" -ForegroundColor Cyan
    if ($global:NodeDir)   { Write-Host "   Node.js: $global:NodeDir" -ForegroundColor Gray }
    if ($global:MinGWDir)  { Write-Host "   MinGW64: $global:MinGWDir" -ForegroundColor Gray }
    if ($global:RustInstalled) { Write-Host "   Rust:    $cargoPath" -ForegroundColor Gray }
    Write-Host ""
}

function global:dev {
    param(
        [switch]$Release,
        [switch]$Check,
        [switch]$Build
    )
    
    if (-not (Test-Path "Cargo.toml") -and -not (Test-Path "src-tauri/Cargo.toml")) {
        Write-Host "[ERROR] No se encontró Cargo.toml" -ForegroundColor Red
        return
    }
    
    if ($Check) {
        Write-Host "🔍 Verificando código..." -ForegroundColor Cyan
        cargo check
    } elseif ($Build) {
        if ($Release) {
            Write-Host "🚀 Compilando en modo release..." -ForegroundColor Cyan
            cargo build --release
        } else {
            Write-Host "🔨 Compilando..." -ForegroundColor Cyan
            cargo build
        }
    } else {
        if ($Release) {
            Write-Host "🚀 Ejecutando en modo release..." -ForegroundColor Cyan
            cargo run --release
        } else {
            Write-Host "🔨 Compilando y ejecutando..." -ForegroundColor Cyan
            cargo run
        }
    }
}

function global:tauri {
    param([Parameter(ValueFromRemainingArguments)]$args)
    
    if (-not (Test-Path "package.json")) {
        Write-Host "[ERROR] No se encontró package.json" -ForegroundColor Red
        return
    }
    
    npm run tauri @args
}

# ============================================
# 11. VERIFICACIÓN AUTOMÁTICA
# ============================================

if ($Verify) {
    Test-DevEnv
    exit 0
}

# ============================================
# 12. MENSAJE FINAL
# ============================================

Write-Host ""
Write-Host "✅ Entorno de desarrollo configurado" -ForegroundColor Green
Write-Host ""

# Resumen de herramientas
$available = @()
if ($global:NodeDir)      { $available += "Node.js" }
if ($global:MinGWDir)     { $available += "MinGW64" }
if ($global:RustInstalled) { $available += "Rust" }

if ($available.Count -gt 0) {
    Write-Host "Herramientas disponibles: " -NoNewline -ForegroundColor Gray
    Write-Host ($available -join ", ") -ForegroundColor Cyan
    Write-Host ""
}

# Comandos helper
Write-Host "Comandos útiles:" -ForegroundColor Gray
Write-Host "  • Test-DevEnv  " -NoNewline -ForegroundColor Yellow
Write-Host "→ Verificar herramientas" -ForegroundColor Gray
Write-Host "  • dev          " -NoNewline -ForegroundColor Yellow
Write-Host "→ Compilar y ejecutar (cargo run)" -ForegroundColor Gray
Write-Host "  • dev -Build   " -NoNewline -ForegroundColor Yellow
Write-Host "→ Solo compilar" -ForegroundColor Gray
Write-Host "  • tauri dev    " -NoNewline -ForegroundColor Yellow
Write-Host "→ Ejecutar app Tauri" -ForegroundColor Gray
Write-Host ""

# Restaurar manejo de errores
$ErrorActionPreference = "Continue"