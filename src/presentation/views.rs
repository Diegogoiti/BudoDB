//! se encarga de dibujar las vistas segun las rutas seleccionadas
//! contiene las funciones con el codigo especifico de cada vista

use crate::application::dto::{AlumnoVista, DatosAlumno, DatosPago, DatosRepresentante, DeudaVista, HistorialPagoVista, PagoVista};
use crate::application::validation::*;
use crate::domain::{EstadoDeuda, EstadoPago, MetodoPago};
use crate::presentation::components::datatable::{DataTable, HeaderColumn, RowKeyFn, RenderRowFn};
use crate::presentation::components::filter::Filter;
use crate::presentation::components::form::Form;
use crate::presentation::components::promotion_form::PromotionForm;
use crate::presentation::components::searchbar::SearchBar;
use crate::presentation::my_app::{self, Columnas};
use chrono::Local;
use dioxus::prelude::*;

static ALUMNO_COLUMNS: &[HeaderColumn] = &[
    HeaderColumn { header: "Nombre", class: Some("font-bold text-white whitespace-nowrap") },
    HeaderColumn { header: "Cinta", class: Some("") },
    HeaderColumn { header: "Rango", class: Some("") },
    HeaderColumn { header: "Edad", class: Some("whitespace-nowrap") },
    HeaderColumn { header: "F. Nacimiento", class: Some("") },
    HeaderColumn { header: "Representante", class: Some("whitespace-nowrap") },
    HeaderColumn { header: "Teléfono", class: Some("text-blue-400 font-mono whitespace-nowrap") },
];

/// Acciones de alumnos que se presentan como modales dentro de la vista única.
#[derive(Clone, Copy, PartialEq)]
enum ModalAlumno {
    Nuevo,
    Editar,
    Promover,
    Eliminar,
    RegistrarRepresentante,
}

/// Vista única de alumnos.
fn alumno_key(vista: &AlumnoVista) -> usize {
    vista.alumno.id
}

fn render_alumno_row(vista: &AlumnoVista, _estado: Signal<my_app::MyApp>) -> Element {
    let hoy = Local::now().date_naive();
    rsx! {
        td { class: "px-4 py-3 font-bold text-white whitespace-nowrap", "{vista.alumno.nombre}" }
        td { class: "px-4 py-3",
            span { class: "inline-flex items-center justify-center min-w-36 px-3 py-1.5 rounded bg-gray-700 text-[10px] uppercase font-bold text-gray-300 whitespace-nowrap",
                "{vista.alumno.cinta()}"
            }
        }
        td { class: "px-4 py-3",
            span { class: "inline-flex items-center justify-center min-w-20 px-2 py-1.5 rounded bg-gray-700 text-[10px] uppercase font-bold text-gray-300 whitespace-nowrap",
                "{vista.alumno.rango()}"
            }
        }
        td { class: "px-4 py-3 whitespace-nowrap", "{vista.alumno.edad(hoy)}" }
        td { class: "px-4 py-3", "{vista.alumno.fecha_de_nacimiento}" }
        td { class: "px-4 py-3 whitespace-nowrap", "{vista.nombre_representante}" }
        td { class: "px-4 py-3 text-blue-400 font-mono whitespace-nowrap",
            "{vista.telefono_representante}"
        }
    }
}

