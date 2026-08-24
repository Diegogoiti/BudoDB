//! Capa de dominio: entidades de negocio y reglas puras.
//! CERO dependencias: ni UI, ni base de datos, ni frameworks.

pub mod alumno;
pub mod cintas;
pub mod pago;
pub mod representante;

pub use alumno::Alumno;
pub use cintas::Cintas;
pub use pago::Pago;
pub use representante::Representante;
