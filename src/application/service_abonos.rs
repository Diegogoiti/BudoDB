//! Casos de uso de abonos: registro contra una deuda existente.
//!
//! Un abono reduce el saldo de una deuda. Si el abono es igual o mayor
//! al saldo restante, la deuda pasa a estado "Pagado".

use super::dto::DatosAbono;
use super::error::ErrorAplicacion;
use super::ports::{AbonoRepository, Logger};
use super::validation::validar_datos_abono;
use crate::domain::Abono;
use std::sync::Arc;

pub struct ServicioAbonos {
    repositorio: Arc<dyn AbonoRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioAbonos {
    pub fn nuevo(repositorio: Arc<dyn AbonoRepository>, logger: Arc<dyn Logger>) -> Self {
        Self {
            repositorio,
            logger,
        }
    }

    /// Registra un abono contra una deuda. Valida el formato antes de
    /// persistir.
    pub fn registrar(&self, datos: DatosAbono) -> Result<(), ErrorAplicacion> {
        validar_datos_abono(&datos)?;
        let abono = Abono {
            id: 0,
            deuda_id: datos.deuda_id,
            monto: datos.monto,
            fecha: datos.fecha,
            observacion: datos.observacion,
        };
        self.repositorio.save(&abono)?;
        self.logger.debug("Abono registrado");
        Ok(())
    }

    /// Total abonado contra una deuda específica.
    pub fn total_por_deuda(&self, deuda_id: usize) -> Result<f64, ErrorAplicacion> {
        let abonos = self.repositorio.fetch_por_deuda(deuda_id)?;
        Ok(abonos.iter().map(|a| a.monto).sum())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use std::collections::HashSet;
    use std::sync::Mutex;

    struct RepoAbonosMock {
        guardados: Mutex<Vec<Abono>>,
    }

    impl RepoAbonosMock {
        fn nuevo() -> Self {
            Self {
                guardados: Mutex::new(Vec::new()),
            }
        }
    }

    impl AbonoRepository for RepoAbonosMock {
        fn save(&self, a: &Abono) -> Result<(), ErrorRepositorio> {
            self.guardados.lock().unwrap().push(a.clone());
            Ok(())
        }
        fn fetch_por_deuda(&self, _: usize) -> Result<Vec<Abono>, ErrorRepositorio> {
            Ok(Vec::new())
        }
        fn fetch_por_periodo(&self, _: &str) -> Result<Vec<Abono>, ErrorRepositorio> {
            Ok(Vec::new())
        }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
    }

    struct LoggerMock;
    impl crate::application::ports::Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    fn servicio() -> (ServicioAbonos, Arc<RepoAbonosMock>) {
        let repo = Arc::new(RepoAbonosMock::nuevo());
        (
            ServicioAbonos::nuevo(repo.clone(), Arc::new(LoggerMock)),
            repo,
        )
    }

    #[test]
    fn registrar_abono_valida_antes_de_persistir() {
        let (s, repo) = servicio();

        let mut datos = DatosAbono {
            deuda_id: 0,
            monto: 500.0,
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        };
        assert!(s.registrar(datos.clone()).is_err());
        assert!(repo.guardados.lock().unwrap().is_empty());

        datos.deuda_id = 1;
        datos.monto = -10.0;
        assert!(s.registrar(datos.clone()).is_err());
        assert!(repo.guardados.lock().unwrap().is_empty());

        datos.monto = 500.0;
        s.registrar(datos).expect("debería registrar");
        assert_eq!(repo.guardados.lock().unwrap().len(), 1);
    }
}
