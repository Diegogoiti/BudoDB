//! Adaptador SQLite para todos los repositorios del sistema.
//! Es la única parte del código que conoce SQL y rusqlite.
//!
//! Implementa: AlumnoRepository, RepresentanteRepository, PagoRepository,
//! DeudaRepository, AplicacionPagoRepository, HistorialPagoRepository,
//! AbonoRepository (legacy), ConfiguracionAppRepository.

use crate::application::ports::{
    AbonoRepository, AlumnoRepository, AplicacionPagoRepository, ConfiguracionAppRepository,
    DeudaRepository, ErrorRepositorio, HistorialPagoRepository, Logger, PagoRepository,
    RepresentanteRepository,
};
use crate::domain::{Abono, Alumno, AplicacionPago, Deuda, HistorialPago, Pago, Representante};
use rusqlite::{params, params_from_iter, OptionalExtension, ToSql};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct SqliteRepositorio {
    connection: Mutex<rusqlite::Connection>,
    logger: Arc<dyn Logger>,
}

impl SqliteRepositorio {
    pub fn abrir(ruta: &str, logger: Arc<dyn Logger>) -> rusqlite::Result<Self> {
        if let Some(padre) = Path::new(ruta).parent() {
            if !padre.as_os_str().is_empty() {
                if let Err(e) = std::fs::create_dir_all(padre) {
                    logger.error(&format!("Error creando carpeta {}: {e}", padre.display()));
                }
            }
        }
        let connection = rusqlite::Connection::open(ruta)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        logger.info(&format!("Base de datos abierta en '{ruta}'"));
        let repositorio = Self { connection: Mutex::new(connection), logger };
        repositorio.inicializar_esquema()?;
        Ok(repositorio)
    }

