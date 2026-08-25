//! Adaptador de los puertos [`AlumnoRepository`], [`RepresentanteRepository`]
//! y [`PagoRepository`] sobre SQLite (rusqlite). Es la única parte del código
//! que conoce SQL y rusqlite (regla 1).
//!
//! Los tres repositorios comparten UNA conexión porque las entidades se
//! relacionan por claves (alumnos.representante_id, pagos.representante_id):
//! así las migraciones de esquema corren en un solo lugar y en orden.

use crate::application::ports::{
    AbonoRepository, AlumnoRepository, ConfiguracionAppRepository, DeudaRepository,
    ErrorRepositorio, HistorialPagoRepository, Logger, PagoRepository, RepresentanteRepository,
};
use crate::domain::{Abono, Alumno, Deuda, HistorialPago, Pago, Representante};
use rusqlite::{params, params_from_iter, OptionalExtension, ToSql};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Repositorio concreto respaldado por un archivo SQLite.
///
/// La conexión se guarda en un `Mutex` porque `rusqlite::Connection` no es
/// `Sync`; la interfaz es de un solo flujo, así que el candado nunca contiende.
pub struct SqliteRepositorio {
    connection: Mutex<rusqlite::Connection>,
    logger: Arc<dyn Logger>,
}

impl SqliteRepositorio {
    /// Abre (o crea) la base de datos en `ruta`, aplica las migraciones de
    /// esquema pendientes e inicializa las tablas.
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
        // Integridad referencial: sin esto SQLite ignora los FOREIGN KEY.
        connection.pragma_update(None, "foreign_keys", "ON")?;
        logger.info(&format!("Base de datos abierta en '{ruta}'"));

