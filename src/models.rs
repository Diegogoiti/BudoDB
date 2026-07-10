//! contiene los modelos de los alumnos para manejarlos como instancias individuales
//!  y el modelo del crud de la base de datos para abstraer operaciones

use chrono::{Datelike, Local, NaiveDate};
use rusqlite;
use rusqlite::{params_from_iter, Result, ToSql};
use std::{collections::HashSet, fmt};

///modelo que maneja los datos delos alumnos para tratarlos como instancias independientes
/// de manera mas organizada y clara, contiene metodos como cinta, rango_str o edad que son setters,
/// calculan los valores a partir de las variables  y los retornan
#[derive(PartialEq, Clone, Debug)]
pub struct Alumno {
    pub id: usize,
    pub nombre: String,
    pub rango: i32,

    pub fecha_de_nacimiento: String,
    pub representante: String,
    pub numero_contacto: String,
    pub rallita: bool,
}

impl Alumno {
    pub fn cinta(&self) -> String {
        let texto_cinta = Cintas::from_rango(self.rango).nombre().to_string();
        if self.rallita {
            let texto_rallita = Cintas::from_rango(self.rango.saturating_sub(1))
                .nombre()
                .to_string();
            format!("{texto_cinta} ralla {texto_rallita}")
        } else {
            texto_cinta
        }
    }

    pub fn edad(&self) -> String {
        let fecha_actual = Local::now().naive_local().date();
        let edad = if let Ok(nac) = NaiveDate::parse_from_str(&self.fecha_de_nacimiento, "%Y-%m-%d")
        {
            let mut años = fecha_actual.year() - nac.year();
            if fecha_actual.month() < nac.month()
                || (fecha_actual.month() == nac.month() && fecha_actual.day() < nac.day())
            {
                años -= 1;
            }
            años.to_string()
        } else {
            "??".to_string()
        };
        format!("{edad} años")
    }

    pub fn rango(&self) -> String {
        if self.rango > 0 && !self.rallita {
            let r = self.rango;
            format!("{r} kyu")
        } else if self.rango > 0 && self.rallita {
            let r = self.rango;
            format!("{r} kyu B")
        } else if self.rango <= 0 {
            let mut r = self.rango;
            r = r.abs();
            r += 1;
            format!("{r} Dan")
        } else {
            "??".to_string()
        }
    }
}

impl fmt::Display for Alumno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Ficha Técnica del Alumno
==========================
ID:             {id}
Nombre:         {nombre}
Fecha de Nac.:  {fecha_nac}
Edad:           {edad} años
Grado (Kyu):    {rango}
Cinta/Nivel:    {cinta}
Representante:  {representante}
Contacto:       {contacto}
Con Rallita:    {rallita}
=========================="#,
            id = self.id,
            nombre = self.nombre,
            fecha_nac = self.fecha_de_nacimiento,
            edad = self.edad(),
            rango = self.rango,
            cinta = self.cinta(),
            representante = self.representante,
            contacto = self.numero_contacto,
            rallita = if self.rallita { "Sí" } else { "No" }
        )
    }
}

///modelo que maneja el crud de la base de datos para abstraer operaciones
pub struct Database {
    connection: rusqlite::Connection,
}
impl Database {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        // 1. Intentamos crear el directorio y capturamos si hay un error real de sistema
        if let Err(e) = std::fs::create_dir_all("database") {
            println!("Error creando carpeta database: {}", e);
        }

        // 2. Abrimos la conexión
        let connection = rusqlite::Connection::open(path)?;

        connection.pragma_update(None, "journal_mode", "WAL")?;

        let db = Self { connection };

        // 3. ¡IMPORTANTE! Asegúrate de llamar aquí a la inicialización de tablas
        // Si no, aunque el archivo se cree, estará vacío y las consultas fallarán.
        db.inicializar_tablas()?;

