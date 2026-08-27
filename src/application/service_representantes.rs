//! Casos de uso de representantes: alta, listado y borrado lógico.
//! Mismas reglas de traducción de errores y validación que los alumnos.

use super::dto::DatosRepresentante;
use super::error::ErrorAplicacion;
use super::ports::{Logger, RepresentanteRepository};
use super::validation::validar_datos_representante;
use crate::domain::Representante;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ServicioRepresentantes {
    repositorio: Arc<dyn RepresentanteRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioRepresentantes {
    pub fn nuevo(repositorio: Arc<dyn RepresentanteRepository>, logger: Arc<dyn Logger>) -> Self {
        Self {
            repositorio,
            logger,
        }
    }

    /// Lista todos los representantes activos.
    pub fn obtener_todos(&self) -> Result<Vec<Representante>, ErrorAplicacion> {
        Ok(self.repositorio.fetch_all()?)
    }

    /// Registra un nuevo representante. El ID lo asigna la base de datos.
    pub fn agregar(&self, datos: DatosRepresentante) -> Result<(), ErrorAplicacion> {
        validar_datos_representante(&datos)?;
        let representante = Representante {
            id: 0,
            nombre: datos.nombre,
            numero_contacto: datos.numero_contacto,
            estado_id: 1,
        };
        self.repositorio.save(&representante)?;
        self.logger.debug("Representante agregado");
        Ok(())
    }

    /// Edita un representante existente conservando su ID.
    pub fn actualizar(&self, id: usize, datos: DatosRepresentante) -> Result<(), ErrorAplicacion> {
        validar_datos_representante(&datos)?;
        let representante = Representante {
            id,
            nombre: datos.nombre,
            numero_contacto: datos.numero_contacto,
            estado_id: 1,
        };
        self.repositorio.update(&representante)?;
        self.logger.debug("Representante actualizado");
        Ok(())
    }

    /// Borrado lógico: el historial de pagos conserva su nombre.
    pub fn eliminar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() {
            return Ok(());
        }
        self.repositorio.delete(ids)?;
        self.logger.debug("Representantes marcados como eliminados");
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use std::sync::Mutex;

    struct RepoRepMock {
        representantes: Mutex<Vec<Representante>>,
        fallo_listado: bool,
        guardados: Mutex<Vec<Representante>>,
        eliminados: Mutex<Option<HashSet<usize>>>,
    }

    impl RepoRepMock {
        fn nuevo() -> Self {
            Self {
                representantes: Mutex::new(Vec::new()),
                fallo_listado: false,
                guardados: Mutex::new(Vec::new()),
                eliminados: Mutex::new(None),
            }
        }
    }

    impl RepresentanteRepository for RepoRepMock {
        fn save(&self, r: &Representante) -> Result<(), ErrorRepositorio> {
            self.guardados.lock().unwrap().push(r.clone());
            Ok(())
        }

        fn fetch_all(&self) -> Result<Vec<Representante>, ErrorRepositorio> {
            if self.fallo_listado {
                return Err(ErrorRepositorio::Consulta("fallo simulado".to_string()));
            }
            Ok(self.representantes.lock().unwrap().clone())
        }

        fn update(&self, r: &Representante) -> Result<(), ErrorRepositorio> {
            let mut lista = self.representantes.lock().unwrap();
            match lista.iter_mut().find(|x| x.id == r.id) {
                Some(slot) => *slot = r.clone(),
                None => lista.push(r.clone()),
            }
            Ok(())
        }

        fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            *self.eliminados.lock().unwrap() = Some(ids);
            Ok(())
        }
    }

    struct LoggerMock;
    impl crate::application::ports::Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    fn servicio(repo: RepoRepMock) -> (ServicioRepresentantes, Arc<RepoRepMock>) {
        let repo = Arc::new(repo);
        (
            ServicioRepresentantes::nuevo(repo.clone(), Arc::new(LoggerMock)),
            repo,
        )
    }

    fn rep(id: usize, nombre: &str) -> Representante {
        Representante { id, nombre: nombre.to_string(), numero_contacto: "0412-0000000".to_string(), estado_id: 1 }
    }

    fn datos_rep() -> DatosRepresentante {
        DatosRepresentante {
            nombre: "Pedro Pérez".to_string(),
            numero_contacto: "0412-0000000".to_string(),
        }
    }

    #[test]
    fn agrega_validando_el_formato() {
        let (servicio, repo) = servicio(RepoRepMock::nuevo());

        servicio.agregar(datos_rep()).expect("debería agregar");

        assert_eq!(repo.guardados.lock().unwrap().len(), 1);

        let mut malo = datos_rep();
        malo.numero_contacto = "0412-00".to_string();
        assert!(servicio.agregar(malo).is_err());
        assert_eq!(repo.guardados.lock().unwrap().len(), 1);
    }

    #[test]
    fn listar_traduce_errores_del_puerto() {
        let mut repo = RepoRepMock::nuevo();
        repo.fallo_listado = true;
        let (servicio, _) = servicio(repo);
        assert!(matches!(
            servicio.obtener_todos(),
            Err(ErrorAplicacion::Repositorio(_))
        ));
    }

    #[test]
    fn eliminar_sin_ids_es_noop() {
        let (servicio, repo) = servicio(RepoRepMock::nuevo());
        servicio.eliminar(HashSet::new()).unwrap();
        assert!(repo.eliminados.lock().unwrap().is_none());
    }
}