        let repositorio = Self {
            connection: Mutex::new(connection),
            logger,
        };
        repositorio.inicializar_esquema()?;
        Ok(repositorio)
    }

    /// Esquema final + cadena de migraciones idempotentes para bases viejas.
    /// Orden garantizado: tablas base -> columna eliminado -> relación con
    /// representantes.
    fn inicializar_esquema(&self) -> rusqlite::Result<()> {
        let connection = self.lock();

        // Forma FINAL del esquema (bases nuevas nacen así). En bases viejas
        // el IF NOT EXISTS no toca nada y las migraciones de abajo completan.
        connection.execute(
            "CREATE TABLE IF NOT EXISTS alumnos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            fecha_de_nacimiento TEXT NOT NULL,
            rango INTEGER NOT NULL,
            representante TEXT NOT NULL DEFAULT '',
            telefono_representante TEXT NOT NULL DEFAULT '',
            rallita BOOLEAN NOT NULL DEFAULT 0,
            eliminado BOOLEAN NOT NULL DEFAULT 0
        )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS representantes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            numero_contacto TEXT NOT NULL,
            eliminado BOOLEAN NOT NULL DEFAULT 0
        )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS pagos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            representante_id INTEGER NOT NULL,
            monto REAL NOT NULL,
            periodo TEXT NOT NULL,
            fecha TEXT NOT NULL,
            observacion TEXT NOT NULL DEFAULT '',
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (representante_id) REFERENCES representantes(id)
        )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS ajustes (
            clave TEXT PRIMARY KEY,
            valor TEXT NOT NULL
        )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS deudas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            representante_id INTEGER NOT NULL,
            monto REAL NOT NULL,
            periodo TEXT NOT NULL,
            fecha TEXT NOT NULL,
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (representante_id) REFERENCES representantes(id)
        )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS abonos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            deuda_id INTEGER NOT NULL,
            monto REAL NOT NULL,
            fecha TEXT NOT NULL,
            observacion TEXT NOT NULL DEFAULT '',
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (deuda_id) REFERENCES deudas(id)
        )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS historial_pagos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            representante_id INTEGER NOT NULL,
            tipo TEXT NOT NULL,
            monto REAL NOT NULL DEFAULT 0,
            periodo TEXT NOT NULL,
            fecha TEXT NOT NULL,
            observacion TEXT NOT NULL DEFAULT '',
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (representante_id) REFERENCES representantes(id)
        )",
            [],
        )?;

        drop(connection);
        self.migrar_columna_eliminado()?;
        self.migrar_representantes()?;
        Ok(())
    }

    /// Migración idempotente: agrega la columna `eliminado` a bases creadas
    /// antes del borrado lógico. Las filas existentes quedan activas.
    fn migrar_columna_eliminado(&self) -> rusqlite::Result<()> {
        let connection = self.lock();
        let ya_existe = Self::columna_existe(&connection, "alumnos", "eliminado")?;

        if !ya_existe {
            connection.execute(
                "ALTER TABLE alumnos ADD COLUMN eliminado BOOLEAN NOT NULL DEFAULT 0",
                [],
            )?;
            self.logger.info("Migración aplicada: columna 'eliminado' agregada");
        }
        Ok(())
    }

    /// Migración v1 -> v2: extrae los representantes que vivían como texto
    /// plano dentro de `alumnos` hacia su propia tabla, relacionada por ID.
    ///
    /// Pasos (todos idempotentes, guardados por la existencia de la columna
    /// legada `representante`):
    ///   1. Copia cada par (nombre, contacto) distinto a `representantes`.
    ///   2. Agrega la columna `representante_id`.
    ///   3. Rellena el ID buscando al representante por nombre+contacto.
    ///   4. Elimina las columnas de texto legadas.
    fn migrar_representantes(&self) -> rusqlite::Result<()> {
        let connection = self.lock();
        let es_legada = Self::columna_existe(&connection, "alumnos", "representante")?;
        if !es_legada {
            return Ok(());
        }

        // 1. Extraer los representantes distintos conservando su contacto.
        // Los IDs autogenerados quedan estables para el relleno del paso 3.
        connection.execute(
            "INSERT INTO representantes (nombre, numero_contacto)
             SELECT DISTINCT representante, numero_contacto FROM alumnos
             WHERE representante <> ''",
            [],
        )?;

        // 2. Columna de relación (nullable: históricos sin representante quedan NULL).
        connection.execute(
            "ALTER TABLE alumnos ADD COLUMN representante_id INTEGER REFERENCES representantes(id)",
            [],
        )?;

        // 3. Rellenar por coincidencia exacta de nombre Y contacto (la clave
        // natural que existía antes de normalizar).
        connection.execute(
            "UPDATE alumnos SET representante_id = (
                SELECT r.id FROM representantes r
                WHERE r.nombre = alumnos.representante
                  AND r.numero_contacto = alumnos.numero_contacto
             )",
            [],
        )?;

        // 4. Las columnas de texto ya no aportan nada: fuera.
        connection.execute("ALTER TABLE alumnos DROP COLUMN representante", [])?;
        connection.execute("ALTER TABLE alumnos DROP COLUMN numero_contacto", [])?;

        self.logger
            .info("Migración aplicada: representantes normalizados con relación por ID");
        Ok(())
    }

    /// Consulta idempotente de esquema: ¿existe esta columna en esta tabla?
    /// Función libre asociada para reutilizarla desde cualquier migración.
    fn columna_existe(
        connection: &rusqlite::Connection,
        tabla: &str,
        columna: &str,
    ) -> rusqlite::Result<bool> {
        connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
                params![tabla, columna],
                |row| row.get::<_, i64>(0),
            )
            .map(|n| n > 0)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.connection
            .lock()
            .expect("la conexión SQLite quedó bloqueada por un pánico previo")
    }

    /// Mapeo explícito fila de BD -> entidad de negocio (regla 5).
    /// El DTO de persistencia nunca cruza hacia las capas superiores.
    /// COALESCE traduce el NULL histórico a "sin representante" (ID 0).
    fn mapear_alumno(row: &rusqlite::Row) -> rusqlite::Result<Alumno> {
        Ok(Alumno {
            id: row.get(0)?,
            nombre: row.get::<_, String>(1)?,
            fecha_de_nacimiento: row.get::<_, String>(2)?,
            rango: row.get(3)?,
            representante_id: row.get(4)?,
            rallita: row.get::<_, bool>(5)?,
        })
    }

    /// Borrado lógico genérico por tabla: marca `eliminado = 1`.
    /// Un solo punto para los tres repositorios (DRY dentro de infra).
    fn borrar_logicamente(
        &self,
        tabla: &str,
        ids: HashSet<usize>,
        descripcion: &str,
    ) -> Result<(), ErrorRepositorio> {
        // 1. Generamos los comodines (?, ?, ...) según la cantidad de IDs.
        let comodines: String = std::iter::repeat("?")
            .take(ids.len())
            .collect::<Vec<_>>()
            .join(", ");

        // Nunca toca filas ya eliminadas (operaciones repetibles).
        let query = format!(
            "UPDATE {tabla} SET eliminado = 1 WHERE id IN ({comodines}) AND eliminado = 0;"
        );

        // 2. Juntamos las referencias de los IDs en el vector de parámetros.
        let mut parametros: Vec<&dyn ToSql> = Vec::with_capacity(ids.len());
        for id in &ids {
            parametros.push(id);
        }

        let connection = self.lock();
        connection
            .execute(&query, params_from_iter(parametros))
            .map_err(error_consulta)?;
        self.logger
            .debug(&format!("{} {descripcion} marcados como eliminados", ids.len()));
        Ok(())
    }
}