        Ok(db)
    }

    fn inicializar_tablas(&self) -> rusqlite::Result<()> {
        self.connection.execute(
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

    pub fn save(&self, alumno: &Alumno) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                alumno.nombre,
                alumno.fecha_de_nacimiento,
                alumno.rango,
                alumno.representante,
                alumno.numero_contacto,
                alumno.rallita
            ],
        )?;
        Ok(())
    }

    pub fn fetch_all(&self) -> rusqlite::Result<Vec<Alumno>> {
        let mut stmt = self.connection.prepare(
        "SELECT id, nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita FROM alumnos"
    )?;

        let alumno_iter = stmt.query_map([], |row| {
            Ok(Alumno {
                id: row.get(0)?,
                nombre: row.get::<_, String>(1)?,
                fecha_de_nacimiento: row.get::<_, String>(2)?,
                rango: row.get(3)?,
                representante: row.get::<_, String>(4)?,
                numero_contacto: row.get::<_, String>(5)?,
                rallita: row.get::<_, bool>(6)?,
            })
        })?;

        let mut alumnos = Vec::new();
        for alumno in alumno_iter {
            alumnos.push(alumno?);
        }
        Ok(alumnos)
    }

    pub fn update(&self, alumno: &Alumno) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE alumnos SET nombre = ?1, fecha_de_nacimiento = ?2, rango = ?3, representante = ?4, numero_contacto = ?5, rallita = ?6 WHERE id = ?7",
            rusqlite::params![
                alumno.nombre,
                alumno.fecha_de_nacimiento,
                alumno.rango,
                alumno.representante,
                alumno.numero_contacto,
                alumno.rallita,
                alumno.id
            ],
        )?;
        Ok(())
    }

    pub fn update_rangos(
        &self,
        lista_ids: HashSet<usize>,
        rango: i32,
        rallita: bool,
    ) -> Result<()> {
        // 1. Generamos los comodines (?, ?, ...) según la cantidad de IDs
        let cantidad_ids = lista_ids.len();
        let comodines: String = std::iter::repeat("?")
            .take(cantidad_ids)
            .collect::<Vec<_>>()
            .join(", ");

        // 2. Armamos la query dinámica sin números en los '?' para que vayan en orden
        let query = format!(
            "UPDATE alumnos SET rango = ?, rallita = ? WHERE id IN ({});",
            comodines
        );

        // 3. Juntamos TODOS los parámetros en un solo vector de referencias en el orden exacto
        let mut parametros: Vec<&dyn ToSql> = vec![&rango, &rallita];

        // Agregamos las referencias de cada ID al vector
        for id in &lista_ids {
            parametros.push(id);
        }

        // 4. Ejecutamos la query pasando el iterador de parámetros
        self.connection
            .execute(&query, params_from_iter(parametros))?;

        Ok(())
    }

    pub fn delete(&self, lista_ids: HashSet<usize>) -> Result<()> {
        // 1. Generamos los comodines (?, ?, ...) según la cantidad de IDs
        let cantidad_ids = lista_ids.len();
        let comodines: String = std::iter::repeat("?")
            .take(cantidad_ids)
            .collect::<Vec<_>>()
            .join(", ");

        // 2. Armamos la query dinámica de eliminación
        let query = format!("DELETE FROM alumnos WHERE id IN ({});", comodines);

        // 3. Juntamos las referencias de los IDs en el vector de parámetros
        let mut parametros: Vec<&dyn ToSql> = Vec::with_capacity(cantidad_ids);

        for id in &lista_ids {
            parametros.push(id);
        }

        // 4. Ejecutamos la query pasando el iterador de parámetros
        self.connection
            .execute(&query, params_from_iter(parametros))?;

        Ok(())
    }

    /*pub fn get_alumno_by_id(&self, id: i32) -> rusqlite::Result<Alumno> {
        self.connection.query_row(
        "SELECT id, nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita FROM alumnos WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Alumno {
                id: row.get(0)?,
                nombre: row.get::<_, String>(1)?,
                fecha_de_nacimiento: row.get::<_, String>(2)?,
                rango:row.get(3)?,
                representante:row.get::<_, String>(4)?,
                numero_contacto:row.get::<_, String>(5)?,
                rallita:row.get::<_, bool>(6)?,
                })
        },
    )
    }*/
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cintas {
    Blanca,
    Celeste,
    Amarilla,

    Naranja,
    Verde,
    Azul1,
    Azul2,
    Marron1,
    Marron2,
    Marron3,
    Negra,
}

impl Cintas {
    pub fn all_variants() -> &'static [Cintas] {
        &[
            Cintas::Blanca,
            Cintas::Celeste,
            Cintas::Amarilla,
            Cintas::Naranja,
            Cintas::Verde,
            Cintas::Azul1,
            Cintas::Azul2,
            Cintas::Marron1,
            Cintas::Marron2,
            Cintas::Marron3,
            Cintas::Negra,
        ]
    }

    pub fn from_rango(rango: i32) -> Self {
        match rango {
            10 => Cintas::Blanca,
            9 => Cintas::Celeste,
            8 => Cintas::Amarilla,
            7 => Cintas::Naranja,
            6 => Cintas::Verde,
            5 => Cintas::Azul1,
            4 => Cintas::Azul2,
            3 => Cintas::Marron1,
            2 => Cintas::Marron2,
            1 => Cintas::Marron3,
            _ => Cintas::Negra,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Cintas::Blanca => "Blanca",
            Cintas::Celeste => "Celeste",
            Cintas::Amarilla => "Amarilla",
            Cintas::Naranja => "Naranja",
            Cintas::Verde => "Verde",
            Cintas::Azul1 => "Azul 1",
            Cintas::Azul2 => "Azul 2",
            Cintas::Marron1 => "Marrón 1",
            Cintas::Marron2 => "Marrón 2",
            Cintas::Marron3 => "Marrón 3",
            Cintas::Negra => "Negra",
        }
    }

    pub fn nombre(&self) -> &'static str {
        match self {
            Cintas::Blanca => "Blanca",
            Cintas::Celeste => "Celeste",
            Cintas::Amarilla => "Amarilla",
            Cintas::Naranja => "Naranja",
            Cintas::Verde => "Verde",
            Cintas::Azul1 | Cintas::Azul2 => "Azul",
            Cintas::Marron1 | Cintas::Marron2 | Cintas::Marron3 => "Marrón",
            Cintas::Negra => "Negra",
        }
    }

    pub fn valor(&self) -> u32 {
        match self {
            Cintas::Blanca => 10,
            Cintas::Celeste => 9,
            Cintas::Amarilla => 8,
            Cintas::Naranja => 7,
            Cintas::Verde => 6,
            Cintas::Azul1 => 5,
            Cintas::Azul2 => 4,
            Cintas::Marron1 => 3,
            Cintas::Marron2 => 2,
            Cintas::Marron3 => 1,
            Cintas::Negra => 0,
        }
    }
}
