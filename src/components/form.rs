use crate::models::{Alumno, Cintas, Database};
use crate::my_app::MyApp;
use dioxus::prelude::*;

#[component]
pub fn Form(
    nombre: Signal<String>,
    fecha_nac: Signal<String>,
    rango: Signal<u32>,
    representante: Signal<String>,
    contacto: Signal<String>,
    rallita: Signal<bool>,
    campos_validos: (bool, bool, bool, bool),
) -> Element {
    fn get_border_class(es_valido: bool, intentando: bool) -> &'static str {
        if !intentando || es_valido {
            // Normal: Borde oscuro, ring azul sutil al enfocar
            "border border-gray-700 focus:ring-2 focus:ring-blue-500/50"
        } else {
            // Error: Borde rojo, ring rojo (más intenso)
            "border border-red-500 ring-2 ring-red-500/30 focus:ring-red-500/50"
        }
    }

    let mut estado = use_context::<Signal<MyApp>>();

    let mut intentado = use_signal(|| false);

    let (nombre_valido, fecha_valida, representante_valido, contacto_valido) = campos_validos;

    let form_valido = nombre_valido && fecha_valida && representante_valido && contacto_valido;

    let color_btn = if form_valido {
        "w-full mt-4 py-3 bg-blue-600 text-white font-bold rounded-xl hover:bg-blue-700 shadow-lg shadow-blue-950 transition-all active:scale-[0.98] cursor-pointer"
    } else {
        "w-full mt-4 py-3 bg-gray-700 text-gray-400 font-bold rounded-xl cursor-pointer active:scale-[0.98] transition-all"
    };

    rsx! {
        // Contenedor del Formulario
        div { class: "flex-1 flex flex-col justify-around bg-gray-800 p-8 rounded-2xl shadow-xl transition-all duration-300 ease-out hover:-translate-y-1 hover:shadow-2xl text-gray-100",

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
                    label { class: "text-sm font-semibold text-gray-400", "Grado (Kyu)" }
                    select {
                        class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 focus:ring-2 focus:ring-blue-500/50 outline-none transition-colors cursor-pointer",
                        value: "{rango}",
                        onchange: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                rango.set(val);
                            }
                        },
                        {Cintas::all_variants().iter().map(|cinta| rsx! {
                            option { class: "bg-gray-900", value: "{cinta.valor()}", "{cinta.label()}" }
                        })}
                    }
                }
            }

            // Campo: Representante
            div { class: "flex flex-col space-y-1",
                label { class: "text-sm font-semibold text-gray-400", "Representante" }
                input {
                    r#type: "text",
                    class: "p-2 rounded-lg outline-none transition-colors bg-gray-900 text-gray-100 {get_border_class(representante_valido, intentado.read().clone())}",
                    placeholder: "Nombre del padre o tutor",
                    value: "{representante}",
                    oninput: move |e| representante.set(e.value())
                }
            }

            // Campo: Contacto y Rallita
            div { class: "grid grid-cols-2 gap-4 items-end",
                div { class: "flex flex-col space-y-1",
                    label { class: "text-sm font-semibold text-gray-400", "Teléfono de Contacto" }
                    input {
                        r#type: "tel",
                        maxlength: "12",
                        value: "{contacto}",
                        class: "p-2 rounded-lg bg-gray-900 text-gray-100 outline-none transition-colors {get_border_class(contacto_valido, intentado.read().clone())}",
                        placeholder: "0412-0000000",
                        oninput: move |e| {
                            let mut val = e.value();
                            val.retain(|c| c.is_ascii_digit());
                            if val.len() > 4 {
                                val.insert(4, '-');
                            }
                            val.truncate(12);
                            contacto.set(val);
                        }
                    }
                }

                // Checkbox de Rallita (Unificado al Modo Oscuro)
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

            button {
                class: color_btn,
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

                    if form_valido {
                        println!("Formulario válido, se puede agregar el alumno.");
                        let _ = estado.read().database.save(&alumno);
                        estado.write().update();
                        nombre.set("".to_string());
                        fecha_nac.set("".to_string());
                        representante.set("".to_string());
                        contacto.set("".to_string());
                        rallita.set(false);
                        rango.set(10); // Opcional: Descomenta si quieres reiniciar la cinta a blanca (10)

                        // 3. Reiniciamos el indicador de intentos de validación
                        intentado.set(false);

                    } else {
                        println!("Formulario inválido, por favor corrige los errores.");
                        intentado.set(true);
                    }
                },
                "Añadir Al Dojo"
            }
        }
    }
}
