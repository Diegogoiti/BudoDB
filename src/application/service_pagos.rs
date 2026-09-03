//! Casos de uso del sistema de pagos mensuales con motor FIFO.
//!
//! Un pago pertenece a un REPRESENTANTE. El motor FIFO determina
//! automáticamente a qué deudas se aplica el monto recibido:
//!
//! 1. **Fase 1 — FIFO de deudas viejas**: aplica el monto a la deuda más
//!    antigua (por `fecha_vencimiento`) que tenga `estado_id IN (1,2)`
//!    (Pendiente o Parcial). Si el monto no alcanza, queda en Parcial.
//!    Si alcanza o sobra, pasa a Pagada y el excedente avanza.
//! 2. **Fase 2 — Adelantos**: si queda dinero y no hay deudas viejas,
//!    crea deudas para meses futuros y les aplica el saldo restante.
//!
//! Cada aplicación genera un registro en `aplicaciones_pago` y un entry
//! en `historial_pagos`.

use super::dto::{DatosPago, PagoVista};
use super::error::ErrorAplicacion;
use super::ports::{AplicacionPagoRepository, ConfiguracionAppRepository, DeudaRepository, HistorialPagoRepository, Logger, PagoRepository};
use super::validation::validar_datos_pago;
use crate::domain::{
    AplicacionPago, Deuda, EstadoDeuda, EstadoPago, HistorialPago, MetodoPago, Pago,
    Representante, TipoHistorial,
};
use chrono::Datelike;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ServicioPagos {
    repo_pagos: Arc<dyn PagoRepository>,
    repo_aplicaciones: Arc<dyn AplicacionPagoRepository>,
    repo_deudas: Arc<dyn DeudaRepository>,
    repo_historial: Arc<dyn HistorialPagoRepository>,
    repo_ajustes: Arc<dyn ConfiguracionAppRepository>,
    logger: Arc<dyn Logger>,
}

/// Resultado de aplicar un pago: lista de (deuda_id, monto_aplicado).
pub type ResultadoFifo = Vec<(usize, f64)>;

impl ServicioPagos {
    pub fn nuevo(
        repo_pagos: Arc<dyn PagoRepository>,
        repo_aplicaciones: Arc<dyn AplicacionPagoRepository>,
        repo_deudas: Arc<dyn DeudaRepository>,
        repo_historial: Arc<dyn HistorialPagoRepository>,
        repo_ajustes: Arc<dyn ConfiguracionAppRepository>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self { repo_pagos, repo_aplicaciones, repo_deudas, repo_historial, repo_ajustes, logger }
    }

    /// Registra un pago y ejecuta el algoritmo FIFO completo.
    ///
    /// Devuelve el resumen de aplicaciones: `Vec<(deuda_id, monto_aplicado)>`.
    pub fn registrar_pago(
        &self,
        datos: DatosPago,
    ) -> Result<ResultadoFifo, ErrorAplicacion> {
        validar_datos_pago(&datos)?;

        // Pre-validar: si puede sobrar dinero, verificar mensualidad ANTES de insertar.
        let deudas_cobrables = self.repo_deudas.fetch_cobrables_por_representante(datos.representante_id)?;
        let total_cobrable: f64 = deudas_cobrables.iter().map(|d| d.monto_pendiente).sum();
        if datos.monto_recibido > total_cobrable {
            self.obtener_mensualidad()?;
        }

        // 1. Crear el pago con estado Completado
        let pago = Pago {
            id: 0,
            representante_id: datos.representante_id,
            monto_recibido: datos.monto_recibido,
            estado_id: EstadoPago::Completado.id(),
            metodo_id: datos.metodo_id,
            fecha_pago: datos.fecha_pago.clone(),
        };
        let pago_id = self.repo_pagos.save(&pago)?;

        self.logger.info(&format!(
            "Pago #{pago_id}: ${:.2} del representante #{} (método: {})",
            datos.monto_recibido,
            datos.representante_id,
            MetodoPago::from_id(datos.metodo_id)
                .map(|m| m.label())
                .unwrap_or("Desconocido"),
        ));

        // 2. Ejecutar FIFO
        let aplicaciones = match self.aplicar_fifo(
            pago_id,
            datos.representante_id,
            datos.monto_recibido,
            &datos.fecha_pago,
        ) {
            Ok(apps) => apps,
            Err(error) => {
                self.logger.error(&format!(
                    "Pago #{pago_id}: error en FIFO - {error}. Eliminando pago huérfano."
                ));
                let _ = self.repo_pagos.delete(HashSet::from([pago_id]));
                return Err(error);
            }
        };

        // 3. Historial: registro del pago
        let total_aplicado: f64 = aplicaciones.iter().map(|(_, m)| m).sum();
        let historial = HistorialPago {
            id: 0,
            representante_id: datos.representante_id,
            tipo_id: TipoHistorial::PagoRegistrado.id(),
            monto: total_aplicado,
            periodo: String::new(),
            fecha: datos.fecha_pago.clone(),
            observacion: format!(
                "Pago de ${:.2} aplicado a {} deuda(s)",
                total_aplicado,
                aplicaciones.len(),
            ),
        };
        // Ignore historial errors (non-critical)
        if let Err(e) = self.repo_historial.save(&historial) {
            self.logger.error(&format!("Error guardando historial: {e}"));
        }

        if aplicaciones.is_empty() {
            self.logger.info(&format!(
                "Pago #{pago_id}: ${:.2} registrado sin deudas pendientes",
                datos.monto_recibido,
            ));
        } else {
            self.logger.info(&format!(
                "Pago #{pago_id}: ${:.2} aplicado a {} deuda(s)",
                total_aplicado,
                aplicaciones.len(),
            ));
        }

        Ok(aplicaciones)
    }

