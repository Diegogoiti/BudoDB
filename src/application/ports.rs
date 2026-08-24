//! Puertos (interfaces) que la capa de aplicación exige al mundo exterior.
//! Las implementaciones concretas viven en la capa `infrastructure`.

/// Puerto de registro de eventos. Permite loguear sin acoplarse a una
/// implementación concreta (regla 10).
pub trait Logger: Send + Sync {
    /// Eventos de diagnóstico detallado (solo visibles en builds de desarrollo).
    fn debug(&self, mensaje: &str);
    /// Eventos normales del ciclo de vida de la aplicación.
    fn info(&self, mensaje: &str);
    /// Errores de operaciones críticas.
    fn error(&self, mensaje: &str);
}
