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
