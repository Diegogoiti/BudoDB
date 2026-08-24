//! Casos de uso del sistema de pagos mensuales.
//!
//! Un pago pertenece a un REPRESENTANTE (quien paga la mensualidad de sus
//! alumnos). El periodo es el mes que se cancela, no la fecha de registro.

use super::dto::{DatosPago, PagoVista};
use super::error::ErrorAplicacion;
use super::ports::{Logger, PagoRepository};
use super::validation::validar_datos_pago;
use crate::domain::{Pago, Representante};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ServicioPagos {
    repositorio: Arc<dyn PagoRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioPagos {
    pub fn nuevo(repositorio: Arc<dyn PagoRepository>, logger: Arc<dyn Logger>) -> Self {
        Self {
            repositorio,
            logger,
        }
    }

    /// Registra un pago validando el formato de monto/periodo/fecha.
    pub fn registrar(&self, datos: DatosPago) -> Result<(), ErrorAplicacion> {
        validar_datos_pago(&datos)?;
        let pago = Pago {
            id: 0,
            representante_id: datos.representante_id,
            monto: datos.monto,
            periodo: datos.periodo,
            fecha: datos.fecha,
            observacion: datos.observacion,
        };
        self.repositorio.save(&pago)?;
        self.logger.debug("Pago registrado");
        Ok(())
    }

    /// Pagos de un periodo con el nombre del representante ya resuelto,
    /// ordenados del más reciente al más antiguo.
    pub fn listar_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<Vec<PagoVista>, ErrorAplicacion> {
        let pagos = self.repositorio.fetch_por_periodo(periodo)?;
        let mut vistas: Vec<PagoVista> = pagos
            .iter()
            .map(|pago| {
                let nombre = representantes
                    .iter()
                    .find(|r| r.id == pago.representante_id)
                    .map(|r| r.nombre.clone())
                    .unwrap_or_else(|| format!("ID {}", pago.representante_id));
                PagoVista {
                    pago: pago.clone(),
                    nombre_representante: nombre,
                }
            })
            .collect();
        vistas.sort_by(|a, b| b.pago.fecha.cmp(&a.pago.fecha));
        Ok(vistas)
    }

    /// Total recaudado en un periodo. La suma es un cálculo del caso de uso,
    /// no de la UI (la vista solo pinta).
    pub fn total_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<f64, ErrorAplicacion> {
        let vistas = self.listar_del_periodo(periodo, representantes)?;
        Ok(vistas.iter().map(|v| v.pago.monto).sum())
    }

    /// Representantes activos que NO tienen ningún pago registrado en el
    /// periodo dado: los morosos del mes.
    pub fn morosos_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<Vec<Representante>, ErrorAplicacion> {
        let pagos = self.repositorio.fetch_por_periodo(periodo)?;
        let pagadores: HashSet<usize> =
            pagos.iter().map(|p| p.representante_id).collect();
        Ok(representantes
            .iter()
            .filter(|r| !pagadores.contains(&r.id))
            .cloned()
            .collect())
    }

    /// Anula (borrado lógico) uno o varios pagos por ID.
    pub fn eliminar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() {
            return Ok(());
        }
        self.repositorio.delete(ids)?;
        self.logger.debug("Pagos anulados");
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use std::sync::Mutex;

    struct RepoPagoMock {
        pagos: Mutex<Vec<Pago>>,
        fallo_listado: bool,
        guardados: Mutex<Vec<Pago>>,
        eliminados: Mutex<Option<HashSet<usize>>>,
    }

    impl RepoPagoMock {
        fn nuevo() -> Self {
            Self {
                pagos: Mutex::new(Vec::new()),
                fallo_listado: false,
                guardados: Mutex::new(Vec::new()),
                eliminados: Mutex::new(None),
            }
        }

        fn con_pagos(pagos: Vec<Pago>) -> Self {
            let repo = Self::nuevo();
            *repo.pagos.lock().unwrap() = pagos;
            repo
        }
    }

    impl PagoRepository for RepoPagoMock {
        fn save(&self, p: &Pago) -> Result<(), ErrorRepositorio> {
            self.guardados.lock().unwrap().push(p.clone());
            Ok(())
        }

        fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Pago>, ErrorRepositorio> {
            if self.fallo_listado {
                return Err(ErrorRepositorio::Consulta("fallo simulado".to_string()));
            }
            Ok(self
                .pagos
                .lock()
                .unwrap()
                .iter()
                .filter(|p| p.periodo == periodo)
                .cloned()
                .collect())
        }

        fn fetch_all(&self) -> Result<Vec<Pago>, ErrorRepositorio> {
            if self.fallo_listado {
                return Err(ErrorRepositorio::Consulta("fallo simulado".to_string()));
            }
            Ok(self.pagos.lock().unwrap().clone())
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

    fn servicio(repo: RepoPagoMock) -> (ServicioPagos, Arc<RepoPagoMock>) {
        let repo = Arc::new(repo);
        (
            ServicioPagos::nuevo(repo.clone(), Arc::new(LoggerMock)),
            repo,
        )
    }

    fn rep(id: usize, nombre: &str) -> Representante {
        Representante {
            id,
            nombre: nombre.to_string(),
            numero_contacto: "0412-0000000".to_string(),
        }
    }

    fn pago(id: usize, rep_id: usize, periodo: &str, fecha: &str, monto: f64) -> Pago {
        Pago {
            id,
            representante_id: rep_id,
            monto,
            periodo: periodo.to_string(),
            fecha: fecha.to_string(),
            observacion: String::new(),
        }
    }

    #[test]
    fn registrar_valida_antes_de_persistir() {
        let (s, repo) = servicio(RepoPagoMock::nuevo());
        let mut datos = DatosPago {
            representante_id: 1,
            monto: -5.0,
            periodo: "2026-08".to_string(),
            fecha: "2026-08-24".to_string(),
            observacion: String::new(),
        };
        assert!(s.registrar(datos.clone()).is_err());
        assert!(repo.guardados.lock().unwrap().is_empty());

        datos.monto = 100.0;
        s.registrar(datos).expect("monto válido debe pasar");
        assert_eq!(repo.guardados.lock().unwrap().len(), 1);
    }

    #[test]
    fn listar_del_periodo_resuelve_nombres_y_ordena_descendente() {
        let (s, _) = servicio(RepoPagoMock::con_pagos(vec![
            pago(1, 10, "2026-08", "2026-08-01", 100.0),
            pago(2, 20, "2026-08", "2026-08-20", 200.0),
            // De otro mes: no debe aparecer en agosto
            pago(3, 10, "2026-07", "2026-07-15", 999.0),
        ]));
        let reps = vec![rep(10, "Ana"), rep(20, "Beto")];

        let vistas = s.listar_del_periodo("2026-08", &reps).unwrap();

        assert_eq!(vistas.len(), 2);
        // Orden descendente por fecha: el más reciente primero
        assert_eq!(vistas[0].nombre_representante, "Beto");
        assert_eq!(vistas[0].pago.id, 2);
        assert_eq!(vistas[1].nombre_representante, "Ana");
    }

    #[test]
    fn total_del_periodo_suma_solo_su_mes() {
        let (s, _) = servicio(RepoPagoMock::con_pagos(vec![
            pago(1, 10, "2026-08", "2026-08-01", 100.0),
            pago(2, 20, "2026-08", "2026-08-20", 250.5),
            pago(3, 10, "2026-09", "2026-09-02", 1000.0),
        ]));
        let reps = vec![rep(10, "Ana"), rep(20, "Beto")];

        let total = s.total_del_periodo("2026-08", &reps).unwrap();
        assert!((total - 350.5).abs() < f64::EPSILON);
    }

    #[test]
    fn morosos_son_los_que_no_pagaron_el_periodo() {
        let (s, _) = servicio(RepoPagoMock::con_pagos(vec![
            pago(1, 10, "2026-08", "2026-08-01", 100.0),
        ]));
        let reps = vec![rep(10, "Ana"), rep(20, "Beto"), rep(30, "Carla")];

        let morosos = s.morosos_del_periodo("2026-08", &reps).unwrap();

        assert_eq!(
            morosos.iter().map(|r| r.nombre.as_str()).collect::<Vec<_>>(),
            vec!["Beto", "Carla"]
        );
    }

    #[test]
    fn un_representante_borrado_aparece_como_id_desconocido_en_vistas() {
        let (s, _) = servicio(RepoPagoMock::con_pagos(vec![
            pago(1, 99, "2026-08", "2026-08-01", 100.0),
        ]));

        let vistas = s.listar_del_periodo("2026-08", &[]).unwrap();

        assert_eq!(vistas[0].nombre_representante, "ID 99");
    }
}