#[component]
pub fn Alumnos() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut modal_activo = use_signal(|| None::<ModalAlumno>);

    let mut busqueda = use_signal(|| (Columnas::Nombre, String::new()));
    let mut filtro = use_signal(|| (Columnas::Cinta, String::new(), false));
    let mut alumnos_filtrados = use_signal(|| estado.read().alumnos.clone());

    {
        let estado = estado.clone();
        use_effect(move || {
            let app = estado.read();
            let (col_buscar, texto) = busqueda.read().clone();
            let (col_filtro, valor_filtro, solo_rallita) = filtro.read().clone();
            let base = app.buscar_alumnos(col_buscar, &texto);
            alumnos_filtrados.set(app.filtrar_lista(base, col_filtro, valor_filtro, solo_rallita));
        });
    }

    let total_seleccionados = estado.read().seleccionados.len();
    let hay_seleccion = total_seleccionados > 0;
    let uno_seleccionado = total_seleccionados == 1;

    let todos_seleccionados = !alumnos_filtrados.read().is_empty()
        && alumnos_filtrados
            .read()
            .iter()
            .all(|v| estado.read().seleccionados.contains(&v.alumno.id));
    let texto_boton = if todos_seleccionados {
        "Deseleccionar todos"
    } else {
        "Seleccionar todos"
    };

    rsx! {
        div { class: "flex flex-col h-full space-y-4 overflow-auto",

            // ── Encabezado ──
            div { class: "flex items-center justify-between py-1",
                div {
                    h2 { class: "text-3xl font-bold text-gray-800", "🥋 Alumnos" }
                    p { class: "text-gray-500 text-sm mt-1",
                        "Consulta, registro y edición de karatekas."
                    }
                }
                button {
                    class: "px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors text-sm",
                    onclick: move |_| {
                        estado.write().toggle_all(alumnos_filtrados.read().clone());
                    },
                    "{texto_boton}"
                }
            }

            // ── Barra superior: búsqueda + filtro ──
            div { class: "grid grid-cols-2 gap-4",
                SearchBar {
                    on_input: move |data| busqueda.set(data),
                    options: vec![
                        ("Id".to_string(), Columnas::Id),
                        ("Nombre".to_string(), Columnas::Nombre),
                        ("Representante".to_string(), Columnas::Representante),
                        ("Teléfono".to_string(), Columnas::Telefono),
                    ],
                    placeholder: "Buscar alumno...".to_string(),
                    initial_param: Columnas::Nombre,
                }
                Filter {
                    on_input: move |data| filtro.set(data),
                    options: vec![
                        ("Cinta".to_string(), Columnas::Cinta),
                        ("Edad".to_string(), Columnas::Edad),
                    ],
                    placeholder: "Filtrar alumnos...".to_string(),
                    initial_param: Columnas::Cinta,
                }
            }

            // ── Tabla de datos ──
            DataTable {
                data: alumnos_filtrados,
                header_columns: ALUMNO_COLUMNS,
                row_key: RowKeyFn(alumno_key),
                render_row: RenderRowFn(render_alumno_row),
                estado,
                aplicar_color_seleccion: true,
                single_select: false,
                checkbox: true,
                on_doble_click: move |_| modal_activo.set(Some(ModalAlumno::Editar)),
            }

            div { class: "flex justify-between items-center",
                div { class: "text-gray-500 text-xs",
                    "Mostrando {alumnos_filtrados.read().len()} alumnos en pantalla"
                }
                div { class: "text-gray-500 text-xs",
                    "alumnos seleccionados: {total_seleccionados}"
                }
            }

            // ── Acciones: botones que abren modales ──
            div { class: "flex flex-wrap justify-center gap-3 pt-1",
                button {
                    class: "px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Nuevo)),
                    "＋ Nuevo Alumno"
                }
                button {
                    class: if uno_seleccionado {
                        "px-5 py-2.5 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-700 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !uno_seleccionado,
                    title: if uno_seleccionado { "Editar el alumno seleccionado" } else { "Seleccione exactamente 1 alumno" },
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Editar)),
                    "✏️ Editar"
                }
                button {
                    class: if hay_seleccion {
                        "px-5 py-2.5 bg-purple-600 hover:bg-purple-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-700 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !hay_seleccion,
                    title: if hay_seleccion { "Promover a los seleccionados" } else { "Seleccione al menos 1 alumno" },
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Promover)),
                    "🥋 Promover"
                }
                button {
                    class: if hay_seleccion {
                        "px-5 py-2.5 bg-red-600 hover:bg-red-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-700 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !hay_seleccion,
                    title: if hay_seleccion { "Eliminar a los seleccionados" } else { "Seleccione al menos 1 alumno" },
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Eliminar)),
                    "🗑️ Eliminar"
                }
                button {
                    class: "px-5 py-2.5 bg-blue-800 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::RegistrarRepresentante)),
                    "👤 Registrar Representante"
                }
            }
        }

        // ═══ Modales de acción ═══
        match modal_activo.read().clone() {
            Some(ModalAlumno::Nuevo) => rsx! { ModalNuevo { on_cerrar: move |_| modal_activo.set(None) } },
            Some(ModalAlumno::Editar) => rsx! { ModalEditar { on_cerrar: move |_| modal_activo.set(None) } },
            Some(ModalAlumno::Promover) => rsx! { ModalPromover { on_cerrar: move |_| modal_activo.set(None) } },
            Some(ModalAlumno::Eliminar) => rsx! { ModalEliminar { on_cerrar: move |_| modal_activo.set(None) } },
            Some(ModalAlumno::RegistrarRepresentante) => rsx! { ModalRegistrarRepresentante { on_cerrar: move |_| modal_activo.set(None) } },
            None => rsx! {},
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Modales de Alumnos
// ═══════════════════════════════════════════════════════════════

/// Contenedor común de modales.
#[component]
fn ModalBase(titulo: String, on_cerrar: EventHandler<()>, children: Element) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
            onclick: move |_| on_cerrar.call(()),
            div {
                class: "bg-gray-800 rounded-xl shadow-2xl border border-gray-700 p-6 w-full max-w-2xl space-y-4 mx-4 max-h-[90vh] overflow-auto",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex items-center justify-between",
                    h3 { class: "text-lg font-bold text-white", "{titulo}" }
                    button {
                        class: "text-gray-400 hover:text-white text-xl leading-none cursor-pointer",
                        onclick: move |_| on_cerrar.call(()),
                        "✕"
                    }
                }

                {children}
            }
        }
    }
}

#[component]
fn ModalNuevo(on_cerrar: EventHandler<()>) -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let nombre = use_signal(|| "".to_string());
    let fecha_nac = use_signal(|| "".to_string());
    let rango = use_signal(|| 10i32);
    let representante_id = use_signal(|| 0usize);
    let rallita = use_signal(|| false);

    let fecha_valida = es_fecha_valida_form(&fecha_nac.read());
    let formulario_valido = (
        !nombre.read().is_empty(),
        fecha_valida,
        representante_asignado(*representante_id.read()),
    );

    rsx! {
        ModalBase { titulo: "＋ Nuevo Alumno".to_string(), on_cerrar,
            Form {
                nombre, fecha_nac, rango, representante_id, rallita,
                representantes: estado.read().representantes.clone(),
                campos_validos: formulario_valido,
                texto_boton: "Guardar",
                on_click: move |_| {
                    let datos = DatosAlumno {
                        nombre: nombre.read().clone(),
                        fecha_de_nacimiento: fecha_nac.read().clone(),
                        rango: *rango.read(),
                        representante_id: *representante_id.read(),
                        rallita: *rallita.read(),
                    };
                    let _ = estado.write().agregar_alumno(datos);
                    on_cerrar.call(());
                },
            }
        }
    }
}

#[component]
fn ModalEditar(on_cerrar: EventHandler<()>) -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut nombre = use_signal(|| "".to_string());
    let mut fecha_nac = use_signal(|| "".to_string());
    let mut rango = use_signal(|| 10i32);
    let mut representante_id = use_signal(|| 0usize);
    let mut rallita = use_signal(|| false);

    {
        let estado = estado.clone();
        use_effect(move || {
            let app = estado.read();
            if let Some(id) = app.seleccionados.iter().copied().next() {
                let alumno = app.get_alumno_by_id(id);
                nombre.set(alumno.nombre.clone());
                fecha_nac.set(alumno.fecha_de_nacimiento.clone());
                rango.set(alumno.rango);
                representante_id.set(alumno.representante_id);
                rallita.set(alumno.rallita);
            }
        });
    }

    let id = estado.read().seleccionados.iter().copied().next();
    let fecha_valida = es_fecha_valida_form(&fecha_nac.read());
    let formulario_valido = (
        !nombre.read().is_empty(),
        fecha_valida,
        representante_asignado(*representante_id.read()),
    );

    rsx! {
        ModalBase { titulo: "✏️ Editar Alumno".to_string(), on_cerrar,
            Form {
                nombre, fecha_nac, rango, representante_id, rallita,
                representantes: estado.read().representantes.clone(),
                campos_validos: formulario_valido,
                texto_boton: "Guardar",
                on_click: move |_| {
                    if let Some(id) = id {
                        let datos = DatosAlumno {
                            nombre: nombre.read().clone(),
                            fecha_de_nacimiento: fecha_nac.read().clone(),
                            rango: *rango.read(),
                            representante_id: *representante_id.read(),
                            rallita: *rallita.read(),
                        };
                        let _ = estado.write().actualizar_alumno(id, datos);
                    }
                    on_cerrar.call(());
                },
            }
        }
    }
}

