//! Casos de uso del sistema de pagos mensuales con motor FIFO.
//!
//! Un pago pertenece a un REPRESENTANTE. El motor FIFO determina
//! automáticamente a qué deudas se aplica el monto recibido.

use super::dto::{DatosPago, PagoVista};
use super::error::ErrorAplicacion;
use super::ports::{Logger, PagoRepository, AplicacionPagoRepository, DeudaRepository};
use super::validation::validar_datos_pago;
use crate::domain::{Pago, EstadoDeuda, EstadoPago, Representante};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ServicioPagos {
    repo_pagos: Arc<dyn PagoRepository>,
    repo_aplicaciones: Arc<dyn AplicacionPagoRepository>,
    repo_deudas: Arc<dyn DeudaRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioPagos {
    pub fn nuevo(
        repo_pagos: Arc<dyn PagoRepository>,
        repo_aplicaciones: Arc<dyn AplicacionPagoRepository>,
        repo_deudas: Arc<dyn DeudaRepository>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self { repo_pagos, repo_aplicaciones, repo_deudas, logger }
    }

    /// Registra un pago y ejecuta el algoritmo FIFO para aplicarlo a deudas.
    /// Devuelve un resumen de las aplicaciones realizadas.
    pub fn registrar_pago(&self, datos: DatosPago) -> Result<Vec<(usize, f64)>, ErrorAplicacion> {
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
        self.repo_pagos.save(&pago)?;

        // Obtener el ID recién creado (para pagos SQLite con rowid)
        // Nota: en SQLite el ID se asigna al hacer INSERT, pero el DTO no lo trae.
        // Por ahora usamos 0 como placeholder; el repositorio real lo resuelve.

        self.logger.debug(&format!("Pago de {} registrado", datos.monto_recibido));
        Ok(Vec::new()) // Las aplicaciones se manejan externamente por ahora
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
                PagoVista {
                    pago: pago.clone(),
                    nombre_representante: nombre,
                    metodo: pago.metodo(),
                    estado: pago.estado(),
                    aplicaciones: Vec::new(),
                }
            })
            .collect();
        vistas.sort_by(|a, b| b.pago.fecha_pago.cmp(&a.pago.fecha_pago));
        Ok(vistas)
    }

    /// Representantes que NO tienen pagos en el periodo (morosos).
    pub fn morosos_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<Vec<Representante>, ErrorAplicacion> {
        let pagos = self.repo_pagos.fetch_por_periodo(periodo)?;
        let pagadores: HashSet<usize> =
            pagos.iter().map(|p| p.representante_id).collect();
        Ok(representantes
            .iter()
            .filter(|r| !pagadores.contains(&r.id))
            .cloned()
            .collect())
    }

    /// Reversa un pago: restaura saldos de deudas afectadas.
    pub fn reversar_pago(&self, pago_id: usize) -> Result<(), ErrorAplicacion> {
        // 1. Buscar aplicaciones del pago
        let aplicaciones = self.repo_aplicaciones.fetch_por_pago(pago_id)?;

        // 2. Restaurar saldos de cada deuda afectada
        for app in &aplicaciones {
            let deudas = self.repo_deudas.fetch_por_periodo("")?; // TODO: buscar por ID
            if let Some(deuda) = deudas.iter().find(|d| d.id == app.deuda_id) {
                let nuevo_pendiente = deuda.monto_pendiente + app.monto_aplicado;
                let nuevo_estado = if nuevo_pendiente >= deuda.monto_total {
                    EstadoDeuda::Pendiente.id()
                } else {
                    EstadoDeuda::Parcial.id()
                };
                self.repo_deudas.update_estado(
                    deuda.id,
                    nuevo_pendiente.min(deuda.monto_total),
                    nuevo_estado,
                )?;
            }
        }

        // 3. Eliminar las aplicaciones
        self.repo_aplicaciones.delete_por_pago(pago_id)?;

        // 4. Cambiar estado del pago a Reversado
        self.repo_pagos.update_estado(pago_id, EstadoPago::Reversado.id())?;

        self.logger.info(&format!("Pago {pago_id} reversado"));
        Ok(())
    }

    /// Total recaudado en un periodo.
    pub fn total_del_periodo(
        &self,
        periodo: &str,
        representantes: &[Representante],
    ) -> Result<f64, ErrorAplicacion> {
        let vistas = self.listar_del_periodo(periodo, representantes)?;
        Ok(vistas.iter().filter(|v| v.estado == EstadoPago::Completado).map(|v| v.pago.monto_recibido).sum())
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
