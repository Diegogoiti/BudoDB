use crate::application::dto::{AlumnoVista, DatosAlumno, DatosAbono, DatosPago, DatosRepresentante, DeudaVista, HistorialPagoVista, PagoVista};
use crate::application::error::ErrorAplicacion;
use crate::application::ports::Logger;
use crate::application::service::ServicioAlumnos;
use crate::application::service_abonos::ServicioAbonos;
use crate::application::service_ajustes::ServicioAjustes;
use crate::application::service_deudas::ServicioDeudas;
use crate::application::service_historial::ServicioHistorialPagos;
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
    /// Pagos legacy del periodo actual (históricos, sin deuda).
    pub pagos: Vec<PagoVista>,
    /// Representantes activos sin pago en `periodo_actual` (legacy).
    pub morosos: Vec<Representante>,
    /// Deudas del periodo actual con saldos y estados ya resueltos.
    pub deudas: Vec<DeudaVista>,
    pub historial_pagos: Vec<HistorialPagoVista>,
    /// Mes que administra el panel de pagos, formato "YYYY-MM".
    pub periodo_actual: String,
    /// Monto predeterminado de mensualidad configurado en Ajustes (0 = sin configurar).
    pub monto_predeterminado: f64,
    /// Ruta del archivo de base de datos (solo lectura, para el panel de Ajustes).
    pub ruta_bd: String,
    /// ID del representante seleccionado en Consulta para filtrar Historial.
    pub representante_historial_id: Option<usize>,
    pub seleccionados: HashSet<usize>,
    /// Selección independiente por contexto
    pub seleccion_alumnos: HashSet<usize>,
    pub seleccion_consulta: HashSet<usize>,
    pub seleccion_representantes: HashSet<usize>,
    /// Estado del modal de reversión de pago.
    pub modal_reversar_activo: bool,
    pub reversar_pago_id: usize,
    pub reversar_rep_nombre: String,
    pub reversar_monto: f64,
    pub reversar_metodo: String,
    pub reversar_fecha: String,
    servicio_alumnos: Arc<ServicioAlumnos>,
    servicio_representantes: Arc<ServicioRepresentantes>,
    #[allow(dead_code)]
    servicio_pagos: Arc<ServicioPagos>,
    servicio_ajustes: Arc<ServicioAjustes>,
    servicio_deudas: Arc<ServicioDeudas>,
    servicio_abonos: Arc<ServicioAbonos>,
    #[allow(dead_code)]
    servicio_historial: Arc<ServicioHistorialPagos>,
    logger: Arc<dyn Logger>,
}

// Re-exportar validaciones para que views.rs pueda usar representante_valido
// al estilo del viejo codebase.

impl MyApp {
    /// Constructor con dependencias inyectadas. Solo lo invoca el composition
    /// root. Carga los datos iniciales vía `refrescar`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ruta_bd: String,
        servicio_alumnos: Arc<ServicioAlumnos>,
        servicio_representantes: Arc<ServicioRepresentantes>,
        servicio_pagos: Arc<ServicioPagos>,
        servicio_ajustes: Arc<ServicioAjustes>,
        servicio_deudas: Arc<ServicioDeudas>,
        servicio_abonos: Arc<ServicioAbonos>,
    #[allow(dead_code)]
    servicio_historial: Arc<ServicioHistorialPagos>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        let ahora = Local::now();
        let mut estado = Self {
            alumnos: Vec::new(),
            representantes: Vec::new(),
            pagos: Vec::new(),
            morosos: Vec::new(),
            deudas: Vec::new(),
            historial_pagos: Vec::new(),
            representante_historial_id: None,
            periodo_actual: format!("{:04}-{:02}", ahora.year(), ahora.month()),
            monto_predeterminado: 0.0,
            ruta_bd,
            seleccionados: HashSet::new(),
            seleccion_alumnos: HashSet::new(),
            seleccion_consulta: HashSet::new(),
            seleccion_representantes: HashSet::new(),
            modal_reversar_activo: false,
            reversar_pago_id: 0,
            reversar_rep_nombre: String::new(),
            reversar_monto: 0.0,
            reversar_metodo: String::new(),
            reversar_fecha: String::new(),
            servicio_alumnos,
            servicio_representantes,
            servicio_pagos,
            servicio_ajustes,
            servicio_deudas,
            servicio_abonos,
            servicio_historial,
            logger,
        };
        estado.refrescar();

        // Creación automática de deudas al iniciar la app.
        if !estado.representantes.is_empty()
            && estado.monto_predeterminado > 0.0
            && estado.deudas.is_empty()
        {
            if let Err(error) = estado.crear_deudas_del_mes() {
                estado.logger.error(&format!(
                    "No se pudieron crear las deudas automáticamente: {error}"
                ));
            }
        }

