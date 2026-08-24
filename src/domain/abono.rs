//! Entidad de negocio `Abono`: pago parcial o total contra una deuda.
//! Cada abono reduce el saldo de una deuda específica.
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

#[derive(PartialEq, Clone, Debug)]
pub struct Abono {
    pub id: usize,
    /// FK hacia la deuda que se está saldando.
    pub deuda_id: usize,
    /// Monto abonado en esta transacción.
    pub monto: f64,
    /// Fecha de registro del abono, formato "YYYY-MM-DD".
    pub fecha: String,
    /// Nota libre opcional (ej: "efectivo", "transferencia", "pago parcial").
    pub observacion: String,
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use super::super::alumno::FORMATO_FECHA;

    #[test]
    fn la_fecha_de_registro_usa_el_formato_canonico() {
        let a = Abono {
            id: 1,
            deuda_id: 5,
            monto: 500.0,
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        };
        assert!(chrono::NaiveDate::parse_from_str(&a.fecha, FORMATO_FECHA).is_ok());
    }
}
