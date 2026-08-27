//! Entidad de negocio `HistorialPago`: bitácora de auditoría de todos los
//! movimientos financieros. Nunca se edita ni se elimina.
//!
//! Cada registro indica qué tipo de movimiento fue, a qué representante
//! afecta, y en qué periodo/fecha ocurrió.
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

#[derive(PartialEq, Clone, Debug)]
pub struct HistorialPago {
    pub id: usize,
    pub representante_id: usize,
    /// FK a cat_tipos_historial: 1=DeudaCreada, 2=PagoRegistrado, etc.
    pub tipo_id: i32,
    pub monto: f64,
    /// Periodo al que aplica, formato "YYYY-MM".
    pub periodo: String,
    /// Fecha del registro, formato "YYYY-MM-DD".
    pub fecha: String,
    pub observacion: String,
}