#[component]
fn ModalPromover(on_cerrar: EventHandler<()>) -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let rango = use_signal(|| 99i32);
    let rallita = use_signal(|| false);

    let nombres: Vec<String> = {
        let app = estado.read();
        app.alumnos
            .iter()
            .filter(|v| app.seleccionados.contains(&v.alumno.id))
            .map(|v| v.alumno.nombre.clone())
            .collect()
    };

    rsx! {
        ModalBase { titulo: "🥋 Promover en masa".to_string(), on_cerrar,
            div { class: "space-y-3",
                p { class: "text-sm text-gray-400",
                    "Se aplicará el nuevo grado a {nombres.len()} alumno(s) seleccionado(s):"
                }
                div { class: "flex flex-wrap gap-2",
                    {nombres.iter().map(|n| rsx! {
                        span { class: "px-2 py-1 rounded bg-gray-900 border border-gray-700 text-xs text-gray-300",
                            "{n}"
                        }
                    })}
                }

                PromotionForm {
                    rango, rallita,
                    texto_boton: "Aplicar cambios",
                    on_click: move |_| {
                        let _ = estado.write().promover_seleccionados(*rango.read(), *rallita.read());
                        on_cerrar.call(());
                    },
                }
            }
        }
    }
}

#[component]
fn ModalEliminar(on_cerrar: EventHandler<()>) -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let seleccionados: Vec<AlumnoVista> = {
        let app = estado.read();
        app.alumnos
            .iter()
            .filter(|v| app.seleccionados.contains(&v.alumno.id))
            .cloned()
            .collect()
    };

    rsx! {
        ModalBase { titulo: "🗑️ Eliminar Alumnos".to_string(), on_cerrar,
            div { class: "space-y-4",
                p { class: "text-sm text-gray-400",
                    "Se eliminarán {seleccionados.len()} alumno(s). Esta acción no se puede deshacer."
                }
                div { class: "max-h-60 overflow-auto rounded-lg border border-gray-700 divide-y divide-gray-700",
                    {seleccionados.iter().map(|v| rsx! {
                        div {
                            key: "{v.alumno.id}",
                            class: "px-3 py-2 text-sm text-gray-200 bg-gray-900",
                            span { class: "font-bold", "{v.alumno.nombre}" }
                            span { class: "text-gray-500 text-xs ml-2", "#{v.alumno.id}" }
                        }
                    })}
                }
                button {
                    class: "w-full py-3 bg-red-600 text-white rounded-xl font-bold hover:bg-red-700 active:scale-[0.98] transition-all cursor-pointer",
                    onclick: move |_| {
                        let _ = estado.write().eliminar_seleccionados();
                        on_cerrar.call(());
                    },
                    "Eliminar definitivamente"
                }
            }
        }
    }
}