    /// Algoritmo FIFO: aplica un monto a deudas de un representante.
    ///
    /// **Fase 1**: deudas Pendientes/Parciales más antiguas (por fecha_vencimiento).
    /// **Fase 2**: si sobra dinero, crea deudas para meses futuros (adelantos).
    fn aplicar_fifo(
        &self,
        pago_id: usize,
        representante_id: usize,
        monto_total: f64,
        fecha_pago: &str,
    ) -> Result<ResultadoFifo, ErrorAplicacion> {
        let mut monto_restante = monto_total;
        let mut resultado: ResultadoFifo = Vec::new();

        // ─── Fase 1: aplicar a deudas viejas (FIFO) ───
        let mut deudas_cobrables = self.repo_deudas.fetch_cobrables_por_representante(representante_id)?;

        for deuda in &mut deudas_cobrables {
            if monto_restante <= 0.0 {
                break;
            }

            let aplicar = monto_restante.min(deuda.monto_pendiente);
            if aplicar <= 0.0 {
                continue;
            }

            // Actualizar la deuda
            let nuevo_pendiente = deuda.monto_pendiente - aplicar;
            let nuevo_estado = if nuevo_pendiente <= 0.0 {
                EstadoDeuda::Pagada.id()
            } else {
                EstadoDeuda::Parcial.id()
            };

            self.repo_deudas.update_estado(
                deuda.id,
                nuevo_pendiente.max(0.0),
                nuevo_estado,
            )?;

            // Registrar aplicación de pago
            let aplicacion = AplicacionPago {
                id: 0,
                pago_id,
                deuda_id: deuda.id,
                monto_aplicado: aplicar,
                fecha: fecha_pago.to_string(),
            };
            self.repo_aplicaciones.save(&aplicacion)?;

            resultado.push((deuda.id, aplicar));
            monto_restante -= aplicar;

            self.logger.debug(&format!(
                "  → ${:.2} aplicados a deuda #{} ({}) — saldo restante: ${:.2}",
                aplicar,
                deuda.id,
                deuda.periodo,
                nuevo_pendiente.max(0.0),
            ));

            // Historial: abono aplicado
            let hist = HistorialPago {
                id: 0,
                representante_id,
                tipo_id: TipoHistorial::AbonoAplicado.id(),
                monto: aplicar,
                periodo: deuda.periodo.clone(),
                fecha: fecha_pago.to_string(),
                observacion: format!("Pago #{pago_id} → deuda #{}", deuda.id),
            };
            let _ = self.repo_historial.save(&hist);
        }

        // ─── Fase 2: crear adelantos si sobra dinero ───
        if monto_restante > 0.0 {
            let mensualidad = self.obtener_mensualidad()?;
            let periodos_existentes: HashSet<String> = self.repo_deudas
                .fetch_todos_periodos_por_representante(representante_id)?
                .into_iter()
                .collect();

            // Generar periodos futuros a partir del mes siguiente al actual
            let ahora = chrono::Local::now();
            let mut year = ahora.year();
            let mut month = ahora.month() + 1;
            if month > 12 {
                month = 1;
                year += 1;
            }

            // Limitar a 12 meses futuros máximo
            for _ in 0..12 {
                if monto_restante <= 0.0 {
                    break;
                }

                let periodo = format!("{year:04}-{month:02}");

                // Solo crear adelanto si no existe ya una deuda para ese periodo
                if !periodos_existentes.contains(&periodo) {
                    let monto_deuda = mensualidad;
                    let aplicar = monto_restante.min(monto_deuda);

                    let nueva_deuda = Deuda {
                        id: 0,
                        representante_id,
                        monto_total: monto_deuda,
                        monto_pendiente: monto_deuda - aplicar,
                        periodo: periodo.clone(),
                        fecha_vencimiento: format!("{periodo}-01"),
                        estado_id: if (monto_deuda - aplicar) <= 0.0 {
                            EstadoDeuda::Pagada.id()
                        } else {
                            EstadoDeuda::Anticipada.id()
                        },
                        alumno_id: None,
                    };
                    self.repo_deudas.save(&nueva_deuda)?;

                    // Nota: necesitamos el ID de la deuda recién creada.
                    // Re-fetch: tomar la última deuda de este representante en este periodo.
                    let deudas_periodo = self.repo_deudas.fetch_por_periodo(&periodo)?;
                    if let Some(deuda_nueva) = deudas_periodo.iter().find(|d| d.representante_id == representante_id) {
                        // Registrar aplicación
                        let aplicacion = AplicacionPago {
                            id: 0,
                            pago_id,
                            deuda_id: deuda_nueva.id,
                            monto_aplicado: aplicar,
                            fecha: fecha_pago.to_string(),
                        };
                        self.repo_aplicaciones.save(&aplicacion)?;
                        resultado.push((deuda_nueva.id, aplicar));

                        self.logger.debug(&format!(
                            "  → ${:.2} adelanto para {periodo} (deuda #{})",
                            aplicar,
                            deuda_nueva.id,
                        ));

                        // Historial: deuda creada como anticipada
                        let hist = HistorialPago {
                            id: 0,
                            representante_id,
                            tipo_id: TipoHistorial::DeudaCreada.id(),
                            monto: monto_deuda,
                            periodo: periodo.clone(),
                            fecha: fecha_pago.to_string(),
                            observacion: format!(
                                "Adelanto: deuda #{} cubierta ${:.2}/${:.2}",
                                deuda_nueva.id, aplicar, monto_deuda,
                            ),
                        };
                        let _ = self.repo_historial.save(&hist);
                    }

                    monto_restante -= aplicar;
                }

                // Avanzar al siguiente mes
                month += 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
            }
        }

        Ok(resultado)
    }

