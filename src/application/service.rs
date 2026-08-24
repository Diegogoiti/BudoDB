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

    /// Elimina (borrado lógico) varios alumnos por ID: dejan de aparecer en
    /// el sistema pero su registro se conserva en la base de datos.
    pub fn eliminar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() {
            return Ok(());
        }
        self.repositorio.delete(ids)?;
        self.logger.debug("Alumnos marcados como eliminados");
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use std::sync::Mutex;

    /// Puerto mockeado: registra llamadas en memoria para verificar
    /// qué le pidió el caso de uso, sin tocar una BD real.
    struct RepoMock {
        alumnos: Mutex<Vec<Alumno>>,
        fallo_listado: bool,
        guardados: Mutex<Vec<Alumno>>,
        rangos_aplicados: Mutex<Option<(HashSet<usize>, i32, bool)>>,
        eliminados: Mutex<Option<HashSet<usize>>>,
    }

    impl RepoMock {
        fn nuevo() -> Self {
            Self {
                alumnos: Mutex::new(Vec::new()),
                fallo_listado: false,
                guardados: Mutex::new(Vec::new()),
                rangos_aplicados: Mutex::new(None),
                eliminados: Mutex::new(None),
            }
        }

        fn con_alumnos(alumnos: Vec<Alumno>) -> Self {
            let repo = Self::nuevo();
            *repo.alumnos.lock().unwrap() = alumnos;
            repo
        }
    }

    impl AlumnoRepository for RepoMock {
        fn save(&self, alumno: &Alumno) -> Result<(), ErrorRepositorio> {
            self.guardados.lock().unwrap().push(alumno.clone());
            Ok(())
        }

        fn fetch_all(&self) -> Result<Vec<Alumno>, ErrorRepositorio> {
            if self.fallo_listado {
                return Err(ErrorRepositorio::Consulta("fallo simulado".to_string()));
            }
            Ok(self.alumnos.lock().unwrap().clone())
        }

        fn update(&self, alumno: &Alumno) -> Result<(), ErrorRepositorio> {
            let mut lista = self.alumnos.lock().unwrap();
            match lista.iter_mut().find(|a| a.id == alumno.id) {
                Some(slot) => *slot = alumno.clone(),
                None => lista.push(alumno.clone()),
            }
            Ok(())
        }

        fn update_rangos(
            &self,
            ids: HashSet<usize>,
            rango: i32,
            rallita: bool,
        ) -> Result<(), ErrorRepositorio> {
            *self.rangos_aplicados.lock().unwrap() = Some((ids, rango, rallita));
            Ok(())
        }

        fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            *self.eliminados.lock().unwrap() = Some(ids);
            Ok(())
        }
    }

    struct LoggerMock;

    impl Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    fn servicio(repo: RepoMock) -> (ServicioAlumnos, Arc<RepoMock>) {
        let repo = Arc::new(repo);
        (
            ServicioAlumnos::nuevo(repo.clone(), Arc::new(LoggerMock)),
            repo,
        )
    }

    fn datos_validos() -> DatosAlumno {
        DatosAlumno {
            nombre: "Juan Pérez".to_string(),
            fecha_de_nacimiento: "2010-01-15".to_string(),
            rango: 6,
            representante: "Pedro Pérez".to_string(),
            numero_contacto: "0412-0000000".to_string(),
            rallita: false,
        }
    }

    fn alumno_existente(id: usize) -> Alumno {
        Alumno {
            id,
            nombre: "Viejo Nombre".to_string(),
            rango: 8,
            fecha_de_nacimiento: "2000-01-01".to_string(),
            representante: "R".to_string(),
            numero_contacto: "0412-0000000".to_string(),
            rallita: false,
        }
    }

    #[test]
    fn agregar_guarda_aplicando_la_regla_de_rallita_para_dan() {
        let (servicio, repo) = servicio(RepoMock::nuevo());
        let mut datos = datos_validos();
        datos.rango = -9; // 10° Dan
        datos.rallita = true;

        servicio.agregar(datos).expect("debería agregar");

        let guardados = repo.guardados.lock().unwrap();
        assert_eq!(guardados.len(), 1);
        assert_eq!(guardados[0].rango, -9);
        assert!(!guardados[0].rallita); // la regla de dominio la apaga
    }

    #[test]
    fn agregar_rechaza_datos_invalidos_y_no_persiste() {
        let (servicio, repo) = servicio(RepoMock::nuevo());
        let mut datos = datos_validos();
        datos.fecha_de_nacimiento = "31/12/2010".to_string();

        assert!(matches!(
            servicio.agregar(datos),
            Err(ErrorAplicacion::Validacion(_))
        ));
        assert!(repo.guardados.lock().unwrap().is_empty());
    }

    #[test]
    fn actualizar_conserva_el_id_y_los_nuevos_campos() {
        let (servicio, repo) = servicio(RepoMock::con_alumnos(vec![alumno_existente(7)]));

        servicio
            .actualizar(7, datos_validos())
            .expect("debería actualizar");

        let lista = repo.alumnos.lock().unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].id, 7);
        assert_eq!(lista[0].nombre, "Juan Pérez");
    }

    #[test]
    fn promover_pasa_los_ids_al_puerto() {
        let (servicio, repo) = servicio(RepoMock::nuevo());
        let ids: HashSet<usize> = HashSet::from([1, 2, 3]);

        servicio.promover(ids.clone(), 5, true).expect("debería promover");

        let aplicados = repo.rangos_aplicados.lock().unwrap().clone();
        assert_eq!(aplicados, Some((ids, 5, true)));
    }

    #[test]
    fn promover_sin_seleccion_es_un_noop() {
        let (servicio, repo) = servicio(RepoMock::nuevo());

        servicio
            .promover(HashSet::new(), 5, true)
            .expect("sin selección no debe fallar");

        assert!(repo.rangos_aplicados.lock().unwrap().is_none());
    }

    #[test]
    fn eliminar_pasa_los_ids_al_puerto() {
        let (servicio, repo) = servicio(RepoMock::nuevo());
        let ids: HashSet<usize> = HashSet::from([9]);

        servicio.eliminar(ids.clone()).expect("debería eliminar");

        assert_eq!(*repo.eliminados.lock().unwrap(), Some(ids));
    }

    #[test]
    fn obtener_todos_traduce_el_error_del_puerto() {
        let mut repo = RepoMock::nuevo();
        repo.fallo_listado = true;
        let (servicio, _) = servicio(repo);

        match servicio.obtener_todos() {
            Err(ErrorAplicacion::Repositorio(_)) => {}
            otro => panic!("se esperaba error de repositorio, obtuve {otro:?}"),
        }
    }
}
