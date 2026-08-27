//! Entidad de negocio `Alumno`: datos + reglas de cálculo puras.
//! Solo depende de `chrono` como librería de utilidad de fechas; jamás
//! conoce bases de datos ni UI (regla 1).

use super::cintas::Cintas;
use chrono::{Datelike, Local, NaiveDate};
use std::fmt;

/// Formato canónico de fecha en TODO el sistema: única fuente de verdad.
pub const FORMATO_FECHA: &str = "%Y-%m-%d";

/// Modelo que maneja los datos de los alumnos para tratarlos como
/// instancias independientes de manera más organizada y clara.
///
/// El alumno se relaciona con su representante mediante `representante_id`
/// (FK hacia la tabla `representantes`). El nombre y teléfono del
/// representante se resuelven en la capa de presentación (AlumnoVista).
#[derive(PartialEq, Clone, Debug)]
pub struct Alumno {
    pub id: usize,
    pub nombre: String,
    pub rango: i32,
    pub fecha_de_nacimiento: String,
    /// FK hacia el representante (adulto responsable) que paga la mensualidad.
    pub representante_id: usize,
    pub rallita: bool,
    /// FK a cat_estados_alumno: 1=Activo, 2=Inactivo, 3=Suspendido, 4=Retirado.
    pub estado_id: i32,
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Ficha Técnica del Alumno
==========================
ID:                 {id}
Nombre:             {nombre}
Fecha de Nac.:      {fecha_nac}
Edad:               {edad} años
Grado (Kyu):        {rango}
Cinta/Nivel:        {cinta}
Representante ID:   {rep_id}
Con Rallita:        {rallita}
========================="#,
            id = self.id,
            nombre = self.nombre,
            fecha_nac = self.fecha_de_nacimiento,
            edad = self.edad(Local::now().date_naive()),
            rango = self.rango,
            cinta = self.cinta(),
            rep_id = self.representante_id,
            rallita = if self.rallita { "Sí" } else { "No" }
        )
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use chrono::NaiveDate;

    fn alumno(rango: i32, rallita: bool) -> Alumno {
        Alumno {
            id: 1,
            nombre: "Test".to_string(),
            rango,
            fecha_de_nacimiento: "2010-01-15".to_string(),
            representante_id: 1,
            rallita,
            estado_id: 1,
        }
    }

    #[test]
    fn cinta_sin_rallita_muestra_solo_su_color() {
        assert_eq!(alumno(6, false).cinta(), "Verde");
        assert_eq!(alumno(10, false).cinta(), "Blanca");
        assert_eq!(alumno(0, false).cinta(), "Negra");
    }

    #[test]
    fn cinta_con_rallita_agrega_la_cinta_anterior() {
        assert_eq!(alumno(6, true).cinta(), "Verde ralla Azul");
        assert_eq!(alumno(10, true).cinta(), "Blanca ralla Celeste");
    }

    #[test]
    fn edad_respeta_si_el_cumpleanos_ya_paso() {
        let hoy = NaiveDate::from_ymd_opt(2026, 8, 24).expect("fecha fija de prueba");
        assert_eq!(alumno(6, false).edad(hoy), "16 años");
    }

    #[test]
    fn edad_no_cuenta_el_cumpleanos_del_dia_siguiente() {
        let hoy = NaiveDate::from_ymd_opt(2026, 8, 24).expect("fecha fija de prueba");
        let casi = Alumno {
            fecha_de_nacimiento: "2010-08-25".to_string(),
            ..alumno(6, false)
        };
        assert_eq!(casi.edad(hoy), "15 años");
    }

    #[test]
    fn el_cumpleanos_de_hoy_cuenta_como_cumplido() {
        let hoy = NaiveDate::from_ymd_opt(2026, 8, 24).expect("fecha fija de prueba");
        let justo = Alumno {
            fecha_de_nacimiento: "2010-08-24".to_string(),
            ..alumno(6, false)
        };
        assert_eq!(justo.edad(hoy), "16 años");
    }

    #[test]
    fn fecha_invalida_muestra_interrogantes() {
        let mut a = alumno(6, false);
        a.fecha_de_nacimiento = "15/01/2010".to_string();
        let hoy = NaiveDate::from_ymd_opt(2026, 8, 24).expect("fecha fija de prueba");
        assert_eq!(a.edad(hoy), "?? años");
    }

    #[test]
    fn rango_formatea_kyu_dan_y_rallita() {
        assert_eq!(alumno(10, false).rango(), "10 kyu");
        assert_eq!(alumno(3, true).rango(), "3 kyu B");
        assert_eq!(alumno(0, false).rango(), "1 Dan");
        assert_eq!(alumno(-9, false).rango(), "10 Dan");
    }

    #[test]
    fn los_dan_nunca_llevan_rallita() {
        assert!(!Alumno::aplica_rallita(0, true));
        assert!(!Alumno::aplica_rallita(-5, true));
        assert!(Alumno::aplica_rallita(6, true));
        assert!(!Alumno::aplica_rallita(6, false));
    }
}
