//! DTOs de la capa de aplicación (regla 5): lo que viaja entre la UI y los
//! casos de uso. Nunca son structs de BD ni de dominio puro.
//!
//! - `Datos*`: entrada de formularios.
//! - `*Vista`: lectura compuesta para pintar tablas (proyección).

/// Entrada para crear o editar un alumno. El contacto ya no vive aquí:
/// el alumno referencia a su representante por ID.
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

/// Proyección de lectura: un alumno junto al nombre/teléfono de su
/// representante ya resueltos, listo para pintarse en una tabla sin que la
/// UI conozca cómo se relacionan las entidades.
#[derive(Debug, Clone, PartialEq)]
pub struct AlumnoVista {
    pub alumno: Alumno,
    /// Vacío si el alumno no tiene representante asignado.
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

use crate::domain::{Alumno, Pago};
