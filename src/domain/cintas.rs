//! Catálogo de cintas y reglas de conversión entre grados (kyu/Dan).
//! Sin dependencias externas.

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

    /// Convierte el número de Dan elegido en la UI al valor interno de rango.
    /// Única fuente de esta fórmula (antes estaba duplicada en dos formularios).
    pub fn rango_desde_dan(dan: i32) -> i32 {
        -dan + 1
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn from_rango_mapea_todo_el_arco_kyu() {
        assert_eq!(Cintas::from_rango(10), Cintas::Blanca);
        assert_eq!(Cintas::from_rango(9), Cintas::Celeste);
        assert_eq!(Cintas::from_rango(8), Cintas::Amarilla);
        assert_eq!(Cintas::from_rango(7), Cintas::Naranja);
        assert_eq!(Cintas::from_rango(6), Cintas::Verde);
        assert_eq!(Cintas::from_rango(5), Cintas::Azul1);
        assert_eq!(Cintas::from_rango(4), Cintas::Azul2);
        assert_eq!(Cintas::from_rango(3), Cintas::Marron1);
        assert_eq!(Cintas::from_rango(2), Cintas::Marron2);
        assert_eq!(Cintas::from_rango(1), Cintas::Marron3);
    }

    #[test]
    fn grados_dan_y_desconocidos_son_negra() {
        assert_eq!(Cintas::from_rango(0), Cintas::Negra);
        assert_eq!(Cintas::from_rango(-7), Cintas::Negra);
        assert_eq!(Cintas::from_rango(99), Cintas::Negra);
    }

    #[test]
    fn label_es_lo_que_ve_el_usuario() {
        assert_eq!(Cintas::Blanca.label(), "Blanca");
        assert_eq!(Cintas::Azul2.label(), "Azul 2");
        assert_eq!(Cintas::Marron3.label(), "Marrón 3");
        assert_eq!(Cintas::Negra.label(), "Negra");
    }

    #[test]
    fn nombre_agrupa_azules_y_marrones() {
        assert_eq!(Cintas::Azul1.nombre(), "Azul");
        assert_eq!(Cintas::Azul2.nombre(), "Azul");
        assert_eq!(Cintas::Marron1.nombre(), "Marrón");
        assert_eq!(Cintas::Marron3.nombre(), "Marrón");
        assert_eq!(Cintas::Verde.nombre(), "Verde");
    }

    #[test]
    fn valor_y_from_rango_son_inversos_salvo_negra() {
        for cinta in Cintas::all_variants() {
            if *cinta == Cintas::Negra {
                continue; // Negra comparte el catch-all con los Dan.
            }
            assert_eq!(Cintas::from_rango(cinta.valor() as i32), *cinta);
        }
        assert_eq!(Cintas::all_variants().len(), 11);
    }

    #[test]
    fn conversion_dan_usa_la_formula_historica() {
        // Dan 1 -> rango 0 (Negra), Dan 10 -> rango -9.
        assert_eq!(Cintas::rango_desde_dan(1), 0);
        assert_eq!(Cintas::rango_desde_dan(10), -9);
        assert_eq!(Cintas::from_rango(Cintas::rango_desde_dan(1)), Cintas::Negra);
    }
}