#[component]
fn ModalRegistrarRepresentante(on_cerrar: EventHandler<()>) -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut nombre = use_signal(String::new);
    let mut contacto = use_signal(String::new);
    let mut msg = use_signal(|| (String::new(), false));

    let formulario_ok = nombre_valido(&nombre.read()) && contacto_valido(&contacto.read());

    rsx! {
        ModalBase { titulo: "Registrar Representante".to_string(), on_cerrar,
            div { class: "space-y-4",
                div { class: "flex flex-col space-y-1",
                    label { class: "text-sm font-semibold text-gray-400", "Nombre completo" }
                    input {
                        r#type: "text",
                        class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                        placeholder: "Ej: Maria Garcia",
                        value: "{nombre}",
                        oninput: move |e| nombre.set(e.value())
                    }
                }
                div { class: "flex flex-col space-y-1",
                    label { class: "text-sm font-semibold text-gray-400", "Telefono de contacto" }
                    input {
                        r#type: "tel",
                        maxlength: "12",
                        class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                        placeholder: "0412-0000000",
                        value: "{contacto}",
                        oninput: move |e| {
                            let mut val = e.value();
                            val.retain(|c| c.is_ascii_digit());
                            if val.len() > 4 { val.insert(4, '-'); }
                            val.truncate(12);
                            contacto.set(val);
                        }
                    }
                }
                if !msg.read().0.is_empty() {
                    p { class: if msg.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" },
                        "{msg.read().0}" }
                }
                button {
                    class: if formulario_ok {
                        "w-full py-3 bg-blue-600 text-white font-bold rounded-xl hover:bg-blue-700 active:scale-[0.98] transition-all cursor-pointer"
                    } else {
                        "w-full py-3 bg-gray-700 text-gray-400 font-bold rounded-xl cursor-pointer"
                    },
                    disabled: !formulario_ok,
                    onclick: move |_| {
                        let datos = DatosRepresentante {
                            nombre: nombre.read().clone(),
                            numero_contacto: contacto.read().clone(),
                        };
                        match estado.write().agregar_representante(datos) {
                            Ok(()) => {
                                nombre.set(String::new());
                                contacto.set(String::new());
                                msg.set(("Representante registrado correctamente".to_string(), false));
                            }
                            Err(e) => msg.set((e.to_string(), true)),
                        }
                    },
                    "Guardar representante"
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers del panel de Pagos
// ═══════════════════════════════════════════════════════════════

/// Texto compacto para montos.
fn fmt_monto(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if (r - r.trunc()).abs() < f64::EPSILON {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// (etiqueta, clases) según el estado de la deuda.
fn badge_estado_deuda(estado: &EstadoDeuda) -> (&'static str, &'static str) {
    match estado {
        EstadoDeuda::Pagada => ("Pagado", "bg-emerald-900 text-emerald-300"),
        EstadoDeuda::Parcial => ("Parcial", "bg-amber-900 text-amber-300"),
        EstadoDeuda::Pendiente => ("Pendiente", "bg-red-900 text-red-300"),
        EstadoDeuda::Anticipada => ("Anticipada", "bg-blue-900 text-blue-300"),
        EstadoDeuda::Anulada => ("Anulada", "bg-gray-800 text-gray-400"),
    }
}

/// Color de la barra de progreso según el estado.
fn clase_barra(estado: &EstadoDeuda) -> &'static str {
    match estado {
        EstadoDeuda::Pagada => "bg-emerald-500",
        EstadoDeuda::Parcial => "bg-amber-500",
        EstadoDeuda::Pendiente => "bg-red-500",
        EstadoDeuda::Anticipada => "bg-blue-500",
        EstadoDeuda::Anulada => "bg-gray-500",
    }
}

/// Etiqueta de un método de pago para selects.
fn metodo_label(m: &MetodoPago) -> &'static str {
    match m {
        MetodoPago::Efectivo => "Efectivo",
        MetodoPago::Transferencia => "Transferencia",
        MetodoPago::Tarjeta => "Tarjeta",
        MetodoPago::Cheque => "Cheque",
    }
}

// =====================================================
// Panel de Pagos — pestañas Consulta / Historial
// =====================================================

/// Etiqueta legible de un tipo de historial.
fn tipo_historial_label(tipo_id: i32) -> &'static str {
    match tipo_id {
        1 => "Deuda Creada",
        2 => "Pago Registrado",
        3 => "Abono Aplicado",
        4 => "Ajuste Manual",
        5 => "Anulación",
        _ => "Otro",
    }
}

/// Color del badge según tipo de historial.
fn tipo_historial_clase(tipo_id: i32) -> &'static str {
    match tipo_id {
        1 => "bg-blue-900 text-blue-300",
        2 => "bg-emerald-900 text-emerald-300",
        3 => "bg-amber-900 text-amber-300",
        4 => "bg-purple-900 text-purple-300",
        5 => "bg-red-900 text-red-300",
        _ => "bg-gray-800 text-gray-400",
    }
}

#[component]
pub fn Pagos() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut pestana = use_signal(|| "consulta".to_string());

    rsx! {
        div { class: "flex flex-col h-full space-y-0",

            // -- Barra de pestanas --
            div { class: "flex border-b border-gray-700 bg-gray-900",
                button {
                    class: if *pestana.read() == "consulta" {
                        "px-5 py-2.5 text-sm font-bold text-blue-400 border-b-2 border-blue-400 transition-colors cursor-pointer"
                    } else {
                        "px-5 py-2.5 text-sm font-medium text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
                    },
                    onclick: move |_| pestana.set("consulta".to_string()),
                    "Consulta"
                }
                button {
                    class: if *pestana.read() == "historial" {
                        "px-5 py-2.5 text-sm font-bold text-blue-400 border-b-2 border-blue-400 transition-colors cursor-pointer"
                    } else {
                        "px-5 py-2.5 text-sm font-medium text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
                    },
                    onclick: move |_| {
                        let rep_id = estado.read().representante_historial_id;
                        estado.write().refrescar_historial(rep_id);
                        pestana.set("historial".to_string());
                    },
                    "Historial"
                }
            }

            // -- Contenido de pestanas --
            div { class: "flex-1 overflow-auto",
                if *pestana.read() == "consulta" {
                    ConsultaTab {}
                } else {
                    HistorialTab {}
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct DeudaRow {
    vista: DeudaVista,
    ultimo: Option<PagoVista>,
    puede_reversar: bool,
    ultimo_id: usize,
}

static DEUDAS_COLUMNS: &[HeaderColumn] = &[
    HeaderColumn { header: "Representante", class: Some("") },
    HeaderColumn { header: "Mensualidad", class: Some("text-right") },
    HeaderColumn { header: "Abonado", class: Some("text-right") },
    HeaderColumn { header: "Progreso", class: Some("") },
    HeaderColumn { header: "Saldo", class: Some("text-right") },
    HeaderColumn { header: "Estado", class: Some("text-center") },
    HeaderColumn { header: "Ultimo pago", class: Some("text-right") },
    HeaderColumn { header: "Accion", class: Some("text-center") },
];

fn deuda_key(row: &DeudaRow) -> usize {
    row.vista.deuda.representante_id
}

fn render_deuda_row(vista: &DeudaRow, mut estado: Signal<my_app::MyApp>) -> Element {
    let ultimo_id = vista.ultimo_id;
    let saldo_texto = fmt_monto(vista.vista.deuda.saldo());
    let pct = vista.vista.deuda.porcentaje();
    let (etiqueta, clases) = badge_estado_deuda(&vista.vista.estado);
    rsx! {
        td { class: "px-3 py-2",
            p { class: "font-medium text-white truncate max-w-40", "{vista.vista.nombre_representante}" }
            p { class: "text-[10px] text-gray-500 font-mono", "{vista.vista.telefono_representante}" }
        }
        td { class: "px-3 py-2 text-right font-mono text-gray-300", "{fmt_monto(vista.vista.deuda.monto_total)}" }
        td { class: "px-3 py-2 text-right font-mono text-emerald-400", "{fmt_monto(vista.vista.deuda.total_abonado())}" }
        td { class: "px-3 py-2",
            div { class: "h-1.5 w-full bg-gray-700 rounded-full overflow-hidden",
                div { class: "h-full rounded-full {clase_barra(&vista.vista.estado)}", style: "width:{pct:.0}%" }
            }
            p { class: "text-[9px] text-gray-500 mt-0.5", "{pct:.0}%" }
        }
        td { class: "px-3 py-2 text-right font-mono font-bold text-amber-400", "{saldo_texto}" }
        td { class: "px-3 py-2 text-center",
            span { class: "inline-block px-1.5 py-0.5 rounded-full text-[9px] font-bold {clases}", "{etiqueta}" }
        }
        td { class: "px-3 py-2 text-gray-400 text-[10px]",
            if let Some(pv) = &vista.ultimo {
                span {
                    class: if pv.estado == EstadoPago::Completado { "text-emerald-400" } else { "text-gray-500" },
                    "{fmt_monto(pv.pago.monto_recibido)}"
                }
                span { class: "text-gray-600 mx-0.5", "/" }
                span { class: "text-gray-500", "{metodo_label(&pv.metodo)}" }
            } else {
                span { class: "text-gray-600", "-" }
            }
        }
        td { class: "px-3 py-2 text-center",
            if vista.puede_reversar {
                button {
                    class: "px-1.5 py-0.5 rounded bg-gray-800 border border-gray-700 text-red-400 hover:text-white hover:border-red-500 font-bold text-[9px] transition-colors cursor-pointer",
                        onclick: move |_| {
                            let _ = estado.write().reversar_pago(ultimo_id);
                        },
                    "Revertir"
                }
            }
        }
    }
}

static HISTORIAL_COLUMNS: &[HeaderColumn] = &[
    HeaderColumn { header: "Fecha", class: Some("") },
    HeaderColumn { header: "Representante", class: Some("") },
    HeaderColumn { header: "Tipo", class: Some("text-center") },
    HeaderColumn { header: "Monto", class: Some("text-right") },
    HeaderColumn { header: "Periodo", class: Some("") },
    HeaderColumn { header: "Detalle", class: Some("") },
];

fn historial_key(row: &HistorialPagoVista) -> usize {
    row.historial.id
}

fn render_historial_row(vista: &HistorialPagoVista, _estado: Signal<my_app::MyApp>) -> Element {
    let tipo_label = tipo_historial_label(vista.historial.tipo_id);
    let tipo_clase = tipo_historial_clase(vista.historial.tipo_id);
    rsx! {
        td { class: "px-3 py-2 font-mono text-gray-400", "{vista.historial.fecha}" }
        td { class: "px-3 py-2",
            p { class: "font-medium text-white truncate max-w-32", "{vista.nombre_representante}" }
        }
        td { class: "px-3 py-2 text-center",
            span { class: "inline-block px-1.5 py-0.5 rounded-full text-[9px] font-bold {tipo_clase}", "{tipo_label}" }
        }
        td { class: "px-3 py-2 text-right font-mono font-bold text-gray-200",
            if vista.historial.monto > 0.0 {
                "{fmt_monto(vista.historial.monto)}"
            } else {
                span { class: "text-gray-600", "-" }
            }
        }
        td { class: "px-3 py-2 font-mono text-gray-500 text-[10px]", "{vista.historial.periodo}" }
        td { class: "px-3 py-2 text-gray-400 text-[10px] truncate max-w-48",
            "{vista.historial.observacion}"
        }
    }
}

// =====================================================
// Pestaña Consulta (vista de deudas/pagos)
// =====================================================

#[component]
fn ConsultaTab() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    let mut busqueda = use_signal(String::new);
    let mut filtro_estado = use_signal(String::new);

    let mut modal_pago = use_signal(|| false);
    let mut pago_representante_id = use_signal(|| 0usize);
    let mut pago_monto = use_signal(String::new);
    let mut pago_metodo_id = use_signal(|| 1i32);
    let mut pago_msg = use_signal(|| (String::new(), false));
    let mut msg_accion = use_signal(|| (String::new(), false));

    let (etiqueta_mes, total_deudas, total_abonado, pagados, parciales, pendientes) = {
        let e = estado.read();
        (
            e.etiqueta_periodo_actual(),
            e.total_deudas_periodo(),
            e.total_abonado_periodo(),
            e.reps_pagados(),
            e.reps_parciales(),
            e.reps_pendientes(),
        )
    };
    let por_cobrar = (total_deudas - total_abonado).max(0.0);

    let all_deudas = estado.read().deudas.clone();
    let all_pagos = estado.read().pagos.clone();
    let representantes = estado.read().representantes.clone();
    let hay_pendientes = all_deudas.iter().any(|v| v.estado != EstadoDeuda::Pagada);

    let pct_global = if total_deudas > 0.0 {
        (total_abonado / total_deudas * 100.0).min(100.0)
    } else { 0.0 };
    let texto_avance = if total_deudas > 0.0 {
        format!("{pct_global:.0}% recaudado")
    } else { "Sin deudas".to_string() };
    let clase_avance = if total_deudas > 0.0 && pendientes == 0 {
        clase_barra(&EstadoDeuda::Pagada)
    } else if total_deudas > 0.0 && pagados == 0 {
        clase_barra(&EstadoDeuda::Pendiente)
    } else {
        clase_barra(&EstadoDeuda::Parcial)
    };

    let mut ultimo_pago_por_rep: std::collections::HashMap<usize, &PagoVista> = std::collections::HashMap::new();
    for pv in all_pagos.iter().rev() {
        ultimo_pago_por_rep.entry(pv.pago.representante_id).or_insert(pv);
    }

    let q = busqueda.read().to_lowercase();
    let filtro = filtro_estado.read().clone();
    let deudas: Vec<_> = all_deudas.iter().filter(|v| {
        let match_nombre = q.is_empty() || v.nombre_representante.to_lowercase().contains(&q)
            || v.telefono_representante.contains(&q);
        let match_estado = filtro.is_empty() || match filtro.as_str() {
            "Pendiente" => v.estado == EstadoDeuda::Pendiente,
            "Parcial" => v.estado == EstadoDeuda::Parcial,
            "Pagada" => v.estado == EstadoDeuda::Pagada,
            "Anticipada" => v.estado == EstadoDeuda::Anticipada,
            "Anulada" => v.estado == EstadoDeuda::Anulada,
            _ => true,
        };
        match_nombre && match_estado
    }).cloned().collect();

    let deudas_rows: Vec<DeudaRow> = deudas.iter().map(|v| {
        let ultimo = ultimo_pago_por_rep.get(&v.deuda.representante_id).cloned();
        let puede_reversar = ultimo.map_or(false, |p| p.estado == EstadoPago::Completado);
        let ultimo_id = ultimo.map_or(0, |p| p.pago.id);
        DeudaRow { vista: v.clone(), ultimo: ultimo.cloned(), puede_reversar, ultimo_id }
    }).collect();

    {
        let mut estado = estado.clone();
        use_effect(move || {
            let seleccionados: Vec<usize> = estado.read().seleccionados.iter().copied().collect();
            let rep_id = if seleccionados.len() == 1 {
                Some(seleccionados[0])
            } else {
                None
            };
            let actual = estado.read().representante_historial_id;
            if rep_id != actual {
                estado.write().seleccionar_rep_historial(rep_id);
            }
        });
    }

    rsx! {
        div { class: "flex flex-col h-full space-y-3 p-4 overflow-auto",

            // -- Encabezado --
            div { class: "flex items-center justify-between py-1",
                div {
                    h2 { class: "text-2xl font-bold text-gray-800", "Panel de Pagos" }
                    p { class: "text-gray-500 text-xs mt-0.5",
                        "Deudas y pagos del mes."
                    }
                }
                span { class: "px-2.5 py-0.5 rounded-full bg-gray-200 text-gray-700 text-[11px] font-bold tracking-widest uppercase whitespace-nowrap",
                    "{etiqueta_mes}"
                }
            }

            // -- Estadisticas --
            div { class: "grid grid-cols-4 gap-3",
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Deudas" }
                    p { class: "text-xl font-bold text-gray-100 mt-0.5", {fmt_monto(total_deudas)} }
                    p { class: "text-[10px] text-gray-500", "{all_deudas.len()} registros" }
                }
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Recaudado" }
                    p { class: "text-xl font-bold text-emerald-400 mt-0.5", {fmt_monto(total_abonado)} }
                    p { class: "text-[10px] text-gray-500", "{pagados} al dia" }
                }
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Por cobrar" }
                    p { class: "text-xl font-bold text-amber-400 mt-0.5", {fmt_monto(por_cobrar)} }
                    p { class: "text-[10px] text-gray-500", "{parciales} parcial - {pendientes} pend." }
                }
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Avance" }
                    div { class: "mt-2 h-1.5 w-full bg-gray-700 rounded-full overflow-hidden",
                        div { class: "h-full rounded-full {clase_avance}", style: "width:{pct_global:.0}%" }
                    }
                    p { class: "text-[10px] text-gray-500 mt-0.5", "{texto_avance}" }
                }
            }

            // -- Busqueda + Filtro --
            div { class: "flex items-center gap-3",
                div { class: "flex-1 relative",
                    input {
                        r#type: "text",
                        class: "w-full p-2 pl-8 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50 text-xs",
                        placeholder: "Buscar por representante o telefono...",
                        value: "{busqueda}",
                        oninput: move |e| busqueda.set(e.value())
                    }
                    span { class: "absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500 text-xs", "\u{1F50D}" }
                }
                select {
                    class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50 text-xs",
                    value: "{filtro_estado}",
                    onchange: move |e| filtro_estado.set(e.value()),
                    option { class: "bg-gray-900", value: "", "Todos los estados" }
                    option { class: "bg-gray-900", value: "Pendiente", "Pendiente" }
                    option { class: "bg-gray-900", value: "Parcial", "Parcial" }
                    option { class: "bg-gray-900", value: "Pagada", "Pagada" }
                    option { class: "bg-gray-900", value: "Anticipada", "Anticipada" }
                }
                div { class: "flex items-center gap-2 text-[10px] text-gray-500",
                    span { class: "flex items-center gap-1", i { class: "inline-block w-1.5 h-1.5 rounded-full bg-emerald-500" } "Pagado" }
                    span { class: "flex items-center gap-1", i { class: "inline-block w-1.5 h-1.5 rounded-full bg-amber-500" } "Parcial" }
                    span { class: "flex items-center gap-1", i { class: "inline-block w-1.5 h-1.5 rounded-full bg-red-500" } "Pendiente" }
                }
            }

            // -- Tabla unificada --
            DataTable {
                data: Signal::new(deudas_rows),
                header_columns: DEUDAS_COLUMNS,
                row_key: RowKeyFn(deuda_key),
                render_row: RenderRowFn(render_deuda_row),
                estado,
                aplicar_color_seleccion: false,
                single_select: true,
                checkbox: true,
                on_doble_click: move |_| {},
            }

            // -- Info --
            div { class: "flex justify-between items-center",
                div { class: "text-gray-500 text-xs",
                    "Mostrando {deudas.len()} de {all_deudas.len()} deudas"
                }
                if !msg_accion.read().0.is_empty() {
                    span {
                        class: if msg_accion.read().1 { "text-[11px] text-red-400" } else { "text-[11px] text-emerald-400" },
                        "{msg_accion.read().0}"
                    }
                }
            }

            // -- Acciones --
            div { class: "flex justify-center gap-3 pt-1",
                button {
                    class: "px-5 py-2.5 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                    onclick: move |_| {
                        match estado.write().crear_deudas_del_mes() {
                            Ok(creadas) => {
                                if creadas == 0 {
                                    msg_accion.set(("Todos ya tienen deuda este mes.".to_string(), false));
                                } else {
                                    msg_accion.set((format!("Se crearon {creadas} deudas."), false));
                                }
                            }
                            Err(error) => msg_accion.set((error.to_string(), true)),
                        }
                    },
                    "＋ Crear deudas del mes"
                }
                button {
                    class: if hay_pendientes {
                        "px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-800 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !hay_pendientes,
                    onclick: move |_| {
                        pago_monto.set(String::new());
                        pago_representante_id.set(0);
                        pago_metodo_id.set(1);
                        pago_msg.set((String::new(), false));
                        modal_pago.set(true);
                    },
                    "💰 Registrar pago"
                }
            }
        }

        // Modal: registrar pago (FIFO)
        if *modal_pago.read() {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
                onclick: move |_| modal_pago.set(false),
                div {
                    class: "bg-gray-800 rounded-xl shadow-2xl border border-gray-700 p-5 w-full max-w-sm space-y-3 mx-4",
                    onclick: move |e| e.stop_propagation(),

                    div { class: "flex items-center justify-between",
                        h3 { class: "text-base font-bold text-white", "Registrar pago" }
                        button {
                            class: "text-gray-400 hover:text-white text-lg leading-none cursor-pointer",
                            onclick: move |_| modal_pago.set(false),
                            "✕"
                        }
                    }

                    p { class: "text-[10px] text-gray-400",
                        "El monto se aplica a la deuda mas antigua (FIFO). Si sobra, crea adelantos."
                    }

                    div { class: "flex flex-col space-y-0.5",
                        label { class: "text-[10px] font-semibold text-gray-400", "Representante" }
                        select {
                            class: "p-1.5 rounded bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-1 focus:ring-blue-500/50 text-xs",
                            value: "{pago_representante_id}",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<usize>() {
                                    pago_representante_id.set(id);
                                    pago_msg.set((String::new(), false));
                                }
                            },
                            option { class: "bg-gray-900", value: "0", "-- Seleccione --" }
                            {representantes.iter().map(|r| rsx! {
                                option {
                                    key: "{r.id}",
                                    class: "bg-gray-900", value: "{r.id}",
                                    "{r.nombre}"
                                }
                            })}
                        }
                    }

                    div { class: "flex flex-col space-y-0.5",
                        label { class: "text-[10px] font-semibold text-gray-400", "Monto recibido" }
                        input {
                            r#type: "text",
                            class: "p-1.5 rounded bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-1 focus:ring-blue-500/50 text-xs",
                            placeholder: "Ej: 1500",
                            value: "{pago_monto}",
                            oninput: move |e| {
                                pago_monto.set(e.value());
                                pago_msg.set((String::new(), false));
                            }
                        }
                    }

                    div { class: "flex flex-col space-y-0.5",
                        label { class: "text-[10px] font-semibold text-gray-400", "Metodo de pago" }
                        select {
                            class: "p-1.5 rounded bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-1 focus:ring-blue-500/50 text-xs",
                            value: "{pago_metodo_id}",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<i32>() {
                                    pago_metodo_id.set(id);
                                }
                            },
                            option { class: "bg-gray-900", value: "1", "Efectivo" }
                            option { class: "bg-gray-900", value: "2", "Transferencia" }
                            option { class: "bg-gray-900", value: "3", "Tarjeta" }
                            option { class: "bg-gray-900", value: "4", "Cheque" }
                        }
                    }

                    if !pago_msg.read().0.is_empty() {
                        p {
                            class: if pago_msg.read().1 { "text-[10px] text-red-400" } else { "text-[10px] text-emerald-400" },
                            "{pago_msg.read().0}"
                        }
                    }

                    div { class: "flex gap-2 justify-end pt-1",
                        button {
                            class: "px-3 py-1.5 text-[11px] text-gray-400 hover:text-white transition-colors cursor-pointer",
                            onclick: move |_| modal_pago.set(false),
                            "Cancelar"
                        }
                        button {
                            class: "px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-xs",
                            onclick: move |_| {
                                let rep_id = *pago_representante_id.read();
                                if rep_id == 0 {
                                    pago_msg.set(("Seleccione un representante.".to_string(), true));
                                    return;
                                }
                                let monto = pago_monto.read().trim().replace(',', ".").parse::<f64>().unwrap_or(0.0);
                                if monto <= 0.0 {
                                    pago_msg.set(("El monto debe ser mayor a cero.".to_string(), true));
                                    return;
                                }
                                let metodo = *pago_metodo_id.read();
                                let fecha = Local::now().format("%Y-%m-%d").to_string();
                                let datos = DatosPago {
                                    representante_id: rep_id,
                                    monto_recibido: monto,
                                    metodo_id: metodo,
                                    fecha_pago: fecha,
                                };
                                match estado.write().registrar_pago(datos) {
                                    Ok(()) => {
                                        modal_pago.set(false);
                                        pago_monto.set(String::new());
                                        pago_representante_id.set(0);
                                        pago_msg.set(("Pago registrado.".to_string(), false));
                                    }
                                    Err(error) => pago_msg.set((error.to_string(), true)),
                                }
                            },
                            "Registrar"
                        }
                    }
                }
            }
        }
    }
}