    fn inicializar_esquema(&self) -> rusqlite::Result<()> {
        let c = self.lock();

        // ─── Tablas catálogo (solo lectura en UI) ───
        c.execute("CREATE TABLE IF NOT EXISTS cat_estados_deuda (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)", [])?;
        c.execute("CREATE TABLE IF NOT EXISTS cat_estados_pago (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)", [])?;
        c.execute("CREATE TABLE IF NOT EXISTS cat_metodos_pago (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)", [])?;
        c.execute("CREATE TABLE IF NOT EXISTS cat_estados_alumno (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)", [])?;
        c.execute("CREATE TABLE IF NOT EXISTS cat_estados_representante (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)", [])?;
        c.execute("CREATE TABLE IF NOT EXISTS cat_tipos_historial (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)", [])?;
        c.execute("CREATE TABLE IF NOT EXISTS cat_tipos_evento (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)", [])?;

        // Poblar catálogos (idempotente)
        c.execute("INSERT OR IGNORE INTO cat_estados_deuda VALUES (1,'Pendiente'),(2,'Parcial'),(3,'Pagada'),(4,'Anticipada'),(5,'Anulada')", [])?;
        c.execute("INSERT OR IGNORE INTO cat_estados_pago VALUES (1,'Completado'),(2,'Reversado'),(3,'Pendiente')", [])?;
        c.execute("INSERT OR IGNORE INTO cat_metodos_pago VALUES (1,'Efectivo'),(2,'Transferencia'),(3,'Tarjeta'),(4,'Cheque')", [])?;
        c.execute("INSERT OR IGNORE INTO cat_estados_alumno VALUES (1,'Activo'),(2,'Inactivo'),(3,'Suspendido'),(4,'Retirado')", [])?;
        c.execute("INSERT OR IGNORE INTO cat_estados_representante VALUES (1,'Activo'),(2,'Inactivo')", [])?;
        c.execute("INSERT OR IGNORE INTO cat_tipos_historial VALUES (1,'Deuda Creada'),(2,'Pago Registrado'),(3,'Abono Aplicado'),(4,'Ajuste Manual'),(5,'Anulación')", [])?;
        c.execute("INSERT OR IGNORE INTO cat_tipos_evento VALUES (1,'Competencia'),(2,'Examen'),(3,'Graduación'),(4,'Clase Especial')", [])?;

        // ─── Tablas de negocio ───
        c.execute("CREATE TABLE IF NOT EXISTS representantes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            numero_contacto TEXT NOT NULL,
            estado_id INTEGER NOT NULL DEFAULT 1,
            eliminado BOOLEAN NOT NULL DEFAULT 0
        )", [])?;

        c.execute("CREATE TABLE IF NOT EXISTS alumnos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            fecha_de_nacimiento TEXT NOT NULL,
            rango INTEGER NOT NULL,
            representante_id INTEGER NOT NULL DEFAULT 0,
            rallita BOOLEAN NOT NULL DEFAULT 0,
            estado_id INTEGER NOT NULL DEFAULT 1,
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (representante_id) REFERENCES representantes(id)
        )", [])?;

        c.execute("CREATE TABLE IF NOT EXISTS deudas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            representante_id INTEGER NOT NULL,
            monto_total REAL NOT NULL,
            monto_pendiente REAL NOT NULL,
            periodo TEXT NOT NULL,
            fecha_vencimiento TEXT NOT NULL,
            estado_id INTEGER NOT NULL DEFAULT 1,
            alumno_id INTEGER,
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (representante_id) REFERENCES representantes(id)
        )", [])?;

        c.execute("CREATE TABLE IF NOT EXISTS pagos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            representante_id INTEGER NOT NULL,
            monto_recibido REAL NOT NULL,
            estado_id INTEGER NOT NULL DEFAULT 1,
            metodo_id INTEGER NOT NULL DEFAULT 1,
            fecha_pago TEXT NOT NULL,
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (representante_id) REFERENCES representantes(id)
        )", [])?;

        c.execute("CREATE TABLE IF NOT EXISTS aplicaciones_pago (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pago_id INTEGER NOT NULL,
            deuda_id INTEGER NOT NULL,
            monto_aplicado REAL NOT NULL,
            fecha TEXT NOT NULL,
            FOREIGN KEY (pago_id) REFERENCES pagos(id),
            FOREIGN KEY (deuda_id) REFERENCES deudas(id)
        )", [])?;

        c.execute("CREATE TABLE IF NOT EXISTS historial_pagos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            representante_id INTEGER NOT NULL,
            tipo_id INTEGER NOT NULL,
            monto REAL NOT NULL DEFAULT 0,
            periodo TEXT NOT NULL,
            fecha TEXT NOT NULL,
            observacion TEXT NOT NULL DEFAULT '',
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (representante_id) REFERENCES representantes(id)
        )", [])?;

        c.execute("CREATE TABLE IF NOT EXISTS ajustes (
            clave TEXT PRIMARY KEY,
            valor TEXT NOT NULL
        )", [])?;

        // Tablas legacy (mantener para compatibilidad)
        c.execute("CREATE TABLE IF NOT EXISTS abonos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            deuda_id INTEGER NOT NULL,
            monto REAL NOT NULL,
            fecha TEXT NOT NULL,
            observacion TEXT NOT NULL DEFAULT '',
            eliminado BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (deuda_id) REFERENCES deudas(id)
        )", [])?;

        // ─── Migraciones idempotentes para bases viejas (ANTES de índices) ───
        // Agregar estado_id si falta
        if !Self::columna_existe(&c, "alumnos", "estado_id")? {
            c.execute("ALTER TABLE alumnos ADD COLUMN estado_id INTEGER NOT NULL DEFAULT 1", [])?;
        }
        if !Self::columna_existe(&c, "representantes", "estado_id")? {
            c.execute("ALTER TABLE representantes ADD COLUMN estado_id INTEGER NOT NULL DEFAULT 1", [])?;
        }
        // Migrar alumnos viejos (columnas de texto -> FK)
        let es_legada = Self::columna_existe(&c, "alumnos", "representante")?;
        if es_legada {
            c.execute("INSERT OR IGNORE INTO representantes (nombre, numero_contacto) SELECT DISTINCT representante, telefono_representante FROM alumnos WHERE representante <> '' AND representante IS NOT NULL", [])?;
            if !Self::columna_existe(&c, "alumnos", "representante_id")? {
                c.execute("ALTER TABLE alumnos ADD COLUMN representante_id INTEGER REFERENCES representantes(id)", [])?;
            }
            c.execute("UPDATE alumnos SET representante_id = (SELECT r.id FROM representantes r WHERE r.nombre = alumnos.representante AND r.numero_contacto = alumnos.telefono_representante) WHERE representante_id IS NULL OR representante_id = 0", [])?;
            let _ = c.execute("ALTER TABLE alumnos DROP COLUMN representante", []);
            let _ = c.execute("ALTER TABLE alumnos DROP COLUMN telefono_representante", []);
        }
        // Migrar deudas viejas (monto -> monto_total/monto_pendiente)
        let deudas_legado = Self::columna_existe(&c, "deudas", "monto")?;
        if deudas_legado {
            c.execute("CREATE TABLE IF NOT EXISTS deudas_nuevo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                representante_id INTEGER NOT NULL,
                monto_total REAL NOT NULL,
                monto_pendiente REAL NOT NULL,
                periodo TEXT NOT NULL,
                fecha_vencimiento TEXT NOT NULL,
                estado_id INTEGER NOT NULL DEFAULT 1,
                alumno_id INTEGER,
                eliminado BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (representante_id) REFERENCES representantes(id)
            )", [])?;
            // Mapear estado: si monto == monto (sin abonos) -> Pendiente(1)
            c.execute("INSERT INTO deudas_nuevo (id, representante_id, monto_total, monto_pendiente, periodo, fecha_vencimiento, estado_id, eliminado) SELECT id, representante_id, monto, monto, periodo, fecha, 1, eliminado FROM deudas", [])?;
            c.execute("DROP TABLE deudas", [])?;
            c.execute("ALTER TABLE deudas_nuevo RENAME TO deudas", [])?;
        }
        // Migrar pagos viejos (monto -> monto_recibido)
        let pagos_legado = Self::columna_existe(&c, "pagos", "monto")?;
        if pagos_legado {
            c.execute("CREATE TABLE IF NOT EXISTS pagos_nuevo (id INTEGER PRIMARY KEY AUTOINCREMENT, representante_id INTEGER NOT NULL, monto_recibido REAL NOT NULL, estado_id INTEGER NOT NULL DEFAULT 1, metodo_id INTEGER NOT NULL DEFAULT 1, fecha_pago TEXT NOT NULL, eliminado BOOLEAN NOT NULL DEFAULT 0, FOREIGN KEY (representante_id) REFERENCES representantes(id))", [])?;
            c.execute("INSERT INTO pagos_nuevo (id, representante_id, monto_recibido, estado_id, metodo_id, fecha_pago, eliminado) SELECT id, representante_id, monto, 1, 1, fecha, eliminado FROM pagos", [])?;
            c.execute("DROP TABLE pagos", [])?;
            c.execute("ALTER TABLE pagos_nuevo RENAME TO pagos", [])?;
        }

        // ─── Índices de performance (DESPUÉS de migraciones) ───
        c.execute("CREATE INDEX IF NOT EXISTS idx_alumnos_rep_estado ON alumnos(representante_id, estado_id)", [])?;
        c.execute("CREATE INDEX IF NOT EXISTS idx_deudas_rep_estado_fecha ON deudas(representante_id, estado_id, fecha_vencimiento)", [])?;
        c.execute("CREATE INDEX IF NOT EXISTS idx_deudas_periodo ON deudas(periodo)", [])?;
        c.execute("CREATE INDEX IF NOT EXISTS idx_pagos_rep_fecha ON pagos(representante_id, fecha_pago)", [])?;
        c.execute("CREATE INDEX IF NOT EXISTS idx_aplicaciones_pago_id ON aplicaciones_pago(pago_id)", [])?;
        c.execute("CREATE INDEX IF NOT EXISTS idx_aplicaciones_deuda_id ON aplicaciones_pago(deuda_id)", [])?;
        c.execute("CREATE INDEX IF NOT EXISTS idx_historial_rep_fecha ON historial_pagos(representante_id, fecha)", [])?;

        drop(c);
        Ok(())
    }

