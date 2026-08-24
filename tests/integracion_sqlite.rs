//! Pruebas de integración del adaptador [`SqliteRepositorio`] contra un
//! archivo SQLite real en el directorio temporal del sistema. Cubre los tres
//! puertos (alumnos, representantes, pagos) y las migraciones de esquema.
//!
//! Como UNA estructura concreta implementa TRES puertos, cada prueba se liga
//! al puerto que ejercita mediante un `&dyn Trait` explícito.

use budodb::application::ports::{
    AlumnoRepository, ConfiguracionAppRepository, Logger, PagoRepository, RepresentanteRepository,
};
use budodb::domain::{Alumno, Pago, Representante};
use budodb::infrastructure::sqlite_repository::SqliteRepositorio;
use std::collections::HashSet;
use std::sync::Arc;

struct LoggerSilencioso;

impl Logger for LoggerSilencioso {
    fn debug(&self, _: &str) {}
    fn info(&self, _: &str) {}
    fn error(&self, mensaje: &str) {
        eprintln!("[ERROR] {mensaje}");
    }
}

fn abrir_bd(etiqueta: &str) -> (SqliteRepositorio, String) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let ruta = std::env::temp_dir()
        .join(format!(
            "budodb_test_{}_{}_{}.db",
            etiqueta,
            std::process::id(),
            nanos
        ))
        .to_string_lossy()
        .into_owned();

    let repo = SqliteRepositorio::abrir(&ruta, Arc::new(LoggerSilencioso))
        .expect("no se pudo abrir la BD de prueba");
    (repo, ruta)
}

fn limpiar(ruta: &str) {
    let _ = std::fs::remove_file(ruta);
    let _ = std::fs::remove_file(format!("{ruta}-wal"));
    let _ = std::fs::remove_file(format!("{ruta}-shm"));
}

/// Crea un representante y devuelve su ID autogenerado.
fn crear_representante(reps: &dyn RepresentanteRepository, nombre: &str) -> usize {
    reps.save(&Representante {
        id: 0,
        nombre: nombre.to_string(),
        numero_contacto: "0412-0000000".to_string(),
    })
    .unwrap();
    reps.fetch_all()
        .unwrap()
        .iter()
        .find(|r| r.nombre == nombre)
        .map(|r| r.id)
        .unwrap()
}

fn alumno(nombre: &str, rango: i32, rallita: bool, representante_id: usize) -> Alumno {
    Alumno {
        id: 0,
        nombre: nombre.to_string(),
        rango,
        fecha_de_nacimiento: "2010-01-15".to_string(),
        representante_id,
        rallita,
    }
}

#[test]
fn ciclo_completo_crud_sobre_archivo_real() {
    let (repo, ruta) = abrir_bd("crud");
    let alumnos: &dyn AlumnoRepository = &repo;
    let rep_id = crear_representante(&repo, "Representante");

    alumnos.save(&alumno("Ana", 6, false, rep_id)).unwrap();
    alumnos.save(&alumno("Beto", 3, true, rep_id)).unwrap();

    // Lectura: los dos registros con su ID autogenerado.
    let todos = alumnos.fetch_all().unwrap();
    assert_eq!(todos.len(), 2);
    assert!(todos.iter().all(|a| a.representante_id == rep_id));
    let id_ana = todos.iter().find(|a| a.nombre == "Ana").map(|a| a.id).unwrap();

    // Update respeta el id y cambia todos los campos.
    let mut ana = todos.into_iter().find(|a| a.id == id_ana).unwrap();
    ana.nombre = "Ana Editada".to_string();
    ana.rango = 5;
    ana.rallita = true;
    ana.representante_id = rep_id;
    alumnos.update(&ana).unwrap();

    let tras_update = alumnos.fetch_all().unwrap();
    let ana_db = tras_update.iter().find(|a| a.id == id_ana).unwrap();
    assert_eq!(ana_db.nombre, "Ana Editada");
    assert_eq!(ana_db.rango, 5);
    assert!(ana_db.rallita);

    // Promoción masiva solo sobre Ana; Beto queda intacto.
    alumnos.update_rangos(HashSet::from([id_ana]), 4, false).unwrap();
    let tras_promo = alumnos.fetch_all().unwrap();
    let ana2 = tras_promo.iter().find(|a| a.id == id_ana).unwrap();
    let beto = tras_promo.iter().find(|a| a.nombre == "Beto").unwrap();
    assert_eq!(ana2.rango, 4);
    assert!(!ana2.rallita);
    assert_eq!(beto.rango, 3);
    assert!(beto.rallita);

    // Delete masivo deja la tabla vacía.
    let ids: HashSet<usize> = tras_promo.iter().map(|a| a.id).collect();
    alumnos.delete(ids).unwrap();
    assert!(alumnos.fetch_all().unwrap().is_empty());

    limpiar(&ruta);
}

