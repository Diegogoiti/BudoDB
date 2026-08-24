//! Casos de uso de deudas mensuales: creación, listado y cálculo de saldos.
//!
//! Una deuda representa la obligación mensual de un representante. Se crea
//! automáticamente con el monto configurado en Ajustes y se va saldando
//! mediante abonos.

use super::dto::DeudaVista;
use super::error::ErrorAplicacion;
use super::ports::{AbonoRepository, DeudaRepository, Logger};
use crate::domain::{Deuda, Representante};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ServicioDeudas {
    repo_deudas: Arc<dyn DeudaRepository>,
    repo_abonos: Arc<dyn AbonoRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioDeudas {
    pub fn nuevo(
        repo_deudas: Arc<dyn DeudaRepository>,
        repo_abonos: Arc<dyn AbonoRepository>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            repo_deudas,
            repo_abonos,
            logger,
        }
    }

    /// Crea deudas mensuales para todos los representantes activos que aún
    /// no tienen una en el periodo dado. Devuelve la cantidad de deudas
    /// nuevas creadas.
    pub fn crear_deudas_del_mes(
        &self,
        periodo: &str,
        monto: f64,
        fecha: &str,
        representantes: &[Representante],
    ) -> Result<usize, ErrorAplicacion> {
        if monto <= 0.0 {
            return Err(ErrorAplicacion::Validacion(
                "El monto de la mensualidad debe ser mayor a cero.".to_string(),
            ));
        }

        // Deudas existentes en este periodo (para no duplicar).
        let existentes = self.repo_deudas.fetch_por_periodo(periodo)?;
        let ya_tienen: HashSet<usize> =
            existentes.iter().map(|d| d.representante_id).collect();

        let mut creadas = 0;
        for rep in representantes {
            if ya_tienen.contains(&rep.id) {
                continue;
            }
            let deuda = Deuda {
                id: 0,
                representante_id: rep.id,
                monto,
                periodo: periodo.to_string(),
                fecha: fecha.to_string(),
            };
            self.repo_deudas.save(&deuda)?;
            creadas += 1;
        }

        if creadas > 0 {
            self.logger.info(&format!(
                "{creadas} deudas creadas para el periodo {periodo}"
            ));
        }
        Ok(creadas)
    }

    /// Lista todas las deudas de un periodo con sus saldos y estados ya
    /// calculados, ordenadas por nombre de representante.
    pub fn listar_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<Vec<DeudaVista>, ErrorAplicacion> {
        let deudas = self.repo_deudas.fetch_por_periodo(periodo)?;
        let todos_abonos = self.repo_abonos.fetch_por_periodo(periodo)?;

        let mut vistas: Vec<DeudaVista> = deudas
            .iter()
            .map(|deuda| {
                let total_abonado: f64 = todos_abonos
                    .iter()
                    .filter(|a| a.deuda_id == deuda.id)
                    .map(|a| a.monto)
                    .sum();

                let representante = representantes
                    .iter()
                    .find(|r| r.id == deuda.representante_id);

                DeudaVista {
                    deuda: deuda.clone(),
                    nombre_representante: representante
                        .map(|r| r.nombre.clone())
                        .unwrap_or_else(|| format!("ID {}", deuda.representante_id)),
                    telefono_representante: representante
                        .map(|r| r.numero_contacto.clone())
                        .unwrap_or_default(),
                    saldo: deuda.saldo(total_abonado),
                    estado: deuda.estado(total_abonado),
                    total_abonado,
                }
            })
            .collect();

        vistas.sort_by(|a, b| a.nombre_representante.cmp(&b.nombre_representante));
        Ok(vistas)
    }

    /// Elimina (borrado lógico) una deuda por ID.
    pub fn eliminar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() {
            return Ok(());
        }
        self.repo_deudas.delete(ids)?;
        self.logger.debug("Deudas eliminadas");
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use crate::domain::{Abono, EstadoDeuda};
    use std::sync::Mutex;

    struct RepoDeudasMock {
        deudas: Mutex<Vec<Deuda>>,
        guardados: Mutex<Vec<Deuda>>,
    }

    impl RepoDeudasMock {
        fn con_deudas(deudas: Vec<Deuda>) -> Self {
            Self {
                deudas: Mutex::new(deudas),
                guardados: Mutex::new(Vec::new()),
            }
        }
    }

    impl DeudaRepository for RepoDeudasMock {
        fn save(&self, d: &Deuda) -> Result<(), ErrorRepositorio> {
            self.guardados.lock().unwrap().push(d.clone());
            Ok(())
        }
        fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(self
                .deudas
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.periodo == periodo)
                .cloned()
                .collect())
        }
        fn fetch_all(&self) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(self.deudas.lock().unwrap().clone())
        }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
    }

    struct RepoAbonosMock {
        abonos: Mutex<Vec<Abono>>,
    }

    impl RepoAbonosMock {
        fn nuevo() -> Self {
            Self {
                abonos: Mutex::new(Vec::new()),
            }
        }

        fn con_abonos(abonos: Vec<Abono>) -> Self {
            let repo = Self::nuevo();
            *repo.abonos.lock().unwrap() = abonos;
            repo
        }
    }

    impl AbonoRepository for RepoAbonosMock {
        fn save(&self, _: &Abono) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
        fn fetch_por_deuda(&self, _: usize) -> Result<Vec<Abono>, ErrorRepositorio> {
            Ok(Vec::new())
        }
        fn fetch_por_periodo(&self, _: &str) -> Result<Vec<Abono>, ErrorRepositorio> {
            Ok(self.abonos.lock().unwrap().clone())
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

    fn servicio(
        deudas: Vec<Deuda>,
        abonos: Vec<Abono>,
    ) -> ServicioDeudas {
        let repo_d = Arc::new(RepoDeudasMock::con_deudas(deudas));
        let repo_a = Arc::new(RepoAbonosMock::con_abonos(abonos));
        ServicioDeudas::nuevo(repo_d, repo_a, Arc::new(LoggerMock))
    }

    fn rep(id: usize, nombre: &str) -> Representante {
        Representante {
            id,
            nombre: nombre.to_string(),
            numero_contacto: "0412-0000000".to_string(),
        }
    }

    #[test]
    fn crear_deudas_del_mes_solo_para_quienes_no_tienen() {
        let s = servicio(Vec::new(), Vec::new());
        let reps = vec![rep(1, "A"), rep(2, "B"), rep(3, "C")];

        let creadas = s
            .crear_deudas_del_mes("2026-08", 1500.0, "2026-08-01", &reps)
            .unwrap();
        assert_eq!(creadas, 3);
    }

    #[test]
    fn crear_deudas_no_duplica_quien_ya_tiene() {
        let existente = Deuda {
            id: 1,
            representante_id: 1,
            monto: 1500.0,
            periodo: "2026-08".to_string(),
            fecha: "2026-08-01".to_string(),
        };
        let s = servicio(vec![existente], Vec::new());
        let reps = vec![rep(1, "A"), rep(2, "B")];

        let creadas = s
            .crear_deudas_del_mes("2026-08", 1500.0, "2026-08-01", &reps)
            .unwrap();
        assert_eq!(creadas, 1); // Solo B
    }

    #[test]
    fn listar_del_periodo_calcula_saldo_y_estado() {
        let deudas = vec![
            Deuda { id: 1, representante_id: 1, monto: 1500.0, periodo: "2026-08".to_string(), fecha: "2026-08-01".to_string() },
            Deuda { id: 2, representante_id: 2, monto: 1500.0, periodo: "2026-08".to_string(), fecha: "2026-08-01".to_string() },
        ];
        let abonos = vec![
            Abono { id: 1, deuda_id: 1, monto: 1500.0, fecha: "2026-08-05".to_string(), observacion: String::new() },
            Abono { id: 2, deuda_id: 2, monto: 500.0, fecha: "2026-08-10".to_string(), observacion: String::new() },
        ];
        let s = servicio(deudas, abonos);
        let reps = vec![rep(1, "Ana"), rep(2, "Beto")];

        let vistas = s.listar_del_periodo("2026-08", &reps).unwrap();
        assert_eq!(vistas.len(), 2);

        let ana = vistas.iter().find(|v| v.nombre_representante == "Ana").unwrap();
        assert_eq!(ana.estado, EstadoDeuda::Pagado);
        assert!((ana.saldo).abs() < f64::EPSILON);

        let beto = vistas.iter().find(|v| v.nombre_representante == "Beto").unwrap();
        assert_eq!(beto.estado, EstadoDeuda::Parcial);
        assert!((beto.saldo - 1000.0).abs() < f64::EPSILON);
    }
}
