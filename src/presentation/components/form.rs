use crate::domain::{Cintas, Representante};
use dioxus::prelude::*;

/// Formulario de alumno compartido por las vistas Agregar y Editar.
/// El contacto ya no se captura aquí: pertenece al REPRESENTANTE, que se
/// elige del catálogo administrado en el panel de Administrador.
#[component]
pub fn Form(
    nombre: Signal<String>,
    fecha_nac: Signal<String>,
    rango: Signal<i32>,
    representantes: Vec<Representante>,
    representante_id: Signal<usize>,
    rallita: Signal<bool>,
    texto_boton: &'static str,
    campos_validos: (bool, bool, bool),
    on_click: EventHandler<()>,
) -> Element {
    fn get_border_class(es_valido: bool, intentando: bool) -> &'static str {
        if !intentando || es_valido {
            "border border-gray-700 focus:ring-2 focus:ring-blue-500/50"
        } else {
            "border border-red-500 ring-2 ring-red-500/30 focus:ring-red-500/50"
        }
    }

    let mut intentado = use_signal(|| false);
    let mut busqueda_rep = use_signal(String::new);
    let mut rep_dropdown_open = use_signal(|| false);
    let (nombre_valido, fecha_valida, rep_valido) = campos_validos;
    let form_valido = nombre_valido && fecha_valida && rep_valido;

    // Placeholder del representante
    let rep_placeholder = if *representante_id.read() == 0 {
        "Escriba para buscar..."
    } else {
        ""
    };

    // Nombre del representante seleccionado (para el input)
    let rep_seleccionado_nombre = if *representante_id.read() != 0 {
        representantes.iter().find(|r| r.id == *representante_id.read()).map(|r| r.nombre.clone()).unwrap_or_default()
    } else {
        "".to_string()
    };

    let color_btn = if form_valido {
        "w-full mt-6 py-3 bg-blue-600 text-white font-bold rounded-xl shadow-lg shadow-blue-950 transition-all active:scale-[0.98] cursor-pointer"
    } else {
        "w-full mt-6 py-3 bg-gray-700 text-gray-400 font-bold rounded-xl cursor-pointer active:scale-[0.98] transition-all"
    };

    rsx! {
        div { class: "flex-1 flex flex-col justify-around bg-gray-800 p-8 rounded-2xl shadow-xl text-gray-100 space-y-4",

            // Campo: Nombre
            div { class: "flex flex-col space-y-2",
                label { class: "text-sm font-semibold text-gray-400", "Nombre Completo" }
                input {
                    r#type: "text",
                    class: "p-2 rounded-lg outline-none transition-colors bg-gray-900 text-gray-100 {get_border_class(nombre_valido, intentado.read().clone())}",
                    placeholder: "Ej: Juan Pérez",
                    value: "{nombre}",
                    oninput: move |e| nombre.set(e.value())
                }
            }

            div { class: "grid grid-cols-2 gap-5",
                // Campo: Fecha de Nacimiento
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Fecha de Nacimiento" }
                    input {
                        r#type: "date",
                        class: "p-2 rounded-lg outline-none transition-colors bg-gray-900 text-gray-100 [color-scheme:dark] {get_border_class(fecha_valida, intentado.read().clone())}",
                        value: "{fecha_nac}",
                        oninput: move |e| fecha_nac.set(e.value())
                    }
                }

                // Campo: Grado / Cinta
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Cinta" }
                    div { class: "relative w-full flex items-center",
                        select {
                            class: "w-full p-2 pr-10 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none transition-colors cursor-pointer",
                            value: {
                                let r = *rango.read();
                                if r <= 0 { "0".to_string() } else { r.to_string() }
                            },
                            style: "-webkit-appearance: none; appearance: none; background-color: #111827;",
                            onchange: move |e| {
                                if let Ok(val) = e.value().parse::<i32>() {
                                    rango.set(val);
                                }
                            },
                            {Cintas::all_variants().iter().map(|cinta| {
                                let v_cinta = cinta.valor();
                                let r_actual = *rango.read();

                                let is_selected = if r_actual <= 0 {
                                    v_cinta == 0
                                } else {
                                    v_cinta == r_actual as u32
                                };

                                rsx! {
                                    option {
                                        class: "bg-gray-900",
                                        value: "{v_cinta}",
                                        selected: is_selected,
                                        "{cinta.label()}"
                                    }
                                }
                            })}
                        }

                        div {
                            class: "pointer-events-none absolute text-gray-400 flex items-center",
                            style: "right: 12px; top: 50%; transform: translateY(-50%);",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                stroke_width: "2",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19 9l-7 7-7-7" }
                            }
                        }
                    }
                }
            }

            div { class: "grid grid-cols-2 gap-5 items-end",
                // Campo: Representante (búsqueda por nombre)
                div { class: "flex flex-col space-y-2",
                    label { class: "text-sm font-semibold text-gray-400", "Representante" }
                    div { class: "relative w-full",
                        input {
                            r#type: "text",
                            class: "w-full p-2 pr-10 rounded-lg bg-gray-900 text-gray-100 {get_border_class(rep_valido, intentado.read().clone())} outline-none transition-colors",
                            placeholder: "{rep_placeholder}",
                            value: if *rep_dropdown_open.read() {
                                busqueda_rep.read().clone()
                            } else {
                                rep_seleccionado_nombre.clone()
                            },
                            oninput: move |e| {
                                let val = e.value();
                                busqueda_rep.set(val.clone());
                                rep_dropdown_open.set(true);
                                if *representante_id.read() != 0 {
                                    let nombre_actual = representantes.iter().find(|r| r.id == *representante_id.read()).map(|r| r.nombre.clone()).unwrap_or_default();
                                    if nombre_actual != val {
                                        representante_id.set(0);
                                    }
                                }
                            },
                            onfocus: move |_| {
                                rep_dropdown_open.set(true);
                                busqueda_rep.set(String::new());
                            },
                        }
                        div {
                            class: "pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3 text-gray-400",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                stroke_width: "2",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19 9l-7 7-7-7" }
                            }
                        }
                        if *rep_dropdown_open.read() {
                            div {
                                class: "absolute z-50 mt-1 w-full bg-gray-900 border border-gray-700 rounded-lg shadow-lg max-h-32 overflow-y-auto",
                                {
                                    let q = busqueda_rep.read().to_lowercase();
                                    let filtrados: Vec<_> = representantes.iter().filter(|r| q.is_empty() || r.nombre.to_lowercase().contains(&q)).collect();
                                    if filtrados.is_empty() {
                                        rsx! {
                                            div { class: "px-3 py-2 text-xs text-gray-500",
                                                "Sin resultados"
                                            }
                                        }
                                    } else {
                                        rsx! {
                                            div { class: "px-3 py-2 text-[10px] text-gray-500 uppercase tracking-wider",
                                                "{filtrados.len()} resultado(s)"
                                            }
                                            {filtrados.iter().map(|rep| {
                                                let id = rep.id;
                                                let nombre = rep.nombre.clone();
                                                let seleccionado = *representante_id.read() == id;
                                                let clase_item = if seleccionado {
                                                    "flex items-center justify-between px-3 py-2 text-sm cursor-pointer transition-colors bg-blue-600/20 text-blue-300"
                                                } else {
                                                    "flex items-center justify-between px-3 py-2 text-sm cursor-pointer transition-colors text-gray-200 hover:bg-gray-800"
                                                };
                                                rsx! {
                                                    div {
                                                        key: "{id}",
                                                        class: "{clase_item}",
                                                        onclick: move |_| {
                                                            representante_id.set(id);
                                                            rep_dropdown_open.set(false);
                                                            busqueda_rep.set(nombre.clone());
                                                        },
                                                        span { "{nombre}" }
                                                        if seleccionado {
                                                            span { class: "text-blue-400 text-xs", "✓" }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Campo: Rallita / Danes
                {
                    if rango.read().clone() > 0 {
                        rsx! {
                            label { class: "flex items-center space-x-3 p-[11px] bg-gray-900/50 rounded-lg border border-gray-700 cursor-pointer transition-colors h-[42px]",
                                input {
                                    r#type: "checkbox",
                                    class: "w-5 h-5 rounded accent-blue-500 bg-gray-900 border-gray-700 focus:ring-blue-500/50 cursor-pointer",
                                    checked: "{rallita}",
                                    onchange: move |_| rallita.set(!rallita.cloned())
                                }
                                span { class: "text-sm font-medium text-gray-300 select-none", "Grado con Rallita" }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "flex flex-col space-y-2",
                                div { class: "relative w-full flex items-center",
                                    select {
                                        class: "w-full p-2 pr-10 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none transition-colors cursor-pointer h-[42px] text-sm font-medium",
                                        style: "-webkit-appearance: none; appearance: none; background-color: #111827;",
                                        onchange: move |e| {
                                            if let Ok(dan) = e.value().parse::<i32>() {
                                                rango.set(Cintas::rango_desde_dan(dan));
                                            }
                                        },
                                        {(1..=10).map(|dan| rsx! {
                                            option { class: "bg-gray-900", value: "{dan}", selected: *rango.read() == Cintas::rango_desde_dan(dan), "{dan}° Dan" }
                                        })}
                                    }
                                    div {
                                        class: "pointer-events-none absolute text-gray-400 flex items-center",
                                        style: "right: 12px; top: 50%; transform: translateY(-50%);",
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            stroke_width: "2",
                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19 9l-7 7-7-7" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            button {
                class: color_btn,
                onclick: move |_| {
                    if form_valido {
                        on_click.call(());
                        intentado.set(false);
                    } else {
                        intentado.set(true);
                    }
                },
                {texto_boton}
            }
        }
    }
}
