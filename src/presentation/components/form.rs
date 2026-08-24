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
    let (nombre_valido, fecha_valida, rep_valido) = campos_validos;
    let form_valido = nombre_valido && fecha_valida && rep_valido;

    let color_btn = if form_valido {
        "w-full mt-4 py-3 bg-blue-600 text-white font-bold rounded-xl hover:bg-blue-700 shadow-lg shadow-blue-950 transition-all active:scale-[0.98] cursor-pointer"
    } else {
        "w-full mt-4 py-3 bg-gray-700 text-gray-400 font-bold rounded-xl cursor-pointer active:scale-[0.98] transition-all"
    };

    rsx! {
        div { class: "flex-1 flex flex-col justify-around bg-gray-800 p-8 rounded-2xl shadow-xl transition-all duration-300 ease-out hover:-translate-y-1 hover:shadow-2xl text-gray-100 ",

            // Campo: Nombre
            div { class: "flex flex-col space-y-1",
                label { class: "text-sm font-semibold text-gray-400", "Nombre Completo" }
                input {
                    r#type: "text",
                    class: "p-2 rounded-lg outline-none transition-colors bg-gray-900 text-gray-100 {get_border_class(nombre_valido, intentado.read().clone())}",
                    placeholder: "Ej: Juan Pérez",
                    value: "{nombre}",
                    oninput: move |e| nombre.set(e.value())
                }
            }

            div { class: "grid grid-cols-2 gap-4",
                // Campo: Fecha de Nacimiento
                div { class: "flex flex-col space-y-1",
                    label { class: "text-sm font-semibold text-gray-400", "Fecha de Nacimiento" }
                    input {
                        r#type: "date",
                        class: "p-2 rounded-lg outline-none transition-colors bg-gray-900 text-gray-100 [color-scheme:dark] {get_border_class(fecha_valida, intentado.read().clone())}",
                        value: "{fecha_nac}",
                        oninput: move |e| fecha_nac.set(e.value())
                    }
                }

                // Campo: Grado / Cinta
                div { class: "flex flex-col space-y-1",
                    label { class: "text-sm font-semibold text-gray-400", "Cinta" }
                    div { class: "relative w-full flex items-center",
                        select {
                            // pr-10 evita que el texto largo pise la flecha
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

                        // Flecha posicionada con inline style para WebKit
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
                }}

            // Campo: Representante (selector del catálogo, relación por ID)
            div { class: "flex flex-col space-y-1",
                label { class: "text-sm font-semibold text-gray-400", "Representante" }
                div { class: "relative w-full flex items-center",
                    select {
                        class: "w-full p-2 pr-10 rounded-lg bg-gray-900 text-gray-100 {get_border_class(rep_valido, intentado.read().clone())} outline-none transition-colors cursor-pointer",
                        style: "-webkit-appearance: none; appearance: none; background-color: #111827;",
                        value: "{representante_id}",
                        onchange: move |e| {
                            if let Ok(id) = e.value().parse::<usize>() {
                                representante_id.set(id);
                            }
                        },
                        option {
                            class: "bg-gray-900",
                            value: "0",
                            selected: *representante_id.read() == 0,
                            disabled: true,
                            "-- Seleccione un representante --"
                        }
                        {representantes.iter().map(|rep| {
                            let id = rep.id;
                            rsx! {
                                option {
                                    class: "bg-gray-900",
                                    value: "{id}",
                                    selected: *representante_id.read() == id,
                                    "{rep.nombre}"
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

            // Campo: Rallita / Danes
            div { class: "grid grid-cols-2 gap-4 items-end",
                {
                    if rango.read().clone() > 0 {
                                            rsx! {
                                                label { class: "flex items-center space-x-3 p-[11px] bg-gray-900/50 rounded-lg border border-gray-700 cursor-pointer hover:bg-gray-700/50 transition-colors h-[42px]",
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
                                                div { class: "flex flex-col space-y-1 w-full",
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

                                                        // Flecha manual idéntica para el dropdown de Danes
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
