//! Pruebas de integración del adaptador [`SqliteAlumnoRepository`] contra
//! un archivo SQLite real en el directorio temporal del sistema.

use budodb::application::ports::{AlumnoRepository, Logger};
use budodb::domain::Alumno;
use budodb::infrastructure::sqlite_repository::SqliteAlumnoRepository;
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

fn abrir_bd(etiqueta: &str) -> (SqliteAlumnoRepository, String) {
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

    let repo = SqliteAlumnoRepository::abrir(&ruta, Arc::new(LoggerSilencioso))
        .expect("no se pudo abrir la BD de prueba");
    (repo, ruta)
}

fn limpiar(ruta: &str) {
    let _ = std::fs::remove_file(ruta);
    let _ = std::fs::remove_file(format!("{ruta}-wal"));
    let _ = std::fs::remove_file(format!("{ruta}-shm"));
}

fn alumno(nombre: &str, rango: i32, rallita: bool) -> Alumno {
    Alumno {
        id: 0,
        nombre: nombre.to_string(),
        rango,
        fecha_de_nacimiento: "2010-01-15".to_string(),
        representante: "Representante".to_string(),
        numero_contacto: "0412-0000000".to_string(),
        rallita,
    }
}

#[test]
fn ciclo_completo_crud_sobre_archivo_real() {
    let (repo, ruta) = abrir_bd("crud");

    repo.save(&alumno("Ana", 6, false)).unwrap();
    repo.save(&alumno("Beto", 3, true)).unwrap();

    // Lectura: los dos registros con su ID autogenerado.
    let todos = repo.fetch_all().unwrap();
    assert_eq!(todos.len(), 2);
    let id_ana = todos.iter().find(|a| a.nombre == "Ana").map(|a| a.id).unwrap();

    // Update respeta el id y cambia todos los campos.
    let mut ana = todos.into_iter().find(|a| a.id == id_ana).unwrap();
    ana.nombre = "Ana Editada".to_string();
    ana.rango = 5;
    ana.rallita = true;
    repo.update(&ana).unwrap();

    let tras_update = repo.fetch_all().unwrap();
    let ana_db = tras_update.iter().find(|a| a.id == id_ana).unwrap();
    assert_eq!(ana_db.nombre, "Ana Editada");
    assert_eq!(ana_db.rango, 5);
    assert!(ana_db.rallita);

    // Promoción masiva solo sobre Ana; Beto queda intacto.
    repo.update_rangos(HashSet::from([id_ana]), 4, false).unwrap();
    let tras_promo = repo.fetch_all().unwrap();
    let ana2 = tras_promo.iter().find(|a| a.id == id_ana).unwrap();
    let beto = tras_promo.iter().find(|a| a.nombre == "Beto").unwrap();
    assert_eq!(ana2.rango, 4);
    assert!(!ana2.rallita);
    assert_eq!(beto.rango, 3);
    assert!(beto.rallita);

    // Delete masivo deja la tabla vacía.
    let ids: HashSet<usize> = tras_promo.iter().map(|a| a.id).collect();
    repo.delete(ids).unwrap();
    assert!(repo.fetch_all().unwrap().is_empty());

    limpiar(&ruta);
}

#[test]
fn la_tabla_se_crea_vacia_en_un_archivo_nuevo() {
    let (repo, ruta) = abrir_bd("nuevo");

    assert!(repo.fetch_all().unwrap().is_empty());

    limpiar(&ruta);
}

#[test]
fn el_borrado_es_logico_y_conserva_la_fila_fisica() {
    let (repo, ruta) = abrir_bd("borrado_logico");

    repo.save(&alumno("Ana", 6, false)).unwrap();
    repo.save(&alumno("Beto", 3, true)).unwrap();
    let todos = repo.fetch_all().unwrap();
    let id_ana = todos.iter().find(|a| a.nombre == "Ana").map(|a| a.id).unwrap();

    // Borrado lógico: desaparece del listado...
    repo.delete(HashSet::from([id_ana])).unwrap();
    let visibles = repo.fetch_all().unwrap();
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
    repo.update_rangos(HashSet::from([id_ana]), 1, true).unwrap();
    let tras_promo = repo.fetch_all().unwrap();
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
    let repo = SqliteAlumnoRepository::abrir(&ruta, Arc::new(LoggerSilencioso))
        .expect("no se pudo abrir la BD legada");

    let alumnos = repo.fetch_all().unwrap();
    assert_eq!(alumnos.len(), 1);
    assert_eq!(alumnos[0].nombre, "Histórico");
    let id_historico = alumnos[0].id;

    // Y el borrado lógico funciona sobre la base migrada.
    repo.delete(HashSet::from([id_historico])).unwrap();
    assert!(repo.fetch_all().unwrap().is_empty());

    limpiar(&ruta);
}
