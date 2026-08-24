//! DTO de entrada de la capa de aplicación (regla 5): lo que la UI envía
/// para crear o editar un alumno. Nunca es un struct de BD ni de dominio.

#[derive(Debug, Clone, PartialEq)]
pub struct DatosAlumno {
    pub nombre: String,
    pub fecha_de_nacimiento: String,
    pub rango: i32,
    pub representante: String,
    pub numero_contacto: String,
    pub rallita: bool,
}