// =====================================================
// Pestaña Historial
// =====================================================

#[component]
fn HistorialTab() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut busqueda = use_signal(String::new);
    let mut cargado = use_signal(|| false);

    // Cargar historial al montar la primera vez
    if !*cargado.read() {
        let rep_id = estado.read().representante_historial_id;
        estado.write().refrescar_historial(rep_id);
        cargado.set(true);
    }

    let historial = estado.read().historial_pagos.clone();
    let rep_label = {
        let app = estado.read();
        match app.representante_historial_id {
            Some(id) => app.representantes.iter().find(|r| r.id == id).map(|r| format!("{} ({})", r.nombre, r.numero_contacto)).unwrap_or_else(|| "Desconocido".to_string()),
            None => "Todos los representantes".to_string(),
        }
    };

    let q = busqueda.read().to_lowercase();
    let filtrado: Vec<_> = historial.iter().filter(|v| {
        if q.is_empty() { return true; }
        v.nombre_representante.to_lowercase().contains(&q)
            || v.historial.observacion.to_lowercase().contains(&q)
            || v.historial.periodo.contains(&q)
    }).cloned().collect();

    let total_monto: f64 = filtrado.iter().map(|v| v.historial.monto).sum();
    let num_pagos = filtrado.iter().filter(|v| v.historial.tipo_id == 2).count();
    let num_adeudas = filtrado.iter().filter(|v| v.historial.tipo_id == 1).count();
    let num_anulaciones = filtrado.iter().filter(|v| v.historial.tipo_id == 5).count();

    rsx! {
        div { class: "flex flex-col h-full space-y-3 p-4 overflow-auto",

            // -- Encabezado --
            div { class: "flex items-center justify-between py-1",
                div {
                    h2 { class: "text-2xl font-bold text-gray-800", "Historial de Pagos" }
                    p { class: "text-gray-500 text-xs mt-0.5",
                        "Registro de todos los movimientos financieros."
                    }
                }
                span { class: "px-2.5 py-0.5 rounded-full bg-purple-100 text-purple-700 text-[11px] font-bold tracking-wider whitespace-nowrap",
                    "{rep_label}"
                }
            }

            // -- Stats del historial --
            div { class: "grid grid-cols-4 gap-3",
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Registros" }
                    p { class: "text-xl font-bold text-gray-100 mt-0.5", "{filtrado.len()}" }
                }
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Monto total" }
                    p { class: "text-xl font-bold text-emerald-400 mt-0.5", "{fmt_monto(total_monto)}" }
                }
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Pagos" }
                    p { class: "text-xl font-bold text-blue-400 mt-0.5", "{num_pagos}" }
                }
                div { class: "bg-gray-800 rounded-lg p-3 border border-gray-700",
                    p { class: "text-[10px] uppercase tracking-wider text-gray-400", "Deudas creadas" }
                    p { class: "text-xl font-bold text-amber-400 mt-0.5", "{num_adeudas}" }
                }
            }

            // -- Busqueda --
            div { class: "flex items-center gap-3",
                div { class: "flex-1 relative",
                    input {
                        r#type: "text",
                        class: "w-full p-2 pl-8 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50 text-xs",
                        placeholder: "Buscar en historial...",
                        value: "{busqueda}",
                        oninput: move |e| busqueda.set(e.value())
                    }
                    span { class: "absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500 text-xs", "\u{1F50D}" }
                }
            }

            // -- Tabla de historial --
            div { class: "overflow-auto rounded-xl border border-gray-800 bg-gray-900 shadow-xl flex-1",
                if filtrado.is_empty() {
                    div { class: "flex items-center justify-center h-full",
                        p { class: "text-gray-400 font-medium text-xs", "Sin registros de historial" }
                        p { class: "text-gray-500 text-[10px] mt-0.5",
                            if estado.read().representante_historial_id.is_some() {
                                "No hay movimientos para este representante."
                            } else {
                                "Los movimientos apareceran aqui cuando registres pagos o crees deudas."
                            }
                        }
                    }
                } else {
                    DataTable {
                        data: Signal::new(filtrado.clone()),
                        header_columns: HISTORIAL_COLUMNS,
                        row_key: RowKeyFn(historial_key),
                        render_row: RenderRowFn(render_historial_row),
                        estado,
                        aplicar_color_seleccion: false,
                        single_select: false,
                        checkbox: false,
                        on_doble_click: move |_| {},
                    }
                }
            }

            // -- Info --
            div { class: "flex justify-between items-center",
                div { class: "text-gray-500 text-xs",
                    "Mostrando {filtrado.len()} de {historial.len()} registros"
                }
                if num_anulaciones > 0 {
                    span { class: "text-[11px] text-red-400",
                        "{num_anulaciones} anulacion(es)"
                    }
                }
            }
        }
    }
}


