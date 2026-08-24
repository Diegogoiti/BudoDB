//! Entidad de negocio `Deuda`: obligation mensual de un representante.
//! Cada mes, cada representante activo tiene una deuda por el monto
//! configurado en Ajustes. Los abonos se registran contra ella.
//!
//! El `estado` es DERIVADO (se calcula, no se almacena):
//!   - Pagado:   saldo == 0
//!   - Parcial:  0 < saldo < monto
//!   - Pendiente: saldo == monto (sin abonos)
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

#[derive(PartialEq, Clone, Debug)]
pub struct Deuda {
    pub id: usize,
    pub representante_id: usize,
    /// Monto total que se debe este mes.
    pub monto: f64,
    /// Mes que se cancela, formato "YYYY-MM".
    pub periodo: String,
    /// Fecha de creación de la deuda, formato "YYYY-MM-DD".
    pub fecha: String,
}

/// Estado derivado de una deuda según sus abonos. No se persiste: se calcula
/// al momento de armar la vista.
#[derive(PartialEq, Clone, Debug)]
pub enum EstadoDeuda {
    Pagado,
    Parcial,
    Pendiente,
}

impl Deuda {
    /// Saldo pendiente: monto total menos lo ya abonado.
    pub fn saldo(&self, total_abonado: f64) -> f64 {
        (self.monto - total_abonado).max(0.0)
    }

    /// Estado derivado a partir del total abonado.
    pub fn estado(&self, total_abonado: f64) -> EstadoDeuda {
        let saldo = self.saldo(total_abonado);
        if saldo <= 0.0 {
            EstadoDeuda::Pagado
        } else if saldo < self.monto {
            EstadoDeuda::Parcial
        } else {
            EstadoDeuda::Pendiente
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn deuda(monto: f64) -> Deuda {
        Deuda {
            id: 1,
            representante_id: 3,
            monto,
            periodo: "2026-08".to_string(),
            fecha: "2026-08-01".to_string(),
        }
    }

    #[test]
    fn saldo_se_calcula_como_monto_menos_abonos() {
        let d = deuda(1500.0);
        assert!((d.saldo(1500.0) - 0.0).abs() < f64::EPSILON);
        assert!((d.saldo(500.0) - 1000.0).abs() < f64::EPSILON);
        assert!((d.saldo(0.0) - 1500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saldo_nunca_es_negativo() {
        let d = deuda(1500.0);
        // Si por algún dato inconsistente hubiera más abonos que deuda,
        // el saldo se clampa a 0 (no hay deuda negativa).
        assert!((d.saldo(2000.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn estado_pagado_cuando_saldo_es_cero() {
        let d = deuda(1500.0);
        assert_eq!(d.estado(1500.0), EstadoDeuda::Pagado);
        assert_eq!(d.estado(2000.0), EstadoDeuda::Pagado);
    }

    #[test]
    fn estado_parcial_cuando_hay_abono_parcial() {
        let d = deuda(1500.0);
        assert_eq!(d.estado(500.0), EstadoDeuda::Parcial);
        assert_eq!(d.estado(1499.0), EstadoDeuda::Parcial);
    }

    #[test]
    fn estado_pendiente_sin_abonos() {
        let d = deuda(1500.0);
        assert_eq!(d.estado(0.0), EstadoDeuda::Pendiente);
    }
}