fn error_consulta(e: rusqlite::Error) -> ErrorRepositorio {
    ErrorRepositorio::Consulta(e.to_string())
}

impl AlumnoRepository for SqliteRepositorio {
    fn save(&self, alumno: &Alumno) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante_id, rallita) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    alumno.nombre,
                    alumno.fecha_de_nacimiento,
                    alumno.rango,
                    alumno.representante_id,
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
                // El borrado lógico oculta las filas marcadas de TODAS las vistas.
                "SELECT id, nombre, fecha_de_nacimiento, rango, representante_id, rallita FROM alumnos WHERE eliminado = 0",
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
                "UPDATE alumnos SET nombre = ?1, fecha_de_nacimiento = ?2, rango = ?3, representante_id = ?4, rallita = ?5 WHERE id = ?6",
                params![
                    alumno.nombre,
                    alumno.fecha_de_nacimiento,
                    alumno.rango,
                    alumno.representante_id,
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

        // 2. Armamos la query dinámica sin números en los '?' para que vayan en orden.
        // Nunca modifica filas ya eliminadas lógicamente.
        let query = format!(
            "UPDATE alumnos SET rango = ?, rallita = ? WHERE id IN ({}) AND eliminado = 0;",
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
        // BORRADO LÓGICO: se marca la fila para que deje de aparecer en las
        // consultas pero conserve su historial.
        self.borrar_logicamente("alumnos", ids, "alumnos")
    }
}

impl RepresentanteRepository for SqliteRepositorio {
    fn save(&self, representante: &Representante) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "INSERT INTO representantes (nombre, numero_contacto) VALUES (?1, ?2)",
                params![representante.nombre, representante.numero_contacto],
            )
            .map_err(error_consulta)?;
        self.logger.debug("Representante guardado");
        Ok(())
    }

    fn fetch_all(&self) -> Result<Vec<Representante>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT id, nombre, numero_contacto FROM representantes WHERE eliminado = 0 ORDER BY nombre",
            )
            .map_err(error_consulta)?;

        let iter = stmt
            .query_map([], |row| {
                Ok(Representante {
                    id: row.get(0)?,
                    nombre: row.get(1)?,
                    numero_contacto: row.get(2)?,
                })
            })
            .map_err(error_consulta)?;

        let mut lista = Vec::new();
        for fila in iter {
            lista.push(fila.map_err(error_consulta)?);
        }
        Ok(lista)
    }

    fn update(&self, representante: &Representante) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "UPDATE representantes SET nombre = ?1, numero_contacto = ?2 WHERE id = ?3",
                params![representante.nombre, representante.numero_contacto, representante.id],
            )
            .map_err(error_consulta)?;
        self.logger
            .debug(&format!("Representante {} actualizado", representante.id));
        Ok(())
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("representantes", ids, "representantes")
    }
}

