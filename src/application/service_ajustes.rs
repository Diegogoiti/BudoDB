//! Casos de uso del panel de Ajustes: configuraciones de la aplicación
//! gestionables desde la UI y persistidas vía su puerto clave-valor.
//!
//! Las CLAVES viven aquí (capa de aplicación): la infraestructura solo
//! conoce pares genéricos y la UI jamás arma claves a mano.

use super::error::ErrorAplicacion;
use super::ports::{ConfiguracionAppRepository, Logger};
use super::validation::monto_valido;
use std::sync::Arc;

/// Clave del monto sugerido de mensualidad (prellena el formulario de pagos).
pub const CLAVE_MONTO_MENSUALIDAD: &str = "monto_mensualidad";

pub struct ServicioAjustes {
    repositorio: Arc<dyn ConfiguracionAppRepository>,
    logger: Arc<dyn Logger>,
}

impl ServicioAjustes {
    pub fn nuevo(repositorio: Arc<dyn ConfiguracionAppRepository>, logger: Arc<dyn Logger>) -> Self {
        Self {
            repositorio,
            logger,
        }
    }

    /// Monto predeterminado de mensualidad. `None` = nunca se configuró
    /// (o el valor almacenado está corrupto, lo cual se registra y se ignora).
    pub fn monto_mensualidad(&self) -> Result<Option<f64>, ErrorAplicacion> {
        let Some(texto) = self.repositorio.obtener(CLAVE_MONTO_MENSUALIDAD)? else {
            return Ok(None);
        };
        match texto.trim().replace(',', ".").parse::<f64>() {
            Ok(monto) if monto_valido(monto) => Ok(Some(monto)),
            _ => {
                self.logger.error(&format!(
                    "El ajuste '{CLAVE_MONTO_MENSUALIDAD}' tiene un valor inválido ('{texto}'); se ignora"
                ));
                Ok(None)
            }
        }
    }

    /// Fija el monto predeterminado validándolo antes de persistir.
    pub fn fijar_monto_mensualidad(&self, monto: f64) -> Result<(), ErrorAplicacion> {
        if !monto_valido(monto) {
            return Err(ErrorAplicacion::Validacion(
                "El monto debe ser un número positivo.".to_string(),
            ));
        }
        self.repositorio
            .guardar(CLAVE_MONTO_MENSUALIDAD, &format!("{monto}"))?;
        self.logger.debug("Monto de mensualidad actualizado");
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::application::ports::ErrorRepositorio;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct RepoAjustesMock {
        valores: Mutex<HashMap<String, String>>,
        fallo: bool,
    }

    impl RepoAjustesMock {
        fn nuevo() -> Self {
            Self {
                valores: Mutex::new(HashMap::new()),
                fallo: false,
            }
        }

        fn con_valor(clave: &str, valor: &str) -> Self {
            let repo = Self::nuevo();
            repo.valores
                .lock()
                .unwrap()
                .insert(clave.to_string(), valor.to_string());
            repo
        }
    }

    impl ConfiguracionAppRepository for RepoAjustesMock {
        fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorRepositorio> {
            if self.fallo {
                return Err(ErrorRepositorio::Consulta("fallo simulado".to_string()));
            }
            Ok(self.valores.lock().unwrap().get(clave).cloned())
        }

        fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorRepositorio> {
            if self.fallo {
                return Err(ErrorRepositorio::Consulta("fallo simulado".to_string()));
            }
            self.valores
                .lock()
                .unwrap()
                .insert(clave.to_string(), valor.to_string());
            Ok(())
        }
    }

    struct LoggerMock;
    impl crate::application::ports::Logger for LoggerMock {
        fn debug(&self, _: &str) {}
        fn info(&self, _: &str) {}
        fn error(&self, _: &str) {}
    }

    fn servicio(repo: RepoAjustesMock) -> (ServicioAjustes, Arc<RepoAjustesMock>) {
        let repo = Arc::new(repo);
        (
            ServicioAjustes::nuevo(repo.clone(), Arc::new(LoggerMock)),
            repo,
        )
    }

    #[test]
    fn sin_configurar_devuelve_none() {
        let (s, _) = servicio(RepoAjustesMock::nuevo());
        assert_eq!(s.monto_mensualidad().unwrap(), None);
    }

    #[test]
    fn guarda_y_recupera_el_monto() {
        let (s, repo) = servicio(RepoAjustesMock::nuevo());

        s.fijar_monto_mensualidad(1500.5).expect("debería fijar");

        assert_eq!(s.monto_mensualidad().unwrap(), Some(1500.5));
        // Se persistió en la clave canónica, no en una inventada por nadie más.
        assert_eq!(
            repo.valores.lock().unwrap().get(CLAVE_MONTO_MENSUALIDAD),
            Some(&"1500.5".to_string())
        );
    }

    #[test]
    fn rechaza_montos_invalidos_sin_persistir() {
        let (s, repo) = servicio(RepoAjustesMock::nuevo());

        assert!(matches!(
            s.fijar_monto_mensualidad(0.0),
            Err(ErrorAplicacion::Validacion(_))
        ));
        assert!(s.fijar_monto_mensualidad(-10.0).is_err());
        assert!(repo.valores.lock().unwrap().is_empty());
    }

    #[test]
    fn un_valor_almacenado_corrupto_se_reporta_como_no_configurado() {
        let (s, _) = servicio(RepoAjustesMock::con_valor(
            CLAVE_MONTO_MENSUALIDAD,
            "mucho dinero",
        ));
        assert_eq!(s.monto_mensualidad().unwrap(), None);
    }
}
