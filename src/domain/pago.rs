//! Entidad de negocio `Pago`: un registro de dinero recibido de un representante.
//!
//! El pago se aplica a deudas mediante la tabla `aplicaciones_pago`.
//! El estado indica si el pago está completo, reversado o pendiente de confirmación.
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

use super::catalogos::{EstadoPago, MetodoPago};

#[derive(PartialEq, Clone, Debug)]
pub struct Pago {
    pub id: usize,
    pub representante_id: usize,
    /// Monto total recibido en esta transacción.
    pub monto_recibido: f64,
    /// Estado del pago: Completado/Reversado/PendienteConfirmar.
    pub estado_id: i32,
    /// Método de pago: Efectivo/Transferencia/Tarjeta/Cheque.
    pub metodo_id: i32,
    /// Fecha de registro del pago, formato "YYYY-MM-DD".
    pub fecha_pago: String,
}

impl Pago {
    /// Estado del pago como enum tipado.
    pub fn estado(&self) -> EstadoPago {
        EstadoPago::from_id(self.estado_id).unwrap_or(EstadoPago::PendienteConfirmar)
    }

    /// Método de pago como enum tipado.
    pub fn metodo(&self) -> MetodoPago {
        MetodoPago::from_id(self.metodo_id).unwrap_or(MetodoPago::Efectivo)
    }
}

/// Etiqueta legible del periodo "2026-08" -> "Agosto 2026".
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

/// Formato canónico de periodo.
pub const FORMATO_PERIODO: &str = "%Y-%m";

#[cfg(test)]
mod pruebas {
    use super::*;

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
    fn estado_pago_desde_id() {
        assert_eq!(EstadoPago::from_id(1), Some(EstadoPago::Completado));
        assert_eq!(EstadoPago::from_id(2), Some(EstadoPago::Reversado));
        assert_eq!(EstadoPago::from_id(3), Some(EstadoPago::PendienteConfirmar));
    }

    #[test]
    fn metodo_pago_desde_id() {
        assert_eq!(MetodoPago::from_id(1), Some(MetodoPago::Efectivo));
        assert_eq!(MetodoPago::from_id(2), Some(MetodoPago::Transferencia));
        assert_eq!(MetodoPago::from_id(3), Some(MetodoPago::Tarjeta));
        assert_eq!(MetodoPago::from_id(4), Some(MetodoPago::Cheque));
    }
}
