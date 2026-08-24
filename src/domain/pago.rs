//! Entidad de negocio `Pago`: una mensualidad pagada por un representante.
//!
//! - `periodo`: mes que se está pagando, en formato "YYYY-MM".
//! - `fecha`: cuándo SE REGISTRÓ el pago, en el formato canónico ISO.
//!   Son cosas distintas: puedes pagar hoy la mensualidad de otro mes.
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

#[derive(PartialEq, Clone, Debug)]
pub struct Pago {
    pub id: usize,
    pub representante_id: usize,
    /// Monto de la mensualidad en la moneda local.
    pub monto: f64,
    /// Mes cancelado, formato "YYYY-MM".
    pub periodo: String,
    /// Fecha de registro del pago, formato YYYY-MM-DD.
    pub fecha: String,
    /// Nota libre opcional (ej: "incluye hermano", "pago parcial").
    pub observacion: String,
}

impl Pago {
    /// Etiqueta legible del periodo "2026-08" -> "Agosto 2026".
    pub fn etiqueta_periodo(&self) -> String {
        etiqueta_de_periodo(&self.periodo)
    }
}

/// Traduce "YYYY-MM" a "Mes AAAA". Función PURA reutilizable por la vista
/// para pintar encabezados sin duplicar nombres de meses.
pub fn etiqueta_de_periodo(periodo: &str) -> String {
    let partes: Vec<&str> = periodo.split('-').collect();
    if partes.len() != 2 {
        return periodo.to_string();
    }
    let nombre = match partes[1] {
        "01" => "Enero",
        "02" => "Febrero",
        "03" => "Marzo",
        "04" => "Abril",
        "05" => "Mayo",
        "06" => "Junio",
        "07" => "Julio",
        "08" => "Agosto",
        "09" => "Septiembre",
        "10" => "Octubre",
        "11" => "Noviembre",
        "12" => "Diciembre",
        _ => return periodo.to_string(),
    };
    format!("{nombre} {}", partes[0])
}

/// Formato canónico de periodo: constante compartida con las validaciones.
pub const FORMATO_PERIODO: &str = "%Y-%m";

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::domain::alumno::FORMATO_FECHA;

    fn pago() -> Pago {
        Pago {
            id: 1,
            representante_id: 3,
            monto: 1500.0,
            periodo: "2026-08".to_string(),
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        }
    }

    #[test]
    fn la_etiqueta_del_periodo_es_legible() {
        assert_eq!(etiqueta_de_periodo("2026-08"), "Agosto 2026");
        assert_eq!(etiqueta_de_periodo("2025-01"), "Enero 2025");
    }

    #[test]
    fn un_periodo_mal_formateado_no_rompe_la_etiqueta() {
        assert_eq!(etiqueta_de_periodo("basura"), "basura");
        assert_eq!(etiqueta_de_periodo("2026-13"), "2026-13");
    }

    #[test]
    fn la_entidad_usa_el_formato_iso_para_la_fecha_de_registro() {
        let p = pago();
        // La fecha de registro debe poder leerse con el mismo parseo que
        // usan las fechas de nacimiento: un solo formato en todo el sistema.
        assert!(chrono::NaiveDate::parse_from_str(&p.fecha, FORMATO_FECHA).is_ok());
    }
}