#[test]
fn las_tablas_se_crea_vacias_en_un_archivo_nuevo() {
    let (repo, ruta) = abrir_bd("nuevo");

    assert!(AlumnoRepository::fetch_all(&repo).unwrap().is_empty());
    assert!(RepresentanteRepository::fetch_all(&repo).unwrap().is_empty());
    assert!(PagoRepository::fetch_all(&repo).unwrap().is_empty());

    limpiar(&ruta);
}

#[test]
fn el_borrado_es_logico_y_conserva_la_fila_fisica() {
    let (repo, ruta) = abrir_bd("borrado_logico");
    let alumnos: &dyn AlumnoRepository = &repo;
    let rep_id = crear_representante(&repo, "Representante");

    alumnos.save(&alumno("Ana", 6, false, rep_id)).unwrap();
    alumnos.save(&alumno("Beto", 3, true, rep_id)).unwrap();
    let todos = alumnos.fetch_all().unwrap();
    let id_ana = todos.iter().find(|a| a.nombre == "Ana").map(|a| a.id).unwrap();

    // Borrado lógico: desaparece del listado...
    alumnos.delete(HashSet::from([id_ana])).unwrap();
    let visibles = alumnos.fetch_all().unwrap();
    assert_eq!(visibles.len(), 1);
    assert_eq!(visibles[0].nombre, "Beto");

    // ...pero la fila sigue en la base con eliminado = 1.
    let cruda = rusqlite::Connection::open(&ruta).unwrap();
    let (cantidad, eliminados): (i64, i64) = cruda
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(eliminado), 0) FROM alumnos",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(cantidad, 2);
    assert_eq!(eliminados, 1);

    // Promover un alumno ya eliminado no debe reactivarlo ni modificarlo.
    alumnos.update_rangos(HashSet::from([id_ana]), 1, true).unwrap();
    let tras_promo = alumnos.fetch_all().unwrap();
    assert_eq!(tras_promo.len(), 1);
    assert_ne!(tras_promo[0].id, id_ana);

    drop(cruda);
    limpiar(&ruta);
}

#[test]
fn migra_bases_creadas_sin_columna_eliminado() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let ruta = std::env::temp_dir()
        .join(format!("budodb_legacy_{}_{}.db", std::process::id(), nanos))
        .to_string_lossy()
        .into_owned();

    // Simulamos una base histórica con el esquema original (sin `eliminado`)
    // y datos precargados.
    let legacy = rusqlite::Connection::open(&ruta).unwrap();
    legacy
        .execute(
            "CREATE TABLE alumnos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            fecha_de_nacimiento TEXT NOT NULL,
            rango INTEGER NOT NULL,
            representante TEXT NOT NULL,
            numero_contacto TEXT NOT NULL,
            rallita BOOLEAN NOT NULL DEFAULT 0
        )",
            [],
        )
        .unwrap();
    legacy
        .execute(
            "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita)
             VALUES ('Histórico', '2000-01-01', 6, 'Rep', '0412-0000000', 0)",
            [],
        )
        .unwrap();
    drop(legacy);

    // Al abrirla con el repositorio actual se migra sola y los datos siguen visibles.
    let repo = SqliteRepositorio::abrir(&ruta, Arc::new(LoggerSilencioso))
        .expect("no se pudo abrir la BD legada");
    let alumnos: &dyn AlumnoRepository = &repo;

    let historicos = alumnos.fetch_all().unwrap();
    assert_eq!(historicos.len(), 1);
    assert_eq!(historicos[0].nombre, "Histórico");

    // Y el borrado lógico funciona sobre la base migrada.
    let id_historico = historicos[0].id;
    alumnos.delete(HashSet::from([id_historico])).unwrap();
    assert!(alumnos.fetch_all().unwrap().is_empty());

    limpiar(&ruta);
}

