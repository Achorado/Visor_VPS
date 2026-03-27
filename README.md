# Visor VPS 🚀

[Español](#español) | [English](#english)

---

## Español

# Visor VPS - Administrador de Servidores Remotos

Visor VPS es una aplicación de escritorio moderna construida con **Tauri v2** y **Svelte** para la gestión y monitorización visual de servidores remotos a través de SSH. Permite controlar procesos PM2, contenedores Docker y configuraciones Nginx con una interfaz intuitiva y potentes gráficas en tiempo real.

### ✨ Características
- 📊 **Monitorización en Tiempo Real:** Gráficas de CPU, Memoria RAM y Red.
- ⚡ **Gestión de PM2:** Visualiza, inicia, detén y reinicia procesos de PM2.
- 🐳 **Docker Integrado:** Gestión de contenedores Docker (Listado, logs y acciones).
- ⚙️ **Nginx Editor:** Crea, edita y habilita configuraciones de Nginx directamente.
- 📂 **Gestor de Despliegues:** Automatiza el despliegue de proyectos Svelte/Next.js en tu VPS.
- 🔒 **Seguridad:** Soporte para claves SSH y contraseñas cifradas localmente.
- 🖥️ **Logs Visuales:** Acceso a registros de servicios sin salir de la app.

### 🚀 Requisitos e Instalación

Para ejecutar este proyecto en modo desarrollo, necesitas tener instalado:

1.  **Rust & Cargo:** [Instalar Rust](https://www.rust-lang.org/tools/install)
2.  **Node.js & npm:** [Descargar Node.js](https://nodejs.org/)
3.  **Tauri CLI:**
    ```bash
    cargo install tauri-cli
    ```

**Pasos para iniciar:**

1.  Clona el repositorio.
2.  Instala las dependencias de Node:
    ```bash
    npm install
    ```
3.  Inicia la aplicación en modo desarrollo:
    ```bash
    npm run tauri:dev
    ```

### 🛠️ Funcionamiento
La aplicación utiliza una base de datos SQLite local (`visor_vps.db`) para almacenar las configuraciones de tus servidores de forma segura. Todas las comunicaciones se realizan vía SSH, cifrando los comandos y gestionando la autenticación de forma nativa desde el backend en Rust.

---

## English

# Visor VPS - Remote Server Manager

Visor VPS is a modern desktop application built with **Tauri v2** and **Svelte** for visual management and monitoring of remote servers via SSH. It allows you to control PM2 processes, Docker containers, and Nginx configurations with an intuitive interface and powerful real-time charts.

### ✨ Features
- 📊 **Real-time Monitoring:** CPU, RAM, and Network usage charts.
- ⚡ **PM2 Management:** View, start, stop, and restart PM2 processes.
- 🐳 **Docker Integrated:** Docker container management (Listing, logs, and actions).
- ⚙️ **Nginx Editor:** Create, edit, and enable Nginx configurations directly.
- 📂 **Deployment Manager:** Automate the deployment of Svelte/Next.js projects to your VPS.
- 🔒 **Security:** Support for SSH keys and locally encrypted passwords.
- 🖥️ **Visual Logs:** Access service logs without leaving the app.

### 🚀 Requirements and Installation

To run this project in development mode, you need to have installed:

1.  **Rust & Cargo:** [Install Rust](https://www.rust-lang.org/tools/install)
2.  **Node.js & npm:** [Download Node.js](https://nodejs.org/)
3.  **Tauri CLI:**
    ```bash
    cargo install tauri-cli
    ```

**Steps to start:**

1.  Clone the repository.
2.  Install Node dependencies:
    ```bash
    npm install
    ```
3.  Launch the application in development mode:
    ```bash
    npm run tauri:dev
    ```

### 🛠️ How it works
The application uses a local SQLite database (`visor_vps.db`) to store your server configurations securely. All communications are handled via SSH, with commands and authentication managed natively from the Rust backend.
