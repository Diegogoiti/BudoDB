use crate::models::{Alumno, Cintas};
use dioxus::prelude::*;

#[component]
pub fn Form(
    nombre: Signal<String>,
    fecha_nac: Signal<String>,
    rango: Signal<u32>,
    representante: Signal<String>,
    contacto: Signal<String>,
    rallita: Signal<bool>,
) -> Element {
    rsx! {
        // Contenedor del Formulario[cite: 6, 8]
                div { class: "flex-1 flex flex-col justify-around bg-white p-8 rounded-2xl shadow-xl transition-all duration-300 ease-out hover:-translate-y-1 hover:shadow-2xl",

                    // Campo: Nombre
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-sm font-semibold text-gray-600", "Nombre Completo" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg border border-gray-300 focus:ring-2 focus:ring-blue-500 outline-none",
                            placeholder: "Ej. Ichiro Suzuki",
                            oninput: move |e| nombre.set(e.value())
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        // Campo: Fecha de Nacimiento
                        div { class: "flex flex-col space-y-1",
                            label { class: "text-sm font-semibold text-gray-600", "Fecha de Nacimiento" }
                            input {
                                r#type: "date",
                                class: "p-2 rounded-lg border border-gray-300 focus:ring-2 focus:ring-blue-500 outline-none",
                                oninput: move |e| fecha_nac.set(e.value())
                            }
                        }

                        // Campo: Grado / Cinta[cite: 2]
                        div { class: "flex flex-col space-y-1",
                            label { class: "text-sm font-semibold text-gray-600", "Grado (Kyu)" }
                            select {
                                class: "p-2 rounded-lg border border-gray-300 bg-gray-50",
                                onchange: move |e| {
                                    if let Ok(val) = e.value().parse::<u32>() {
                                        rango.set(val);
                                    }
                                },
                                {Cintas::all_variants().iter().map(|cinta| rsx! {
                                    option { value: "{cinta.valor()}", "{cinta.label()}" }
                                })}
                            }
                        }
                    }

                    // Campo: Representante
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-sm font-semibold text-gray-600", "Representante" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg border border-gray-300 focus:ring-2 focus:ring-blue-500 outline-none",
                            placeholder: "Nombre del padre o tutor",
                            oninput: move |e| representante.set(e.value())
                        }
                    }

                    // Campo: Contacto y Rallita
                    div { class: "grid grid-cols-2 gap-4 space-y-1 ",
                        div { class: "flex flex-col space-y-1",
                            label { class: "text-sm font-semibold text-gray-600", "Teléfono de Contacto" }
                            input {
                                r#type: "tel",
                                maxlength: "12",
                                value: "{contacto}",
                                class: "p-2 rounded-lg border border-gray-300 focus:ring-2 focus:ring-blue-500 outline-none",
                                placeholder: "0412-0000000",
                                oninput: move |e| {
        let mut val = e.value();

        // 1. Limpiar: solo números
        val.retain(|c| c.is_ascii_digit());

        // 2. Formatear: solo insertamos si hay suficientes números
        if val.len() > 4 {
            // Inserta el guion en la posición 4
            val.insert(4, '-');
        }

        // 3. Limitar (opcional, pero útil para no exceder el espacio)
        val.truncate(12);

        contacto.set(val);
    }
                            }
                        }

                        // Checkbox de Rallita[cite: 6]
                        label { class: "flex items-center space-x-3 p-3 bg-gray-50 rounded-lg border border-gray-200 cursor-pointer hover:bg-gray-100 transition-colors",
                            input {
                                r#type: "checkbox",
                                class: "w-5 h-5 text-blue-600 rounded border-gray-300 focus:ring-blue-500",
                                onchange: move |_| rallita.set(!rallita.cloned())
                            }
                            span { class: "text-sm font-medium text-gray-700", "Grado con Rallita" }
                        }
                    }

                    // Botón de Acción
                    button {
                        class: "w-full mt-4 py-3 bg-blue-600 text-white font-bold rounded-xl hover:bg-blue-700 shadow-lg shadow-blue-200 transition-all active:scale-[0.98]",
                        onclick: move |_| {
                            let alumno = Alumno {
                                id: 0,
                                nombre: nombre.read().clone(),
                                fecha_de_nacimiento: fecha_nac.read().clone(),
                                rango: rango.read().clone(),
                                representante: representante.read().clone(),
                                numero_contacto: contacto.read().clone(),
                                rallita: rallita.read().clone(),
                            };
                            println!("{alumno}");
                            // Aquí conectarás con app_state.write().database.save(...) más adelante
                        },
                        "Añadir Al Dojo"
                    }
                }



            }
}
