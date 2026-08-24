use crate::application::dto::{AlumnoVista, DatosAlumno, DatosPago, DatosRepresentante};
use crate::application::error::ErrorAplicacion;
use crate::application::ports::Logger;
use crate::application::service::ServicioAlumnos;
use crate::application::service_pagos::ServicioPagos;
use crate::application::service_representantes::ServicioRepresentantes;
use crate::domain::{Alumno, Cintas, Representante};
use chrono::{Datelike, Local};
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

/// ViewModel de presentación: caché en memoria de alumnos, representantes y
/// pagos del mes + selección. Toda la persistencia se delega en los casos de
/// uso de `application`; nunca expone sus servicios internos (regla 3).
///
/// Las tablas consumen PROYECCIONES (`AlumnoVista`/`PagoVista`) ya resueltas
/// por la capa de aplicación: la UI jamás junta entidades por su cuenta.
pub struct MyApp {
    pub alumnos: Vec<AlumnoVista>,
    pub representantes: Vec<Representante>,
    /// Pagos registrados para `periodo_actual`.
    pub pagos: Vec<crate::application::dto::PagoVista>,
    /// Representantes activos sin pago en `periodo_actual`.
    pub morosos: Vec<Representante>,
    /// Mes que administra el panel, formato "YYYY-MM".
    pub periodo_actual: String,
    pub seleccionados: HashSet<usize>,
    servicio_alumnos: Arc<ServicioAlumnos>,
    servicio_representantes: Arc<ServicioRepresentantes>,
    servicio_pagos: Arc<ServicioPagos>,
    logger: Arc<dyn Logger>,
}

impl MyApp {
    /// Constructor con dependencias inyectadas. Solo lo invoca el composition
    /// root. Carga los datos iniciales vía `refrescar`.
    pub fn new(
        servicio_alumnos: Arc<ServicioAlumnos>,
        servicio_representantes: Arc<ServicioRepresentantes>,
        servicio_pagos: Arc<ServicioPagos>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        let ahora = Local::now();
        let mut estado = Self {
            alumnos: Vec::new(),
            representantes: Vec::new(),
            pagos: Vec::new(),
            morosos: Vec::new(),
            periodo_actual: format!("{:04}-{:02}", ahora.year(), ahora.month()),
            seleccionados: HashSet::new(),
            servicio_alumnos,
            servicio_representantes,
            servicio_pagos,
            logger,
        };
        estado.refrescar();
        estado
    }

    fn refrescar(&mut self) {
        // Alumnos + representantes se juntan en la proyección de lectura.
        match (
            self.servicio_alumnos.obtener_todos(),
            self.servicio_representantes.obtener_todos(),
        ) {
            (Ok(alumnos), Ok(representantes)) => {
                self.alumnos = crate::application::service::armar_vistas_alumnos(
                    &alumnos,
                    &representantes,
                );
                self.representantes = representantes;

                // Pagos y morosos dependen de ambas listas.
                match self.servicio_pagos.listar_del_periodo(
                    &self.periodo_actual.clone(),
                    &self.representantes.clone(),
                ) {
                    Ok(pagos) => self.pagos = pagos,
                    Err(error) => self.logger.error(&format!(
                        "No se pudieron cargar los pagos del periodo: {error}"
                    )),
                }
                match self.servicio_pagos.morosos_del_periodo(
                    &self.periodo_actual.clone(),
                    &self.representantes.clone(),
                ) {
                    Ok(morosos) => self.morosos = morosos,
                    Err(error) => self
                        .logger
                        .error(&format!("No se pudo calcular la morosidad: {error}")),
                }
            }
            (Err(error), _) | (_, Err(error)) => self.logger.error(&format!(
                "No se pudo refrescar la lista de alumnos: {error}"
            )),
        }
    }

    // ---------- Casos de uso de alumnos ----------

    pub fn agregar_alumno(&mut self, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        self.servicio_alumnos.agregar(datos)?;
        self.refrescar();
        Ok(())
    }

    pub fn actualizar_alumno(&mut self, id: usize, datos: DatosAlumno) -> Result<(), ErrorAplicacion> {
        self.servicio_alumnos.actualizar(id, datos)?;
        self.refrescar();
        Ok(())
    }

    /// Caso de uso: promover masivamente a los alumnos seleccionados.
    pub fn promover_seleccionados(&mut self, rango: i32, rallita: bool) -> Result<(), ErrorAplicacion> {
        let ids = self.seleccionados.clone();
        self.servicio_alumnos.promover(ids, rango, rallita)?;
        self.refrescar();
        Ok(())
    }

    /// Caso de uso: eliminar a los seleccionados, limpiar la selección y refrescar.
    pub fn eliminar_seleccionados(&mut self) -> Result<(), ErrorAplicacion> {
        let ids = self.seleccionados.clone();
        self.servicio_alumnos.eliminar(ids)?;
        self.seleccionados.clear();
        self.refrescar();
        Ok(())
    }

