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

    let mut search_text = use_signal(|| {
        if initial_param == Columnas::Cinta {
            cintas[0].label().to_string()
        } else {
            "".to_string()
        }
    });
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
        div { class: "flex flex-row items-center space-x-4 p-4 bg-white rounded-xl shadow-md border border-gray-200",

            // 1. Dropdown de parámetro (Nombre, Edad, Cinta...)
            select {
                class: "p-2 rounded bg-gray-50 border border-gray-300 text-gray-700",
                value: "{options.iter().position(|(_, value)| *value == *selected_param.read()).unwrap_or(0)}",
                onchange: move |evt| {
                    if let Ok(index) = evt.value().parse::<usize>() {
                        if let Some((_, option_value)) = options.get(index) {
                            selected_param.set(*option_value);
                            if *option_value == Columnas::Cinta {
                                search_text.set(cintas[0].label().to_string());
                            } else {
                                search_text.set("".to_string());
                            }
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
                        class: "flex-1 p-2 rounded bg-gray-50 border border-gray-300 text-gray-700",
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
            label { class: "flex items-center space-x-2 text-sm font-medium text-gray-700 cursor-pointer bg-gray-50 p-2 rounded border border-gray-200",
                input {
                    r#type: "checkbox",
                    class: "w-4 h-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500",
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
                        class: "flex-1 p-2 rounded border border-gray-300 focus:ring-2 focus:ring-blue-500 outline-none",
                        placeholder: "Filtrar por edad...",
                        value: "{search_text}",
                        r#type: "number",
                        oninput: move |evt| {
                            search_text.set(evt.value());
                            notificar();
                        },
                    }
                },
                _ => rsx! {
                    input {
                        class: "flex-1 p-2 rounded border border-gray-300 focus:ring-2 focus:ring-blue-500 outline-none",
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