#[test]
fn los_alumnos_se_relacionan_por_clave_con_su_representante() {
    let (repo, ruta) = abrir_bd("relacion");
    let alumnos: &dyn AlumnoRepository = &repo;

    // Dos representantes; cada alumno apunta al suyo por ID.
    let pedro = crear_representante(&repo, "Pedro Pérez");
    let maria = crear_representante(&repo, "María Gómez");

    alumnos.save(&alumno("Ana", 6, false, pedro)).unwrap();
    alumnos.save(&alumno("Luis", 8, false, maria)).unwrap();

    let lista = alumnos.fetch_all().unwrap();
    let ana = lista.iter().find(|a| a.nombre == "Ana").unwrap();
    let luis = lista.iter().find(|a| a.nombre == "Luis").unwrap();
    assert_eq!(ana.representante_id, pedro);
    assert_eq!(luis.representante_id, maria);

    // Un mismo representante puede tener varios alumnos.
    alumnos.save(&alumno("Carlitos", 10, false, pedro)).unwrap();
    let hijos_de_pedro: Vec<_> = alumnos
        .fetch_all()
        .unwrap()
        .into_iter()
        .filter(|a| a.representante_id == pedro)
        .collect();
    assert_eq!(hijos_de_pedro.len(), 2);

    limpiar(&ruta);
}

