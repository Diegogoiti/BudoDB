//! DTOs de la capa de aplicación (regla 5): lo que viaja entre la UI y los
//! casos de uso. Nunca son structs de BD ni de dominio puro.
//!
//! - `Datos*`: entrada de formularios.
//! - `*Vista`: lectura compuesta para pintar tablas (proyección).

use crate::domain::{Alumno, Deuda, EstadoDeuda, HistorialPago, Pago};

/// Entrada para crear o editar un alumno. El representante se vincula
/// por ID (FK) — no se guardan strings directos en el alumno.
#[derive(Debug, Clone, PartialEq)]
pub struct DatosAlumno {
    pub nombre: String,
    pub fecha_de_nacimiento: String,
    pub rango: i32,
    pub representante_id: usize,
    pub rallita: bool,
}

/// Entrada para crear o editar un representante.
#[derive(Debug, Clone, PartialEq)]
pub struct DatosRepresentante {
    pub nombre: String,
    pub numero_contacto: String,
}

/// Entrada para registrar un pago de mensualidad.
#[derive(Debug, Clone, PartialEq)]
pub struct DatosPago {
    pub representante_id: usize,
    pub monto: f64,
    /// Mes que se cancela, formato "YYYY-MM".
    pub periodo: String,
    /// Fecha de registro, formato "YYYY-MM-DD".
    pub fecha: String,
    pub observacion: String,
}

/// Proyección de lectura para pintar la tabla de alumnos.
/// Resuelve el nombre y teléfono del representante para que la UI
/// no tenga que hacer joins manualmente.
#[derive(Debug, Clone, PartialEq)]
pub struct AlumnoVista {
    pub alumno: Alumno,
    pub nombre_representante: String,
    pub telefono_representante: String,
}

/// Proyección de lectura de un pago con el nombre del representante
/// resuelto para la tabla del panel administrativo.
#[derive(Debug, Clone, PartialEq)]
pub struct PagoVista {
    pub pago: Pago,
    pub nombre_representante: String,
}

/// Proyección de lectura de una deuda con todos los datos resueltos
/// para la tabla principal del panel de pagos.
#[derive(Debug, Clone, PartialEq)]
pub struct DeudaVista {
    pub deuda: Deuda,
    pub nombre_representante: String,
    pub telefono_representante: String,
    /// Suma de todos los abonos registrados contra esta deuda.
    pub total_abonado: f64,
    /// saldo = deuda.monto - total_abonado (nunca negativo).
    pub saldo: f64,
    /// Estado derivado: Pagado / Parcial / Pendiente.
    pub estado: EstadoDeuda,
}

/// Entrada para registrar un abono contra una deuda existente.
#[derive(Debug, Clone, PartialEq)]
pub struct DatosAbono {
    pub deuda_id: usize,
    pub monto: f64,
    /// Fecha de registro, formato "YYYY-MM-DD".
    pub fecha: String,
    pub observacion: String,
}

/// Entrada para registrar un movimiento en el historial de pagos.
#[derive(Debug, Clone, PartialEq)]
pub struct DatosHistorialPago {
    pub representante_id: usize,
    pub tipo: String,
    pub monto: f64,
    pub periodo: String,
    pub fecha: String,
    pub observacion: String,
}

/// Proyección de lectura de un registro del historial de pagos
/// con el nombre del representante resuelto.
#[derive(Debug, Clone, PartialEq)]
pub struct HistorialPagoVista {
    pub historial: HistorialPago,
    pub nombre_representante: String,
}