    /// Obtiene la mensualidad configurada desde ajustes.
    fn obtener_mensualidad(&self) -> Result<f64, ErrorAplicacion> {
        self.repo_ajustes
            .obtener("monto_mensualidad")
            .map_err(|e| ErrorAplicacion::Repositorio(e))
            .and_then(|opt| {
                opt.and_then(|v| v.parse::<f64>().ok())
                    .filter(|&m| m > 0.0)
                    .ok_or_else(|| ErrorAplicacion::Validacion(
                        "Configure el monto de mensualidad en Ajustes primero.".to_string(),
                    ))
            })
    }

    /// Lista pagos de un periodo con datos resueltos.
    pub fn listar_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<Vec<PagoVista>, ErrorAplicacion> {
        let pagos = self.repo_pagos.fetch_por_periodo(periodo)?;
        let mut vistas: Vec<PagoVista> = pagos
            .iter()
            .map(|pago| {
                let nombre = representantes
                    .iter()
                    .find(|r| r.id == pago.representante_id)
                    .map(|r| r.nombre.clone())
                    .unwrap_or_else(|| format!("ID {}", pago.representante_id));

                // Cargar aplicaciones de este pago
                let aplicaciones_raw = self.repo_aplicaciones.fetch_por_pago(pago.id)
                    .unwrap_or_default();
                let deudas = self.repo_deudas.fetch_all().unwrap_or_default();
                let aplicaciones = aplicaciones_raw.iter().map(|ap| {
                    let periodo_deuda = deudas.iter()
                        .find(|d| d.id == ap.deuda_id)
                        .map(|d| d.periodo.clone())
                        .unwrap_or_default();
                    super::dto::AplicacionPagoVista {
                        aplicacion: ap.clone(),
                        nombre_representante: nombre.clone(),
                        periodo_deuda,
                    }
                }).collect();

                PagoVista {
                    pago: pago.clone(),
                    nombre_representante: nombre,
                    metodo: pago.metodo(),
                    estado: pago.estado(),
                    aplicaciones,
                }
            })
            .collect();
        vistas.sort_by(|a, b| b.pago.fecha_pago.cmp(&a.pago.fecha_pago));
        Ok(vistas)
    }

