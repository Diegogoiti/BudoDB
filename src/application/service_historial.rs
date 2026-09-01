use super::dto::{DatosHistorialPago, HistorialPagoVista};
use super::error::ErrorAplicacion;
use super::ports::{HistorialPagoRepository, Logger};
use crate::domain::{HistorialPago, Representante};
use std::sync::Arc;

pub struct ServicioHistorialPagos {
    repositorio: Arc<dyn HistorialPagoRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioHistorialPagos {
    pub fn nuevo(repositorio: Arc<dyn HistorialPagoRepository>, logger: Arc<dyn Logger>) -> Self {
        Self { repositorio, logger }
    }

    pub fn registrar(&self, datos: DatosHistorialPago) -> Result<(), ErrorAplicacion> {
        if datos.monto <= 0.0 {
            return Err(ErrorAplicacion::Validacion("El monto debe ser positivo.".to_string()));
        }
        let registro = HistorialPago {
            id: 0,
            representante_id: datos.representante_id,
            tipo_id: datos.tipo_id,
            monto: datos.monto,
            periodo: datos.periodo,
            fecha: datos.fecha,
            observacion: datos.observacion,
        };
        self.repositorio.save(&registro)?;
        self.logger.debug("Registro de historial guardado");
        Ok(())
    }

    pub fn listar_todos(&self, representantes: &[Representante]) -> Result<Vec<HistorialPagoVista>, ErrorAplicacion> {
        let registros = self.repositorio.fetch_all()?;
        Ok(self.resolver_vistas(registros, representantes))
    }

    pub fn listar_por_representante(
        &self,
        representante_id: usize,
        representantes: &[Representante],
    ) -> Result<Vec<HistorialPagoVista>, ErrorAplicacion> {
        let registros = self.repositorio.fetch_por_representante(representante_id)?;
        Ok(self.resolver_vistas(registros, representantes))
    }

    pub fn listar_por_periodo(&self, periodo: &str, representantes: &[Representante]) -> Result<Vec<HistorialPagoVista>, ErrorAplicacion> {
        let registros = self.repositorio.fetch_por_periodo(periodo)?;
        Ok(self.resolver_vistas(registros, representantes))
    }

    fn resolver_vistas(
        &self,
        registros: Vec<HistorialPago>,
        representantes: &[Representante],
    ) -> Vec<HistorialPagoVista> {
        registros
            .iter()
            .map(|r| {
                let nombre = representantes
                    .iter()
                    .find(|rep| rep.id == r.representante_id)
                    .map(|rep| rep.nombre.clone())
                    .unwrap_or_else(|| "Desconocido".to_string());
                HistorialPagoVista { historial: r.clone(), nombre_representante: nombre }
            })
            .collect()
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use crate::domain::TipoHistorial;
    use std::collections::HashSet;
    use std::sync::Mutex;

    struct LoggerMock;
    impl crate::application::ports::Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    struct RepoHistorialMock {
        registros: Mutex<Vec<HistorialPago>>,
    }

    impl RepoHistorialMock {
        fn nuevo() -> Self {
            Self { registros: Mutex::new(Vec::new()) }
        }
        fn con_registros(vec: Vec<HistorialPago>) -> Self {
            Self { registros: Mutex::new(vec) }
        }
    }

    impl HistorialPagoRepository for RepoHistorialMock {
        fn save(&self, r: &HistorialPago) -> Result<(), ErrorRepositorio> {
            self.registros.lock().unwrap().push(r.clone());
            Ok(())
        }
        fn fetch_por_representante(&self, id: usize) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
            Ok(self.registros.lock().unwrap().iter().filter(|r| r.representante_id == id).cloned().collect())
        }
        fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
            Ok(self.registros.lock().unwrap().iter().filter(|r| r.periodo == periodo).cloned().collect())
        }
        fn fetch_all(&self) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
            Ok(self.registros.lock().unwrap().clone())
        }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
    }

    fn servicio(repo: RepoHistorialMock) -> (ServicioHistorialPagos, Arc<RepoHistorialMock>) {
        let repo = Arc::new(repo);
        (ServicioHistorialPagos::nuevo(repo.clone(), Arc::new(LoggerMock)), repo)
    }

    fn rep(id: usize) -> Representante {
        Representante { id, nombre: format!("Rep {id}"), numero_contacto: "0412-0000000".to_string(), estado_id: 1 }
    }

    fn registro(representante_id: usize, tipo_id: i32, monto: f64, periodo: &str) -> HistorialPago {
        HistorialPago {
            id: 0,
            representante_id,
            tipo_id,
            monto,
            periodo: periodo.to_string(),
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        }
    }

    #[test]
    fn registrar_rechaza_monto_no_positivo_sin_persistir() {
        let (s, repo) = servicio(RepoHistorialMock::nuevo());

        let mut datos = DatosHistorialPago {
            representante_id: 1,
            tipo_id: TipoHistorial::PagoRegistrado.id(),
            monto: 0.0,
            periodo: "2026-08".to_string(),
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        };
        assert!(matches!(s.registrar(datos.clone()), Err(ErrorAplicacion::Validacion(_))));

        datos.monto = -5.0;
        assert!(matches!(s.registrar(datos.clone()), Err(ErrorAplicacion::Validacion(_))));

        assert!(repo.registros.lock().unwrap().is_empty());

        datos.monto = 1500.0;
        s.registrar(datos).unwrap();
        assert_eq!(repo.registros.lock().unwrap().len(), 1);
    }

    #[test]
    fn listar_todos_resuelve_el_nombre_del_representante() {
        let (s, _) = servicio(RepoHistorialMock::con_registros(vec![
            registro(1, TipoHistorial::PagoRegistrado.id(), 1500.0, "2026-08"),
        ]));

        let vistas = s.listar_todos(&[rep(1)]).unwrap();
        assert_eq!(vistas.len(), 1);
        assert_eq!(vistas[0].nombre_representante, "Rep 1");
        assert_eq!(vistas[0].historial.monto, 1500.0);

        // Representante inexistente → "Desconocido"
        let vistas = s.listar_todos(&[]).unwrap();
        assert_eq!(vistas[0].nombre_representante, "Desconocido");
    }

    #[test]
    fn listar_por_representante_filtra() {
        let (s, _) = servicio(RepoHistorialMock::con_registros(vec![
            registro(1, TipoHistorial::PagoRegistrado.id(), 1500.0, "2026-08"),
            registro(2, TipoHistorial::DeudaCreada.id(), 3000.0, "2026-08"),
        ]));

        let vistas = s.listar_por_representante(2, &[rep(1), rep(2)]).unwrap();
        assert_eq!(vistas.len(), 1);
        assert_eq!(vistas[0].historial.representante_id, 2);
    }
}
