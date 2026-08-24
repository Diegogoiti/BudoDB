//! ÚNICA fuente de verdad para validaciones de formato/entrada (regla 6).
//! Las reglas de negocio viven en `domain`; aquí solo el formato de entrada.
//! La presentación reutiliza estas mismas funciones para pintar feedback
//! en vivo, así la regla nunca se duplica.

use super::dto::{DatosAbono, DatosAlumno, DatosPago, DatosRepresentante};
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
/// Hoy lo usa el FORMULARIO DE REPRESENTANTE (el teléfono es suyo).
pub fn contacto_valido(numero: &str) -> bool {
    !(numero.is_empty() || numero.len() < 12)
}

/// El nombre es obligatorio.
pub fn nombre_valido(nombre: &str) -> bool {
    !nombre.is_empty()
}

/// Un alumno debe apuntar a un representante existente: el ID 0 es
/// "sin asignar" y solo se tolera en registros históricos ya migrados.
pub fn representante_asignado(representante_id: usize) -> bool {
    representante_id > 0
}

/// Monto de pago aceptable: positivo, con centavos como máximo dos decimales
/// de precisión práctica y un tope sanador contra typos gigantes.
pub fn monto_valido(monto: f64) -> bool {
    monto.is_finite() && monto > 0.0 && monto <= 1_000_000.0
}

/// Periodo "YYYY-MM" estricto: mes entre 01 y 12.
pub fn es_periodo_valido(periodo: &str) -> bool {
    let partes: Vec<&str> = periodo.split('-').collect();
    if partes.len() != 2 || partes[0].len() != 4 || partes[1].len() != 2 {
        return false;
    }
    let Ok(anio) = partes[0].parse::<i32>() else {
        return false;
    };
    let Ok(mes) = partes[1].parse::<u32>() else {
        return false;
    };
    (2000..=2100).contains(&anio) && (1..=12).contains(&mes)
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
    if !representante_asignado(datos.representante_id) {
        return Err(ErrorAplicacion::Validacion(
            "Debe seleccionar un representante.".to_string(),
        ));
    }
    Ok(())
}

/// Validación completa antes de persistir un representante.
pub fn validar_datos_representante(datos: &DatosRepresentante) -> Result<(), ErrorAplicacion> {
    if !nombre_valido(&datos.nombre) {
        return Err(ErrorAplicacion::Validacion(
            "El nombre del representante no puede estar vacío.".to_string(),
        ));
    }
    if !contacto_valido(&datos.numero_contacto) {
        return Err(ErrorAplicacion::Validacion(
            "El teléfono de contacto no es válido.".to_string(),
        ));
    }
    Ok(())
}

/// Validación completa antes de registrar un pago.
pub fn validar_datos_pago(datos: &DatosPago) -> Result<(), ErrorAplicacion> {
    if !representante_asignado(datos.representante_id) {
        return Err(ErrorAplicacion::Validacion(
            "El pago debe estar asociado a un representante.".to_string(),
        ));
    }
    if !monto_valido(datos.monto) {
        return Err(ErrorAplicacion::Validacion(
            "El monto debe ser un número positivo.".to_string(),
        ));
    }
    if !es_periodo_valido(&datos.periodo) {
        return Err(ErrorAplicacion::Validacion(
            "El periodo debe tener formato AAAA-MM.".to_string(),
        ));
    }
    if !es_fecha_valida(&datos.fecha) {
        return Err(ErrorAplicacion::Validacion(
            "La fecha de registro no es válida.".to_string(),
        ));
    }
    Ok(())
}

/// Validación completa antes de registrar un abono contra una deuda.
pub fn validar_datos_abono(datos: &DatosAbono) -> Result<(), ErrorAplicacion> {
    if datos.deuda_id == 0 {
        return Err(ErrorAplicacion::Validacion(
            "El abono debe estar asociado a una deuda.".to_string(),
        ));
    }
    if !monto_valido(datos.monto) {
        return Err(ErrorAplicacion::Validacion(
            "El monto del abono debe ser un número positivo.".to_string(),
        ));
    }
    if !es_fecha_valida(&datos.fecha) {
        return Err(ErrorAplicacion::Validacion(
            "La fecha de registro no es válida.".to_string(),
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
        assert!(!representante_asignado(0));
        assert!(representante_asignado(3));
    }

    #[test]
    fn montos_solo_positivos_y_razonables() {
        assert!(!monto_valido(0.0));
        assert!(!monto_valido(-100.0));
        assert!(!monto_valido(f64::NAN));
        assert!(monto_valido(1500.0));
    }

    #[test]
    fn periodos_validos_solo_en_formato_aaaa_mm() {
        assert!(es_periodo_valido("2026-08"));
        assert!(es_periodo_valido("2025-12"));
        assert!(!es_periodo_valido("2026-13"));
        assert!(!es_periodo_valido("2026-00"));
        assert!(!es_periodo_valido("26-08"));
        assert!(!es_periodo_valido("agosto"));
        assert!(!es_periodo_valido(""));
    }

    #[test]
    fn validacion_completa_acepta_datos_buenos() {
        let datos = DatosAlumno {
            nombre: "Juan".to_string(),
            fecha_de_nacimiento: "2010-01-15".to_string(),
            rango: 6,
            representante_id: 3,
            rallita: false,
        };
        assert!(validar_datos_alumno(&datos).is_ok());

        let rep = DatosRepresentante {
            nombre: "Pedro".to_string(),
            numero_contacto: "0412-0000000".to_string(),
        };
        assert!(validar_datos_representante(&rep).is_ok());

        let pago = DatosPago {
            representante_id: 3,
            monto: 1500.0,
            periodo: "2026-08".to_string(),
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        };
        assert!(validar_datos_pago(&pago).is_ok());
    }

    #[test]
    fn validacion_completa_reporta_error_de_formato() {
        let mut datos = DatosAlumno {
            nombre: "Juan".to_string(),
            fecha_de_nacimiento: "31/12/2010".to_string(),
            rango: 6,
            representante_id: 3,
            rallita: false,
        };

        match validar_datos_alumno(&datos) {
            Err(ErrorAplicacion::Validacion(_)) => {}
            otro => panic!("se esperaba error de validación, obtuve {otro:?}"),
        }

        datos.fecha_de_nacimiento = "2010-12-31".to_string();
        datos.representante_id = 0;
        match validar_datos_alumno(&datos) {
            Err(ErrorAplicacion::Validacion(_)) => {}
            otro => panic!("se esperaba error de validación, obtuve {otro:?}"),
        }
    }

    #[test]
    fn un_pago_con_monto_cero_se_rechaza() {
        let pago = DatosPago {
            representante_id: 1,
            monto: 0.0,
            periodo: "2026-08".to_string(),
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        };
        match validar_datos_pago(&pago) {
            Err(ErrorAplicacion::Validacion(_)) => {}
            otro => panic!("se esperaba error de validación, obtuve {otro:?}"),
        }
    }
}
