use crate::models::Cintas;
use dioxus::prelude::*;

#[component]
pub fn PromotionForm(
    rango: Signal<i32>,
    rallita: Signal<bool>,
    texto_boton: &'static str,
    on_click: EventHandler<()>,
) -> Element {
    // 1. Estado reactivo para rastrear la modificación entre renderizados
    let mut modificado = use_signal(|| false);

    // Estilos dinámicos para el botón basados en si se modificó el grado
    let color_btn = if *modificado.read() {
        "w-full mt-4 py-3 bg-blue-600 text-white font-bold rounded-xl hover:bg-blue-700 shadow-lg shadow-blue-950 transition-all active:scale-[0.98] cursor-pointer text-sm"
    } else {
        "w-full mt-4 py-3 bg-gray-700 text-gray-400 font-bold rounded-xl cursor-not-allowed transition-all text-sm"
    };

    // Estilos dinámicos para la caja del checkbox (para que se vea gris si está bloqueada)
    let clases_label_rallita = if *modificado.read() {
        "flex items-center space-x-3 p-[11px] bg-gray-900/50 rounded-lg border border-gray-700 cursor-pointer hover:bg-gray-700/50 transition-colors h-[42px]"
    } else {
        "flex items-center space-x-3 p-[11px] bg-gray-800 rounded-lg border border-gray-700 cursor-not-allowed opacity-40 h-[42px]"
    };

    rsx! {
        div { class: "flex-1 flex flex-col justify-around bg-gray-800 p-8 rounded-2xl shadow-xl transition-all duration-300 ease-out hover:-translate-y-1 hover:shadow-2xl text-gray-100",

            div { class: "grid grid-cols-2 gap-4 items-end",

                // Columna Izquierda: Selección de Cinta
                div { class: "flex flex-col space-y-1",
                    label { class: "text-sm font-semibold text-gray-400", "Cinta" }
                    div { class: "relative w-full flex items-center",
                        select {
                            class: "w-full p-2 pr-10 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none transition-colors cursor-pointer",
                            value: {
                                let r = *rango.read();
                                if r == 99 { "99".to_string() } else if r <= 0 { "0".to_string() } else { r.to_string() }
                            },
                            style: "-webkit-appearance: none; appearance: none; background-color: #111827;",
                            onchange: move |e| {
                                if let Ok(val) = e.value().parse::<i32>() {
                                    modificado.set(true); // Persiste el cambio correctamente
                                    rango.set(val);
                                }
                            },

                            // Placeholder neutro inicial
                            option {
                                class: "bg-gray-900 text-gray-400 font-semibold",
                                value: "99",
                                selected: *rango.read() == 99,
                                disabled: true,
                                "-- Seleccionar Grado --"
                            }

                            {Cintas::all_variants().iter().map(|cinta| {
                                let v_cinta = cinta.valor();
                                let r_actual = *rango.read();

                                let is_selected = r_actual != 99 && (if r_actual <= 0 {
                                    v_cinta == 0
                                } else {
                                    v_cinta as i32 == r_actual
                                });

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

                // Columna Derecha: Checkbox de Rallita o Selector de Dan
                {
                    if *rango.read() > 0 {
                        rsx! {
                            label { class: "{clases_label_rallita}",
                                input {
                                    r#type: "checkbox",
                                    class: "w-5 h-5 rounded accent-blue-500 bg-gray-900 border-gray-700 focus:ring-blue-500/50 cursor-pointer",
                                    checked: "{rallita}",
                                    disabled: !*modificado.read(),
                                    onchange: move |_| {
                                        if *modificado.read() {
                                            rallita.set(!rallita.cloned());
                                        }
                                    }
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
                                                let mut val = dan;
                                                val = val * -1;
                                                val = val + 1;
                                                rango.set(val);
                                            }
                                        },
                                        {(1..=10).map(|dan| rsx! {
                                            option { class: "bg-gray-900", value: "{dan}", "{dan}° Dan" }
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

            // Botón de guardado masivo
            button {
                class: color_btn,
                onclick: move |_| {
                    if *modificado.read() {
                        on_click.call(());
                    }
                },
                {texto_boton}
            }
        }
    }
}
