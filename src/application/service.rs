//! Casos de uso de alumnos: los únicos puntos de entrada de la presentación
/// hacia la persistencia (regla 3). Mapean explícitamente DTO de entrada ->
/// entidad de dominio y aplican validación + reglas antes de tocar puertos.

use super::dto::DatosAlumno;
use super::error::ErrorAplicacion;
use super::ports::{AlumnoRepository, Logger};
use super::validation::validar_datos_alumno;
use crate::domain::alumno::Alumno;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ServicioAlumnos {
    repositorio: Arc<dyn AlumnoRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioAlumnos {
    pub fn nuevo(repositorio: Arc<dyn AlumnoRepository>, logger: Arc<dyn Logger>) -> Self {
        Self {
            repositorio,
            logger,
        }
    }

    /// Lista todos los alumnos registrados.
    pub fn obtener_todos(&self) -> Result<Vec<Alumno>, ErrorAplicacion> {
        Ok(self.repositorio.fetch_all()?)
    }

    /// Registra un nuevo alumno: valida formato, aplica regla de rallita/Dan
    /// y persiste. El ID lo asigna la base de datos.
    pub fn agregar(&self, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        validar_datos_alumno(&datos)?;
        let alumno = Alumno {
            id: 0,
            nombre: datos.nombre,
            fecha_de_nacimiento: datos.fecha_de_nacimiento,
            rango: datos.rango,
            representante: datos.representante,
            numero_contacto: datos.numero_contacto,
            // Regla de dominio: un Dan nunca lleva rallita.
            rallita: Alumno::aplica_rallita(datos.rango, datos.rallita),
        };
        self.repositorio.save(&alumno)?;
        self.logger.debug("Alumno agregado");
        Ok(())
    }

    /// Edita un alumno existente conservando su ID.
    pub fn actualizar(&self, id: usize, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        validar_datos_alumno(&datos)?;
        let alumno = Alumno {
            id,
            nombre: datos.nombre,
            fecha_de_nacimiento: datos.fecha_de_nacimiento,
            rango: datos.rango,
            representante: datos.representante,
            numero_contacto: datos.numero_contacto,
            // NOTA: a diferencia de `agregar`, aquí NO se normaliza la rallita
            // para preservar el comportamiento histórico de la vista Editar.
            rallita: datos.rallita,
        };
        self.repositorio.update(&alumno)?;
        self.logger.debug("Alumno actualizado");
        Ok(())
    }

    /// Promoción masiva: asigna el mismo grado a varios IDs de una vez.
    pub fn promover(
        &self,
        ids: HashSet<usize>,
        rango: i32,
        rallita: bool,
    ) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() {
            return Ok(());
        }
        self.repositorio.update_rangos(ids, rango, rallita)?;
        self.logger.debug("Grados actualizados en lote");
        Ok(())
    }

    /// Elimina varios alumnos por ID.
    pub fn eliminar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() {
            return Ok(());
        }
        self.repositorio.delete(ids)?;
        self.logger.debug("Alumnos eliminados");
        Ok(())
    }
}
