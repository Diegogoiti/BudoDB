use crate::models::{Alumno, Database};
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq)]
pub enum Columnas {
    Id,
    Nombre,
    Edad,
    #[allow(dead_code)]
    FechaNacimiento,
    Representante,
    Telefono,
    Cinta,
}

///estructura que maneja los datos de los alumnos y la ui en memoria
/// ademas de almacenar la coneccion de la db y el estado de la aplicacion
pub struct MyApp {
    pub alumnos: Vec<Alumno>,
    pub seleccionados: HashSet<usize>,
    pub database: Database,
}

impl MyApp {
    /// Constructor con dependencias inyectadas. Solo lo invoca el composition root.
    pub fn new(alumnos: Vec<Alumno>, database: Database) -> Self {
        Self {
            alumnos,
            seleccionados: HashSet::new(),
            database,
        }
    }

    pub fn update(&mut self) {
        self.alumnos = self.database.fetch_all().unwrap();
    }

    pub fn toggle_seleccion(&mut self, id: usize) {
        if self.seleccionados.contains(&id) {
            self.seleccionados.remove(&id);
        } else {
            self.seleccionados.insert(id);
        }
    }

    pub fn toggle_all(&mut self, alumnos_visibles: Vec<Alumno>) {
        // 1. Verificamos si TODOS los alumnos que se están viendo ya están seleccionados
        let todos_seleccionados = alumnos_visibles
            .iter()
            .all(|a| self.seleccionados.contains(&a.id));

        if todos_seleccionados {
            // Si ya todos están, quitamos de la selección SOLO los que estamos viendo
            for alumno in alumnos_visibles {
                self.seleccionados.remove(&alumno.id);
            }
        } else {
            // Si falta alguno (o todos), añadimos todos los visibles a la selección
            for alumno in alumnos_visibles {
                self.seleccionados.insert(alumno.id);
            }
        }
    }

    pub fn buscar_alumnos(&self, col: Columnas, query: &str) -> Vec<Alumno> {
        let q = query.to_lowercase();
        if q.is_empty() {
            return self.alumnos.clone();
        }

        // Caso especial: Búsqueda exacta por ID (KISS y óptima)
        if let Columnas::Id = col {
            if let Ok(id_buscado) = q.trim().parse::<usize>() {
                return self
                    .alumnos
                    .iter()
                    .find(|a| a.id == id_buscado)
                    .cloned()
                    .map(|a| vec![a]) // Si lo encuentra, lo mete en un Vec
                    .unwrap_or_default(); // Si no, retorna un Vec vacío
            } else {
                return Vec::new(); // Si el query no es un número válido, no hay coincidencia exacta
            }
        }

        // Resto de columnas (Búsqueda parcial por texto como la tenías)
        self.alumnos
            .iter()
            .cloned()
            .filter(|a| match col {
                Columnas::Nombre => a.nombre.to_lowercase().contains(&q),
                Columnas::Representante => a.representante.to_lowercase().contains(&q),
                Columnas::Telefono => a.numero_contacto.contains(&q),
                _ => true,
            })
            .collect()
    }

    pub fn get_alumno_by_id(&self, id: usize) -> Alumno {
        self.alumnos
            .iter()
            .find(|a| a.id == id)
            .cloned() // Clona el alumno encontrado para poder sacarlo de la estructura
            .expect("Error: El ID del alumno no existe en la base de datos") // Rompe el programa de forma controlada si es None
    }

    pub fn filtrar_edad(&self, edad: String) -> Vec<Alumno> {
        if edad.is_empty() {
            return self.alumnos.clone();
        }
        self.alumnos
            .iter()
            .cloned()
            .filter(|a| a.edad() == edad)
            .collect()
    }

    pub fn filtrar_cinta(&self, cinta_label: String, solo_rallita: bool) -> Vec<Alumno> {
        if cinta_label.is_empty() {
            return self.alumnos.clone();
        }

        self.alumnos
            .iter()
            .cloned()
            .filter(|a| {
                let cinta_alumno = crate::models::Cintas::from_rango(a.rango);

                let cinta = match cinta_label.as_str() {
                    "Azul (todos)" => {
                        // Comparamos contra las variantes exactas del Enum
                        matches!(
                            cinta_alumno,
                            crate::models::Cintas::Azul1 | crate::models::Cintas::Azul2
                        )
                    }
                    "Marrón (todos)" => {
                        // Comparamos contra las variantes exactas del Enum
                        matches!(
                            cinta_alumno,
                            crate::models::Cintas::Marron1
                                | crate::models::Cintas::Marron2
                                | crate::models::Cintas::Marron3
                        )
                    }
                    // Para etiquetas individuales ("Blanca", "Azul 1"), usamos .label()
                    // que es lo que el usuario ve y selecciona en el dropdown
                    _ => cinta_alumno.label() == cinta_label,
                };
                cinta && a.rallita == solo_rallita
            })
            .collect()
    }
}
