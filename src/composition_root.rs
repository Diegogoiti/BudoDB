//! Composition root: punto ÚNICO de construcción del grafo de objetos y de
//! arranque de la aplicación (regla 4). Todos los servicios/repositorios se
//! construyen aquí y se inyectan hacia las capas superiores.

use crate::application::ports::Logger;
use crate::infrastructure::console_logger::ConsoleLogger;
use crate::models::Database;
use crate::presentation::app::{App, CSS};
use crate::presentation::my_app::MyApp;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

/// Ruta por defecto de la base de datos.
/// TEMPORAL: en la fase 2 pasa a leerse de configuración externa.
const RUTA_BASE_DATOS: &str = "./database/database.db";

/// Punto de entrada de la aplicación: configura la ventana y lanza el runtime
/// de Dioxus. `main.rs` solo delega aquí.
pub fn run() {
    let initial_size = LogicalSize::new(1024.0, 720.0);

    let mut window = WindowBuilder::new()
        .with_title("BudoDB")
        .with_min_inner_size(LogicalSize::new(800.0, 600.0))
        .with_inner_size(initial_size);

    // Configuración condicional
    #[cfg(target_os = "windows")]
    {
        window = window.with_visible(false);
    }

    #[cfg(not(target_os = "windows"))]
    {
        window = window.with_transparent(true);
    }

    let config = Config::default()
        .with_window(window)
        .with_menu(None)
        .with_custom_head(format!("<style>{}</style>", CSS));

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

/// Construye el estado inicial de la aplicación con todas sus dependencias
/// inyectadas. Invocado una única vez durante el arranque del runtime.
pub fn construir_estado_aplicacion() -> MyApp {
    match intentar_construir_estado() {
        Ok(estado) => estado,
        Err(error) => {
            // Manejador global de errores de arranque: último recurso antes
            // de salir, ya que sin base de datos la app no puede operar.
            let logger = ConsoleLogger;
            logger.error(&format!("No se pudo iniciar BudoDB: {error}"));
            std::process::exit(1);
        }
    }
}

fn intentar_construir_estado() -> Result<MyApp, String> {
    let logger = ConsoleLogger;
    logger.info("Iniciando BudoDB...");

    let database =
        Database::new(RUTA_BASE_DATOS).map_err(|e| format!("no se pudo abrir la base de datos '{RUTA_BASE_DATOS}': {e}"))?;
    let alumnos = database
        .fetch_all()
        .map_err(|e| format!("no se pudieron cargar los alumnos iniciales: {e}"))?;
    logger.info("Base de datos lista");

    Ok(MyApp::new(alumnos, database))
}
