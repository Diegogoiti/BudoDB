use crate::models::Alumno;
use crate::my_app;
use crate::Route;
use dioxus::prelude::*;

///componente que recibe un contexto con una clase myapp y clona el vertor alumnos
/// para dibujar la tabla de los datos en la ventana
#[component]
pub fn DataTable(
    alumnos_lista: Signal<Vec<Alumno>>,
    estado: Signal<my_app::MyApp>,
    aplicar_color_seleccion: bool,
) -> Element {
    let alumnos = alumnos_lista.read().clone();
    let nav = use_navigator();
    rsx! {

        div { class: "overflow-auto rounded-xl border border-gray-800 bg-gray-900 shadow-xl ",
            table { class: "w-full border-collapse text-left text-xs md:text-sm table-auto",
                thead {
                    // sticky y top-0 mantienen la fila visible al bajar
                    tr { class: "sticky top-0 text-white bg-gray-800",
                        // Aplicamos sticky y top-0 a cada th para asegurar compatibilidad
                        // Agregamos z-20 para que los datos no pasen "por encima" de la cabecera
                        th { class: " z-20  px-4 py-3", "Sel." }
                        th { class: " z-20  px-4 py-3", "ID" }
                        th { class: " z-20  px-4 py-3", "Nombre" }
                        th { class: " z-20  px-4 py-3", "Cinta" }
                        th { class: " z-20  px-4 py-3", "Rango" }
                        th { class: " z-20  px-4 py-3", "Edad" }
                        th { class: " z-20  px-4 py-3", "F. Nacimiento" }
                        th { class: " z-20  px-4 py-3", "Representante" }
                        th { class: " z-20 px-4 py-3", "Teléfono" }
                    }
                }
                tbody { class: "divide-y divide-gray-800 text-gray-300",
                    for (i, alumno) in alumnos.into_iter().enumerate() {
                        tr {
                            class: {
                                let es_seleccionado = estado.read().seleccionados.contains(&alumno.id);
                                let base = if aplicar_color_seleccion && es_seleccionado {
                                    "bg-blue-500 hover:bg-blue-700 transition-colors"
                                } else {
                                    if i % 2 == 0 { "bg-gray-850 hover:bg-gray-700 transition-colors" } else { "bg-gray-800 hover:bg-gray-700 transition-colors" }
                                };
                                //let hover = "bg-gray-750";
                                base
                                },
                            onclick: move |_| {
                                estado.write().toggle_seleccion(alumno.id);
                            },
                            ondoubleclick: move |_| {
                            if !estado.read().seleccionados.contains(&alumno.id) {
                                estado.write().toggle_seleccion(alumno.id);
                            }
                            nav.push(Route::Editar {});
                            },
                            td { class: "px-4 py-3",
                                input {
                                    r#type: "checkbox",
                                    class: "w-4 h-4 rounded border-gray-700 bg-gray-800 text-blue-600 focus:ring-blue-500",
                                    checked: estado.read().seleccionados.contains(&alumno.id),

                                }
                            }
                            td { class: "px-4 py-3 font-mono text-gray-500", "#{alumno.id}" }
                            td { class: "px-4 py-3 font-bold text-white whitespace-nowrap", "{alumno.nombre}" }
                            td { class: "px-4 py-3",
                                span { class: "inline-flex items-center justify-center min-w-36 px-3 py-1.5 rounded bg-gray-700 text-[10px] uppercase font-bold text-gray-300 whitespace-nowrap",
                                    "{alumno.cinta()}"
                                }
                            }
                                                        td { class: "px-4 py-3",
                                span { class: "inline-flex items-center justify-center min-w-20 px-2 py-1.5 rounded bg-gray-700 text-[10px] uppercase font-bold text-gray-300 whitespace-nowrap",
                                    "{alumno.rango()}"
                                }
                            }
                            td { class: "px-4 py-3 whitespace-nowrap", "{alumno.edad()}" }
                            td { class: "px-4 py-3", "{alumno.fecha_de_nacimiento}" }
                            td { class: "px-4 py-3 whitespace-nowrap", "{alumno.representante}" }
                            td { class: "px-4 py-3 text-blue-400 font-mono whitespace-nowrap",
                                "{alumno.numero_contacto}"
                            }
                        }
                    }
                }
            }
        }
    }
}
