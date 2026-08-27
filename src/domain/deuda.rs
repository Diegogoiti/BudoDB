//! Entidad de negocio `Deuda`: obligación mensual de un representante.
//!
//! A diferencia del modelo anterior (donde el estado era derivado), ahora
//! el `estado_id` se persiste directamente en la BD y se actualiza por la
//! máquina de estados del motor FIFO.
//!
//! Campos clave:
//! - `monto_total`: mensualidad configurada al momento de creación.
//! - `monto_pendiente`: se reduce con cada aplicación de pago.
//! - `fecha_vencimiento`: periodo + día de corte (ej: "2026-08-10").
//! - `alumno_id`: opcional — si la política es "Por Alumno".
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

use super::catalogos::EstadoDeuda;

#[derive(PartialEq, Clone, Debug)]
pub struct Deuda {
    pub id: usize,
    pub representante_id: usize,
    /// Monto total de la mensualidad al momento de creación.
    pub monto_total: f64,
    /// Saldo pendiente: se reduce con cada aplicación de pago.
    pub monto_pendiente: f64,
    /// Periodo que cubre esta deuda, formato "YYYY-MM".
    pub periodo: String,
    /// Fecha de vencimiento: periodo + día de corte (YYYY-MM-DD).
    pub fecha_vencimiento: String,
    /// Estado persistido: FK a cat_estados_deuda.
    pub estado_id: i32,
    /// Opcional: si la política es "Por Alumno", apunta al alumno específico.
    pub alumno_id: Option<usize>,
}

impl Deuda {
    /// Estado de la deuda como enum tipado.
    pub fn estado(&self) -> EstadoDeuda {
        EstadoDeuda::from_id(self.estado_id).unwrap_or(EstadoDeuda::Pendiente)
    }

    /// Saldo pendiente (= monto_pendiente, ya persistido).
    pub fn saldo(&self) -> f64 {
        self.monto_pendiente
    }

    /// Total abonado = monto_total - monto_pendiente.
    pub fn total_abonado(&self) -> f64 {
        (self.monto_total - self.monto_pendiente).max(0.0)
    }

    /// Porcentaje de avance (0.0 a 100.0).
    pub fn porcentaje(&self) -> f64 {
        if self.monto_total > 0.0 {
            (self.total_abonado() / self.monto_total * 100.0).min(100.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn deuda(monto_total: f64, monto_pendiente: f64) -> Deuda {
        Deuda {
            id: 1,
            representante_id: 3,
            monto_total,
            monto_pendiente,
            periodo: "2026-08".to_string(),
            fecha_vencimiento: "2026-08-10".to_string(),
            estado_id: EstadoDeuda::Pendiente.id(),
            alumno_id: None,
        }
    }

    #[test]
    fn saldo_es_monto_pendiente() {
        let d = deuda(1500.0, 500.0);
        assert!((d.saldo() - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn total_abonado_es_la_diferencia() {
        let d = deuda(1500.0, 500.0);
        assert!((d.total_abonado() - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn porcentaje_se_calcula_correctamente() {
        let d = deuda(1500.0, 750.0);
        assert!((d.porcentaje() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn porcentaje_nunca_supera_100() {
        let d = deuda(1500.0, 0.0);
        assert!((d.porcentaje() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn estado_desde_id() {
        assert_eq!(EstadoDeuda::from_id(1), Some(EstadoDeuda::Pendiente));
        assert_eq!(EstadoDeuda::from_id(3), Some(EstadoDeuda::Pagada));
        assert_eq!(EstadoDeuda::from_id(99), None);
    }

    #[test]
    fn estado_terminal_no_es_cobrable() {
        assert!(!EstadoDeuda::Pagada.es_cobrable());
        assert!(!EstadoDeuda::Anulada.es_cobrable());
        assert!(EstadoDeuda::Pendiente.es_cobrable());
        assert!(EstadoDeuda::Parcial.es_cobrable());
    }
}
