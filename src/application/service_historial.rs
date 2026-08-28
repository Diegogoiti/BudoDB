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
