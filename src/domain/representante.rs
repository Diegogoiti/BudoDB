//! Entidad de negocio `Representante`: el adulto responsable de uno o
//! varios alumnos. Es quien paga la mensualidad, de ahí que los pagos se
//! relacionen con él y no directamente con el alumno.
//!
//! CERO dependencias: ni UI, ni base de datos, ni frameworks (regla 1).

/// El nombre es obligatorio; el teléfono respeta el formato histórico
/// validado en `application/validation`.
#[derive(PartialEq, Clone, Debug)]
pub struct Representante {
    pub id: usize,
    pub nombre: String,
    pub numero_contacto: String,
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_entidad_se_clona_y_compara_por_valor() {
        let a = Representante {
            id: 1,
            nombre: "Pedro Pérez".to_string(),
            numero_contacto: "0412-0000000".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
