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
        let alumno = Alumno { id: 0, nombre: datos.nombre, fecha_de_nacimiento: datos.fecha_de_nacimiento, rango: datos.rango, representante_id: datos.representante_id, rallita: Alumno::aplica_rallita(datos.rango, datos.rallita), estado_id: 1 };
        self.repositorio.save(&alumno)?;
        self.logger.debug("Alumno agregado");
        Ok(())
    }
    pub fn actualizar(&self, id: usize, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        validar_datos_alumno(&datos)?;
        let alumno = Alumno { id, nombre: datos.nombre, fecha_de_nacimiento: datos.fecha_de_nacimiento, rango: datos.rango, representante_id: datos.representante_id, rallita: datos.rallita, estado_id: 1 };
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
    pub fn desactivar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() { return Ok(()); }
        self.repositorio.desactivar(ids)?;
        self.logger.debug("Alumnos desactivados");
        Ok(())
    }
    pub fn activar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() { return Ok(()); }
        self.repositorio.activar(ids)?;
        self.logger.debug("Alumnos activados");
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

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;

    struct LoggerMock;
    impl crate::application::ports::Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    /// Repositorio en memoria para alumnos que registra llamadas.
    struct RepoAlumnosMock {
        alumnos: std::sync::Mutex<Vec<Alumno>>,
        guardados: std::sync::Mutex<usize>,
        desactivados: std::sync::Mutex<Option<std::collections::HashSet<usize>>>,
        activados: std::sync::Mutex<Option<std::collections::HashSet<usize>>>,
        rangos: std::sync::Mutex<Option<(std::collections::HashSet<usize>, i32, bool)>>,
        eliminados: std::sync::Mutex<Option<std::collections::HashSet<usize>>>,
    }

    impl RepoAlumnosMock {
        fn nuevo() -> Self {
            Self {
                alumnos: std::sync::Mutex::new(Vec::new()),
                guardados: std::sync::Mutex::new(0),
                desactivados: std::sync::Mutex::new(None),
                activados: std::sync::Mutex::new(None),
                rangos: std::sync::Mutex::new(None),
                eliminados: std::sync::Mutex::new(None),
            }
        }
    }

    impl AlumnoRepository for RepoAlumnosMock {
        fn save(&self, a: &Alumno) -> Result<(), ErrorRepositorio> {
            let mut lista = self.alumnos.lock().unwrap();
            *self.guardados.lock().unwrap() += 1;
            lista.push(a.clone());
            Ok(())
        }
        fn fetch_all(&self) -> Result<Vec<Alumno>, ErrorRepositorio> {
            Ok(self.alumnos.lock().unwrap().clone())
        }
        fn update(&self, a: &Alumno) -> Result<(), ErrorRepositorio> {
            let mut lista = self.alumnos.lock().unwrap();
            if let Some(slot) = lista.iter_mut().find(|x| x.id == a.id) {
                *slot = a.clone();
            } else {
                lista.push(a.clone());
            }
            Ok(())
        }
        fn update_rangos(&self, ids: std::collections::HashSet<usize>, rango: i32, rallita: bool) -> Result<(), ErrorRepositorio> {
            *self.rangos.lock().unwrap() = Some((ids, rango, rallita));
            Ok(())
        }
        fn delete(&self, ids: std::collections::HashSet<usize>) -> Result<(), ErrorRepositorio> {
            *self.eliminados.lock().unwrap() = Some(ids);
            Ok(())
        }
        fn desactivar(&self, ids: std::collections::HashSet<usize>) -> Result<(), ErrorRepositorio> {
            *self.desactivados.lock().unwrap() = Some(ids);
            Ok(())
        }
        fn activar(&self, ids: std::collections::HashSet<usize>) -> Result<(), ErrorRepositorio> {
            *self.activados.lock().unwrap() = Some(ids);
            Ok(())
        }
    }

    fn servicio(repo: RepoAlumnosMock) -> (ServicioAlumnos, Arc<RepoAlumnosMock>) {
        let repo = Arc::new(repo);
        (ServicioAlumnos::nuevo(repo.clone(), Arc::new(LoggerMock)), repo)
    }

    fn datos_alumno_ok() -> DatosAlumno {
        DatosAlumno {
            nombre: "Juan".to_string(),
            fecha_de_nacimiento: "2010-01-15".to_string(),
            rango: 6,
            representante_id: 1,
            rallita: false,
        }
    }

    fn alumno(id: usize, representante_id: usize, estado_id: i32) -> Alumno {
        Alumno {
            id,
            nombre: "Test".to_string(),
            rango: 6,
            fecha_de_nacimiento: "2010-01-15".to_string(),
            representante_id,
            rallita: false,
            estado_id,
        }
    }

    #[test]
    fn agregar_valida_antes_de_persistir() {
        let (s, repo) = servicio(RepoAlumnosMock::nuevo());

        // Datos válidos → se persiste con estado Activo y rallita aplicada
        s.agregar(datos_alumno_ok()).expect("debería agregar");
        assert_eq!(*repo.guardados.lock().unwrap(), 1);
        assert_eq!(repo.alumnos.lock().unwrap()[0].estado_id, 1);

        // Nombre vacío → error de validación, no se persiste
        let mut malo = datos_alumno_ok();
        malo.nombre = "".to_string();
        assert!(matches!(s.agregar(malo.clone()), Err(ErrorAplicacion::Validacion(_))));

        // Fecha inválida → error
        let mut malo = datos_alumno_ok();
        malo.fecha_de_nacimiento = "31/12/2010".to_string();
        assert!(matches!(s.agregar(malo.clone()), Err(ErrorAplicacion::Validacion(_))));

        // Representante no asignado → error
        let mut malo = datos_alumno_ok();
        malo.representante_id = 0;
        assert!(matches!(s.agregar(malo), Err(ErrorAplicacion::Validacion(_))));

        assert_eq!(*repo.guardados.lock().unwrap(), 1);
    }

    #[test]
    fn agregar_quita_la_rallita_si_el_alumno_es_dan() {
        let (s, repo) = servicio(RepoAlumnosMock::nuevo());
        let mut datos = datos_alumno_ok();
        datos.rango = 0; // 1er Dan
        datos.rallita = true; // se ignora

        s.agregar(datos).expect("debería agregar");
        assert!(!repo.alumnos.lock().unwrap()[0].rallita);
    }

    #[test]
    fn desactivar_y_activar_con_ids_no_vacios_delegan_al_repo() {
        let (s, repo) = servicio(RepoAlumnosMock::nuevo());

        s.desactivar(HashSet::from([1, 2])).unwrap();
        assert_eq!(
            *repo.desactivados.lock().unwrap(),
            Some(HashSet::from([1, 2]))
        );

        s.activar(HashSet::from([3])).unwrap();
        assert_eq!(*repo.activados.lock().unwrap(), Some(HashSet::from([3])));
    }

    #[test]
    fn desactivar_activar_promover_con_ids_vacios_es_noop() {
        let (s, repo) = servicio(RepoAlumnosMock::nuevo());

        s.desactivar(HashSet::new()).unwrap();
        s.activar(HashSet::new()).unwrap();
        s.promover(HashSet::new(), 5, false).unwrap();
        s.eliminar(HashSet::new()).unwrap();

        assert!(repo.desactivados.lock().unwrap().is_none());
        assert!(repo.activados.lock().unwrap().is_none());
        assert!(repo.rangos.lock().unwrap().is_none());
        assert!(repo.eliminados.lock().unwrap().is_none());
    }

    #[test]
    fn promover_delega_rango_y_rallita() {
        let (s, repo) = servicio(RepoAlumnosMock::nuevo());
        s.promover(HashSet::from([7]), 4, true).unwrap();
        assert_eq!(
            *repo.rangos.lock().unwrap(),
            Some((HashSet::from([7]), 4, true))
        );
    }

    #[test]
    fn armar_vistas_resuelve_nombre_y_telefono_del_representante() {
        let rep = Representante {
            id: 5,
            nombre: "María Gómez".to_string(),
            numero_contacto: "0412-1234567".to_string(),
            estado_id: 1,
        };
        let a = alumno(1, 5, 1);
        let sin_rep = alumno(2, 999, 1);

        let vistas = armar_vistas_alumnos(&[a, sin_rep], &[rep]);

        assert_eq!(vistas[0].nombre_representante, "María Gómez");
        assert_eq!(vistas[0].telefono_representante, "0412-1234567");
        assert_eq!(vistas[1].nombre_representante, "Sin representante");
        assert_eq!(vistas[1].telefono_representante, "");
    }
}
