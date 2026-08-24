//! Composition root: punto ÚNICO de construcción del grafo de objetos y de
//! arranque de la aplicación (regla 4). Todos los servicios/repositorios se
//! construyen aquí y se inyectan hacia las capas superiores.

use crate::application::ports::{AlumnoRepository, Configuracion, Logger};
use crate::infrastructure::console_logger::ConsoleLogger;
use crate::infrastructure::env_config::ConfigEntorno;
use crate::infrastructure::sqlite_repository::SqliteAlumnoRepository;
use crate::presentation::app::{App, CSS};
use crate::presentation::my_app::MyApp;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use std::sync::Arc;

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
    // Orden de construcción del grafo: logger -> configuración -> repositorio -> estado.
    let logger: Arc<dyn Logger> = Arc::new(ConsoleLogger);
    let config = ConfigEntorno;
    let ruta = config.ruta_base_de_datos();

    logger.info("Iniciando BudoDB...");

    let repositorio: Arc<dyn AlumnoRepository> = Arc::new(
        SqliteAlumnoRepository::abrir(&ruta, logger.clone())
            .map_err(|e| format!("no se pudo abrir la base de datos '{ruta}': {e}"))?,
    );
    let alumnos = repositorio
        .fetch_all()
        .map_err(|e| format!("no se pudieron cargar los alumnos iniciales: {e}"))?;
    logger.info("Base de datos lista");

    Ok(MyApp::new(alumnos, repositorio))
}
