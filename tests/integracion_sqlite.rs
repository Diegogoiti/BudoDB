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
