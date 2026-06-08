# BudoDB 🥋

**BudoDB** es una solución de software eficiente y ligera diseñada para la gestión integral de alumnos en dojos de Karate. Desarrollado por **Diego Goitia**, este proyecto combina la seguridad y el rendimiento de **Rust** con una interfaz moderna construida en **Dioxus**.

![Version](https://img.shields.io/badge/version-0.1.0--alpha-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Rust](https://img.shields.io/badge/language-Rust-orange)

## 🚀 Características Principales

- **Multiplataforma:** Ejecución nativa en escritorio (Windows) mediante WebView2.
- **Base de Datos Local:** Gestión de persistencia mediante **SQLite** (`rusqlite`), garantizando que los datos sensibles se mantengan de forma local y segura.
- **Interfaz Fluida:** UI reactiva construida con componentes de Dioxus y estilizada con **Tailwind CSS**.
- **Gestión de Alumnos:**
    - Registro detallado (Nombre, fecha de nacimiento, representante, contacto).
    - Cálculo automático de edad y gestión de rangos/cintas.
    - Sistema de búsqueda y filtrado dinámico por columnas.
- **Filosofía KISS:** Código modular, altamente optimizado para hardware con recursos limitados (probado en procesadores Celeron).

## 🛠️ Stack Tecnológico

- **Lenguaje:** [Rust](https://www.rust-lang.org/) (Edición 2021)
- **Frontend Framework:** [Dioxus](https://dioxuslabs.com/) v0.7.1
- **Base de Datos:** [SQLite](https://sqlite.org/) con soporte `bundled` para evitar dependencias externas.
- **Manejo de Tiempos:** [Chrono](https://github.com/chronotope/chrono) para cálculos precisos de fechas.

## 📁 Estructura del Proyecto

El código sigue una arquitectura de backend profesional y modular:

- `src/main.rs`: Punto de entrada, configuración de la ventana nativa y sistema de rutas.
- `src/models.rs`: Abstracción de la base de datos (CRUD) y lógica de la estructura `Alumno`.
- `src/my_app.rs`: Manejo del estado global de la aplicación y lógica de filtrado/selección.
- `src/views.rs`: Renderizado de las vistas principales (Home, Buscar, Agregar, etc.).
- `src/components/`: Componentes de UI reutilizables (DataTable, SearchBar, Sidebar).

## ⚙️ Instalación y Compilación

### Prerrequisitos
- Tener instalado [Rust y Cargo](https://rustup.rs/).
- En Windows, es necesario el WebView2 Runtime (ya viene en Windows 10/11).

### Instrucciones
1. **Clonar el repositorio:**
```bash
git clone https://github.com/Diegogoiti/budodb.git
cd BudoDB
```

2. **Ejecutar en desarrollo:**
```bash
   cargo run
```

3. **Compilar binario optimizado:**
```bash
   cargo build --release
```

## 📝 Notas del Desarrollador

BudoDB nació de la necesidad de una herramienta de gestión que fuera rápida, privada y que no consumiera los recursos excesivos de las aplicaciones basadas en Electron. Al estar desarrollado íntegramente en Rust, ofrece un consumo de RAM mínimo y una estabilidad superior, ideal para el hardware del dojo.

---
Desarrollado con 🦀 por [Diego Goitia](https://diegogoitia-dev.onrender.com).
