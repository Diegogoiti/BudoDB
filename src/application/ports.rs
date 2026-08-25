//! Puertos (interfaces) que la capa de aplicación exige al mundo exterior.
//! Las implementaciones concretas viven en la capa `infrastructure`.

use crate::domain::{Abono, Alumno, Deuda, HistorialPago, Pago, Representante};
use std::collections::HashSet;
use std::fmt;

/// Puerto de registro de eventos. Permite loguear sin acoplarse a una
/// implementación concreta (regla 10).
pub trait Logger: Send + Sync {
    fn debug(&self, mensaje: &str);
    fn info(&self, mensaje: &str);
    fn error(&self, mensaje: &str);
}

/// Errores que un repositorio de persistencia puede reportar.
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

/// Puerto de persistencia de alumnos.
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

/// Puerto de persistencia de representantes.
pub trait RepresentanteRepository: Send + Sync {
    fn save(&self, representante: &Representante) -> Result<(), ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<Representante>, ErrorRepositorio>;
    fn update(&self, representante: &Representante) -> Result<(), ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de pagos de mensualidad.
pub trait PagoRepository: Send + Sync {
    fn save(&self, pago: &Pago) -> Result<(), ErrorRepositorio>;
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Pago>, ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<Pago>, ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de deudas mensuales.
pub trait DeudaRepository: Send + Sync {
    fn save(&self, deuda: &Deuda) -> Result<(), ErrorRepositorio>;
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Deuda>, ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<Deuda>, ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de abonos (pagos parciales contra una deuda).
pub trait AbonoRepository: Send + Sync {
    fn save(&self, abono: &Abono) -> Result<(), ErrorRepositorio>;
    fn fetch_por_deuda(&self, deuda_id: usize) -> Result<Vec<Abono>, ErrorRepositorio>;
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Abono>, ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia del historial de pagos para auditoría y cálculos.
pub trait HistorialPagoRepository: Send + Sync {
    fn save(&self, registro: &HistorialPago) -> Result<(), ErrorRepositorio>;
    fn fetch_por_representante(&self, representante_id: usize) -> Result<Vec<HistorialPago>, ErrorRepositorio>;
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<HistorialPago>, ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<HistorialPago>, ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de configuración externa: rutas y parámetros nunca hardcodeados
/// en código (regla 8).
pub trait Configuracion: Send + Sync {
    fn ruta_base_de_datos(&self) -> String;
}

/// Puerto de AJUSTES DE LA APLICACIÓN gestionables desde la UI (persistidos).
pub trait ConfiguracionAppRepository: Send + Sync {
    fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorRepositorio>;
    fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorRepositorio>;
}
