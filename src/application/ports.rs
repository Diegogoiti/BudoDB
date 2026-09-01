//! Puertos (interfaces) que la capa de aplicación exige al mundo exterior.
//! Las implementaciones concretas viven en la capa `infrastructure`.

use crate::domain::{Alumno, AplicacionPago, Deuda, HistorialPago, Pago, Representante};
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
    fn desactivar(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
    fn activar(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de representantes.
pub trait RepresentanteRepository: Send + Sync {
    fn save(&self, representante: &Representante) -> Result<(), ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<Representante>, ErrorRepositorio>;
    fn update(&self, representante: &Representante) -> Result<(), ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
    fn desactivar(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
    fn activar(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de pagos de mensualidad.
pub trait PagoRepository: Send + Sync {
    /// Guarda un pago y devuelve su ID recién asignado.
    fn save(&self, pago: &Pago) -> Result<usize, ErrorRepositorio>;
    /// Pagos cuya `fecha_pago` comienza con el prefijo `periodo` ("YYYY-MM").
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Pago>, ErrorRepositorio>;
    fn fetch_por_representante(&self, representante_id: usize) -> Result<Vec<Pago>, ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<Pago>, ErrorRepositorio>;
    fn update_estado(&self, id: usize, estado_id: i32) -> Result<(), ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de deudas mensuales.
pub trait DeudaRepository: Send + Sync {
    fn save(&self, deuda: &Deuda) -> Result<(), ErrorRepositorio>;
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Deuda>, ErrorRepositorio>;
    fn fetch_cobrables_por_representante(&self, representante_id: usize) -> Result<Vec<Deuda>, ErrorRepositorio>;
    fn fetch_todos_periodos_por_representante(&self, representante_id: usize) -> Result<Vec<String>, ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<Deuda>, ErrorRepositorio>;
    fn update_estado(&self, id: usize, monto_pendiente: f64, estado_id: i32) -> Result<(), ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de aplicaciones de pago (tabla puente pagos ↔ deudas).
pub trait AplicacionPagoRepository: Send + Sync {
    fn save(&self, aplicacion: &AplicacionPago) -> Result<(), ErrorRepositorio>;
    fn fetch_por_pago(&self, pago_id: usize) -> Result<Vec<AplicacionPago>, ErrorRepositorio>;
    fn fetch_por_deuda(&self, deuda_id: usize) -> Result<Vec<AplicacionPago>, ErrorRepositorio>;
    fn delete_por_pago(&self, pago_id: usize) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia de abonos (LEGACY — mantener para compatibilidad temporal).
pub trait AbonoRepository: Send + Sync {
    fn save(&self, abono: &crate::domain::Abono) -> Result<(), ErrorRepositorio>;
    fn fetch_por_deuda(&self, deuda_id: usize) -> Result<Vec<crate::domain::Abono>, ErrorRepositorio>;
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<crate::domain::Abono>, ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de persistencia del historial de pagos para auditoría.
pub trait HistorialPagoRepository: Send + Sync {
    fn save(&self, registro: &HistorialPago) -> Result<(), ErrorRepositorio>;
    fn fetch_por_representante(&self, representante_id: usize) -> Result<Vec<HistorialPago>, ErrorRepositorio>;
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<HistorialPago>, ErrorRepositorio>;
    fn fetch_all(&self) -> Result<Vec<HistorialPago>, ErrorRepositorio>;
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio>;
}

/// Puerto de configuración externa: rutas y parámetros nunca hardcodeados.
pub trait Configuracion: Send + Sync {
    fn ruta_base_de_datos(&self) -> String;
}

/// Puerto de AJUSTES DE LA APLICACIÓN gestionables desde la UI (persistidos).
pub trait ConfiguracionAppRepository: Send + Sync {
    fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorRepositorio>;
    fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorRepositorio>;
}
