//! Capa de dominio: entidades de negocio y reglas puras.
//! CERO dependencias: ni UI, ni base de datos, ni frameworks.

pub mod abono;
pub mod alumno;
pub mod aplicacion_pago;
pub mod catalogos;
pub mod cintas;
pub mod deuda;
pub mod historial_pago;
pub mod pago;
pub mod representante;

pub use abono::Abono;
pub use alumno::Alumno;
pub use aplicacion_pago::AplicacionPago;
pub use catalogos::*;
pub use cintas::Cintas;
pub use deuda::Deuda;
pub use historial_pago::HistorialPago;
pub use pago::Pago;
pub use representante::Representante;
