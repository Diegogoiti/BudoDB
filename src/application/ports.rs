//! Puertos (interfaces) que la capa de aplicación exige al mundo exterior.
//! Las implementaciones concretas viven en la capa `infrastructure`.

use crate::models::Alumno; // TEMPORAL: migrará a `domain::alumno` en la fase 3.
use std::collections::HashSet;
use std::fmt;

/// Puerto de registro de eventos. Permite loguear sin acoplarse a una
/// implementación concreta (regla 10).
pub trait Logger: Send + Sync {
    /// Eventos de diagnóstico detallado (solo visibles en builds de desarrollo).
    fn debug(&self, mensaje: &str);
    /// Eventos normales del ciclo de vida de la aplicación.
    fn info(&self, mensaje: &str);
    /// Errores de operaciones críticas.
    fn error(&self, mensaje: &str);
}

/// Errores que un repositorio de persistencia puede reportar.
/// Traducción de errores de infra en los límites (regla 9).
#[derive(Debug)]
pub enum ErrorRepositorio {
    Conexion(String),
    Consulta(String),
}

impl fmt::Display for ErrorRepositorio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorRepositorio::Conexion(detalle) => write!(f, "error de conexión: {detalle}"),
            ErrorRepositorio::Consulta(detalle) => write!(f, "error de consulta: {detalle}"),
        }
    }
}

impl std::error::Error for ErrorRepositorio {}

/// Puerto de persistencia de alumnos. La capa de aplicación solo conoce esta
/// abstracción, nunca la base de datos concreta (regla 1).
pub trait AlumnoRepository: Send + Sync {
    fn save(&self, alumno: &Alumno) -> Result<(), ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<Alumno>, ErrorRepositorio>;
    fn update(&self, alumno: &Alumno) -> Result<(), ErrorRepositorio>;
    fn update_rangos(
        &self,
        ids: HashSet<usize>,
        rango: i32,
        rallita: bool,
    ) -> Result<(), ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de configuración externa: rutas y parámetros nunca hardcodeados
/// en código (regla 8).
pub trait Configuracion: Send + Sync {
    fn ruta_base_de_datos(&self) -> String;
}
