use crate::domain::Cintas;
use crate::presentation::my_app::Columnas;
use dioxus::prelude::*;

#[component]
pub fn Filter(
    on_input: EventHandler<(Columnas, String, bool)>, // Agregamos bool aquí
    options: Vec<(String, Columnas)>,
    placeholder: String,
    initial_param: Columnas,
) -> Element {
    let mut con_rallita = use_signal(|| false);
    let cintas = Cintas::all_variants();
    let special_cintas = ["Azul (todos)", "Marrón (todos)"];

    let mut search_text = use_signal(|| "".to_string());
    let mut selected_param = use_signal(|| initial_param);

    // Actualizamos notificar para incluir el valor del checkbox
    let notificar = move || {
        on_input.call((
            selected_param.cloned(),
            search_text.cloned(),
            con_rallita.cloned(),
        ));
    };

    rsx! {
        div { class: "flex flex-row items-center space-x-3 p-3 bg-gray-800 rounded-xl shadow-md border border-gray-700",

            // 1. Dropdown de parámetro (Nombre, Edad, Cinta...)
            select {
                class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50 text-xs",
                value: "{options.iter().position(|(_, value)| *value == *selected_param.read()).unwrap_or(0)}",
                onchange: move |evt| {
                    if let Ok(index) = evt.value().parse::<usize>() {
                        if let Some((_, option_value)) = options.get(index) {
                            selected_param.set(*option_value);
                            search_text.set("".to_string());
                        }
                    }
                    notificar();
                },
                {options.iter().enumerate().map(|(index, (label, option_value))| rsx! {
                    option {
                        value: "{index}",
                        selected: *selected_param.read() == *option_value,
                        "{label}"
                    }
                })}
            }

            // 2. Input dinámico (Dropdown de cintas o Input de edad)
            match *selected_param.read() {
                Columnas::Cinta => rsx! {
                    select {
                        class: "flex-1 p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50 text-xs",
                        value: "{search_text}",
                        onchange: move |evt| {
                            search_text.set(evt.value());
                            notificar();
                        },
                        // Opción para quitar el filtro por completo
                        option {
                            value: "",
                            selected: search_text.read().is_empty(),
                            "Todas"
                        }
                        {cintas.iter().map(|cinta| rsx! {
                            option {
                                value: "{cinta.label()}",
                                selected: search_text.read().as_str() == cinta.label(),
                                "{cinta.label()}"
                            }
                        })}
                        {special_cintas.iter().map(|label| rsx! {
                            option {
                                value: "{label}",
                                selected: search_text.read().as_str() == *label,
                                "{label}"
                            }
                        })}
                    }
                    // 3. CHECKBOX (Ubicado al lado del input/dropdown anterior)
            label { class: "flex items-center space-x-2 text-xs font-medium text-gray-300 cursor-pointer bg-gray-900 p-2 rounded-lg border border-gray-700",
                input {
                    r#type: "checkbox",
                    class: "w-4 h-4 text-blue-600 rounded border-gray-600 bg-gray-800 focus:ring-blue-500",
                    checked: "{con_rallita}",
                    onchange: move |_| {
                        con_rallita.set(!con_rallita.cloned());
                        notificar();
                    }
                }
                span { "Con Rallita" }
            }
                },
                Columnas::Edad => rsx! {
                    input {
                        r#type: "number",
                        class: "flex-1 p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none text-xs placeholder-gray-500",
                        placeholder: "Filtrar por edad...",
                        value: "{search_text}",
                        oninput: move |evt| {
                            search_text.set(evt.value());
                            notificar();
                        },
                    }
                },
                _ => rsx! {
                    input {
                        class: "flex-1 p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none text-xs placeholder-gray-500",
                        placeholder: "{placeholder}",
                        value: "{search_text}",
                        oninput: move |evt| {
                            search_text.set(evt.value());
                            notificar();
                        },
                    }
                }
            }
        }
    }
}
