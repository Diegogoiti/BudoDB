//! Entidad de negocio `Alumno`: datos + reglas de cálculo puras.
//! Solo depende de `chrono` como librería de utilidad de fechas; jamás
//! conoce bases de datos ni UI (regla 1).

use super::cintas::Cintas;
use chrono::{Datelike, Local, NaiveDate};
use std::fmt;

/// Formato canónico de fecha en TODO el sistema: única fuente de verdad.
pub const FORMATO_FECHA: &str = "%Y-%m-%d";

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

    /// Edad calculada respecto a una fecha actual recibida como parámetro.
    /// Es una función PURA (el reloj lo provee quien llama), testeable.
    pub fn edad(&self, fecha_actual: NaiveDate) -> String {
        let edad = if let Ok(nac) =
            NaiveDate::parse_from_str(&self.fecha_de_nacimiento, FORMATO_FECHA)
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

    /// Regla de negocio: los grados Dan (rango <= 0) nunca llevan rallita.
    pub fn aplica_rallita(rango: i32, rallita: bool) -> bool {
        if rango <= 0 {
            false
        } else {
            rallita
        }
    }
}

impl fmt::Display for Alumno {
    // Única excepción de reloj en el dominio: este formato tipo ficha no se usa
    // desde la interfaz hoy; se conserva para paridad total de comportamiento.
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
            edad = self.edad(Local::now().date_naive()),
            rango = self.rango,
            cinta = self.cinta(),
            representante = self.representante,
            contacto = self.numero_contacto,
            rallita = if self.rallita { "Sí" } else { "No" }
        )
    }
}