    /// Representantes que NO tienen pagos completados en el periodo.
    pub fn morosos_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<Vec<Representante>, ErrorAplicacion> {
        let pagos = self.repo_pagos.fetch_por_periodo(periodo)?;
        let pagadores: HashSet<usize> = pagos
            .iter()
            .filter(|p| p.estado_id == EstadoPago::Completado.id())
            .map(|p| p.representante_id)
            .collect();
        Ok(representantes
            .iter()
            .filter(|r| !pagadores.contains(&r.id))
            .cloned()
            .collect())
    }

    /// Reversa un pago: restaura saldos de deudas afectadas y cambia estado del pago.
    pub fn reversar_pago(&self, pago_id: usize) -> Result<(), ErrorAplicacion> {
        // 1. Buscar aplicaciones del pago
        let aplicaciones = self.repo_aplicaciones.fetch_por_pago(pago_id)?;

        if aplicaciones.is_empty() {
            self.logger.info(&format!(
                "Pago #{pago_id}: sin aplicaciones, solo se cambia estado a Reversado"
            ));
        }

        // 2. Restaurar saldos de cada deuda afectada
        for app in &aplicaciones {
            // Buscar la deuda directamente
            let todas = self.repo_deudas.fetch_all()?;
            if let Some(deuda) = todas.iter().find(|d| d.id == app.deuda_id) {
                let nuevo_pendiente = (deuda.monto_pendiente + app.monto_aplicado).min(deuda.monto_total);
                let nuevo_estado = if nuevo_pendiente >= deuda.monto_total {
                    EstadoDeuda::Pendiente.id()
                } else if nuevo_pendiente > 0.0 {
                    EstadoDeuda::Parcial.id()
                } else {
                    EstadoDeuda::Pagada.id()
                };

                self.repo_deudas.update_estado(
                    deuda.id,
                    nuevo_pendiente,
                    nuevo_estado,
                )?;

                self.logger.debug(&format!(
                    "  → Deuda #{}: saldo restaurado a ${:.2} ({})",
                    deuda.id,
                    nuevo_pendiente,
                    EstadoDeuda::from_id(nuevo_estado)
                        .map(|e| e.label())
                        .unwrap_or("?"),
                ));
            }
        }

        // 3. Eliminar las aplicaciones (físico, es una tabla puente)
        self.repo_aplicaciones.delete_por_pago(pago_id)?;

        // 4. Cambiar estado del pago a Reversado
        self.repo_pagos.update_estado(pago_id, EstadoPago::Reversado.id())?;

        // 5. Historial
        let historial = HistorialPago {
            id: 0,
            representante_id: 0, // Se pierde en reversión, pero es aceptable
            tipo_id: TipoHistorial::Anulacion.id(),
            monto: 0.0,
            periodo: String::new(),
            fecha: chrono::Local::now().format("%Y-%m-%d").to_string(),
            observacion: format!(
                "Pago #{pago_id} reversado - {} aplicacion(es) restaurada(s)",
                aplicaciones.len(),
            ),
        };
        let _ = self.repo_historial.save(&historial);

        self.logger.info(&format!(
            "Pago #{pago_id} reversado: {} aplicación(es) revertida(s)",
            aplicaciones.len(),
        ));
        Ok(())
    }

    /// Total recaudado (solo pagos completados) en un periodo.
    pub fn total_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<f64, ErrorAplicacion> {
        let vistas = self.listar_del_periodo(periodo, representantes)?;
        Ok(vistas
            .iter()
            .filter(|v| v.estado == EstadoPago::Completado)
            .map(|v| v.pago.monto_recibido)
            .sum())
    }