#[test]
fn los_pagos_se_guardan_por_periodo_y_el_borrado_es_logico() {
    let (repo, ruta) = abrir_bd("pagos");
    let pagos: &dyn PagoRepository = &repo;

    let rep_id = crear_representante(&repo, "Pagador");

    pagos.save(&Pago {
        id: 0,
        representante_id: rep_id,
        monto: 1500.0,
        periodo: "2026-08".to_string(),
        fecha: "2026-08-24".to_string(),
        observacion: String::new(),
    })
    .unwrap();
    pagos.save(&Pago {
        id: 0,
        representante_id: rep_id,
        monto: 1500.0,
        periodo: "2026-07".to_string(),
        fecha: "2026-07-20".to_string(),
        observacion: "mes anterior".to_string(),
    })
    .unwrap();

    // La consulta por periodo solo trae SU mes.
    let agosto = pagos.fetch_por_periodo("2026-08").unwrap();
    assert_eq!(agosto.len(), 1);
    assert_eq!(agosto[0].representante_id, rep_id);
    assert!((agosto[0].monto - 1500.0).abs() < f64::EPSILON);
    assert!(agosto[0].observacion.is_empty());

    // El historial completo trae ambos.
    assert_eq!(pagos.fetch_all().unwrap().len(), 2);

    // Anulación lógica: desaparece del listado pero sigue en el archivo.
    let id_pago = agosto[0].id;
    pagos.delete(HashSet::from([id_pago])).unwrap();
    assert!(pagos.fetch_por_periodo("2026-08").unwrap().is_empty());
    assert_eq!(pagos.fetch_all().unwrap().len(), 1);

    let cruda = rusqlite::Connection::open(&ruta).unwrap();
    let anulados: i64 = cruda
        .query_row(
            "SELECT COUNT(*) FROM pagos WHERE eliminado = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(anulados, 1);

    limpiar(&ruta);
}

#[test]
fn los_representantes_tienen_borrado_logico_propio() {
    let (repo, ruta) = abrir_bd("reps");
    let reps: &dyn RepresentanteRepository = &repo;

    let pedro = crear_representante(&repo, "Pedro Pérez");
    crear_representante(&repo, "María Gómez");

    // El listado viene ordenado por nombre.
    let lista = reps.fetch_all().unwrap();
    assert_eq!(lista.len(), 2);
    assert_eq!(lista[0].nombre, "María Gómez");

    // Borrado lógico del representante.
    reps.delete(HashSet::from([pedro])).unwrap();
    let activos = reps.fetch_all().unwrap();
    assert_eq!(activos.len(), 1);
    assert_eq!(activos[0].nombre, "María Gómez");

    limpiar(&ruta);
}

#[test]
fn una_base_legada_se_normaliza_a_la_relacion_por_clave() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let ruta = std::env::temp_dir()
        .join(format!("budodb_v1_{}_{}.db", std::process::id(), nanos))
        .to_string_lossy()
        .into_owned();

    // Esquema v1 EXACTO (texto plano del representante dentro del alumno).
    let legacy = rusqlite::Connection::open(&ruta).unwrap();
    legacy
        .execute(
            "CREATE TABLE alumnos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            fecha_de_nacimiento TEXT NOT NULL,
            rango INTEGER NOT NULL,
            representante TEXT NOT NULL,
            numero_contacto TEXT NOT NULL,
            rallita BOOLEAN NOT NULL DEFAULT 0
        )",
            [],
        )
        .unwrap();
    legacy
        .execute(
            "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita)
             VALUES ('Ana', '2010-01-15', 6, 'Pedro Pérez', '0412-1111111', 0)",
            [],
        )
        .unwrap();
    legacy
        .execute(
            "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita)
             VALUES ('Luis', '2009-05-20', 8, 'María Gómez', '0414-2222222', 0)",
            [],
        )
        .unwrap();
    legacy
        .execute(
            "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita)
             VALUES ('Carlitos', '2011-03-03', 10, 'Pedro Pérez', '0412-1111111', 0)",
            [],
        )
        .unwrap();
    drop(legacy);

    let repo = SqliteRepositorio::abrir(&ruta, Arc::new(LoggerSilencioso))
        .expect("la BD legada debió migrar sin errores");

    // Los representantes se extrajeron SIN duplicados (Pedro tenía 2 hijos).
    let representantes = RepresentanteRepository::fetch_all(&repo).unwrap();
    assert_eq!(representantes.len(), 2);

    // Cada alumno quedó enlazado por ID a su representante.
    let alumnos = AlumnoRepository::fetch_all(&repo).unwrap();
    assert_eq!(alumnos.len(), 3);
    let pedro = representantes.iter().find(|r| r.nombre == "Pedro Pérez").unwrap();
    let maria = representantes.iter().find(|r| r.nombre == "María Gómez").unwrap();
    let ana = alumnos.iter().find(|a| a.nombre == "Ana").unwrap();
    let carlitos = alumnos.iter().find(|a| a.nombre == "Carlitos").unwrap();
    let luis = alumnos.iter().find(|a| a.nombre == "Luis").unwrap();
    assert_eq!(ana.representante_id, pedro.id);
    assert_eq!(carlitos.representante_id, pedro.id);
    assert_eq!(luis.representante_id, maria.id);

    // Las columnas legadas ya no existen.
    let cruda = rusqlite::Connection::open(&ruta).unwrap();
    let columnas_restantes: Vec<String> = cruda
        .prepare("SELECT name FROM pragma_table_info('alumnos')")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert!(!columnas_restantes.contains(&"representante".to_string()));
    assert!(!columnas_restantes.contains(&"numero_contacto".to_string()));
    assert!(columnas_restantes.contains(&"representante_id".to_string()));

    limpiar(&ruta);
}

#[test]
fn los_ajustes_persisten_y_se_actualizan_por_clave() {
    let (repo, ruta) = abrir_bd("ajustes");

    // Clave inexistente: None, sin errores.
    assert_eq!(
        ConfiguracionAppRepository::obtener(&repo, "monto_mensualidad").unwrap(),
        None
    );

    // Guardar crea; volver a guardar ACTUALIZA (upsert).
    ConfiguracionAppRepository::guardar(&repo, "monto_mensualidad", "1500").unwrap();
    assert_eq!(
        ConfiguracionAppRepository::obtener(&repo, "monto_mensualidad").unwrap(),
        Some("1500".to_string())
    );
    ConfiguracionAppRepository::guardar(&repo, "monto_mensualidad", "2000.5").unwrap();
    assert_eq!(
        ConfiguracionAppRepository::obtener(&repo, "monto_mensualidad").unwrap(),
        Some("2000.5".to_string())
    );

    // Persiste en el archivo: una segunda conexión lo ve.
    drop(repo);
    let repo2 =
        SqliteRepositorio::abrir(&ruta, Arc::new(LoggerSilencioso)).expect("reapertura");
    assert_eq!(
        ConfiguracionAppRepository::obtener(&repo2, "monto_mensualidad").unwrap(),
        Some("2000.5".to_string())
    );

    limpiar(&ruta);
}
