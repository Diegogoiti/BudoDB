//! Adaptador del puerto [`AlumnoRepository`] sobre SQLite (rusqlite).
//! Es la única parte del código que conoce SQL y rusqlite (regla 1).

use crate::application::ports::{AlumnoRepository, ErrorRepositorio, Logger};
use crate::models::Alumno; // TEMPORAL: migrará a `domain::alumno` en la fase 3.
use rusqlite::{params, params_from_iter, ToSql};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Repositorio concreto de alumnos respaldado por un archivo SQLite.
///
/// La conexión se guarda en un `Mutex` porque `rusqlite::Connection` no es
/// `Sync`; la interfaz es de un solo flujo, así que el candado nunca contiende.
pub struct SqliteAlumnoRepository {
    connection: Mutex<rusqlite::Connection>,
    logger: Arc<dyn Logger>,
}

impl SqliteAlumnoRepository {
    /// Abre (o crea) la base de datos en `ruta` e inicializa el esquema.
    pub fn abrir(ruta: &str, logger: Arc<dyn Logger>) -> rusqlite::Result<Self> {
        // Creamos el directorio contenedor si hace falta; los errores se
        // reportan por el logger sin abortar (la apertura fallará después
        // con un error claro si el directorio no pudo crearse).
        if let Some(padre) = Path::new(ruta).parent() {
            if !padre.as_os_str().is_empty() {
                if let Err(e) = std::fs::create_dir_all(padre) {
                    logger.error(&format!("Error creando carpeta {}: {e}", padre.display()));
                }
            }
        }

        let connection = rusqlite::Connection::open(ruta)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        logger.info(&format!("Base de datos abierta en '{ruta}'"));

        let repositorio = Self {
            connection: Mutex::new(connection),
            logger,
        };
        repositorio.inicializar_tablas()?;
        Ok(repositorio)
    }

    fn inicializar_tablas(&self) -> rusqlite::Result<()> {
        let connection = self.lock();
        connection.execute(
            "CREATE TABLE IF NOT EXISTS alumnos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            fecha_de_nacimiento TEXT NOT NULL,
            rango INTEGER NOT NULL,
            representante TEXT NOT NULL,
            numero_contacto TEXT NOT NULL,
            rallita BOOLEAN NOT NULL DEFAULT 0
        )",
            [],
        )?;
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.connection
            .lock()
            .expect("la conexión SQLite quedó bloqueada por un pánico previo")
    }

    /// Mapeo explícito fila de BD -> entidad de negocio (regla 5).
    /// El DTO de persistencia nunca cruza hacia las capas superiores.
    fn mapear_alumno(row: &rusqlite::Row) -> rusqlite::Result<Alumno> {
        Ok(Alumno {
            id: row.get(0)?,
            nombre: row.get::<_, String>(1)?,
            fecha_de_nacimiento: row.get::<_, String>(2)?,
            rango: row.get(3)?,
            representante: row.get::<_, String>(4)?,
            numero_contacto: row.get::<_, String>(5)?,
            rallita: row.get::<_, bool>(6)?,
        })
    }
}

fn error_consulta(e: rusqlite::Error) -> ErrorRepositorio {
    ErrorRepositorio::Consulta(e.to_string())
}

impl AlumnoRepository for SqliteAlumnoRepository {
    fn save(&self, alumno: &Alumno) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    alumno.nombre,
                    alumno.fecha_de_nacimiento,
                    alumno.rango,
                    alumno.representante,
                    alumno.numero_contacto,
                    alumno.rallita
                ],
            )
            .map_err(error_consulta)?;
        self.logger.debug("Alumno guardado");
        Ok(())
    }

    fn fetch_all(&self) -> Result<Vec<Alumno>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT id, nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita FROM alumnos",
            )
            .map_err(error_consulta)?;

        let alumno_iter = stmt
            .query_map([], Self::mapear_alumno)
            .map_err(error_consulta)?;

        let mut alumnos = Vec::new();
        for alumno in alumno_iter {
            alumnos.push(alumno.map_err(error_consulta)?);
        }
        Ok(alumnos)
    }

    fn update(&self, alumno: &Alumno) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "UPDATE alumnos SET nombre = ?1, fecha_de_nacimiento = ?2, rango = ?3, representante = ?4, numero_contacto = ?5, rallita = ?6 WHERE id = ?7",
                params![
                    alumno.nombre,
                    alumno.fecha_de_nacimiento,
                    alumno.rango,
                    alumno.representante,
                    alumno.numero_contacto,
                    alumno.rallita,
                    alumno.id
                ],
            )
            .map_err(error_consulta)?;
        self.logger.debug(&format!("Alumno {} actualizado", alumno.id));
        Ok(())
    }

    fn update_rangos(
        &self,
        ids: HashSet<usize>,
        rango: i32,
        rallita: bool,
    ) -> Result<(), ErrorRepositorio> {
        // 1. Generamos los comodines (?, ?, ...) según la cantidad de IDs
        let comodines: String = std::iter::repeat("?")
            .take(ids.len())
            .collect::<Vec<_>>()
            .join(", ");

        // 2. Armamos la query dinámica sin números en los '?' para que vayan en orden
        let query = format!(
            "UPDATE alumnos SET rango = ?, rallita = ? WHERE id IN ({});",
            comodines
        );

        // 3. Juntamos TODOS los parámetros en un solo vector de referencias en el orden exacto
        let mut parametros: Vec<&dyn ToSql> = vec![&rango, &rallita];

        for id in &ids {
            parametros.push(id);
        }

        let connection = self.lock();
        connection
            .execute(&query, params_from_iter(parametros))
            .map_err(error_consulta)?;
        self.logger
            .debug(&format!("Rangos actualizados para {} alumnos", ids.len()));
        Ok(())
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        // 1. Generamos los comodines (?, ?, ...) según la cantidad de IDs
        let comodines: String = std::iter::repeat("?")
            .take(ids.len())
            .collect::<Vec<_>>()
            .join(", ");

        // 2. Armamos la query dinámica de eliminación
        let query = format!("DELETE FROM alumnos WHERE id IN ({});", comodines);

        // 3. Juntamos las referencias de los IDs en el vector de parámetros
        let mut parametros: Vec<&dyn ToSql> = Vec::with_capacity(ids.len());

        for id in &ids {
            parametros.push(id);
        }

        let connection = self.lock();
        connection
            .execute(&query, params_from_iter(parametros))
            .map_err(error_consulta)?;
        self.logger
            .debug(&format!("{} alumnos eliminados", ids.len()));
        Ok(())
    }
}
