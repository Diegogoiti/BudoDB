use crate::application::dto::{AlumnoVista, DatosAlumno};
use crate::application::error::ErrorAplicacion;
use crate::application::ports::{AlumnoRepository, Logger};
use crate::application::validation::validar_datos_alumno;
use crate::domain::alumno::Alumno;
use crate::domain::Representante;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ServicioAlumnos {
    repositorio: Arc<dyn AlumnoRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioAlumnos {
    pub fn nuevo(repositorio: Arc<dyn AlumnoRepository>, logger: Arc<dyn Logger>) -> Self {
        Self { repositorio, logger }
    }
    pub fn obtener_todos(&self) -> Result<Vec<Alumno>, ErrorAplicacion> {
        Ok(self.repositorio.fetch_all()?)
    }
    pub fn agregar(&self, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        validar_datos_alumno(&datos)?;
        let alumno = Alumno { id: 0, nombre: datos.nombre, fecha_de_nacimiento: datos.fecha_de_nacimiento, rango: datos.rango, representante_id: datos.representante_id, rallita: Alumno::aplica_rallita(datos.rango, datos.rallita) };
        self.repositorio.save(&alumno)?;
        self.logger.debug("Alumno agregado");
        Ok(())
    }
    pub fn actualizar(&self, id: usize, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        validar_datos_alumno(&datos)?;
        let alumno = Alumno { id, nombre: datos.nombre, fecha_de_nacimiento: datos.fecha_de_nacimiento, rango: datos.rango, representante_id: datos.representante_id, rallita: datos.rallita };
        self.repositorio.update(&alumno)?;
        self.logger.debug("Alumno actualizado");
        Ok(())
    }
    pub fn promover(&self, ids: HashSet<usize>, rango: i32, rallita: bool) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() { return Ok(()); }
        self.repositorio.update_rangos(ids, rango, rallita)?;
        self.logger.debug("Grados actualizados en lote");
        Ok(())
    }
    pub fn eliminar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() { return Ok(()); }
        self.repositorio.delete(ids)?;
        self.logger.debug("Alumnos marcados como eliminados");
        Ok(())
    }
}

pub fn armar_vistas_alumnos(alumnos: &[Alumno], representantes: &[Representante]) -> Vec<AlumnoVista> {
    alumnos.iter().map(|alumno| {
        let rep = representantes.iter().find(|r| r.id == alumno.representante_id);
        AlumnoVista {
            alumno: alumno.clone(),
            nombre_representante: rep.map(|r| r.nombre.clone()).unwrap_or_else(|| "Sin representante".to_string()),
            telefono_representante: rep.map(|r| r.numero_contacto.clone()).unwrap_or_default(),
        }
    }).collect()
}
