use crate::application::dto::DatosAlumno;
use crate::application::error::ErrorAplicacion;
use crate::application::ports::Logger;
use crate::application::service::ServicioAlumnos;
use crate::domain::{Alumno, Cintas};
use chrono::Local;
use std::collections::HashSet;
use std::sync::Arc;

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

/// ViewModel de presentación: caché en memoria de alumnos + selección.
/// Toda la persistencia se delega en los casos de uso de `application`;
/// ya NO expone su repositorio interno (regla 3).
pub struct MyApp {
    pub alumnos: Vec<Alumno>,
    pub seleccionados: HashSet<usize>,
    servicio: Arc<ServicioAlumnos>,
    logger: Arc<dyn Logger>,
}

impl MyApp {
    /// Constructor con dependencias inyectadas. Solo lo invoca el composition root.
    pub fn new(
        alumnos: Vec<Alumno>,
        servicio: Arc<ServicioAlumnos>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            alumnos,
            seleccionados: HashSet::new(),
            servicio,
            logger,
        }
    }

    /// Recarga la lista desde el caso de uso. Si falla: log y se conserva
    /// la lista previa (política de error uniforme de la app).
    fn refrescar(&mut self) {
        match self.servicio.obtener_todos() {
            Ok(alumnos) => self.alumnos = alumnos,
            Err(error) => self
                .logger
                .error(&format!("No se pudo refrescar la lista de alumnos: {error}")),
        }
    }

    /// Caso de uso: registrar un alumno y refrescar la lista.
    pub fn agregar_alumno(&mut self, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        self.servicio.agregar(datos)?;
        self.refrescar();
        Ok(())
    }

    /// Caso de uso: editar un alumno existente y refrescar la lista.
    pub fn actualizar_alumno(&mut self, id: usize, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        self.servicio.actualizar(id, datos)?;
        self.refrescar();
        Ok(())
    }

    /// Caso de uso: promover masivamente a los alumnos seleccionados.
    pub fn promover_seleccionados(&mut self, rango: i32, rallita: bool) -> Result<(), ErrorAplicacion> {
        let ids = self.seleccionados.clone();
        self.servicio.promover(ids, rango, rallita)?;
        self.refrescar();
        Ok(())
    }

    /// Caso de uso: eliminar a los seleccionados, limpiar la selección y refrescar.
    pub fn eliminar_seleccionados(&mut self) -> Result<(), ErrorAplicacion> {
        let ids = self.seleccionados.clone();
        self.servicio.eliminar(ids)?;
        self.seleccionados.clear();
        self.refrescar();
        Ok(())
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
        let hoy = Local::now().date_naive();
        self.alumnos
            .iter()
            .cloned()
            .filter(|a| a.edad(hoy) == edad)
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
                let cinta_alumno = Cintas::from_rango(a.rango);

                let cinta = match cinta_label.as_str() {
                    "Azul (todos)" => {
                        // Comparamos contra las variantes exactas del Enum
                        matches!(cinta_alumno, Cintas::Azul1 | Cintas::Azul2)
                    }
                    "Marrón (todos)" => {
                        // Comparamos contra las variantes exactas del Enum
                        matches!(
                            cinta_alumno,
                            Cintas::Marron1 | Cintas::Marron2 | Cintas::Marron3
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