    /// Anula pagos por IDs (borrado lógico).
    pub fn eliminar(&self, ids: HashSet<usize>) -> Result<(), ErrorAplicacion> {
        if ids.is_empty() {
            return Ok(());
        }
        self.repo_pagos.delete(ids)?;
        self.logger.debug("Pagos anulados");
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::{
        AplicacionPagoRepository, ConfiguracionAppRepository, DeudaRepository, ErrorRepositorio,
        HistorialPagoRepository, Logger, PagoRepository,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct LoggerMock;
    impl Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    // ── Repositorios en memoria ──

    struct RepoPagosMock {
        pagos: Mutex<Vec<Pago>>,
        proximo_id: Mutex<usize>,
        estados: Mutex<Vec<(usize, i32)>>,
    }

    impl RepoPagosMock {
        fn nuevo() -> Self {
            Self { pagos: Mutex::new(Vec::new()), proximo_id: Mutex::new(1), estados: Mutex::new(Vec::new()) }
        }
    }

    impl PagoRepository for RepoPagosMock {
        fn save(&self, p: &Pago) -> Result<usize, ErrorRepositorio> {
            let mut p = p.clone();
            p.id = *self.proximo_id.lock().unwrap();
            *self.proximo_id.lock().unwrap() += 1;
            self.pagos.lock().unwrap().push(p.clone());
            Ok(p.id)
        }
        fn fetch_por_periodo(&self, _: &str) -> Result<Vec<Pago>, ErrorRepositorio> {
            Ok(self.pagos.lock().unwrap().clone())
        }
        fn fetch_por_representante(&self, _: usize) -> Result<Vec<Pago>, ErrorRepositorio> {
            Ok(self.pagos.lock().unwrap().clone())
        }
        fn fetch_all(&self) -> Result<Vec<Pago>, ErrorRepositorio> {
            Ok(self.pagos.lock().unwrap().clone())
        }
        fn update_estado(&self, id: usize, estado_id: i32) -> Result<(), ErrorRepositorio> {
            self.estados.lock().unwrap().push((id, estado_id));
            Ok(())
        }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
    }

    struct RepoDeudasMock {
        // deudas cobrables (Fase 1, FIFO) — ya ordenadas por vencimiento
        cobrables: Mutex<Vec<Deuda>>,
        // estado de cada deuda tras update_estado
        saldos: Mutex<Vec<(usize, f64, i32)>>,
        // deudas creadas (Fase 2 adelantos)
        guardadas: Mutex<Vec<Deuda>>,
        // periodos existentes por representante
        periodos_rep: Mutex<Vec<String>>,
        // fetch_por_periodo devuelve las guardadas
        todas: Mutex<Vec<Deuda>>,
    }

    impl RepoDeudasMock {
        fn nuevo() -> Self {
            Self {
                cobrables: Mutex::new(Vec::new()),
                saldos: Mutex::new(Vec::new()),
                guardadas: Mutex::new(Vec::new()),
                periodos_rep: Mutex::new(Vec::new()),
                todas: Mutex::new(Vec::new()),
            }
        }

        fn con_cobrables(vec: Vec<Deuda>) -> Self {
            let repo = Self::nuevo();
            *repo.cobrables.lock().unwrap() = vec;
            repo
        }
    }

    impl DeudaRepository for RepoDeudasMock {
        fn save(&self, d: &Deuda) -> Result<(), ErrorRepositorio> {
            let mut d = d.clone();
            d.id = 100 + self.guardadas.lock().unwrap().len();
            self.guardadas.lock().unwrap().push(d.clone());
            self.todas.lock().unwrap().push(d);
            Ok(())
        }
        fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Deuda>, ErrorRepositorio> {
            Ok(self.todas.lock().unwrap().iter().filter(|d| d.periodo == periodo).cloned().collect())
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
        fn update_estado(&self, id: usize, monto_pendiente: f64, estado_id: i32) -> Result<(), ErrorRepositorio> {
            self.saldos.lock().unwrap().push((id, monto_pendiente, estado_id));
            Ok(())
        }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> {
            Ok(())
        }
    }

    struct RepoAplicacionesMock {
        aplicaciones: Mutex<Vec<AplicacionPago>>,
        borradas_por_pago: Mutex<Vec<usize>>,
    }

    impl RepoAplicacionesMock {
        fn nuevo() -> Self {
            Self { aplicaciones: Mutex::new(Vec::new()), borradas_por_pago: Mutex::new(Vec::new()) }
        }
    }

    impl AplicacionPagoRepository for RepoAplicacionesMock {
        fn save(&self, a: &AplicacionPago) -> Result<(), ErrorRepositorio> {
            self.aplicaciones.lock().unwrap().push(a.clone());
            Ok(())
        }
        fn fetch_por_pago(&self, pago_id: usize) -> Result<Vec<AplicacionPago>, ErrorRepositorio> {
            Ok(self.aplicaciones.lock().unwrap().iter().filter(|a| a.pago_id == pago_id).cloned().collect())
        }
        fn fetch_por_deuda(&self, _: usize) -> Result<Vec<AplicacionPago>, ErrorRepositorio> {
            Ok(Vec::new())
        }
        fn delete_por_pago(&self, pago_id: usize) -> Result<(), ErrorRepositorio> {
            self.borradas_por_pago.lock().unwrap().push(pago_id);
            Ok(())
        }
    }

    struct RepoAjustesMock {
        valores: Mutex<HashMap<String, String>>,
    }

    impl RepoAjustesMock {
        fn nuevo() -> Self {
            Self { valores: Mutex::new(HashMap::new()) }
        }
        fn con_mensualidad(monto: f64) -> Self {
            let repo = Self::nuevo();
            repo.valores.lock().unwrap().insert("monto_mensualidad".to_string(), monto.to_string());
            repo
        }
    }

    impl ConfiguracionAppRepository for RepoAjustesMock {
        fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorRepositorio> {
            Ok(self.valores.lock().unwrap().get(clave).cloned())
        }
        fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorRepositorio> {
            self.valores.lock().unwrap().insert(clave.to_string(), valor.to_string());
            Ok(())
        }
    }

    struct RepoHistorialMock;
    impl HistorialPagoRepository for RepoHistorialMock {
        fn save(&self, _: &HistorialPago) -> Result<(), ErrorRepositorio> { Ok(()) }
        fn fetch_por_representante(&self, _: usize) -> Result<Vec<HistorialPago>, ErrorRepositorio> { Ok(Vec::new()) }
        fn fetch_por_periodo(&self, _: &str) -> Result<Vec<HistorialPago>, ErrorRepositorio> { Ok(Vec::new()) }
        fn fetch_all(&self) -> Result<Vec<HistorialPago>, ErrorRepositorio> { Ok(Vec::new()) }
        fn delete(&self, _: HashSet<usize>) -> Result<(), ErrorRepositorio> { Ok(()) }
    }

    // ── Helpers ──

    struct Mocks {
        pagos: Arc<RepoPagosMock>,
        deudas: Arc<RepoDeudasMock>,
        aplicaciones: Arc<RepoAplicacionesMock>,
    }

    fn construir(
        repo_deudas: RepoDeudasMock,
        repo_ajustes: RepoAjustesMock,
    ) -> (ServicioPagos, Mocks) {
        let pagos = Arc::new(RepoPagosMock::nuevo());
        let deudas = Arc::new(repo_deudas);
        let aplicaciones = Arc::new(RepoAplicacionesMock::nuevo());
        let servicio = ServicioPagos::nuevo(
            pagos.clone(),
            aplicaciones.clone(),
            deudas.clone(),
            Arc::new(RepoHistorialMock),
            Arc::new(repo_ajustes),
            Arc::new(LoggerMock),
        );
        (servicio, Mocks { pagos, deudas, aplicaciones })
    }

    fn deuda(id: usize, periodo: &str, monto_total: f64, monto_pendiente: f64, estado: EstadoDeuda) -> Deuda {
        Deuda {
            id,
            representante_id: 1,
            monto_total,
            monto_pendiente,
            periodo: periodo.to_string(),
            fecha_vencimiento: format!("{periodo}-10"),
            estado_id: estado.id(),
            alumno_id: None,
        }
    }

    fn datos_pago(monto: f64) -> DatosPago {
        DatosPago {
            representante_id: 1,
            monto_recibido: monto,
            metodo_id: MetodoPago::Efectivo.id(),
            fecha_pago: "2026-08-24".to_string(),
        }
    }

    // ── Tests ──

    #[test]
    fn rechaza_pagos_invalidos_sin_crear_pago() {
        let (s, m) = construir(RepoDeudasMock::nuevo(), RepoAjustesMock::con_mensualidad(1500.0));

        // Sin representante
        let mut p = datos_pago(1500.0);
        p.representante_id = 0;
        assert!(matches!(s.registrar_pago(p), Err(ErrorAplicacion::Validacion(_))));

        // Monto negativo
        let p = datos_pago(-1.0);
        assert!(matches!(s.registrar_pago(p), Err(ErrorAplicacion::Validacion(_))));

        assert!(m.pagos.pagos.lock().unwrap().is_empty());
    }

    #[test]
    fn pago_total_a_una_deuda_la_deja_pagada() {
        // 1 deuda pendiente de 1500; se paga exacto → Pagada y saldo 0
        let repo = RepoDeudasMock::con_cobrables(vec![deuda(1, "2026-08", 1500.0, 1500.0, EstadoDeuda::Pendiente)]);
        let (s, m) = construir(repo, RepoAjustesMock::con_mensualidad(1500.0));

        let res = s.registrar_pago(datos_pago(1500.0)).unwrap();

        assert_eq!(res, vec![(1, 1500.0)]);
        // Se actualizó la deuda 1 a Pagada (estado 3) con saldo 0
        let saldos = m.deudas.saldos.lock().unwrap();
        assert_eq!(saldos[0].0, 1);
        assert!(saldos[0].1.abs() < f64::EPSILON);
        assert_eq!(saldos[0].2, EstadoDeuda::Pagada.id());
        // Se registró exactamente una aplicación
        assert_eq!(m.aplicaciones.aplicaciones.lock().unwrap().len(), 1);
    }

    #[test]
    fn pago_parcial_deja_la_deuda_parcial() {
        // 1 deuda de 1500; se pagan 500 → Parcial, saldo 1000
        let repo = RepoDeudasMock::con_cobrables(vec![deuda(1, "2026-08", 1500.0, 1500.0, EstadoDeuda::Pendiente)]);
        let (s, m) = construir(repo, RepoAjustesMock::con_mensualidad(1500.0));

        let res = s.registrar_pago(datos_pago(500.0)).unwrap();

        assert_eq!(res, vec![(1, 500.0)]);
        let saldos = m.deudas.saldos.lock().unwrap();
        assert_eq!(saldos[0].0, 1);
        assert!((saldos[0].1 - 1000.0).abs() < f64::EPSILON);
        assert_eq!(saldos[0].2, EstadoDeuda::Parcial.id());
    }

    #[test]
    fn fifo_aplica_primero_a_la_deuda_mas_antigua() {
        // 2 deudas pendientes: 2026-05 (1000) y 2026-06 (1000).
        // Se pagan 1500 → cubre 05 completa y 500 de 06.
        let repo = RepoDeudasMock::con_cobrables(vec![
            deuda(1, "2026-05", 1000.0, 1000.0, EstadoDeuda::Pendiente),
            deuda(2, "2026-06", 1000.0, 1000.0, EstadoDeuda::Pendiente),
        ]);
        let (s, m) = construir(repo, RepoAjustesMock::con_mensualidad(1500.0));

        let res = s.registrar_pago(datos_pago(1500.0)).unwrap();

        assert_eq!(res, vec![(1, 1000.0), (2, 500.0)]);
        let saldos = m.deudas.saldos.lock().unwrap();
        assert!((saldos[0].1).abs() < f64::EPSILON);            // deuda 1 → 0
        assert_eq!(saldos[0].2, EstadoDeuda::Pagada.id());
        assert!((saldos[1].1 - 500.0).abs() < f64::EPSILON);     // deuda 2 → 500
        assert_eq!(saldos[1].2, EstadoDeuda::Parcial.id());
    }

    #[test]
    fn pago_exacto_no_genera_adelanto() {
        // Deuda 2026-07 de 1000, se paga exacto 1000 → sin sobrante → sin adelanto
        let repo = RepoDeudasMock::con_cobrables(vec![deuda(1, "2026-07", 1000.0, 1000.0, EstadoDeuda::Pendiente)]);
        let (s, m) = construir(repo, RepoAjustesMock::con_mensualidad(1500.0));

        let res = s.registrar_pago(datos_pago(1000.0)).unwrap();

        assert_eq!(res, vec![(1, 1000.0)]);
        assert!(m.deudas.guardadas.lock().unwrap().is_empty());
    }

    #[test]
    fn excedente_genera_adelanto_para_el_siguiente_mes() {
        // Deuda 2026-07 de 1000, se pagan 1000 exacto → sin sobrante, sin adelanto.
        // Este test aísla la Fase 2 cuando NO sobra.
        let repo = RepoDeudasMock::con_cobrables(vec![deuda(1, "2026-07", 1000.0, 1000.0, EstadoDeuda::Pendiente)]);
        let (s, m) = construir(repo, RepoAjustesMock::con_mensualidad(1500.0));

        s.registrar_pago(datos_pago(1000.0)).unwrap();
        assert!(m.deudas.guardadas.lock().unwrap().is_empty());

        // Ahora: deuda de 1000 pero se pagan 2500 → sobra 1500 → adelanto 1 mes
        let repo = RepoDeudasMock::con_cobrables(vec![deuda(1, "2026-07", 1000.0, 1000.0, EstadoDeuda::Pendiente)]);
        let (s, m) = construir(repo, RepoAjustesMock::con_mensualidad(1500.0));

        let res = s.registrar_pago(datos_pago(2500.0)).unwrap();

        // 1000 a la deuda vieja + adelanto de 1500 para el mes siguiente
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], (1, 1000.0));
        let adelanto = res[1];
        assert_eq!(adelanto.1, 1500.0);
        // Se creó una deuda nueva (adelanto)
        assert_eq!(m.deudas.guardadas.lock().unwrap().len(), 1);
    }

    #[test]
    fn sin_deudas_sin_configuracion_de_mensualidad_falla_al_faltar_mensualidad() {
        // Sin deudas cobrables pero con dinero sobrante, el motor intenta
        // crear un adelanto y necesita la mensualidad configurada.
        let (s, _) = construir(RepoDeudasMock::nuevo(), RepoAjustesMock::nuevo());

        let res = s.registrar_pago(datos_pago(1500.0));
        assert!(matches!(res, Err(ErrorAplicacion::Validacion(msg)) if msg.contains("monto de mensualidad")));
    }

    #[test]
    fn sin_deudas_con_mensualidad_crea_adelanto() {
        // Sin deudas viejas pero con mensualidad configurada, todo el monto
        // se convierte en un adelanto para el siguiente mes.
        let (s, m) = construir(RepoDeudasMock::nuevo(), RepoAjustesMock::con_mensualidad(1500.0));

        let res = s.registrar_pago(datos_pago(1500.0)).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1, 1500.0);
        // Se creó una deuda anticipada como adelanto
        assert_eq!(m.deudas.guardadas.lock().unwrap().len(), 1);
    }

