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
