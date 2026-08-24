//! Composition root: punto ÚNICO de construcción del grafo de objetos y de
//! arranque de la aplicación (regla 4). Todos los servicios/repositorios se
//! construyen aquí y se inyectan hacia las capas superiores.

use crate::application::ports::{AlumnoRepository, Configuracion, Logger, PagoRepository, RepresentanteRepository};
use crate::application::service::ServicioAlumnos;
use crate::application::service_pagos::ServicioPagos;
use crate::application::service_representantes::ServicioRepresentantes;
use crate::infrastructure::console_logger::ConsoleLogger;
use crate::infrastructure::env_config::ConfigEntorno;
use crate::infrastructure::sqlite_repository::SqliteRepositorio;
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
    // Orden de construcción del grafo: logger -> configuración -> repositorio
    // (uno solo, tres puertos) -> servicios -> estado.
    let logger: Arc<dyn Logger> = Arc::new(ConsoleLogger);
    let config = ConfigEntorno;
    let ruta = config.ruta_base_de_datos();

    logger.info("Iniciando BudoDB...");

    // Una única instancia de repositorio coercida a sus tres puertos: las
    // entidades comparten conexión porque se relacionan por claves y así las
    // migraciones corren en un solo lugar.
    let sqlite = Arc::new(
        SqliteRepositorio::abrir(&ruta, logger.clone())
            .map_err(|e| format!("no se pudo abrir la base de datos '{ruta}': {e}"))?,
    );
    let repo_alumnos: Arc<dyn AlumnoRepository> = sqlite.clone();
    let repo_representantes: Arc<dyn RepresentanteRepository> = sqlite.clone();
    let repo_pagos: Arc<dyn PagoRepository> = sqlite;

    let servicio_alumnos = Arc::new(ServicioAlumnos::nuevo(repo_alumnos, logger.clone()));
    let servicio_representantes =
        Arc::new(ServicioRepresentantes::nuevo(repo_representantes, logger.clone()));
    let servicio_pagos = Arc::new(ServicioPagos::nuevo(repo_pagos, logger.clone()));

    logger.info("Base de datos lista");

    Ok(MyApp::new(
        servicio_alumnos,
        servicio_representantes,
        servicio_pagos,
        logger,
    ))
}