    #[test]
    fn reversar_restaura_el_saldo_de_la_deuda() {
        // Deuda 1 de 1500 pagada con 1500 → saldo 0/pagada.
        // Al reversar, se restaura a 1500/pendiente.
        let repo = RepoDeudasMock::con_cobrables(vec![deuda(1, "2026-08", 1500.0, 1500.0, EstadoDeuda::Pendiente)]);
        let (s, m) = construir(repo, RepoAjustesMock::con_mensualidad(1500.0));

        s.registrar_pago(datos_pago(1500.0)).unwrap();

        // Guardar la deuda en "todas" y resetear saldos para medir la reversa
        m.deudas.saldos.lock().unwrap().clear();
        // Simular estado post-pago
        *m.deudas.todas.lock().unwrap() = vec![deuda(1, "2026-08", 1500.0, 0.0, EstadoDeuda::Pagada)];
        let mut deuda_guardada = deuda(1, "2026-08", 1500.0, 0.0, EstadoDeuda::Pagada);
        deuda_guardada.id = 1;
        *m.deudas.guardadas.lock().unwrap() = vec![deuda_guardada];

        // El pago #1 tiene una aplicación. Reversar lo restaura.
        s.reversar_pago(1).unwrap();

        // Se llamó update_estado para restaurar: saldo 1500, estado Pendiente
        let saldos = m.deudas.saldos.lock().unwrap();
        assert_eq!(saldos[0].0, 1);
        assert!((saldos[0].1 - 1500.0).abs() < f64::EPSILON);
        assert_eq!(saldos[0].2, EstadoDeuda::Pendiente.id());
    }
}
