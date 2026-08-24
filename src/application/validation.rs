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

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn fechas_validas_solo_en_formato_iso() {
        assert!(es_fecha_valida("2026-08-24"));
        assert!(!es_fecha_valida("24-08-2026"));
        assert!(!es_fecha_valida("2026/08/24"));
        assert!(!es_fecha_valida(""));
    }

    #[test]
    fn formulario_exige_fecha_no_vacia() {
        assert!(es_fecha_valida_form("2026-08-24"));
        assert!(!es_fecha_valida_form(""));
    }

    #[test]
    fn contacto_respeta_el_largo_minimo_historico() {
        assert!(!contacto_valido(""));
        assert!(!contacto_valido("0412-00000")); // 11 caracteres
        assert!(contacto_valido("0412-0000000")); // 12 caracteres
    }

    #[test]
    fn nombre_y_representante_son_obligatorios() {
        assert!(!nombre_valido(""));
        assert!(nombre_valido("Juan"));
        assert!(!representante_valido(""));
        assert!(representante_valido("Pedro"));
    }

    #[test]
    fn validacion_completa_acepta_datos_buenos() {
        let datos = DatosAlumno {
            nombre: "Juan".to_string(),
            fecha_de_nacimiento: "2010-01-15".to_string(),
            rango: 6,
            representante: "Pedro".to_string(),
            numero_contacto: "0412-0000000".to_string(),
            rallita: false,
        };
        assert!(validar_datos_alumno(&datos).is_ok());
    }

    #[test]
    fn validacion_completa_reporta_error_de_formato() {
        let mut datos = DatosAlumno {
            nombre: "Juan".to_string(),
            fecha_de_nacimiento: "31/12/2010".to_string(),
            rango: 6,
            representante: "Pedro".to_string(),
            numero_contacto: "0412-0000000".to_string(),
            rallita: false,
        };

        match validar_datos_alumno(&datos) {
            Err(ErrorAplicacion::Validacion(_)) => {}
            otro => panic!("se esperaba error de validación, obtuve {otro:?}"),
        }

        datos.fecha_de_nacimiento = "2010-12-31".to_string();
        datos.numero_contacto = "0412-00".to_string();
        match validar_datos_alumno(&datos) {
            Err(ErrorAplicacion::Validacion(_)) => {}
            otro => panic!("se esperaba error de validación, obtuve {otro:?}"),
        }
    }
}