impl PagoRepository for SqliteRepositorio {
    fn save(&self, pago: &Pago) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "INSERT INTO pagos (representante_id, monto, periodo, fecha, observacion) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    pago.representante_id,
                    pago.monto,
                    pago.periodo,
                    pago.fecha,
                    pago.observacion
                ],
            )
            .map_err(error_consulta)?;
        self.logger.debug("Pago guardado");
        Ok(())
    }

    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Pago>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT id, representante_id, monto, periodo, fecha, observacion FROM pagos WHERE periodo = ?1 AND eliminado = 0",
            )
            .map_err(error_consulta)?;

        Self::mapear_pagos(stmt.query_map(params![periodo], Self::mapear_pago))
    }

    fn fetch_all(&self) -> Result<Vec<Pago>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT id, representante_id, monto, periodo, fecha, observacion FROM pagos WHERE eliminado = 0",
            )
            .map_err(error_consulta)?;

        Self::mapear_pagos(stmt.query_map([], Self::mapear_pago))
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("pagos", ids, "pagos")
    }
}

// Helpers de mapeo de pagos, deudas y abonos.

impl SqliteRepositorio {
    fn mapear_pago(row: &rusqlite::Row) -> rusqlite::Result<Pago> {
        Ok(Pago {
            id: row.get(0)?,
            representante_id: row.get(1)?,
            monto: row.get(2)?,
            periodo: row.get(3)?,
            fecha: row.get(4)?,
            observacion: row.get(5)?,
        })
    }

    fn mapear_pagos(
        iter: Result<impl Iterator<Item = rusqlite::Result<Pago>>, rusqlite::Error>,
    ) -> Result<Vec<Pago>, ErrorRepositorio> {
        let mut pagos = Vec::new();
        for fila in iter.map_err(error_consulta)? {
            pagos.push(fila.map_err(error_consulta)?);
        }
        Ok(pagos)
    }

    fn mapear_deuda(row: &rusqlite::Row) -> rusqlite::Result<Deuda> {
        Ok(Deuda {
            id: row.get(0)?,
            representante_id: row.get(1)?,
            monto: row.get(2)?,
            periodo: row.get(3)?,
            fecha: row.get(4)?,
        })
    }

    fn mapear_deudas(
        iter: Result<impl Iterator<Item = rusqlite::Result<Deuda>>, rusqlite::Error>,
    ) -> Result<Vec<Deuda>, ErrorRepositorio> {
        let mut deudas = Vec::new();
        for fila in iter.map_err(error_consulta)? {
            deudas.push(fila.map_err(error_consulta)?);
        }
        Ok(deudas)
    }

    fn mapear_abono(row: &rusqlite::Row) -> rusqlite::Result<Abono> {
        Ok(Abono {
            id: row.get(0)?,
            deuda_id: row.get(1)?,
            monto: row.get(2)?,
            fecha: row.get(3)?,
            observacion: row.get(4)?,
        })
    }

    fn mapear_abonos(
        iter: Result<impl Iterator<Item = rusqlite::Result<Abono>>, rusqlite::Error>,
    ) -> Result<Vec<Abono>, ErrorRepositorio> {
        let mut abonos = Vec::new();
        for fila in iter.map_err(error_consulta)? {
            abonos.push(fila.map_err(error_consulta)?);
        }
        Ok(abonos)
    }
}

impl ConfiguracionAppRepository for SqliteRepositorio {
    fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorRepositorio> {
        let connection = self.lock();
        let resultado = connection
            .query_row(
                "SELECT valor FROM ajustes WHERE clave = ?1",
                params![clave],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(error_consulta)?;
        Ok(resultado)
    }

    fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "INSERT INTO ajustes (clave, valor) VALUES (?1, ?2)
                 ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
                params![clave, valor],
            )
            .map_err(error_consulta)?;
        self.logger
            .debug(&format!("Ajuste '{clave}' guardado"));
        Ok(())
    }
}

impl DeudaRepository for SqliteRepositorio {
    fn save(&self, deuda: &Deuda) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "INSERT INTO deudas (representante_id, monto, periodo, fecha) VALUES (?1, ?2, ?3, ?4)",
                params![deuda.representante_id, deuda.monto, deuda.periodo, deuda.fecha],
            )
            .map_err(error_consulta)?;
        self.logger.debug("Deuda guardada");
        Ok(())
    }

    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Deuda>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT id, representante_id, monto, periodo, fecha FROM deudas WHERE periodo = ?1 AND eliminado = 0",
            )
            .map_err(error_consulta)?;

        Self::mapear_deudas(stmt.query_map(params![periodo], Self::mapear_deuda))
    }

    fn fetch_all(&self) -> Result<Vec<Deuda>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT id, representante_id, monto, periodo, fecha FROM deudas WHERE eliminado = 0",
            )
            .map_err(error_consulta)?;

        Self::mapear_deudas(stmt.query_map([], Self::mapear_deuda))
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("deudas", ids, "deudas")
    }
}

