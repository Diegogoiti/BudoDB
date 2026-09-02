//! se encarga de dibujar las vistas segun las rutas seleccionadas
//! contiene las funciones con el codigo especifico de cada vista

use crate::application::dto::{AlumnoVista, DatosAlumno, DatosPago, DatosRepresentante, DeudaVista, HistorialPagoVista, PagoVista};
use crate::application::validation::*;
use crate::domain::{EstadoDeuda, EstadoPago, MetodoPago, Representante};
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
    let mut filtro_estado = use_signal(|| "Activo".to_string());
    let mut alumnos_filtrados = use_signal(|| estado.read().alumnos.clone());

    {
        let estado = estado.clone();
        use_effect(move || {
            let app = estado.read();
            let (col_buscar, texto) = busqueda.read().clone();
            let (col_filtro, valor_filtro, solo_rallita) = filtro.read().clone();
            let estado_filtro = filtro_estado.read().clone();
            let base = app.buscar_alumnos(col_buscar, &texto);
            let filtrados = app.filtrar_lista(base, col_filtro, valor_filtro, solo_rallita);
            let resultado: Vec<_> = filtrados.into_iter().filter(|v| {
                match estado_filtro.as_str() {
                    "Activo" => v.alumno.estado_id == 1,
                    "Inactivo" => v.alumno.estado_id == 2,
                    _ => true,
                }
            }).collect();
            alumnos_filtrados.set(resultado);
        });
    }

    let total_seleccionados = estado.read().seleccion_alumnos.len();
    let hay_seleccion = total_seleccionados > 0;
    let uno_seleccionado = total_seleccionados == 1;

    let todos_seleccionados = !alumnos_filtrados.read().is_empty()
        && alumnos_filtrados
            .read()
            .iter()
            .all(|v| estado.read().seleccion_alumnos.contains(&v.alumno.id));
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
            div { class: "flex flex-wrap justify-between items-center gap-4 px-1 py-2",
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
                select {
                    class: "p-2 rounded-lg bg-gray-800 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50 text-xs",
                    value: "{filtro_estado}",
                    onchange: move |e| filtro_estado.set(e.value()),
                    option { class: "bg-gray-900", value: "Activo", "Activos" }
                    option { class: "bg-gray-900", value: "Inactivo", "Inactivos" }
                    option { class: "bg-gray-900", value: "Todos", "Todos" }
                }
            }

            // ── Tabla de datos ──
            DataTable {
                data: alumnos_filtrados.read().clone(),
                header_columns: ALUMNO_COLUMNS,
                row_key: RowKeyFn(alumno_key),
                render_row: RenderRowFn(render_alumno_row),
                estado,
                aplicar_color_seleccion: true,
                single_select: false,
                checkbox: true,
                on_doble_click: move |_| {},
                contexto: "alumnos".to_string(),
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
                    title: if hay_seleccion { "Desactivar los seleccionados" } else { "Seleccione al menos 1 alumno" },
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Eliminar)),
                    "🚫 Desactivar"
                }
                button {
                    class: if hay_seleccion {
                        "px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-700 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !hay_seleccion,
                    title: if hay_seleccion { "Activar los seleccionados" } else { "Seleccione al menos 1 alumno" },
                    onclick: move |_| {
                        let _ = estado.write().activar_seleccionados();
                    },
                    "✅ Activar"
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
fn ModalBase(titulo: String, on_cerrar: EventHandler<()>, ancho: Option<String>, children: Element) -> Element {
    let max_w = ancho.unwrap_or_else(|| "max-w-2xl".to_string());
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm",
            onclick: move |_| on_cerrar.call(()),
            div {
                class: "bg-gray-800 rounded-2xl shadow-2xl border border-gray-700 p-8 w-full {max_w} space-y-6 mx-4 max-h-[90vh] overflow-auto",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex items-center justify-between pb-3 border-b border-gray-700/50",
                    h3 { class: "text-lg font-bold text-white", "{titulo}" }
                    button {
                        class: "text-gray-400 text-xl leading-none cursor-pointer",
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
            if let Some(id) = app.seleccion_alumnos.iter().copied().next() {
                let alumno = app.get_alumno_by_id(id);
                nombre.set(alumno.nombre.clone());
                fecha_nac.set(alumno.fecha_de_nacimiento.clone());
                rango.set(alumno.rango);
                representante_id.set(alumno.representante_id);
                rallita.set(alumno.rallita);
            }
        });
    }

    let id = estado.read().seleccion_alumnos.iter().copied().next();
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
            .filter(|v| app.seleccion_alumnos.contains(&v.alumno.id))
            .map(|v| v.alumno.nombre.clone())
            .collect()
    };

    rsx! {
        ModalBase { titulo: "🥋 Promover en masa".to_string(), on_cerrar,
            div { class: "space-y-5",
                div { class: "flex items-center justify-between",
                    p { class: "text-sm text-gray-400",
                        "Se aplicará el nuevo grado a {nombres.len()} alumno(s):"
                    }
                    span { class: "px-2.5 py-0.5 rounded-full bg-gray-700 text-gray-300 text-[11px] font-bold",
                        "{nombres.len()}"
                    }
                }
                div { class: "max-h-40 overflow-y-auto rounded-lg border border-gray-700 divide-y divide-gray-700",
                    {nombres.iter().map(|n| rsx! {
                        div { class: "flex items-center gap-2 px-3 py-1.5 bg-gray-900",
                            div { class: "w-1.5 h-1.5 rounded-full bg-blue-500 shrink-0" }
                            span { class: "text-sm text-gray-200 truncate", "{n}" }
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
            .filter(|v| app.seleccion_alumnos.contains(&v.alumno.id))
            .cloned()
            .collect()
    };

    rsx! {
        ModalBase { titulo: "🚫 Desactivar Alumnos".to_string(), on_cerrar,
            div { class: "space-y-5",
                div { class: "flex items-center justify-between",
                    p { class: "text-sm text-gray-400",
                        "Se desactivarán {seleccionados.len()} alumno(s). Podrás reactivarlos desde los filtros."
                    }
                    span { class: "px-2.5 py-0.5 rounded-full bg-gray-700 text-gray-300 text-[11px] font-bold",
                        "{seleccionados.len()}"
                    }
                }
                div { class: "max-h-48 overflow-y-auto rounded-lg border border-gray-700 divide-y divide-gray-700",
                    {seleccionados.iter().map(|v| rsx! {
                        div {
                            key: "{v.alumno.id}",
                            class: "flex items-center gap-2 px-3 py-2 bg-gray-900",
                            div { class: "w-1.5 h-1.5 rounded-full bg-red-500 shrink-0" }
                            span { class: "text-sm font-medium text-gray-200 truncate", "{v.alumno.nombre}" }
                            span { class: "text-gray-500 text-xs ml-auto shrink-0", "#{v.alumno.id}" }
                        }
                    })}
                }
                button {
                    class: "w-full py-3 bg-red-600 text-white rounded-xl font-bold active:scale-[0.98] transition-all cursor-pointer",
                    onclick: move |_| {
                        let _ = estado.write().desactivar_seleccionados();
                        on_cerrar.call(());
                    },
                    "Desactivar"
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
            div { class: "space-y-5",
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Nombre completo" }
                    input {
                        r#type: "text",
                        class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                        placeholder: "Ej: Maria Garcia",
                        value: "{nombre}",
                        oninput: move |e| nombre.set(e.value())
                    }
                }
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Telefono de contacto" }
                    input {
                        r#type: "tel",
                        maxlength: "12",
                        class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
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
                        "w-full py-3 bg-blue-600 text-white font-bold rounded-xl active:scale-[0.98] transition-all cursor-pointer"
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

/// Un movimiento es un "adelanto" cuando se creó una deuda para un mes
/// futuro (motor FIFO fase 2). En el historial se registra como
/// DeudaCreada con observación que comienza con "Adelanto".
fn es_adelanto(vista: &HistorialPagoVista) -> bool {
    vista.historial.tipo_id == 1 && vista.historial.observacion.starts_with("Adelanto")
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
            div { class: "flex border-b border-gray-800 bg-gray-900 rounded-t-xl px-2",
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
}

static DEUDAS_COLUMNS: &[HeaderColumn] = &[
    HeaderColumn { header: "Representante", class: Some("") },
    HeaderColumn { header: "Mensualidad", class: Some("text-right") },
    HeaderColumn { header: "Abonado", class: Some("text-right") },
    HeaderColumn { header: "Progreso", class: Some("") },
    HeaderColumn { header: "Saldo", class: Some("text-right") },
    HeaderColumn { header: "Estado", class: Some("text-center") },
    HeaderColumn { header: "Ultimo pago", class: Some("text-right") },
];

fn deuda_key(row: &DeudaRow) -> usize {
    row.vista.deuda.representante_id
}

fn render_deuda_row(vista: &DeudaRow, _estado: Signal<my_app::MyApp>) -> Element {
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
    let hay_rep_seleccionado = estado.read().seleccion_consulta.len() == 1;
    let puede_registrar_pago = hay_pendientes && hay_rep_seleccionado;

    // Lógica para botón "Revertir pago": seleccionado + último pago completado
    let rep_seleccionado_id = if hay_rep_seleccionado {
        Some(*estado.read().seleccion_consulta.iter().next().unwrap())
    } else {
        None
    };
    let reversar_pago_data: Option<PagoVista> = rep_seleccionado_id.and_then(|id| {
        all_pagos.iter().rev().find(|p| p.pago.representante_id == id && p.estado == EstadoPago::Completado).cloned()
    });
    let puede_reversar_pago = reversar_pago_data.is_some();

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
        DeudaRow { vista: v.clone(), ultimo: ultimo.cloned() }
    }).collect();

    {
        let mut estado = estado.clone();
        use_effect(move || {
            let seleccionados: Vec<usize> = estado.read().seleccion_consulta.iter().copied().collect();
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

    let pago_rep_id_val = *pago_representante_id.read();
    let pago_rep_nombre = representantes.iter()
        .find(|r| r.id == pago_rep_id_val)
        .map(|r| format!("{} ({})", r.nombre, r.numero_contacto))
        .unwrap_or_else(|| "Representante".to_string());

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
                data: deudas_rows,
                header_columns: DEUDAS_COLUMNS,
                row_key: RowKeyFn(deuda_key),
                render_row: RenderRowFn(render_deuda_row),
                estado,
                aplicar_color_seleccion: false,
                single_select: true,
                checkbox: true,
                on_doble_click: move |_| {},
                contexto: "consulta".to_string(),
            }

            // -- Info --
            div { class: "flex justify-between items-center",
                div { class: "text-gray-500 text-xs",
                    "Mostrando {deudas.len()} de {all_deudas.len()} deudas"
                }
            }

            // -- Acciones --
            div { class: "flex justify-center gap-3 pt-1",
                button {
                    class: if puede_registrar_pago {
                        "px-5 py-2.5 bg-emerald-600 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-800 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !puede_registrar_pago,
                    onclick: move |_| {
                        let seleccionados: Vec<usize> = estado.read().seleccion_consulta.iter().copied().collect();
                        pago_representante_id.set(seleccionados[0]);
                        pago_monto.set(String::new());
                        pago_metodo_id.set(1);
                        pago_msg.set((String::new(), false));
                        modal_pago.set(true);
                    },
                    "💰 Registrar pago"
                }
                button {
                    class: if puede_reversar_pago {
                        "px-5 py-2.5 bg-red-600 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-800 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !puede_reversar_pago,
                    onclick: move |_| {
                        if let Some(pv) = &reversar_pago_data {
                            estado.write().abrir_modal_reversar(
                                pv.pago.id,
                                pv.nombre_representante.clone(),
                                pv.pago.monto_recibido,
                                metodo_label(&pv.metodo).to_string(),
                                pv.pago.fecha_pago.clone(),
                            );
                        }
                    },
                    "↩ Revertir pago"
                }
            }
        }

        // Modal: registrar pago (FIFO)
        if *modal_pago.read() {
            ModalBase {
                titulo: "💰 Registrar pago".to_string(),
                ancho: Some("max-w-sm".to_string()),
                on_cerrar: move |_| modal_pago.set(false),
                div { class: "space-y-4",
                    div { class: "flex items-center gap-3 p-3 rounded-lg bg-gray-900 border border-gray-700",
                        div { class: "w-9 h-9 rounded-full bg-blue-600/20 text-blue-400 flex items-center justify-center font-bold text-sm shrink-0",
                            {pago_rep_nombre.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_else(|| "R".to_string())}
                        }
                        div { class: "min-w-0",
                            p { class: "text-sm font-bold text-white truncate", "{pago_rep_nombre}" }
                            p { class: "text-[10px] text-gray-500", "El monto se aplica a la deuda mas antigua (FIFO). Si sobra, crea adelantos." }
                        }
                    }

                    div { class: "flex flex-col space-y-1",
                        label { class: "text-sm font-semibold text-gray-400", "Monto recibido" }
                        input {
                            r#type: "text",
                            class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50 text-sm placeholder:text-gray-600",
                            placeholder: "Ej: 1500",
                            value: "{pago_monto}",
                            oninput: move |e| {
                                pago_monto.set(e.value());
                                pago_msg.set((String::new(), false));
                            }
                        }
                    }

                    div { class: "flex flex-col space-y-1",
                        label { class: "text-sm font-semibold text-gray-400", "Tipo de pago" }
                        select {
                            class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50 text-sm",
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
                            class: if pago_msg.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" },
                            "{pago_msg.read().0}"
                        }
                    }

                    div { class: "flex gap-2 justify-end pt-1",
                        button {
                            class: "px-4 py-2 text-sm text-gray-400 hover:text-white hover:bg-gray-700/50 rounded-lg transition-colors cursor-pointer",
                            onclick: move |_| modal_pago.set(false),
                            "Cancelar"
                        }
                        button {
                            class: "px-5 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                            onclick: move |_| {
                                let rep_id = *pago_representante_id.read();
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

        // Modal: revertir pago
        if estado.read().modal_reversar_activo {
            ModalBase {
                titulo: "⚠️ Revertir pago".to_string(),
                ancho: Some("max-w-sm".to_string()),
                on_cerrar: move |_| estado.write().cerrar_modal_reversar(),
                { let e = estado.read(); rsx! {
                    div { class: "space-y-4",
                        div { class: "flex items-center gap-3 p-3 rounded-lg bg-gray-900 border border-gray-700",
                            div { class: "w-9 h-9 rounded-full bg-red-600/20 text-red-400 flex items-center justify-center font-bold text-sm shrink-0",
                                {e.reversar_rep_nombre.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_else(|| "R".to_string())}
                            }
                            div { class: "min-w-0",
                                p { class: "text-sm font-bold text-white truncate", "{e.reversar_rep_nombre}" }
                                p { class: "text-[10px] text-gray-500", "Se restaurarán los saldos de las deudas afectadas." }
                            }
                        }

                        div { class: "grid grid-cols-2 gap-3 text-sm",
                            div { class: "flex flex-col space-y-0.5",
                                span { class: "text-[10px] uppercase tracking-wider text-gray-500", "Monto" }
                                span { class: "font-mono font-bold text-amber-400", "{fmt_monto(e.reversar_monto)}" }
                            }
                            div { class: "flex flex-col space-y-0.5",
                                span { class: "text-[10px] uppercase tracking-wider text-gray-500", "Método" }
                                span { class: "text-gray-300", "{e.reversar_metodo}" }
                            }
                            div { class: "flex flex-col space-y-0.5",
                                span { class: "text-[10px] uppercase tracking-wider text-gray-500", "Fecha" }
                                span { class: "text-gray-300 font-mono", "{e.reversar_fecha}" }
                            }
                        }

                        p { class: "text-xs text-red-400/80", "Esta acción no se puede deshacer." }

                        div { class: "flex gap-2 justify-end pt-1",
                            button {
                                class: "px-4 py-2 text-sm text-gray-400 hover:text-white hover:bg-gray-700/50 rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| estado.write().cerrar_modal_reversar(),
                                "Cancelar"
                            }
                            button {
                                class: "px-5 py-2 bg-red-600 hover:bg-red-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                                onclick: move |_| {
                                    let pago_id = estado.read().reversar_pago_id;
                                    let resultado = estado.write().reversar_pago(pago_id);
                                    estado.write().cerrar_modal_reversar();
                                    let _ = resultado;
                                },
                                "Revertir"
                            }
                        }
                    }
                }}
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
    let mut desde = use_signal(|| String::new());
    let mut hasta = use_signal(|| String::new());
    let mut filtro_tipo = use_signal(|| "Todos".to_string());
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

    // Filtro por rango de fechas. Fechas vacías = sin límite = todo.
    // Las fechas usan formato ISO "YYYY-MM-DD", por lo que la comparación
    // lexicográfica del string es equivalente a la cronológica.
    let desde_val = desde.read().clone();
    let hasta_val = hasta.read().clone();
    let tipo = filtro_tipo.read().clone();
    let filtrado: Vec<_> = historial.iter().filter(|v| {
        let fecha = &v.historial.fecha;
        if !desde_val.is_empty() && fecha < &desde_val {
            return false;
        }
        if !hasta_val.is_empty() && fecha > &hasta_val {
            return false;
        }
        // Filtro por tipo de movimiento
        match tipo.as_str() {
            "Deuda" => v.historial.tipo_id == 1 && !es_adelanto(v),
            "Pago" => v.historial.tipo_id == 2,
            "Abono" => v.historial.tipo_id == 3,
            "Anticipado" => es_adelanto(v),
            "Ajuste" => v.historial.tipo_id == 4,
            "Anulación" => v.historial.tipo_id == 5,
            _ => true,
        }
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

            // -- Filtro por rango de fechas + tipo --
            div { class: "flex items-end gap-4 bg-gray-800 rounded-xl px-4 py-3 border border-gray-700",
                div { class: "flex flex-col space-y-1 flex-1",
                    label { class: "text-[10px] uppercase tracking-wider text-gray-400", "Desde" }
                    input {
                        r#type: "date",
                        class: "w-full p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50 text-sm [color-scheme:dark]",
                        value: "{desde}",
                        oninput: move |e| desde.set(e.value())
                    }
                }
                div { class: "flex flex-col space-y-1 flex-1",
                    label { class: "text-[10px] uppercase tracking-wider text-gray-400", "Hasta" }
                    input {
                        r#type: "date",
                        class: "w-full p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50 text-sm [color-scheme:dark]",
                        value: "{hasta}",
                        oninput: move |e| hasta.set(e.value())
                    }
                }
                div { class: "flex flex-col space-y-1 flex-1",
                    label { class: "text-[10px] uppercase tracking-wider text-gray-400", "Tipo" }
                    select {
                        class: "w-full p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50 text-sm",
                        value: "{filtro_tipo}",
                        onchange: move |e| filtro_tipo.set(e.value()),
                        {["Todos", "Deuda", "Pago", "Abono", "Anticipado", "Ajuste", "Anulación"].iter().map(|op| rsx! {
                            option { class: "bg-gray-900", value: "{op}", "{op}" }
                        })}
                    }
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
                        data: filtrado.clone(),
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

// =====================================================
// Pestaña Representantes
// =====================================================

static REP_COLUMNS: &[HeaderColumn] = &[
    HeaderColumn { header: "Nombre", class: Some("font-bold text-white whitespace-nowrap") },
    HeaderColumn { header: "Telefono", class: Some("text-blue-400 font-mono whitespace-nowrap") },
    HeaderColumn { header: "Estado", class: Some("text-center") },
];

fn rep_key(row: &Representante) -> usize {
    row.id
}

fn render_rep_row(vista: &Representante, _estado: Signal<my_app::MyApp>) -> Element {
    let (etiqueta, clases) = if vista.estado_id == 1 {
        ("Activo", "bg-emerald-900 text-emerald-300")
    } else {
        ("Inactivo", "bg-gray-700 text-gray-400")
    };
    rsx! {
        td { class: "px-4 py-3 font-bold text-white whitespace-nowrap", "{vista.nombre}" }
        td { class: "px-4 py-3 text-blue-400 font-mono whitespace-nowrap", "{vista.numero_contacto}" }
        td { class: "px-4 py-3 text-center",
            span { class: "inline-block px-2 py-0.5 rounded-full text-[10px] font-bold {clases}", "{etiqueta}" }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ModalRep {
    Nuevo,
    Editar,
}

#[component]
pub fn Representantes() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut modal_activo = use_signal(|| None::<ModalRep>);
    let mut edit_id = use_signal(|| 0usize);
    let mut busqueda = use_signal(String::new);
    let mut filtro_estado = use_signal(|| "Activo".to_string());

    let seleccionados = estado.read().seleccion_representantes.clone();
    let total_seleccionados = seleccionados.len();
    let hay_seleccion = total_seleccionados > 0;
    let uno_seleccionado = total_seleccionados == 1;

    let reps_filtrados: Vec<Representante> = {
        let app = estado.read();
        let q = busqueda.read().to_lowercase();
        let estado_filtro = filtro_estado.read().clone();
        app.representantes.iter().filter(|r| {
            let match_nombre = q.is_empty() || r.nombre.to_lowercase().contains(&q) || r.numero_contacto.contains(&q);
            let match_estado = match estado_filtro.as_str() {
                "Activo" => r.estado_id == 1,
                "Inactivo" => r.estado_id == 2,
                _ => true,
            };
            match_nombre && match_estado
        }).cloned().collect()
    };

    rsx! {
        div { class: "flex flex-col h-full space-y-4 overflow-auto",

            // ── Encabezado ──
            div { class: "flex items-center justify-between py-1",
                div {
                    h2 { class: "text-3xl font-bold text-gray-800", "👤 Representantes" }
                    p { class: "text-gray-500 text-sm mt-1",
                        "Gestion de representantes responsables."
                    }
                }
            }

            // ── Barra de búsqueda + filtro ──
            div { class: "flex flex-wrap gap-3 items-center",
                div { class: "flex-1 relative min-w-48",
                    input {
                        r#type: "text",
                        class: "w-full p-2 pl-8 rounded-lg bg-gray-800 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50 text-xs placeholder-gray-500",
                        placeholder: "Buscar representante...",
                        value: "{busqueda}",
                        oninput: move |e| busqueda.set(e.value())
                    }
                    span { class: "absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500 text-xs", "\u{1F50D}" }
                }
                select {
                    class: "p-2 rounded-lg bg-gray-800 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50 text-xs",
                    value: "{filtro_estado}",
                    onchange: move |e| filtro_estado.set(e.value()),
                    option { class: "bg-gray-900", value: "Activo", "Activos" }
                    option { class: "bg-gray-900", value: "Inactivo", "Inactivos" }
                    option { class: "bg-gray-900", value: "Todos", "Todos" }
                }
            }

            // ── Tabla ──
            DataTable {
                data: reps_filtrados.clone(),
                header_columns: REP_COLUMNS,
                row_key: RowKeyFn(rep_key),
                render_row: RenderRowFn(render_rep_row),
                estado,
                aplicar_color_seleccion: true,
                single_select: false,
                checkbox: true,
                on_doble_click: move |_| {},
                contexto: "representantes".to_string(),
            }

            div { class: "flex justify-between items-center",
                div { class: "text-gray-500 text-xs",
                    "Mostrando {reps_filtrados.len()} representante(s)"
                }
                div { class: "text-gray-500 text-xs",
                    "seleccionados: {total_seleccionados}"
                }
            }

            // ── Acciones ──
            div { class: "flex flex-wrap justify-center gap-3 pt-1",
                button {
                    class: "px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                    onclick: move |_| modal_activo.set(Some(ModalRep::Nuevo)),
                    "＋ Nuevo Representante"
                }
                button {
                    class: if uno_seleccionado {
                        "px-5 py-2.5 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-700 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !uno_seleccionado,
                    onclick: move |_| {
                        if let Some(&id) = estado.read().seleccion_representantes.iter().next() {
                            edit_id.set(id);
                            modal_activo.set(Some(ModalRep::Editar));
                        }
                    },
                    "✏️ Editar"
                }
                button {
                    class: if hay_seleccion {
                        "px-5 py-2.5 bg-red-600 hover:bg-red-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-700 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !hay_seleccion,
                    onclick: move |_| {
                        let ids: Vec<usize> = estado.read().seleccion_representantes.iter().copied().collect();
                        for id in ids {
                            let _ = estado.write().desactivar_representante(id);
                        }
                        estado.write().seleccion_representantes.clear();
                    },
                    "🚫 Desactivar"
                }
                button {
                    class: if hay_seleccion {
                        "px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-400 text-gray-700 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !hay_seleccion,
                    onclick: move |_| {
                        let ids: Vec<usize> = estado.read().seleccion_representantes.iter().copied().collect();
                        for id in ids {
                            let _ = estado.write().activar_representante(id);
                        }
                        estado.write().seleccion_representantes.clear();
                    },
                    "✅ Activar"
                }
            }
        }

        // ── Modales ──
        match modal_activo.read().clone() {
            Some(ModalRep::Nuevo) => rsx! {
                ModalRepNuevo { on_cerrar: move |_| modal_activo.set(None) }
            },
            Some(ModalRep::Editar) => rsx! {
                ModalRepEditar { id: edit_id.read().clone(), on_cerrar: move |_| modal_activo.set(None) }
            },
            None => rsx! {},
        }
    }
}

#[component]
fn ModalRepNuevo(on_cerrar: EventHandler<()>) -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut nombre = use_signal(String::new);
    let mut contacto = use_signal(String::new);
    let mut msg = use_signal(|| (String::new(), false));

    let formulario_ok = !nombre.read().is_empty() && contacto_valido(&contacto.read());

    rsx! {
        ModalBase { titulo: "👤 Nuevo Representante".to_string(), on_cerrar,
            div { class: "space-y-5",
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Nombre completo" }
                    input {
                        class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                        placeholder: "Ej: Maria Garcia",
                        value: "{nombre}",
                        oninput: move |e| nombre.set(e.value())
                    }
                }
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Telefono" }
                    input {
                        class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
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
                        "w-full py-3 bg-blue-600 text-white font-bold rounded-xl active:scale-[0.98] transition-all cursor-pointer"
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
                                msg.set(("Representante registrado.".to_string(), false));
                            }
                            Err(e) => msg.set((e.to_string(), true)),
                        }
                    },
                    "Guardar"
                }
            }
        }
    }
}

#[component]
fn ModalRepEditar(id: usize, on_cerrar: EventHandler<()>) -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut nombre = use_signal(String::new);
    let mut contacto = use_signal(String::new);
    let mut msg = use_signal(|| (String::new(), false));

    {
        let estado = estado.clone();
        let rep_id = id;
        use_effect(move || {
            let app = estado.read();
            if let Some(rep) = app.representantes.iter().find(|r| r.id == rep_id) {
                nombre.set(rep.nombre.clone());
                contacto.set(rep.numero_contacto.clone());
            }
        });
    }

    let formulario_ok = !nombre.read().is_empty() && contacto_valido(&contacto.read());

    rsx! {
        ModalBase { titulo: "✏️ Editar Representante".to_string(), on_cerrar,
            div { class: "space-y-5",
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Nombre completo" }
                    input {
                        class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                        placeholder: "Ej: Maria Garcia",
                        value: "{nombre}",
                        oninput: move |e| nombre.set(e.value())
                    }
                }
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Telefono" }
                    input {
                        class: "p-2.5 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
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
                        "w-full py-3 bg-blue-600 text-white font-bold rounded-xl active:scale-[0.98] transition-all cursor-pointer"
                    } else {
                        "w-full py-3 bg-gray-700 text-gray-400 font-bold rounded-xl cursor-pointer"
                    },
                    disabled: !formulario_ok,
                    onclick: move |_| {
                        let datos = DatosRepresentante {
                            nombre: nombre.read().clone(),
                            numero_contacto: contacto.read().clone(),
                        };
                        match estado.write().actualizar_representante(id, datos) {
                            Ok(()) => {
                                msg.set(("Representante actualizado.".to_string(), false));
                            }
                            Err(e) => msg.set((e.to_string(), true)),
                        }
                    },
                    "Guardar cambios"
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
