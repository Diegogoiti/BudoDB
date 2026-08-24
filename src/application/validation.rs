//! ÚNICA fuente de verdad para validaciones de formato/entrada (regla 6).
//! Las reglas de negocio viven en `domain`; aquí solo el formato de entrada.
//! La presentación reutiliza estas mismas funciones para pintar feedback
//! en vivo, así la regla nunca se duplica.

use super::dto::DatosAlumno;
use super::error::ErrorAplicacion;
use crate::domain::alumno::FORMATO_FECHA;
use chrono::NaiveDate;

/// Verifica que el texto tenga formato de fecha YYYY-MM-DD.
pub fn es_fecha_valida(fecha: &str) -> bool {
    NaiveDate::parse_from_str(fecha, FORMATO_FECHA).is_ok()
}

/// Fecha válida para formularios: además no puede estar vacía.
pub fn es_fecha_valida_form(fecha: &str) -> bool {
    !fecha.is_empty() && es_fecha_valida(fecha)
}

/// El contacto no puede estar vacío y respeta el largo mínimo histórico (12).
pub fn contacto_valido(numero: &str) -> bool {
    !(numero.is_empty() || numero.len() < 12)
}

/// El nombre es obligatorio.
pub fn nombre_valido(nombre: &str) -> bool {
    !nombre.is_empty()
}

/// El representante es obligatorio.
pub fn representante_valido(representante: &str) -> bool {
    !representante.is_empty()
}

/// Validación completa antes de persistir un alumno.
/// Devuelve el primer error de validación encontrado, si lo hay.
pub fn validar_datos_alumno(datos: &DatosAlumno) -> Result<(), ErrorAplicacion> {
    if !nombre_valido(&datos.nombre) {
        return Err(ErrorAplicacion::Validacion(
            "El nombre no puede estar vacío.".to_string(),
        ));
    }
    if !es_fecha_valida_form(&datos.fecha_de_nacimiento) {
        return Err(ErrorAplicacion::Validacion(
            "La fecha de nacimiento no es válida.".to_string(),
        ));
    }
    if !representante_valido(&datos.representante) {
        return Err(ErrorAplicacion::Validacion(
            "El representante no puede estar vacío.".to_string(),
        ));
    }
    if !contacto_valido(&datos.numero_contacto) {
        return Err(ErrorAplicacion::Validacion(
            "El teléfono de contacto no es válido.".to_string(),
        ));
    }
    Ok(())
}
