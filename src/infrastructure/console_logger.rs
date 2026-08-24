use crate::application::ports::Logger;

/// Implementación concreta del puerto [`Logger`] sobre la salida estándar.
///
/// En builds de desarrollo (`debug_assertions`) también emite los mensajes
/// de nivel `debug`; en release solo `info` y `error`.
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn debug(&self, mensaje: &str) {
        if cfg!(debug_assertions) {
            println!("[DEBUG] {mensaje}");
        }
    }

    fn info(&self, mensaje: &str) {
        println!("[INFO] {mensaje}");
    }

    fn error(&self, mensaje: &str) {
        eprintln!("[ERROR] {mensaje}");
    }
}
