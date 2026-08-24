use crate::presentation::my_app::Columnas;
use dioxus::prelude::*;

#[component]
pub fn SearchBar(
    on_input: EventHandler<(Columnas, String)>,
    options: Vec<(String, Columnas)>,
    placeholder: String,
    initial_param: Columnas,
) -> Element {
    let mut search_text = use_signal(|| "".to_string());
    let mut selected_param = use_signal(|| initial_param);

    let notificar = move || {
        on_input.call((selected_param.cloned(), search_text.cloned()));
    };

    rsx! {
        div { class: "flex flex-row space-x-2 p-4 bg-white rounded-xl shadow-md border border-gray-200",
            // Dropdown para el parámetro
            select {
                class: "p-2 rounded bg-gray-50 border border-gray-300 text-gray-700",
                value: "{options.iter().position(|(_, value)| *value == *selected_param.read()).unwrap_or(0)}",
                onchange: move |evt| {
                    if let Ok(index) = evt.value().parse::<usize>() {
                        if let Some((_, option_value)) = options.get(index) {
                            selected_param.set(*option_value);
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

            // Input de texto
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