impl AbonoRepository for SqliteRepositorio {
    fn save(&self, abono: &Abono) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection
            .execute(
                "INSERT INTO abonos (deuda_id, monto, fecha, observacion) VALUES (?1, ?2, ?3, ?4)",
                params![abono.deuda_id, abono.monto, abono.fecha, abono.observacion],
            )
            .map_err(error_consulta)?;
        self.logger.debug("Abono guardado");
        Ok(())
    }

    fn fetch_por_deuda(&self, deuda_id: usize) -> Result<Vec<Abono>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT id, deuda_id, monto, fecha, observacion FROM abonos WHERE deuda_id = ?1 AND eliminado = 0",
            )
            .map_err(error_consulta)?;

        Self::mapear_abonos(stmt.query_map(params![deuda_id], Self::mapear_abono))
    }

    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Abono>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection
            .prepare(
                "SELECT a.id, a.deuda_id, a.monto, a.fecha, a.observacion
                 FROM abonos a
                 INNER JOIN deudas d ON a.deuda_id = d.id
                 WHERE d.periodo = ?1 AND a.eliminado = 0",
            )
            .map_err(error_consulta)?;

        Self::mapear_abonos(stmt.query_map(params![periodo], Self::mapear_abono))
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("abonos", ids, "abonos")
    }
}

// Helpers de mapeo adicionales
impl SqliteRepositorio {
    fn mapear_historial_registro(row: &rusqlite::Row) -> rusqlite::Result<HistorialPago> {
        Ok(HistorialPago {
            id: row.get(0)?,
            representante_id: row.get(1)?,
            tipo: row.get(2)?,
            monto: row.get(3)?,
            periodo: row.get(4)?,
            fecha: row.get(5)?,
            observacion: row.get(6)?,
        })
    }
    fn mapear_historial(iter: Result<impl Iterator<Item = rusqlite::Result<HistorialPago>>, rusqlite::Error>) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
        let mut registros = Vec::new();
        for fila in iter.map_err(error_consulta)? { registros.push(fila.map_err(error_consulta)?); }
        Ok(registros)
    }
}


impl HistorialPagoRepository for SqliteRepositorio {
    fn save(&self, registro: &HistorialPago) -> Result<(), ErrorRepositorio> {
        let connection = self.lock();
        connection.execute(
            "INSERT INTO historial_pagos (representante_id, tipo, monto, periodo, fecha, observacion) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![registro.representante_id, registro.tipo, registro.monto, registro.periodo, registro.fecha, registro.observacion],
        ).map_err(error_consulta)?;
        self.logger.debug("Historial de pago guardado");
        Ok(())
    }
    fn fetch_por_representante(&self, representante_id: usize) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection.prepare(
            "SELECT id, representante_id, tipo, monto, periodo, fecha, observacion FROM historial_pagos WHERE representante_id = ?1 AND eliminado = 0",
        ).map_err(error_consulta)?;
        Self::mapear_historial(stmt.query_map(params![representante_id], Self::mapear_historial_registro))
    }
    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection.prepare(
            "SELECT id, representante_id, tipo, monto, periodo, fecha, observacion FROM historial_pagos WHERE periodo = ?1 AND eliminado = 0",
        ).map_err(error_consulta)?;
        Self::mapear_historial(stmt.query_map(params![periodo], Self::mapear_historial_registro))
    }
    fn fetch_all(&self) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
        let connection = self.lock();
        let mut stmt = connection.prepare(
            "SELECT id, representante_id, tipo, monto, periodo, fecha, observacion FROM historial_pagos WHERE eliminado = 0",
        ).map_err(error_consulta)?;
        Self::mapear_historial(stmt.query_map([], Self::mapear_historial_registro))
    }
    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("historial_pagos", ids, "historial de pagos")
    }
}
