<#
.SYNOPSIS
    Script wrapper para ejecutar comandos en el entorno portable (para Antigravity/AI).
.DESCRIPTION
    Carga dev-env.ps1 y luego ejecuta el comando pasado como argumento.
    Permite a agentes externos ejecutar herramientas (npm, cargo) sin configuración global.
.EXAMPLE
    .\ag-run.ps1 "npm install"
    .\ag-run.ps1 "cargo run"
#>
param(
    [Parameter(Mandatory=$true)]
    [string]$Command
)

$ErrorActionPreference = "Stop"

# 1. Cargar el entorno de desarrollo
# Usamos & { ... } para aislar el output de la carga si fuera necesario, 
# pero dev-env necesita exportar variables al scope actual. 
# Dot-sourcing (.) es necesario.

# Redirigimos la salida "verbose" de dev-env.ps1 para limpiar el output del comando real
# Si hay error en dev-env, fallará el script.
try {
    # Cargar silenciosamente.
    . "$PSScriptRoot\dev-env.ps1" *>$null
} catch {
    Write-Error "Error cargando dev-env: $_"
    exit 1
}

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Error "npm no encontrado."
    exit 1
}

Invoke-Expression $Command

# 3. Ejecutar el comando
# Write-Host "AG -> $Command" -ForegroundColor Magenta
Invoke-Expression $Command