    fn columna_existe(c: &rusqlite::Connection, tabla: &str, columna: &str) -> rusqlite::Result<bool> {
        c.query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
            params![tabla, columna],
            |row| row.get::<_, i64>(0),
        ).map(|n| n > 0)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.connection.lock().expect("conexión SQLite bloqueada")
    }

    fn borrar_logicamente(&self, tabla: &str, ids: HashSet<usize>, desc: &str) -> Result<(), ErrorRepositorio> {
        if ids.is_empty() { return Ok(()); }
        let comodines: String = std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(", ");
        let query = format!("UPDATE {tabla} SET eliminado = 1 WHERE id IN ({comodines}) AND eliminado = 0");
        let mut parametros: Vec<&dyn ToSql> = Vec::with_capacity(ids.len());
        for id in &ids { parametros.push(id); }
        self.lock().execute(&query, params_from_iter(parametros)).map_err(|e| ErrorRepositorio::Consulta(e.to_string()))?;
        self.logger.debug(&format!("{} {desc} marcados como eliminados", ids.len()));
        Ok(())
    }
}

fn error_consulta(e: rusqlite::Error) -> ErrorRepositorio {
    ErrorRepositorio::Consulta(e.to_string())
}

// ═══════════════════════════════════════════════════════════════
// AlumnoRepository
// ═══════════════════════════════════════════════════════════════

