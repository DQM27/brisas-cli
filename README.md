# 🌬️ Brisas Environment CLI (be)

> **Tu entorno de desarrollo completo, portátil y automático en segundos.**

**Brisas Environment (be)** es una herramienta de línea de comandos (CLI) diseñada para automatizar la configuración de entornos de desarrollo en Windows. Olvídate de instalaciones complejas, variables de entorno rotas o contaminar tu sistema operativo.

**Ideal para entornos restringidos:** ¿Estás en una computadora de la escuela, universidad o trabajo donde **no tienes contraseña de Administrador**? Brisas es la solución. Te permite tener un entorno de programación profesional (Node, GCC, etc.) funcionando en segundos, sin pedir permisos elevados y sin dejar rastros al terminar.

Con un solo comando, **Brisas** descarga, configura y te entrega un entorno con **Node.js**, **Compiladores C/C++ (MinGW)** y **PowerShell 7**, todo listo para usar.

## 🚀 Características Principales

*   **⚡ Instalación Automática**: Descarga las versiones exactas definidas en el manifiesto `tools.json`.
*   **🎒 Totalmente Portátil**: Todo se instala en `%LOCALAPPDATA%`. No ensucia tu sistema ni requiere Admin.
*   **🛡️ Entorno Aislado**: Las herramientas se agregan al PATH solo para tu usuario o temporalmente en la terminal.
*   **🔄 Actualizaciones Fáciles**: Si cambia la versión en `tools.json`, `be setup` actualiza tu entorno automáticamente.
*   **📦 Shell Portátil**: Inicia una terminal `pwsh` con todo cargado sin tocar tu configuración global.
*   **🚫 Cero Emojis (Modo Serio)**: Interfaz limpia y profesional para entornos corporativos o minimalistas.

## 🛠️ Herramientas Incluidas (Por Defecto)

*   **Node.js**: Entorno de ejecución para JavaScript.
*   **MinGW-w64 (GCC)**: Compilador de C y C++ robusto para Windows.
*   **PowerShell**: La terminal moderna y potente de Microsoft.
*(Y cualquier otra que agregues a tu `tools.json` personalizado)*

## 📥 Instalación

Simplemente descarga el ejecutable `be.exe` (desde Releases) y colócalo en una carpeta de tu preferencia (ej: `C:\Brisas`).

## 📖 Uso

Puedes usar **Brisas** de dos formas:

### 1. Menú Interactivo
Si ejecutas `be.exe` (doble clic) sin argumentos, verás un menú visual para elegir qué hacer:
*   **Instalar / Reparar**: Descarga todo lo necesario.
*   **Iniciar Shell**: Abre una terminal lista para trabajar.
*   **Verificar Estado**: Te dice si te falta algo.
*   **Desinstalar**: Borra todo.

### 2. Línea de Comandos (Automatización)
Para usuarios avanzados o scripts:

```powershell
# Instalar / Actualizar entorno
be setup

# Abrir terminal portable
be shell

# Ejecutar un comando específico dentro del entorno
be run npm install
be run gcc main.c -o app

# Verificar estado
be status

# Desinstalar todo (Limpieza total)
be clean

# Ayuda
be help
```

### 🧬 (Avanzado) Generador de Manifiestos
Si quieres controlar qué versiones instalar o agregar herramientas propias, puedes editar el archivo `tools.json`.
Brisas incluye un asistente para esto:

```powershell
be manifest-gen
```
Este comando te permitirá editar las URLs, versiones y calcular automáticamente los Hashes SHA256 de seguridad, e incluso subir los cambios a Git.

## 📂 Estructura de Archivos

Al instalarse, Brisas crea la siguiente estructura en `C:\Users\TU_USUARIO\AppData\Local`:

```
AppData/Local/
├── node/           # Node.js portable
├── mingw64/        # GCC/G++ y herramientas de compilación
├── pwsh/           # PowerShell core
```

## 📄 Licencia

Este proyecto es **Software Libre** bajo la licencia **MIT**.
Eres libre de usarlo, modificarlo y compartirlo. ¡Disfrútalo!

---
Hecho con ❤️ y **Rust** 🦀 por el equipo Brisas.
