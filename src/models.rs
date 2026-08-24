//! contiene las entidades de negocio (`Alumno`, `Cintas`).
//! TEMPORAL: en la fase 3 del refactor se dividirán hacia `domain`.
//! La persistencia (`Database`) ya migró a `infrastructure/sqlite_repository.rs`.

use chrono::{Datelike, Local, NaiveDate};
use std::fmt;

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
========================="#,
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
