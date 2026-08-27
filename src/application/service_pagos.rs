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
        let aplicaciones = self.aplicar_fifo(
            pago_id,
            datos.representante_id,
            datos.monto_recibido,
            &datos.fecha_pago,
        )?;

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
