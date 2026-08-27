//! se encarga de dibujar las vistas segun las rutas seleccionadas
//! contiene las funciones con el codigo especifico de cada vista

use crate::application::dto::{AlumnoVista, DatosAlumno, DatosPago, DatosRepresentante};
use crate::application::validation::*;
use crate::domain::{EstadoDeuda, EstadoPago, MetodoPago};
use crate::presentation::components::datatable::DataTable;
use crate::presentation::components::filter::Filter;
use crate::presentation::components::form::Form;
use crate::presentation::components::promotion_form::PromotionForm;
use crate::presentation::components::searchbar::SearchBar;
use crate::presentation::my_app::{self, Columnas};
use chrono::Local;
use dioxus::prelude::*;

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
        div { class: "flex flex-col h-full space-y-4",

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
                alumnos_lista: alumnos_filtrados,
                estado,
                aplicar_color_seleccion: true,
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
            div { class: "flex justify-center gap-3 pt-1",
                button {
                    class: "px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Nuevo)),
                    "＋ Nuevo Alumno"
                }
                button {
                    class: if uno_seleccionado {
                        "px-5 py-2.5 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-300 text-gray-500 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !uno_seleccionado,
                    title: if uno_seleccionado { "Editar el alumno seleccionado" } else { "Seleccione exactamente 1 alumno" },
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Editar)),
                    "✏️ Editar"
                }
                button {
                    class: if hay_seleccion {
                        "px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                    } else {
                        "px-5 py-2.5 bg-gray-300 text-gray-500 font-bold rounded-lg cursor-not-allowed text-sm"
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
                        "px-5 py-2.5 bg-gray-300 text-gray-500 font-bold rounded-lg cursor-not-allowed text-sm"
                    },
                    disabled: !hay_seleccion,
                    title: if hay_seleccion { "Eliminar a los seleccionados" } else { "Seleccione al menos 1 alumno" },
                    onclick: move |_| modal_activo.set(Some(ModalAlumno::Eliminar)),
                    "🗑️ Eliminar"
                }
                button {
                    class: "px-5 py-2.5 bg-cyan-600 hover:bg-cyan-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
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
    let mut nombre = use_signal(|| "".to_string());
    let mut fecha_nac = use_signal(|| "".to_string());
    let mut rango = use_signal(|| 10i32);
    let mut representante_id = use_signal(|| 0usize);
    let mut rallita = use_signal(|| false);

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
    let mut rango = use_signal(|| 99i32);
    let mut rallita = use_signal(|| false);

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

// ═══════════════════════════════════════════════════════════════
// Panel de Pagos (motor FIFO)
// ═══════════════════════════════════════════════════════════════

#[component]
pub fn Pagos() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    // ── Modal: registrar pago (FIFO) ──
    let mut modal_pago = use_signal(|| false);
    let mut pago_representante_id = use_signal(|| 0usize);
    let mut pago_monto = use_signal(String::new);
    let mut pago_metodo_id = use_signal(|| 1i32); // Efectivo por defecto
    let mut pago_msg = use_signal(|| (String::new(), false));

    // ── Formulario de representantes (inline) ──
    let mut rep_nombre = use_signal(String::new);
    let mut rep_contacto = use_signal(String::new);

    // ── Mensaje de acción ──
    let mut msg_accion = use_signal(|| (String::new(), false));

    // Una sola pasada de lectura
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

    let deudas = estado.read().deudas.clone();
    let pagos = estado.read().pagos.clone();
    let representantes = estado.read().representantes.clone();
    let hay_pendientes = deudas.iter().any(|v| v.estado != EstadoDeuda::Pagada);

    let rep_formulario_ok = nombre_valido(&rep_nombre.read()) && contacto_valido(&rep_contacto.read());

    // Progreso global
    let pct_global = if total_deudas > 0.0 {
        (total_abonado / total_deudas * 100.0).min(100.0)
    } else {
        0.0
    };
    let texto_avance = if total_deudas > 0.0 {
        format!("{pct_global:.0}% recaudado")
    } else {
        "Sin deudas aún".to_string()
    };
    let clase_avance = if total_deudas > 0.0 && pendientes == 0 {
        clase_barra(&EstadoDeuda::Pagada)
    } else if total_deudas > 0.0 && pagados == 0 {
        clase_barra(&EstadoDeuda::Pendiente)
    } else {
        clase_barra(&EstadoDeuda::Parcial)
    };

    rsx! {
        div { class: "flex flex-col h-full space-y-5 overflow-auto pr-1",

            // ── Encabezado ──
            div { class: "flex items-end justify-between py-1",
                div {
                    h2 { class: "text-3xl font-bold text-gray-800", "💳 Panel de Pagos" }
                    p { class: "text-gray-500 text-sm mt-1",
                        "Deudas y pagos de mensualidad — vista del mes completo."
                    }
                }
                span { class: "px-3 py-1 rounded-full bg-gray-200 text-gray-700 text-xs font-bold tracking-widest uppercase whitespace-nowrap",
                    "{etiqueta_mes}"
                }
            }

            // ── Estadísticas del mes ──
            div { class: "grid grid-cols-4 gap-4",
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "💰 Deudas del mes" }
                    p { class: "text-2xl font-bold text-gray-100 mt-1", {fmt_monto(total_deudas)} }
                    p { class: "text-[11px] text-gray-500 mt-1", "{deudas.len()} deudas" }
                }
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "✅ Recaudado" }
                    p { class: "text-2xl font-bold text-emerald-400 mt-1", {fmt_monto(total_abonado)} }
                    p { class: "text-[11px] text-gray-500 mt-1", "{pagados} pagaron completo" }
                }
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "⏳ Por cobrar" }
                    p { class: "text-2xl font-bold text-amber-400 mt-1", {fmt_monto(por_cobrar)} }
                    p { class: "text-[11px] text-gray-500 mt-1", "{parciales} abonaron parcial · {pendientes} pendientes" }
                }
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "📊 Avance" }
                    div { class: "mt-3 h-2 w-full bg-gray-700 rounded-full overflow-hidden",
                        div {
                            class: "h-full rounded-full {clase_avance}",
                            style: "width:{pct_global:.0}%",
                        }
                    }
                    p { class: "text-[11px] text-gray-500 mt-1", "{texto_avance}" }
                }
            }

            // ── Barra de acciones ──
            div { class: "flex items-center justify-between gap-3 flex-wrap",
                div { class: "flex items-center gap-3",
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
                            "px-5 py-2.5 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed text-sm"
                        },
                        disabled: !hay_pendientes,
                        onclick: move |_| {
                            pago_monto.set(String::new());
                            pago_representante_id.set(0);
                            pago_metodo_id.set(1);
                            pago_msg.set((String::new(), false));
                            modal_pago.set(true);
                        },
                        "💵 Registrar pago"
                    }
                    if !msg_accion.read().0.is_empty() {
                        span {
                            class: if msg_accion.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" },
                            "{msg_accion.read().0}"
                        }
                    }
                }
                div { class: "flex items-center gap-3 text-[11px] text-gray-500",
                    span { class: "flex items-center gap-1", i { class: "inline-block w-2 h-2 rounded-full bg-emerald-500" } "Pagado" }
                    span { class: "flex items-center gap-1", i { class: "inline-block w-2 h-2 rounded-full bg-amber-500" } "Parcial" }
                    span { class: "flex items-center gap-1", i { class: "inline-block w-2 h-2 rounded-full bg-red-500" } "Pendiente" }
                }
            }

            // ── Tabla de deudas del mes ──
            div { class: "overflow-auto rounded-xl border border-gray-800 bg-gray-900 shadow-xl max-h-72",
                table { class: "w-full border-collapse text-left text-xs md:text-sm",
                    thead {
                        tr { class: "sticky top-0 text-white bg-gray-800 z-10",
                            th { class: "px-4 py-3", "Representante" }
                            th { class: "px-4 py-3 text-right", "Mensualidad" }
                            th { class: "px-4 py-3 text-right", "Abonado" }
                            th { class: "px-4 py-3 w-36", "Progreso" }
                            th { class: "px-4 py-3 text-right", "Saldo" }
                            th { class: "px-4 py-3 text-center", "Estado" }
                        }
                    }
                    tbody { class: "divide-y divide-gray-800 text-gray-300",
                        if deudas.is_empty() {
                            tr {
                                td { colspan: 6, class: "px-4 py-10 text-center",
                                    p { class: "text-3xl mb-2", "🗓️" }
                                    p { class: "text-gray-400 font-medium", "Este mes aún no tiene deudas" }
                                    p { class: "text-gray-500 text-xs mt-1",
                                        "Pulsa \"+ Crear deudas del mes\" para generarlas."
                                    }
                                }
                            }
                        }
                        {(deudas.iter()).map(|vista| {
                            let id_deuda = vista.deuda.id;
                            let saldo_texto = fmt_monto(vista.deuda.saldo());
                            let pct = vista.deuda.porcentaje();
                            let (etiqueta, clases) = badge_estado_deuda(&vista.estado);
                            rsx! {
                            tr {
                                key: "{id_deuda}",
                                class: "hover:bg-gray-700/50",
                                td { class: "px-4 py-2.5",
                                    p { class: "font-medium text-white truncate max-w-48", "{vista.nombre_representante}" }
                                    p { class: "text-[11px] text-gray-500 font-mono", "{vista.telefono_representante}" }
                                }
                                td { class: "px-4 py-2.5 text-right font-mono text-gray-300", "{fmt_monto(vista.deuda.monto_total)}" }
                                td { class: "px-4 py-2.5 text-right font-mono text-emerald-400", "{fmt_monto(vista.deuda.total_abonado())}" }
                                td { class: "px-4 py-2.5",
                                    div { class: "h-2 w-full bg-gray-700 rounded-full overflow-hidden",
                                        div { class: "h-full rounded-full {clase_barra(&vista.estado)}", style: "width:{pct:.0}%" }
                                    }
                                    p { class: "text-[10px] text-gray-500 mt-1", "{pct:.0}%" }
                                }
                                td { class: "px-4 py-2.5 text-right font-mono font-bold text-amber-400", "{saldo_texto}" }
                                td { class: "px-4 py-2.5 text-center",
                                    span { class: "inline-block px-2 py-0.5 rounded-full text-[10px] font-bold {clases}", "{etiqueta}" }
                                }
                            }
                            }
                        })}
                    }
                }
            }

            // ── Historial de pagos del mes ──
            if !pagos.is_empty() {
                div { class: "rounded-xl border border-gray-800 bg-gray-900 shadow-xl",
                    div { class: "px-4 py-3 bg-gray-800 rounded-t-xl",
                        h3 { class: "text-sm font-bold text-white", "📜 Pagos registrados ({pagos.len()})" }
                    }
                    table { class: "w-full border-collapse text-left text-xs",
                        thead {
                            tr { class: "text-gray-400 border-b border-gray-800",
                                th { class: "px-4 py-2", "Representante" }
                                th { class: "px-4 py-2 text-right", "Monto" }
                                th { class: "px-4 py-2", "Método" }
                                th { class: "px-4 py-2", "Fecha" }
                                th { class: "px-4 py-2 text-center", "Estado" }
                                th { class: "px-4 py-2 text-right", "" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-800 text-gray-300",
                            {(pagos.iter()).map(|vista| {
                                let pago_id = vista.pago.id;
                                let es_completado = vista.estado == EstadoPago::Completado;
                                rsx! {
                                tr { key: "{pago_id}", class: "hover:bg-gray-800/50",
                                    td { class: "px-4 py-2 font-medium text-white", "{vista.nombre_representante}" }
                                    td { class: "px-4 py-2 text-right font-mono", "{fmt_monto(vista.pago.monto_recibido)}" }
                                    td { class: "px-4 py-2 text-gray-400", "{metodo_label(&vista.metodo)}" }
                                    td { class: "px-4 py-2 text-gray-500 font-mono text-[11px]", "{vista.pago.fecha_pago}" }
                                    td { class: "px-4 py-2 text-center",
                                        span {
                                            class: if vista.estado == EstadoPago::Completado {
                                                "px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-900 text-emerald-300"
                                            } else if vista.estado == EstadoPago::Reversado {
                                                "px-2 py-0.5 rounded-full text-[10px] font-bold bg-red-900 text-red-300"
                                            } else {
                                                "px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-900 text-amber-300"
                                            },
                                            "{vista.estado.label()}"
                                        }
                                    }
                                    td { class: "px-4 py-2 text-right",
                                        if es_completado {
                                            button {
                                                class: "px-2 py-1 rounded bg-gray-800 border border-gray-700 text-red-400 hover:text-white hover:border-red-500 font-bold text-[10px] transition-colors cursor-pointer",
                                                onclick: move |_| {
                                                    let _ = estado.write().reversar_pago(pago_id);
                                                },
                                                "↩ Reversar"
                                            }
                                        }
                                    }
                                }
                                }
                            })}
                        }
                    }
                }
            }

            // ── Representantes (compacto) ──
            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-3",
                div { class: "flex items-center justify-between",
                    h3 { class: "font-bold text-white", "👥 Representantes ({representantes.len()})" }
                }
                div { class: "flex items-end gap-2",
                    div { class: "flex flex-col space-y-1 flex-1",
                        label { class: "text-xs font-semibold text-gray-400", "Nombre" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                            placeholder: "Ej: Pedro Pérez",
                            value: "{rep_nombre}",
                            oninput: move |e| rep_nombre.set(e.value())
                        }
                    }
                    div { class: "flex flex-col space-y-1 w-44",
                        label { class: "text-xs font-semibold text-gray-400", "Teléfono" }
                        input {
                            r#type: "tel",
                            maxlength: "12",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                            placeholder: "0412-0000000",
                            value: "{rep_contacto}",
                            oninput: move |e| {
                                let mut val = e.value();
                                val.retain(|c| c.is_ascii_digit());
                                if val.len() > 4 { val.insert(4, '-'); }
                                val.truncate(12);
                                rep_contacto.set(val);
                            }
                        }
                    }
                    button {
                        class: if rep_formulario_ok {
                            "px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                        } else {
                            "px-4 py-2 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed text-sm"
                        },
                        disabled: !rep_formulario_ok,
                        onclick: move |_| {
                            if !rep_formulario_ok { return; }
                            let datos = DatosRepresentante {
                                nombre: rep_nombre.read().clone(),
                                numero_contacto: rep_contacto.read().clone(),
                            };
                            if estado.write().agregar_representante(datos).is_ok() {
                                rep_nombre.set(String::new());
                                rep_contacto.set(String::new());
                            }
                        },
                        "＋ Agregar"
                    }
                }
                div { class: "grid grid-cols-2 gap-2 mt-2",
                    {representantes.iter().map(|r| rsx! {
                        div {
                            key: "{r.id}",
                            class: "flex items-center gap-3 px-3 py-2 bg-gray-900 rounded-lg border border-gray-800",
                            span { class: "w-8 h-8 rounded-full bg-blue-600 text-white flex items-center justify-center text-xs font-bold flex-none",
                                {r.nombre.chars().next().unwrap_or('?').to_string()}
                            }
                            div { class: "min-w-0",
                                p { class: "text-sm text-gray-100 truncate", "{r.nombre}" }
                                p { class: "text-[11px] text-gray-500 font-mono", "{r.numero_contacto}" }
                            }
                        }
                    })}
                }
            }
        }

        // ═══ Modal: registrar pago (FIFO) ═══
        if *modal_pago.read() {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
                onclick: move |_| modal_pago.set(false),
                div {
                    class: "bg-gray-800 rounded-xl shadow-2xl border border-gray-700 p-6 w-full max-w-md space-y-4 mx-4",
                    onclick: move |e| e.stop_propagation(),

                    div { class: "flex items-center justify-between",
                        h3 { class: "text-lg font-bold text-white", "💵 Registrar pago" }
                        button {
                            class: "text-gray-400 hover:text-white text-xl leading-none cursor-pointer",
                            onclick: move |_| modal_pago.set(false),
                            "✕"
                        }
                    }

                    p { class: "text-xs text-gray-400",
                        "El monto se aplicará automáticamente a las deudas más antiguas (FIFO). Si sobra, se crean adelantos."
                    }

                    // Representante
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Representante" }
                        select {
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50",
                            value: "{pago_representante_id}",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<usize>() {
                                    pago_representante_id.set(id);
                                    pago_msg.set((String::new(), false));
                                }
                            },
                            option { class: "bg-gray-900", value: "0", "-- Seleccione representante --" }
                            {representantes.iter().map(|r| rsx! {
                                option {
                                    key: "{r.id}",
                                    class: "bg-gray-900", value: "{r.id}",
                                    "{r.nombre}"
                                }
                            })}
                        }
                    }

                    // Monto
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Monto recibido" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                            placeholder: "Ej: 1500",
                            value: "{pago_monto}",
                            oninput: move |e| {
                                pago_monto.set(e.value());
                                pago_msg.set((String::new(), false));
                            }
                        }
                    }

                    // Método de pago
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Método de pago" }
                        select {
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50",
                            value: "{pago_metodo_id}",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<i32>() {
                                    pago_metodo_id.set(id);
                                }
                            },
                            option { class: "bg-gray-900", value: "1", "💵 Efectivo" }
                            option { class: "bg-gray-900", value: "2", "🏦 Transferencia" }
                            option { class: "bg-gray-900", value: "3", "💳 Tarjeta" }
                            option { class: "bg-gray-900", value: "4", "📄 Cheque" }
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
                            class: "px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors cursor-pointer",
                            onclick: move |_| modal_pago.set(false),
                            "Cancelar"
                        }
                        button {
                            class: "px-5 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
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
                                        pago_msg.set(("Pago registrado y aplicado correctamente.".to_string(), false));
                                    }
                                    Err(error) => pago_msg.set((error.to_string(), true)),
                                }
                            },
                            "Registrar pago"
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
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
