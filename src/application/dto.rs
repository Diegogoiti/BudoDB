//! DTOs de la capa de aplicación (regla 5): lo que viaja entre la UI y los
//! casos de uso. Nunca son structs de BD ni de dominio puro.
//!
//! - `Datos*`: entrada de formularios.
//! - `*Vista`: lectura compuesta para pintar tablas (proyección).

use crate::domain::{Alumno, AplicacionPago, Deuda, EstadoDeuda, EstadoPago, HistorialPago, MetodoPago, Pago};

/// Entrada para crear o editar un alumno.
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

/// Entrada para registrar un pago (reemplaza al viejo DatosPago + DatosAbono).
/// El motor FIFO determina automáticamente a qué deudas se aplica.
#[derive(Debug, Clone, PartialEq)]
pub struct DatosPago {
    pub representante_id: usize,
    pub monto_recibido: f64,
    pub metodo_id: i32,
    pub fecha_pago: String,
}

/// Proyección de lectura para pintar la tabla de alumnos.
#[derive(Debug, Clone, PartialEq)]
pub struct AlumnoVista {
    pub alumno: Alumno,
    pub nombre_representante: String,
    pub telefono_representante: String,
}

/// Proyección de lectura de una deuda con todos los datos resueltos.
#[derive(Debug, Clone, PartialEq)]
pub struct DeudaVista {
    pub deuda: Deuda,
    pub nombre_representante: String,
    pub telefono_representante: String,
    /// Estado tipado de la deuda.
    pub estado: EstadoDeuda,
}

/// Proyección de lectura de un pago con datos resueltos.
#[derive(Debug, Clone, PartialEq)]
pub struct PagoVista {
    pub pago: Pago,
    pub nombre_representante: String,
    pub metodo: MetodoPago,
    pub estado: EstadoPago,
    /// Aplicaciones de este pago a deudas.
    pub aplicaciones: Vec<AplicacionPagoVista>,
}

/// Proyección de lectura de una aplicación de pago.
#[derive(Debug, Clone, PartialEq)]
pub struct AplicacionPagoVista {
    pub aplicacion: AplicacionPago,
    pub nombre_representante: String,
    pub periodo_deuda: String,
}

/// Entrada para registrar un movimiento en el historial de pagos.
#[derive(Debug, Clone, PartialEq)]
pub struct DatosHistorialPago {
    pub representante_id: usize,
    pub tipo_id: i32,
    pub monto: f64,
    pub periodo: String,
    pub fecha: String,
    pub observacion: String,
}

/// Proyección de lectura de un registro del historial.
#[derive(Debug, Clone, PartialEq)]
pub struct HistorialPagoVista {
    pub historial: HistorialPago,
    pub nombre_representante: String,
}


/// Entrada para registrar un abono (LEGACY — mantenido para compatibilidad).
#[derive(Debug, Clone, PartialEq)]
pub struct DatosAbono {
    pub deuda_id: usize,
    pub monto: f64,
    pub fecha: String,
    pub observacion: String,
}
