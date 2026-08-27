//! Casos de uso de deudas mensuales con el nuevo esquema.
//!
//! Las deudas ahora tienen `monto_pendiente` y `estado_id` persistidos,
//! en vez de calcularlos.derivan de abonos.

use super::dto::DeudaVista;
use super::error::ErrorAplicacion;
use super::ports::{DeudaRepository, Logger};
use crate::domain::{Deuda, EstadoDeuda, Representante};
use std::sync::Arc;

pub struct ServicioDeudas {
    repo_deudas: Arc<dyn DeudaRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioDeudas {
    pub fn nuevo(
        repo_deudas: Arc<dyn DeudaRepository>,
        _repo_abonos: Arc<dyn crate::application::ports::AbonoRepository>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self { repo_deudas, logger }
    }

    /// Crea deudas mensuales para todos los representantes activos que aún
    /// no tienen una en el periodo dado. Devuelve la cantidad creada.
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

        let existentes = self.repo_deudas.fetch_por_periodo(periodo)?;
        let ya_tienen: std::collections::HashSet<usize> =
            existentes.iter().map(|d| d.representante_id).collect();

        let mut creadas = 0;
        for rep in representantes {
            if ya_tienen.contains(&rep.id) {
                continue;
            }
            let deuda = Deuda {
                id: 0,
                representante_id: rep.id,
                monto_total: monto,
                monto_pendiente: monto,
                periodo: periodo.to_string(),
                fecha_vencimiento: fecha.to_string(),
                estado_id: EstadoDeuda::Pendiente.id(),
                alumno_id: None,
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

    /// Lista todas las deudas de un periodo con datos resueltos.
    pub fn listar_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<Vec<DeudaVista>, ErrorAplicacion> {
        let deudas = self.repo_deudas.fetch_por_periodo(periodo)?;

        let vistas: Vec<DeudaVista> = deudas
            .iter()
            .map(|deuda| {
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
                    estado: deuda.estado(),
                }
            })
            .collect();

        Ok(vistas)
    }

    /// Elimina (borrado lógico) deudas por IDs.
    pub fn eliminar(&self, ids: std::collections::HashSet<usize>) -> Result<(), ErrorAplicacion> {
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
    use std::collections::HashSet;
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
        fn fetch_cobrables_por_representante(&self, _: usize) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(Vec::new())
        }
        fn fetch_todos_periodos_por_representante(&self, _: usize) -> Result<Vec<String>, ErrorRepositorio> {
            Ok(Vec::new())
        }
        fn fetch_all(&self) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(self.deudas.lock().unwrap().clone())
        }
        fn update_estado(&self, _: usize, _: f64, _: i32) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
    }

    struct RepoAbonosMock;
    impl crate::application::ports::AbonoRepository for RepoAbonosMock {
        fn save(&self, _: &crate::domain::Abono) -> Result<(), ErrorRepositorio> { Ok(()) }
        fn fetch_por_deuda(&self, _: usize) -> Result<Vec<crate::domain::Abono>, ErrorRepositorio> { Ok(Vec::new()) }
        fn fetch_por_periodo(&self, _: &str) -> Result<Vec<crate::domain::Abono>, ErrorRepositorio> { Ok(Vec::new()) }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> { Ok(()) }
    }

    struct LoggerMock;
    impl crate::application::ports::Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    fn servicio(deudas: Vec<Deuda>) -> ServicioDeudas {
        let repo_d = Arc::new(RepoDeudasMock::con_deudas(deudas));
        ServicioDeudas::nuevo(repo_d, Arc::new(RepoAbonosMock), Arc::new(LoggerMock))
    }

    fn rep(id: usize, nombre: &str) -> Representante {
        Representante {
            id,
            nombre: nombre.to_string(),
            numero_contacto: "0412-0000000".to_string(),
            estado_id: 1,
        }
    }

    #[test]
    fn crear_deudas_solo_para_quienes_no_tienen() {
        let s = servicio(Vec::new());
        let reps = vec![rep(1, "A"), rep(2, "B"), rep(3, "C")];
        let creadas = s.crear_deudas_del_mes("2026-08", 1500.0, "2026-08-01", &reps).unwrap();
        assert_eq!(creadas, 3);
    }

    #[test]
    fn crear_deudas_no_duplica() {
        let existente = Deuda {
            id: 1, representante_id: 1, monto_total: 1500.0, monto_pendiente: 1500.0,
            periodo: "2026-08".to_string(), fecha_vencimiento: "2026-08-10".to_string(),
            estado_id: 1, alumno_id: None,
        };
        let s = servicio(vec![existente]);
        let reps = vec![rep(1, "A"), rep(2, "B")];
        let creadas = s.crear_deudas_del_mes("2026-08", 1500.0, "2026-08-01", &reps).unwrap();
        assert_eq!(creadas, 1);
    }
}
