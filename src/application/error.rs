//! Errores propios de la capa de aplicación (regla 9). Se traducen en los
/// límites; solo la presentación decide cómo mostrarlos.

use crate::application::ports::ErrorRepositorio;
use std::fmt;

#[derive(Debug)]
pub enum ErrorAplicacion {
    /// Datos de entrada con formato inválido.
    Validacion(String),
    /// Fallo del puerto de persistencia.
    Repositorio(ErrorRepositorio),
}

impl fmt::Display for ErrorAplicacion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorAplicacion::Validacion(detalle) => write!(f, "validación: {detalle}"),
            ErrorAplicacion::Repositorio(error) => write!(f, "persistencia: {error}"),
        }
    }
}

impl std::error::Error for ErrorAplicacion {}

impl From<ErrorRepositorio> for ErrorAplicacion {
    fn from(error: ErrorRepositorio) -> Self {
        ErrorAplicacion::Repositorio(error)
    }
}