    // ---------- Casos de uso de representantes ----------

    pub fn agregar_representante(&mut self, datos: DatosRepresentante) -> Result<(), ErrorAplicacion> {
        self.servicio_representantes.agregar(datos)?;
        self.refrescar();
        Ok(())
    }

    pub fn actualizar_representante(
        &mut self,
        id: usize,
        datos: DatosRepresentante,
    ) -> Result<(), ErrorAplicacion> {
        self.servicio_representantes.actualizar(id, datos)?;
        self.refrescar();
        Ok(())
    }

    // ---------- Casos de uso de pagos ----------

    pub fn registrar_pago(&mut self, datos: DatosPago) -> Result<(), ErrorAplicacion> {
        self.servicio_pagos.registrar(datos)?;
        self.refrescar();
        Ok(())
    }

    /// Anula un pago (borrado lógico) y refresca totales/morosos.
    pub fn anular_pago(&mut self, id: usize) -> Result<(), ErrorAplicacion> {
        self.servicio_pagos.eliminar(HashSet::from([id]))?;
        self.refrescar();
        Ok(())
    }

    /// Total recaudado en el periodo administrado. Suma sobre la caché:
    /// la vista solo pinta, el dato ya fue cargado por el caso de uso.
    pub fn total_del_mes(&self) -> f64 {
        self.pagos.iter().map(|v| v.pago.monto).sum()
    }

    /// Etiqueta legible del periodo actual ("Agosto 2026") para el encabezado.
    pub fn etiqueta_periodo_actual(&self) -> String {
        crate::domain::pago::etiqueta_de_periodo(&self.periodo_actual)
    }

    // ---------- Consultas locales sobre la caché ----------

    pub fn toggle_seleccion(&mut self, id: usize) {
        if self.seleccionados.contains(&id) {
            self.seleccionados.remove(&id);
        } else {
            self.seleccionados.insert(id);
        }
    }

    pub fn toggle_all(&mut self, alumnos_visibles: Vec<AlumnoVista>) {
        // 1. Verificamos si TODOS los alumnos que se están viendo ya están seleccionados
        let todos_seleccionados = alumnos_visibles
            .iter()
            .all(|v| self.seleccionados.contains(&v.alumno.id));

        if todos_seleccionados {
            // Si ya todos están, quitamos de la selección SOLO los que estamos viendo
            for vista in alumnos_visibles {
                self.seleccionados.remove(&vista.alumno.id);
            }
        } else {
            // Si falta alguno (o todos), añadimos todos los visibles a la selección
            for vista in alumnos_visibles {
                self.seleccionados.insert(vista.alumno.id);
            }
        }
    }

    pub fn buscar_alumnos(&self, col: Columnas, query: &str) -> Vec<AlumnoVista> {
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
                    .filter(|v| v.alumno.id == id_buscado)
                    .cloned()
                    .collect();
            } else {
                return Vec::new(); // Si el query no es un número válido, no hay coincidencia exacta
            }
        }

        // Resto de columnas (Búsqueda parcial por texto como la tenías)
        self.alumnos
            .iter()
            .filter(|v| match col {
                Columnas::Nombre => v.alumno.nombre.to_lowercase().contains(&q),
                // El representante y el teléfono viven en la proyección resuelta
                Columnas::Representante => v.nombre_representante.to_lowercase().contains(&q),
                Columnas::Telefono => v.telefono_representante.contains(&q),
                _ => true,
            })
            .cloned()
            .collect()
    }

    pub fn get_alumno_by_id(&self, id: usize) -> Alumno {
        self.alumnos
            .iter()
            .find(|v| v.alumno.id == id)
            .map(|v| v.alumno.clone())
            .expect("Error: El ID del alumno no existe en la base de datos") // Rompe el programa de forma controlada si es None
    }

    pub fn filtrar_edad(&self, edad: String) -> Vec<AlumnoVista> {
        if edad.is_empty() {
            return self.alumnos.clone();
        }
        let hoy = Local::now().date_naive();
        self.alumnos
            .iter()
            .filter(|v| v.alumno.edad(hoy) == edad)
            .cloned()
            .collect()
    }

    pub fn filtrar_cinta(&self, cinta_label: String, solo_rallita: bool) -> Vec<AlumnoVista> {
        if cinta_label.is_empty() {
            return self.alumnos.clone();
        }

        self.alumnos
            .iter()
            .filter(|v| {
                let alumno = &v.alumno;
                let cinta_alumno = Cintas::from_rango(alumno.rango);

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
                cinta && alumno.rallita == solo_rallita
            })
            .cloned()
            .collect()
    }
}