// Panel de Ajustes
// ═══════════════════════════════════════════════════════════════

#[component]
pub fn Ajustes() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    let mut monto_texto = use_signal(|| {
        let monto = estado.read().monto_predeterminado;
        if monto > 0.0 { fmt_monto(monto) } else { String::new() }
    });
    let mut mensaje = use_signal(|| (String::new(), false));

    let ruta_bd = estado.read().ruta_bd.clone();
    let periodo = estado.read().etiqueta_periodo_actual();
    let version = env!("CARGO_PKG_VERSION");

    let monto_ok = monto_texto
        .read()
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map(monto_valido)
        .unwrap_or(false);

    rsx! {
        div { class: "flex flex-col h-full space-y-6 overflow-auto pr-1 max-w-3xl mx-auto",

            div { class: "flex items-end justify-between py-1",
                div {
                    h2 { class: "text-3xl font-bold text-gray-800", "⚙️ Panel de Ajustes" }
                    p { class: "text-gray-500 text-sm mt-1", "Configuración de la aplicación." }
                }
                span { class: "px-3 py-1 rounded-full bg-gray-200 text-gray-700 text-xs font-bold tracking-widest uppercase whitespace-nowrap",
                    "{periodo}"
                }
            }

            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-3",
                h3 { class: "font-bold text-white", "💵 Mensualidad" }
                p { class: "text-xs text-gray-400",
                    "Monto predeterminado: se usa para crear deudas automáticamente y para adelantos en el motor FIFO."
                }
                div { class: "flex items-end gap-2",
                    div { class: "flex flex-col space-y-1 w-56",
                        label { class: "text-xs font-semibold text-gray-400", "Monto predeterminado" }
                        input {
                            r#type: "text",
                            class: if monto_ok || monto_texto.read().is_empty() { "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50" } else { "p-2 rounded-lg bg-gray-900 text-gray-100 border border-red-500 outline-none ring-2 ring-red-500/30" },
                            placeholder: "Ej: 1500",
                            value: "{monto_texto}",
                            oninput: move |e| monto_texto.set(e.value())
                        }
                    }
                    button {
                        class: if monto_ok { "px-5 py-2 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm" } else { "px-5 py-2 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed text-sm" },
                        disabled: !monto_ok,
                        onclick: move |_| {
                            if !monto_ok { return; }
                            match estado.write().cambiar_monto_predeterminado(monto_texto.read().clone()) {
                                Ok(()) => mensaje.set(("Ajuste guardado".to_string(), false)),
                                Err(error) => mensaje.set((error.to_string(), true)),
                            }
                        },
                        "Guardar"
                    }
                }
                if !mensaje.read().0.is_empty() {
                    p { class: if mensaje.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" }, "{mensaje.read().0}" }
                }
            }

            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-2",
                h3 { class: "font-bold text-white", "ℹ️ Información del sistema" }
                div { class: "flex justify-between items-center px-1 py-2 text-sm border-b border-gray-700",
                    span { class: "text-gray-400", "Versión" }
                    span { class: "text-gray-200 font-mono", "BudoDB v{version}" }
                }
                div { class: "flex justify-between items-center px-1 py-2 text-sm border-b border-gray-700/60",
                    span { class: "text-gray-400", "Periodo administrado" }
                    span { class: "text-gray-200", "{periodo}" }
                }
                div { class: "flex flex-col space-y-1 px-1 py-2 text-sm",
                    span { class: "text-gray-400", "Base de datos" }
                    span { class: "text-blue-400 font-mono text-xs break-all", "{ruta_bd}" }
                    p { class: "text-[10px] text-gray-500 mt-1",
                        "Se configura con la variable de entorno BUDODB_DB_PATH antes de abrir la aplicación."
                    }
                }
            }
        }
    }
}
