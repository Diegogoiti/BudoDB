//! Entidad de negocio `AplicacionPago`: vincula un pago con una deuda
//! y registra cuánto monto se aplicó de ese pago a esa deuda específica.
//!
//! Esta tabla es el corazón del motor FIFO: cada pago se descompone en
//! una o más aplicaciones, cada una imputando un monto a una deuda concreta.
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

#[derive(PartialEq, Clone, Debug)]
pub struct AplicacionPago {
    pub id: usize,
    /// FK al pago que originó esta aplicación.
    pub pago_id: usize,
    /// FK a la deuda que recibe el abono.
    pub deuda_id: usize,
    /// Monto aplicado de este pago a esta deuda.
    pub monto_aplicado: f64,
    /// Fecha de la aplicación, formato "YYYY-MM-DD".
    pub fecha: String,
}