        estado
    }

    fn refrescar(&mut self) {
        // Ajuste de monto predeterminado (barato y estable entre paneles).
        match self.servicio_ajustes.monto_mensualidad() {
            Ok(monto) => self.monto_predeterminado = monto.unwrap_or(0.0),
            Err(error) => self
                .logger
                .error(&format!("No se pudo leer el ajuste de mensualidad: {error}")),
        }

        // Alumnos + representantes se juntan en la proyección de lectura.
        match (
            self.servicio_alumnos.obtener_todos(),
            self.servicio_representantes.obtener_todos(),
        ) {
            (Ok(alumnos), Ok(representantes)) => {
                self.representantes = representantes;
                self.alumnos = crate::application::service::armar_vistas_alumnos(
                    &alumnos,
                    &self.representantes,
                );

                // Pagos legacy y morosos dependen de ambas listas.
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

                // Deudas del periodo (nuevo sistema).
                match self.servicio_deudas.listar_del_periodo(
                    &self.periodo_actual.clone(),
                    &self.representantes.clone(),
                ) {
                    Ok(deudas) => self.deudas = deudas,
                    Err(error) => self.logger.error(&format!(
                        "No se pudieron cargar las deudas del periodo: {error}"
                    )),
                }
            }
            (Err(error), _) | (_, Err(error)) => self.logger.error(&format!(
                "No se pudo refrescar la lista de alumnos: {error}"
            )),
        }
    }

    /// Caso de uso de Ajustes: fija el monto predeterminado de la mensualidad.
    pub fn cambiar_monto_predeterminado(&mut self, texto: String) -> Result<(), ErrorAplicacion> {
        let monto = texto
            .trim()
            .replace(',', ".")
            .parse::<f64>()
            .map_err(|_| {
                ErrorAplicacion::Validacion("El monto no es un número válido.".to_string())
            })?;
        self.servicio_ajustes.fijar_monto_mensualidad(monto)?;
        self.monto_predeterminado = monto;
        Ok(())
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
        let ids = self.seleccion_alumnos.clone();
        self.servicio_alumnos.promover(ids, rango, rallita)?;
        self.refrescar();
        Ok(())
    }

    /// Caso de uso: desactivar a los seleccionados, limpiar la selección y refrescar.
    /// Si todos los alumnos de un representante quedan inactivos, desactivarlo también.
    pub fn desactivar_seleccionados(&mut self) -> Result<(), ErrorAplicacion> {
        let ids = self.seleccion_alumnos.clone();
        self.servicio_alumnos.desactivar(ids)?;
        self.seleccion_alumnos.clear();
        self.refrescar();
        self.verificar_representantes_cascade();
        Ok(())
    }

    /// Caso de uso: activar a los seleccionados.
    pub fn activar_seleccionados(&mut self) -> Result<(), ErrorAplicacion> {
        let ids = self.seleccion_alumnos.clone();
        self.servicio_alumnos.activar(ids)?;
        self.seleccion_alumnos.clear();
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

    pub fn desactivar_representante(&mut self, id: usize) -> Result<(), ErrorAplicacion> {
        self.servicio_representantes.desactivar(HashSet::from([id]))?;
        let alumnos_ids: HashSet<usize> = self.alumnos.iter()
            .filter(|v| v.alumno.representante_id == id && v.alumno.estado_id == 1)
            .map(|v| v.alumno.id)
            .collect();
        if !alumnos_ids.is_empty() {
            self.servicio_alumnos.desactivar(alumnos_ids)?;
        }
        self.refrescar();
        Ok(())
    }

    pub fn activar_representante(&mut self, id: usize) -> Result<(), ErrorAplicacion> {
        self.servicio_representantes.activar(HashSet::from([id]))?;
        self.refrescar();
        Ok(())
    }

    /// Verifica si algún representante tiene todos sus alumnos inactivos y lo desactiva.
    fn verificar_representantes_cascade(&mut self) {
        let mut reps_a_desactivar: HashSet<usize> = HashSet::new();
        for rep in &self.representantes {
            if rep.estado_id != 1 { continue; }
            let tiene_activo = self.alumnos.iter().any(|v|
                v.alumno.representante_id == rep.id && v.alumno.estado_id == 1
            );
            if !tiene_activo {
                reps_a_desactivar.insert(rep.id);
            }
        }
        if !reps_a_desactivar.is_empty() {
            for id in &reps_a_desactivar {
                let _ = self.servicio_representantes.desactivar(HashSet::from([*id]));
            }
            self.refrescar();
        }
    }

    // ---------- Casos de uso de pagos ----------

    pub fn registrar_pago(&mut self, datos: DatosPago) -> Result<(), ErrorAplicacion> {
        self.servicio_pagos.registrar_pago(datos)?;
        self.refrescar();
        Ok(())
    }

    /// Anula un pago (borrado lógico) y refresca totales/morosos.
    pub fn anular_pago(&mut self, id: usize) -> Result<(), ErrorAplicacion> {
        self.servicio_pagos.eliminar(HashSet::from([id]))?;
        self.refrescar();
        Ok(())
    }

    /// Reversa un pago: restaura saldos de deudas y cambia estado a Reversado.
    pub fn reversar_pago(&mut self, pago_id: usize) -> Result<(), ErrorAplicacion> {
        self.servicio_pagos.reversar_pago(pago_id)?;
        self.refrescar();
        Ok(())
    }

    /// Abre el modal de confirmación de reversión con los datos del pago.
    pub fn abrir_modal_reversar(
        &mut self,
        pago_id: usize,
        rep_nombre: String,
        monto: f64,
        metodo: String,
        fecha: String,
    ) {
        self.reversar_pago_id = pago_id;
        self.reversar_rep_nombre = rep_nombre;
        self.reversar_monto = monto;
        self.reversar_metodo = metodo;
        self.reversar_fecha = fecha;
        self.modal_reversar_activo = true;
    }

    /// Cierra el modal de reversión.
    pub fn cerrar_modal_reversar(&mut self) {
        self.modal_reversar_activo = false;
    }

    /// Total recaudado en el periodo administrado. Suma sobre la caché:
    /// la vista solo pinta, el dato ya fue cargado por el caso de uso.
    pub fn total_del_mes(&self) -> f64 {
        self.pagos.iter().map(|v| v.pago.monto_recibido).sum()
    }

    /// Etiqueta legible del periodo actual ("Agosto 2026") para el encabezado.
    pub fn etiqueta_periodo_actual(&self) -> String {
        crate::domain::pago::etiqueta_de_periodo(&self.periodo_actual)
    }

    // ---------- Casos de uso de deudas/abonos ----------

    /// Crea deudas del mes para representantes que aún no tienen una en el
    /// periodo activo. Requiere que el monto esté configurado en Ajustes.
    pub fn crear_deudas_del_mes(&mut self) -> Result<usize, ErrorAplicacion> {
        let monto = self.monto_predeterminado;
        if monto <= 0.0 {
            return Err(ErrorAplicacion::Validacion(
                "Configure el monto de mensualidad en Ajustes primero.".to_string(),
            ));
        }
        let fecha = Local::now().format("%Y-%m-%d").to_string();
        let reps = self.representantes.clone();
        let alumnos = self.alumnos.iter().map(|v| v.alumno.clone()).collect::<Vec<_>>();
        let periodo = self.periodo_actual.clone();
        let creadas = self.servicio_deudas.crear_deudas_del_mes(
            &periodo,
            monto,
            &fecha,
            &reps,
            &alumnos,
        )?;
        self.refrescar();
        Ok(creadas)
    }

    /// Registra un abono contra una deuda existente.
    pub fn registrar_abono(&mut self, datos: DatosAbono) -> Result<(), ErrorAplicacion> {
        self.servicio_abonos.registrar(datos)?;
        self.refrescar();
        Ok(())
    }

    // ---------- Estadísticas del sistema de deudas ----------

    /// Monto total de todas las deudas del periodo (monto × num deudas).
    pub fn total_deudas_periodo(&self) -> f64 {
        self.deudas.iter().map(|v| v.deuda.monto_total).sum()
    }

    /// Monto total abonado en el periodo.
    pub fn total_abonado_periodo(&self) -> f64 {
        self.deudas.iter().map(|v| v.deuda.total_abonado()).sum()
    }

    /// Cantidad de representantes que fully pagaron.
    pub fn reps_pagados(&self) -> usize {
        use crate::domain::EstadoDeuda;
        self.deudas
            .iter()
            .filter(|v| v.estado == EstadoDeuda::Pagada)
            .count()
    }

    /// Cantidad con abono parcial.
    pub fn reps_parciales(&self) -> usize {
        use crate::domain::EstadoDeuda;
        self.deudas
            .iter()
            .filter(|v| v.estado == EstadoDeuda::Parcial)
            .count()
    }

    /// Cantidad sin abono alguno (pendientes totales).
    pub fn reps_pendientes(&self) -> usize {
        use crate::domain::EstadoDeuda;
        self.deudas
            .iter()
            .filter(|v| v.estado == EstadoDeuda::Pendiente)
            .count()
    }

    // ---------- Historial de pagos ----------

    /// Carga el historial de pagos. Si se pasa un representante_id, filtra solo ese.
    pub fn refrescar_historial(&mut self, representante_id: Option<usize>) {
        let reps = self.representantes.clone();
        let resultado = match representante_id {
            Some(id) => self.servicio_historial.listar_por_representante(id, &reps),
            None => self.servicio_historial.listar_todos(&reps),
        };
        match resultado {
            Ok(lista) => self.historial_pagos = lista,
            Err(error) => self.logger.error(&format!(
                "No se pudo cargar el historial: {error}"
            )),
        }
    }

    /// Selecciona un representante para filtrar el historial.
    pub fn seleccionar_rep_historial(&mut self, representante_id: Option<usize>) {
        self.representante_historial_id = representante_id;
        self.refrescar_historial(representante_id);
    }

    // ---------- Consultas locales sobre la caché ----------

    pub fn toggle_seleccion(&mut self, id: usize) {
        if self.seleccionados.contains(&id) {
            self.seleccionados.remove(&id);
        } else {
            self.seleccionados.insert(id);
        }
    }

    pub fn toggle_single_seleccion(&mut self, id: usize) {
        if self.seleccionados.contains(&id) {
            self.seleccionados.remove(&id);
        } else {
            self.seleccionados.clear();
            self.seleccionados.insert(id);
        }
    }

    // --- Selección por contexto ---

    pub fn seleccion_set(&mut self, contexto: &str) -> &mut HashSet<usize> {
        match contexto {
            "alumnos" => &mut self.seleccion_alumnos,
            "consulta" => &mut self.seleccion_consulta,
            "representantes" => &mut self.seleccion_representantes,
            _ => &mut self.seleccionados,
        }
    }

    pub fn seleccion_get(&self, contexto: &str) -> &HashSet<usize> {
        match contexto {
            "alumnos" => &self.seleccion_alumnos,
            "consulta" => &self.seleccion_consulta,
            "representantes" => &self.seleccion_representantes,
            _ => &self.seleccionados,
        }
    }

    pub fn toggle_seleccion_ctx(&mut self, id: usize, contexto: &str) {
        let set = self.seleccion_set(contexto);
        if set.contains(&id) {
            set.remove(&id);
        } else {
            set.insert(id);
        }
    }

    pub fn toggle_single_seleccion_ctx(&mut self, id: usize, contexto: &str) {
        let set = self.seleccion_set(contexto);
        if set.contains(&id) {
            set.remove(&id);
        } else {
            set.clear();
            set.insert(id);
        }
    }

    pub fn toggle_all(&mut self, alumnos_visibles: Vec<AlumnoVista>) {
        // 1. Verificamos si TODOS los alumnos que se están viendo ya están seleccionados
        let todos_seleccionados = alumnos_visibles
            .iter()
            .all(|v| self.seleccion_alumnos.contains(&v.alumno.id));

        if todos_seleccionados {
            // Si ya todos están, quitamos de la selección SOLO los que estamos viendo
            for vista in alumnos_visibles {
                self.seleccion_alumnos.remove(&vista.alumno.id);
            }
        } else {
            // Si falta alguno (o todos), añadimos todos los visibles a la selección
            for vista in alumnos_visibles {
                self.seleccion_alumnos.insert(vista.alumno.id);
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
                return Vec::new();
            }
        }

        // Resto de columnas (búsqueda parcial por texto)
        self.alumnos
            .iter()
            .filter(|v| match col {
                Columnas::Nombre => v.alumno.nombre.to_lowercase().contains(&q),
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
            .expect("Error: El ID del alumno no existe en la base de datos")
    }

    /// Filtra una lista (ya buscada) por cinta o edad. Un valor vacío significa
    /// "sin filtro activo" y devuelve la lista tal cual. Permite encadenar
    /// búsqueda y filtro en la vista única de alumnos.
    pub fn filtrar_lista(
        &self,
        base: Vec<AlumnoVista>,
        col: Columnas,
        valor: String,
        solo_rallita: bool,
    ) -> Vec<AlumnoVista> {
        if valor.is_empty() {
            return base;
        }

        match col {
            Columnas::Edad => {
                let hoy = Local::now().date_naive();
                base.into_iter()
                    .filter(|v| v.alumno.edad(hoy) == valor)
                    .collect()
            }
            Columnas::Cinta => base
                .into_iter()
                .filter(|v| {
                    let alumno = &v.alumno;
                    let cinta_alumno = Cintas::from_rango(alumno.rango);

                    let cinta = match valor.as_str() {
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
                        _ => cinta_alumno.label() == valor,
                    };
                    cinta && alumno.rallita == solo_rallita
                })
                .collect(),
            _ => base,
        }
    }
}
