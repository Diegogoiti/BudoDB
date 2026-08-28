use crate::application::ports::{
    AbonoRepository, AlumnoRepository, AplicacionPagoRepository, ConfiguracionAppRepository, Configuracion,
    DeudaRepository, HistorialPagoRepository, Logger, PagoRepository, RepresentanteRepository,
};
use crate::application::service::ServicioAlumnos;
use crate::application::service_abonos::ServicioAbonos;
use crate::application::service_ajustes::ServicioAjustes;
use crate::application::service_deudas::ServicioDeudas;
use crate::application::service_historial::ServicioHistorialPagos;
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

pub fn run() {
    let initial_size = LogicalSize::new(1024.0, 720.0);
    let mut window = WindowBuilder::new()
        .with_title("BudoDB")
        .with_min_inner_size(LogicalSize::new(800.0, 600.0))
        .with_inner_size(initial_size);
    #[cfg(target_os = "windows")]
    { window = window.with_visible(false); }
    #[cfg(not(target_os = "windows"))]
    { window = window.with_transparent(true); }
    let config = Config::default()
        .with_window(window)
        .with_menu(None)
        .with_custom_head(format!("<style>{}</style>", CSS));
    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

pub fn construir_estado_aplicacion() -> MyApp {
    match intentar_construir_estado() {
        Ok(estado) => estado,
        Err(error) => {
            let logger = ConsoleLogger;
            logger.error(&format!("No se pudo iniciar BudoDB: {error}"));
            std::process::exit(1);
        }
    }
}

fn intentar_construir_estado() -> Result<MyApp, String> {
    let logger: Arc<dyn Logger> = Arc::new(ConsoleLogger);
    let config = ConfigEntorno;
    let ruta = config.ruta_base_de_datos();
    logger.info("Iniciando BudoDB...");
    let sqlite = Arc::new(
        SqliteRepositorio::abrir(&ruta, logger.clone())
            .map_err(|e| format!("no se pudo abrir la base de datos '{ruta}': {e}"))?,
    );
    let repo_alumnos: Arc<dyn AlumnoRepository> = sqlite.clone();
    let repo_representantes: Arc<dyn RepresentanteRepository> = sqlite.clone();
    let repo_pagos: Arc<dyn PagoRepository> = sqlite.clone();
    let repo_ajustes: Arc<dyn ConfiguracionAppRepository> = sqlite.clone();
    let repo_deudas: Arc<dyn DeudaRepository> = sqlite.clone();
    let repo_abonos: Arc<dyn AbonoRepository> = sqlite.clone();
    let repo_aplicaciones: Arc<dyn AplicacionPagoRepository> = sqlite.clone();
    let repo_historial: Arc<dyn HistorialPagoRepository> = sqlite;

    let servicio_alumnos = Arc::new(ServicioAlumnos::nuevo(repo_alumnos, logger.clone()));
    let servicio_representantes = Arc::new(ServicioRepresentantes::nuevo(repo_representantes, logger.clone()));
    let servicio_pagos = Arc::new(ServicioPagos::nuevo(repo_pagos, repo_aplicaciones, repo_deudas.clone(), repo_historial.clone(), repo_ajustes.clone(), logger.clone()));
    let servicio_ajustes = Arc::new(ServicioAjustes::nuevo(repo_ajustes.clone(), logger.clone()));
    let servicio_deudas = Arc::new(ServicioDeudas::nuevo(repo_deudas, repo_ajustes, repo_abonos.clone(), logger.clone()));
    let servicio_abonos = Arc::new(ServicioAbonos::nuevo(repo_abonos, logger.clone()));
    let servicio_historial = Arc::new(ServicioHistorialPagos::nuevo(repo_historial, logger.clone()));

    logger.info("Base de datos lista");
    Ok(MyApp::new(
        ruta, servicio_alumnos, servicio_representantes, servicio_pagos,
        servicio_ajustes, servicio_deudas, servicio_abonos, servicio_historial, logger,
    ))
}
