//! Entidad de negocio `HistorialPago`: registro del historial de pagos,
//! deudas y abonos para auditoría y cálculos posteriores.
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

#[derive(PartialEq, Clone, Debug)]
pub struct HistorialPago {
    pub id: usize,
    pub representante_id: usize,
    /// Tipo de movimiento: "deuda_creada", "abono", "pago", etc.
    pub tipo: String,
    pub monto: f64,
    /// Periodo al que aplica, formato "YYYY-MM".
    pub periodo: String,
    /// Fecha del registro, formato "YYYY-MM-DD".
    pub fecha: String,
    pub observacion: String,
}
