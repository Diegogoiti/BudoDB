use crate::application::ports::Configuracion;
use std::env;

/// Nombre de la variable de entorno que define la ruta del archivo SQLite.
pub const VAR_RUTA_BASE_DATOS: &str = "BUDODB_DB_PATH";

const RUTA_POR_DEFECTO: &str = "./database/database.db";

/// Adaptador de configuración que lee variables de entorno con valores por
/// defecto idénticos al comportamiento histórico de la app.
pub struct ConfigEntorno;

impl Configuracion for ConfigEntorno {
    fn ruta_base_de_datos(&self) -> String {
        env::var(VAR_RUTA_BASE_DATOS).unwrap_or_else(|_| RUTA_POR_DEFECTO.to_string())
    }
}
