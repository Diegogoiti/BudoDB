use crate::models::Cintas;
use dioxus::prelude::*;

#[component]
pub fn PromotionForm(
    rango: Signal<i32>,
    rallita: Signal<bool>,
    texto_boton: &'static str,
    on_click: EventHandler<()>,
) -> Element {
    let color_btn = "w-full mt-4 py-3 bg-blue-600 text-white font-bold rounded-xl hover:bg-blue-700 shadow-lg shadow-blue-950 transition-all active:scale-[0.98] cursor-pointer";

    rsx! {
        // Contenedor calcado del tuyo: fondo gris, p-8, rounded-2xl y sombras
        div { class: "flex-1 flex flex-col justify-around bg-gray-800 p-8 rounded-2xl shadow-xl transition-all duration-300 ease-out hover:-translate-y-1 hover:shadow-2xl text-gray-100",

            // Fila de dos columnas exacta a tu sección de contacto/rallita
            div { class: "grid grid-cols-2 gap-4 items-end",

                // Columna Izquierda: Selección de Cinta (Copiado de tu Form)
                div { class: "flex flex-col space-y-1",
                    label { class: "text-sm font-semibold text-gray-400", "Cinta" }
                    select {
                        class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none transition-colors cursor-pointer",
                        value: {
                            let r = *rango.read();
                            if r <= 0 { "0".to_string() } else { r.to_string() }
                        },
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
                                v_cinta as i32 == r_actual
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
                }

                // Columna Derecha: Checkbox de Rallita o Selector de Dan (Copiado de tu Form)
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
                                select {
                                    class: "w-full p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none transition-colors cursor-pointer h-[42px] text-sm font-medium",
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
                            }
                        }
                    }
                }
            }

            // Botón de guardado masivo con tu estética original
            button {
                class: color_btn,
                onclick: move |_| {
                    on_click.call(());
                },
                {texto_boton}
            }
        }
    }
}
