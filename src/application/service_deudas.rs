//! Casos de uso de deudas mensuales con el nuevo esquema.
//!
//! Las deudas ahora tienen `monto_pendiente` y `estado_id` persistidos,
//! en vez de calcularlos deriban de abonos.
//!
//! **Mensualidad por alumnos**: el monto de cada deuda se calcula como
//! `monto_base × número_de_alumnos_activos_del_representante`, a menos
//! que el administrador haya configurado un override explícito para ese
//! representante (clave `mensualidad_override_{rep_id}` en ajustes).

use super::dto::DeudaVista;
use super::error::ErrorAplicacion;
use super::ports::{ConfiguracionAppRepository, DeudaRepository, Logger};
use crate::domain::{Alumno, Deuda, EstadoDeuda, Representante};
use std::sync::Arc;

pub struct ServicioDeudas {
    repo_deudas: Arc<dyn DeudaRepository>,
    repo_ajustes: Arc<dyn ConfiguracionAppRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioDeudas {
    pub fn nuevo(
        repo_deudas: Arc<dyn DeudaRepository>,
        repo_ajustes: Arc<dyn ConfiguracionAppRepository>,
        _repo_abonos: Arc<dyn crate::application::ports::AbonoRepository>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self { repo_deudas, repo_ajustes, logger }
    }

    /// Calcula la mensualidad para un representante específico.
    ///
    /// 1. Busca si hay un override explícito (`mensualidad_override_{rep_id}` en ajustes).
    /// 2. Si no, usa `monto_base × num_alumnos_activos`.
    /// 3. Si hay 0 alumnos activos, devuelve 0 (no se genera deuda).
    fn mensualidad_para_representante(
        &self,
        representante_id: usize,
        monto_base: f64,
        alumnos_activos: &[Alumno],
    ) -> f64 {
        // 1. Override explícito
        let clave_override = format!("mensualidad_override_{representante_id}");
        if let Ok(Some(valor)) = self.repo_ajustes.obtener(&clave_override) {
            if let Ok(monto) = valor.parse::<f64>() {
                if monto > 0.0 {
                    return monto;
                }
            }
        }

        // 2. Cálculo automático: base × alumnos activos
        let num_alumnos = alumnos_activos.len() as f64;
        if num_alumnos <= 0.0 {
            return 0.0;
        }
        monto_base * num_alumnos
    }

    /// Crea deudas mensuales para todos los representantes activos que aún
    /// no tienen una en el periodo dado. El monto se calcula por representante
    /// según la cantidad de alumnos activos (o override explícito).
    /// Devuelve la cantidad creada.
    pub fn crear_deudas_del_mes(
        &self,
        periodo: &str,
        monto_base: f64,
        fecha: &str,
        representantes: &[Representante],
        alumnos: &[Alumno],
    ) -> Result<usize, ErrorAplicacion> {
        if monto_base <= 0.0 {
            return Err(ErrorAplicacion::Validacion(
                "El monto base de mensualidad debe ser mayor a cero.".to_string(),
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

            // Filtrar alumnos activos de este representante
            let alumnos_del_rep: Vec<&Alumno> = alumnos
                .iter()
                .filter(|a| a.representante_id == rep.id && a.estado_id == 1)
                .collect();

            if alumnos_del_rep.is_empty() {
                // Sin alumnos activos → no se genera deuda
                self.logger.debug(&format!(
                    "Rep #{} ({}) no tiene alumnos activos, se salta",
                    rep.id, rep.nombre
                ));
                continue;
            }

            let monto = self.mensualidad_para_representante(
                rep.id,
                monto_base,
                &alumnos_del_rep.into_iter().cloned().collect::<Vec<_>>(),
            );

            if monto <= 0.0 {
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
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    struct LoggerMock;
    impl crate::application::ports::Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    /// Repo de deudas: `crear_deudas_del_mes` guarda deudas y consulta las
    /// existentes por periodo.
    struct RepoDeudasMock {
        por_periodo: Mutex<Vec<Deuda>>,
        guardadas: Mutex<Vec<Deuda>>,
        cobrables: Mutex<Vec<Deuda>>,
        todas: Mutex<Vec<Deuda>>,
        periodos_rep: Mutex<Vec<String>>,
    }

    impl RepoDeudasMock {
        fn nuevo() -> Self {
            Self {
                por_periodo: Mutex::new(Vec::new()),
                guardadas: Mutex::new(Vec::new()),
                cobrables: Mutex::new(Vec::new()),
                todas: Mutex::new(Vec::new()),
                periodos_rep: Mutex::new(Vec::new()),
            }
        }

        fn con_existente(deuda: Deuda) -> Self {
            let repo = Self::nuevo();
            repo.por_periodo.lock().unwrap().push(deuda.clone());
            repo.todas.lock().unwrap().push(deuda);
            repo
        }
    }

    impl crate::application::ports::DeudaRepository for RepoDeudasMock {
        fn save(&self, d: &Deuda) -> Result<(), ErrorRepositorio> {
            let mut d = d.clone();
            d.id = self.guardadas.lock().unwrap().len() + 1;
            self.guardadas.lock().unwrap().push(d.clone());
            self.todas.lock().unwrap().push(d);
            Ok(())
        }
        fn fetch_por_periodo(&self, _: &str) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(self.por_periodo.lock().unwrap().clone())
        }
        fn fetch_cobrables_por_representante(&self, _: usize) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(self.cobrables.lock().unwrap().clone())
        }
        fn fetch_todos_periodos_por_representante(&self, _: usize) -> Result<Vec<String>, ErrorRepositorio> {
            Ok(self.periodos_rep.lock().unwrap().clone())
        }
        fn fetch_all(&self) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(self.todas.lock().unwrap().clone())
        }
        fn update_estado(&self, _: usize, _: f64, _: i32) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
    }

    /// Repo de ajustes en memoria para los overrides.
    struct RepoAjustesMock {
        valores: Mutex<HashMap<String, String>>,
    }

    impl RepoAjustesMock {
        fn nuevo() -> Self {
            Self { valores: Mutex::new(HashMap::new()) }
        }
        fn con_valor(clave: &str, valor: &str) -> Self {
            let repo = Self::nuevo();
            repo.valores.lock().unwrap().insert(clave.to_string(), valor.to_string());
            repo
        }
    }

    impl crate::application::ports::ConfiguracionAppRepository for RepoAjustesMock {
        fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorRepositorio> {
            Ok(self.valores.lock().unwrap().get(clave).cloned())
        }
        fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorRepositorio> {
            self.valores.lock().unwrap().insert(clave.to_string(), valor.to_string());
            Ok(())
        }
    }

    /// Repo de abonos de relleno (no usado por `crear_deudas_del_mes`).
    struct RepoAbonosMock;
    impl crate::application::ports::AbonoRepository for RepoAbonosMock {
        fn save(&self, _: &crate::domain::Abono) -> Result<(), ErrorRepositorio> { Ok(()) }
        fn fetch_por_deuda(&self, _: usize) -> Result<Vec<crate::domain::Abono>, ErrorRepositorio> { Ok(Vec::new()) }
        fn fetch_por_periodo(&self, _: &str) -> Result<Vec<crate::domain::Abono>, ErrorRepositorio> { Ok(Vec::new()) }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> { Ok(()) }
    }

    fn servicio(
        repo_deudas: RepoDeudasMock,
        repo_ajustes: RepoAjustesMock,
    ) -> (ServicioDeudas, Arc<RepoDeudasMock>) {
        let repo = Arc::new(repo_deudas);
        (
            ServicioDeudas::nuevo(
                repo.clone(),
                Arc::new(repo_ajustes),
                Arc::new(RepoAbonosMock),
                Arc::new(LoggerMock),
            ),
            repo,
        )
    }

    fn rep(id: usize) -> Representante {
        Representante { id, nombre: format!("Rep {id}"), numero_contacto: "0412-0000000".to_string(), estado_id: 1 }
    }

    fn alumno(id: usize, representante_id: usize) -> Alumno {
        Alumno {
            id,
            nombre: "Test".to_string(),
            rango: 6,
            fecha_de_nacimiento: "2010-01-15".to_string(),
            representante_id,
            rallita: false,
            estado_id: 1,
        }
    }

    /// Alumno con estado específico.
    fn alumno_estado(id: usize, representante_id: usize, estado_id: i32) -> Alumno {
        let mut a = alumno(id, representante_id);
        a.estado_id = estado_id;
        a
    }

    #[test]
    fn crea_una_deuda_por_representante_con_monto_base_por_alumno() {
        let (s, repo) = servicio(RepoDeudasMock::nuevo(), RepoAjustesMock::nuevo());
        // Rep 1 tiene 2 alumnos activos → 1500 * 2 = 3000
        let reps = vec![rep(1)];
        let alumnos = vec![alumno(1, 1), alumno(2, 1)];

        let creadas = s.crear_deudas_del_mes("2026-08", 1500.0, "2026-08-10", &reps, &alumnos).unwrap();

        assert_eq!(creadas, 1);
        let guardadas = repo.guardadas.lock().unwrap();
        assert_eq!(guardadas.len(), 1);
        assert_eq!(guardadas[0].representante_id, 1);
        assert!((guardadas[0].monto_total - 3000.0).abs() < f64::EPSILON);
        assert!((guardadas[0].monto_pendiente - 3000.0).abs() < f64::EPSILON);
        assert_eq!(guardadas[0].estado_id, EstadoDeuda::Pendiente.id());
        assert_eq!(guardadas[0].periodo, "2026-08");
    }

    #[test]
    fn respeta_el_override_de_mensualidad_por_representante() {
        let ajustes = RepoAjustesMock::con_valor("mensualidad_override_1", "5000");
        let (s, repo) = servicio(RepoDeudasMock::nuevo(), ajustes);
        let reps = vec![rep(1)];
        let alumnos = vec![alumno(1, 1), alumno(2, 1)]; // 2 activos

        s.crear_deudas_del_mes("2026-08", 1500.0, "2026-08-10", &reps, &alumnos).unwrap();

        let guardadas = repo.guardadas.lock().unwrap();
        assert!((guardadas[0].monto_total - 5000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn no_crea_deuda_si_no_hay_alumnos_activos() {
        let (s, repo) = servicio(RepoDeudasMock::nuevo(), RepoAjustesMock::nuevo());
        let reps = vec![rep(1)];

        // 1 alumno inactivo + 1 suspendido → no generan deuda
        let alumnos = vec![alumno_estado(1, 1, 2), alumno_estado(2, 1, 3)];
        let creadas = s.crear_deudas_del_mes("2026-08", 1500.0, "2026-08-10", &reps, &alumnos).unwrap();

        assert_eq!(creadas, 0);
        assert!(repo.guardadas.lock().unwrap().is_empty());
    }

    #[test]
    fn no_duplica_deuda_si_el_representante_ya_tiene_una_en_el_periodo() {
        let existente = Deuda {
            id: 99,
            representante_id: 1,
            monto_total: 3000.0,
            monto_pendiente: 3000.0,
            periodo: "2026-08".to_string(),
            fecha_vencimiento: "2026-08-10".to_string(),
            estado_id: EstadoDeuda::Pendiente.id(),
            alumno_id: None,
        };
        let (s, repo) = servicio(RepoDeudasMock::con_existente(existente), RepoAjustesMock::nuevo());
        let reps = vec![rep(1)];
        let alumnos = vec![alumno(1, 1), alumno(2, 1)];

        let creadas = s.crear_deudas_del_mes("2026-08", 1500.0, "2026-08-10", &reps, &alumnos).unwrap();

        assert_eq!(creadas, 0);
        assert!(repo.guardadas.lock().unwrap().is_empty());
    }

    #[test]
    fn sin_alumnos_activos_no_genera_deuda() {
        let (s, repo) = servicio(RepoDeudasMock::nuevo(), RepoAjustesMock::nuevo());
        // El rep 2 existe pero ningún alumno le pertenece → 0 deudas
        let rep_sin_alumnos = rep(2);
        let alumnos_del_1 = vec![alumno(1, 1)];

        let creadas = s.crear_deudas_del_mes("2026-08", 1500.0, "2026-08-10", &[rep_sin_alumnos], &alumnos_del_1).unwrap();
        assert_eq!(creadas, 0);
        assert!(repo.guardadas.lock().unwrap().is_empty());
    }

    #[test]
    fn rechaza_un_monto_base_no_positivo() {
        let (s, _) = servicio(RepoDeudasMock::nuevo(), RepoAjustesMock::nuevo());
        let reps = vec![rep(1)];
        assert!(s.crear_deudas_del_mes("2026-08", 0.0, "2026-08-10", &reps, &[]).is_err());
        assert!(s.crear_deudas_del_mes("2026-08", -10.0, "2026-08-10", &reps, &[]).is_err());
    }
}