impl AlumnoRepository for SqliteRepositorio {
    fn save(&self, a: &Alumno) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante_id, rallita, estado_id) VALUES (?1,?2,?3,?4,?5,?6)",
            params![a.nombre, a.fecha_de_nacimiento, a.rango, a.representante_id, a.rallita, a.estado_id],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn fetch_all(&self) -> Result<Vec<Alumno>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, nombre, fecha_de_nacimiento, rango, representante_id, rallita, estado_id FROM alumnos WHERE eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map([], |row| {
            Ok(Alumno {
                id: row.get(0)?,
                nombre: row.get(1)?,
                fecha_de_nacimiento: row.get(2)?,
                rango: row.get(3)?,
                representante_id: row.get(4)?,
                rallita: row.get(5)?,
                estado_id: row.get(6)?,
            })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn update(&self, a: &Alumno) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "UPDATE alumnos SET nombre=?1, fecha_de_nacimiento=?2, rango=?3, representante_id=?4, rallita=?5, estado_id=?6 WHERE id=?7",
            params![a.nombre, a.fecha_de_nacimiento, a.rango, a.representante_id, a.rallita, a.estado_id, a.id],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn update_rangos(&self, ids: HashSet<usize>, rango: i32, rallita: bool) -> Result<(), ErrorRepositorio> {
        let comodines: String = std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(", ");
        let query = format!("UPDATE alumnos SET rango=?, rallita=? WHERE id IN ({comodines}) AND eliminado=0");
        let mut parametros: Vec<&dyn ToSql> = vec![&rango, &rallita];
        for id in &ids { parametros.push(id); }
        self.lock().execute(&query, params_from_iter(parametros)).map_err(error_consulta)?;
        Ok(())
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("alumnos", ids, "alumnos")
    }
}

// ═══════════════════════════════════════════════════════════════
// RepresentanteRepository
// ═══════════════════════════════════════════════════════════════

impl RepresentanteRepository for SqliteRepositorio {
    fn save(&self, r: &Representante) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO representantes (nombre, numero_contacto, estado_id) VALUES (?1,?2,?3)",
            params![r.nombre, r.numero_contacto, r.estado_id],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn fetch_all(&self) -> Result<Vec<Representante>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, nombre, numero_contacto, estado_id FROM representantes WHERE eliminado = 0 ORDER BY nombre"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map([], |row| {
            Ok(Representante { id: row.get(0)?, nombre: row.get(1)?, numero_contacto: row.get(2)?, estado_id: row.get(3)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn update(&self, r: &Representante) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "UPDATE representantes SET nombre=?1, numero_contacto=?2, estado_id=?3 WHERE id=?4",
            params![r.nombre, r.numero_contacto, r.estado_id, r.id],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("representantes", ids, "representantes")
    }
}

// ═══════════════════════════════════════════════════════════════
// PagoRepository
// ═══════════════════════════════════════════════════════════════

impl PagoRepository for SqliteRepositorio {
    fn save(&self, p: &Pago) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO pagos (representante_id, monto_recibido, estado_id, metodo_id, fecha_pago) VALUES (?1,?2,?3,?4,?5)",
            params![p.representante_id, p.monto_recibido, p.estado_id, p.metodo_id, p.fecha_pago],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn fetch_por_periodo(&self, _periodo: &str) -> Result<Vec<Pago>, ErrorRepositorio> {
        // Pagos legacy no tienen periodo; filtramos por todos los no eliminados
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, monto_recibido, estado_id, metodo_id, fecha_pago FROM pagos WHERE eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map([], |row| {
            Ok(Pago { id: row.get(0)?, representante_id: row.get(1)?, monto_recibido: row.get(2)?, estado_id: row.get(3)?, metodo_id: row.get(4)?, fecha_pago: row.get(5)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_por_representante(&self, representante_id: usize) -> Result<Vec<Pago>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, monto_recibido, estado_id, metodo_id, fecha_pago FROM pagos WHERE representante_id = ?1 AND eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![representante_id], |row| {
            Ok(Pago { id: row.get(0)?, representante_id: row.get(1)?, monto_recibido: row.get(2)?, estado_id: row.get(3)?, metodo_id: row.get(4)?, fecha_pago: row.get(5)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_all(&self) -> Result<Vec<Pago>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, monto_recibido, estado_id, metodo_id, fecha_pago FROM pagos WHERE eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map([], |row| {
            Ok(Pago { id: row.get(0)?, representante_id: row.get(1)?, monto_recibido: row.get(2)?, estado_id: row.get(3)?, metodo_id: row.get(4)?, fecha_pago: row.get(5)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn update_estado(&self, id: usize, estado_id: i32) -> Result<(), ErrorRepositorio> {
        self.lock().execute("UPDATE pagos SET estado_id = ?1 WHERE id = ?2", params![estado_id, id]).map_err(error_consulta)?;
        Ok(())
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("pagos", ids, "pagos")
    }
}

// ═══════════════════════════════════════════════════════════════
// DeudaRepository
// ═══════════════════════════════════════════════════════════════

impl DeudaRepository for SqliteRepositorio {
    fn save(&self, d: &Deuda) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO deudas (representante_id, monto_total, monto_pendiente, periodo, fecha_vencimiento, estado_id, alumno_id) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![d.representante_id, d.monto_total, d.monto_pendiente, d.periodo, d.fecha_vencimiento, d.estado_id, d.alumno_id],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<Deuda>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, monto_total, monto_pendiente, periodo, fecha_vencimiento, estado_id, alumno_id FROM deudas WHERE periodo = ?1 AND eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![periodo], |row| {
            Ok(Deuda {
                id: row.get(0)?, representante_id: row.get(1)?, monto_total: row.get(2)?,
                monto_pendiente: row.get(3)?, periodo: row.get(4)?, fecha_vencimiento: row.get(5)?,
                estado_id: row.get(6)?, alumno_id: row.get(7)?,
            })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_cobrables_por_representante(&self, representante_id: usize) -> Result<Vec<Deuda>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, monto_total, monto_pendiente, periodo, fecha_vencimiento, estado_id, alumno_id FROM deudas WHERE representante_id = ?1 AND estado_id IN (1,2) AND eliminado = 0 ORDER BY fecha_vencimiento ASC"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![representante_id], |row| {
            Ok(Deuda {
                id: row.get(0)?, representante_id: row.get(1)?, monto_total: row.get(2)?,
                monto_pendiente: row.get(3)?, periodo: row.get(4)?, fecha_vencimiento: row.get(5)?,
                estado_id: row.get(6)?, alumno_id: row.get(7)?,
            })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_todos_periodos_por_representante(&self, representante_id: usize) -> Result<Vec<String>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT DISTINCT periodo FROM deudas WHERE representante_id = ?1 AND eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![representante_id], |row| row.get(0)).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_all(&self) -> Result<Vec<Deuda>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, monto_total, monto_pendiente, periodo, fecha_vencimiento, estado_id, alumno_id FROM deudas WHERE eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map([], |row| {
            Ok(Deuda {
                id: row.get(0)?, representante_id: row.get(1)?, monto_total: row.get(2)?,
                monto_pendiente: row.get(3)?, periodo: row.get(4)?, fecha_vencimiento: row.get(5)?,
                estado_id: row.get(6)?, alumno_id: row.get(7)?,
            })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn update_estado(&self, id: usize, monto_pendiente: f64, estado_id: i32) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "UPDATE deudas SET monto_pendiente = ?1, estado_id = ?2 WHERE id = ?3",
            params![monto_pendiente, estado_id, id],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("deudas", ids, "deudas")
    }
}

// ═══════════════════════════════════════════════════════════════
// AplicacionPagoRepository
// ═══════════════════════════════════════════════════════════════

impl AplicacionPagoRepository for SqliteRepositorio {
    fn save(&self, a: &AplicacionPago) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO aplicaciones_pago (pago_id, deuda_id, monto_aplicado, fecha) VALUES (?1,?2,?3,?4)",
            params![a.pago_id, a.deuda_id, a.monto_aplicado, a.fecha],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn fetch_por_pago(&self, pago_id: usize) -> Result<Vec<AplicacionPago>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, pago_id, deuda_id, monto_aplicado, fecha FROM aplicaciones_pago WHERE pago_id = ?1"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![pago_id], |row| {
            Ok(AplicacionPago { id: row.get(0)?, pago_id: row.get(1)?, deuda_id: row.get(2)?, monto_aplicado: row.get(3)?, fecha: row.get(4)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_por_deuda(&self, deuda_id: usize) -> Result<Vec<AplicacionPago>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, pago_id, deuda_id, monto_aplicado, fecha FROM aplicaciones_pago WHERE deuda_id = ?1"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![deuda_id], |row| {
            Ok(AplicacionPago { id: row.get(0)?, pago_id: row.get(1)?, deuda_id: row.get(2)?, monto_aplicado: row.get(3)?, fecha: row.get(4)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn delete_por_pago(&self, pago_id: usize) -> Result<(), ErrorRepositorio> {
        self.lock().execute("DELETE FROM aplicaciones_pago WHERE pago_id = ?1", params![pago_id]).map_err(error_consulta)?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// HistorialPagoRepository
// ═══════════════════════════════════════════════════════════════

impl HistorialPagoRepository for SqliteRepositorio {
    fn save(&self, h: &HistorialPago) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO historial_pagos (representante_id, tipo_id, monto, periodo, fecha, observacion) VALUES (?1,?2,?3,?4,?5,?6)",
            params![h.representante_id, h.tipo_id, h.monto, h.periodo, h.fecha, h.observacion],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn fetch_por_representante(&self, representante_id: usize) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, tipo_id, monto, periodo, fecha, observacion FROM historial_pagos WHERE representante_id = ?1 AND eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![representante_id], |row| {
            Ok(HistorialPago { id: row.get(0)?, representante_id: row.get(1)?, tipo_id: row.get(2)?, monto: row.get(3)?, periodo: row.get(4)?, fecha: row.get(5)?, observacion: row.get(6)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_por_periodo(&self, periodo: &str) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, tipo_id, monto, periodo, fecha, observacion FROM historial_pagos WHERE periodo = ?1 AND eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![periodo], |row| {
            Ok(HistorialPago { id: row.get(0)?, representante_id: row.get(1)?, tipo_id: row.get(2)?, monto: row.get(3)?, periodo: row.get(4)?, fecha: row.get(5)?, observacion: row.get(6)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_all(&self) -> Result<Vec<HistorialPago>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, representante_id, tipo_id, monto, periodo, fecha, observacion FROM historial_pagos WHERE eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map([], |row| {
            Ok(HistorialPago { id: row.get(0)?, representante_id: row.get(1)?, tipo_id: row.get(2)?, monto: row.get(3)?, periodo: row.get(4)?, fecha: row.get(5)?, observacion: row.get(6)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("historial_pagos", ids, "historial")
    }
}

// ═══════════════════════════════════════════════════════════════
// AbonoRepository (legacy)
// ═══════════════════════════════════════════════════════════════

impl AbonoRepository for SqliteRepositorio {
    fn save(&self, a: &Abono) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO abonos (deuda_id, monto, fecha, observacion) VALUES (?1,?2,?3,?4)",
            params![a.deuda_id, a.monto, a.fecha, a.observacion],
        ).map_err(error_consulta)?;
        Ok(())
    }

    fn fetch_por_deuda(&self, deuda_id: usize) -> Result<Vec<Abono>, ErrorRepositorio> {
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, deuda_id, monto, fecha, observacion FROM abonos WHERE deuda_id = ?1 AND eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map(params![deuda_id], |row| {
            Ok(Abono { id: row.get(0)?, deuda_id: row.get(1)?, monto: row.get(2)?, fecha: row.get(3)?, observacion: row.get(4)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn fetch_por_periodo(&self, _periodo: &str) -> Result<Vec<Abono>, ErrorRepositorio> {
        // Legacy: abonos no tienen periodo propio
        let c = self.lock();
        let mut stmt = c.prepare(
            "SELECT id, deuda_id, monto, fecha, observacion FROM abonos WHERE eliminado = 0"
        ).map_err(error_consulta)?;
        let iter = stmt.query_map([], |row| {
            Ok(Abono { id: row.get(0)?, deuda_id: row.get(1)?, monto: row.get(2)?, fecha: row.get(3)?, observacion: row.get(4)? })
        }).map_err(error_consulta)?;
        let mut lista = Vec::new();
        for fila in iter { lista.push(fila.map_err(error_consulta)?); }
        Ok(lista)
    }

    fn delete(&self, ids: HashSet<usize>) -> Result<(), ErrorRepositorio> {
        self.borrar_logicamente("abonos", ids, "abonos")
    }
}

// ═══════════════════════════════════════════════════════════════
// ConfiguracionAppRepository
// ═══════════════════════════════════════════════════════════════

impl ConfiguracionAppRepository for SqliteRepositorio {
    fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorRepositorio> {
        let c = self.lock();
        let resultado = c.query_row(
            "SELECT valor FROM ajustes WHERE clave = ?1",
            params![clave],
            |row| row.get::<_, String>(0),
        ).optional().map_err(error_consulta)?;
        Ok(resultado)
    }

    fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorRepositorio> {
        self.lock().execute(
            "INSERT INTO ajustes (clave, valor) VALUES (?1, ?2) ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![clave, valor],
        ).map_err(error_consulta)?;
        Ok(())
    }
}
