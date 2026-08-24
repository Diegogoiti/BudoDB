//! se encarga de dibujar las vistas segun las rutas seleccionadas
//! contiene las funciones con el codigo especifico de cada vista

//use std::string;

//use std::intrinsics::fabs;
use std::{usize, vec};

use crate::application::dto::DatosAlumno;
use crate::application::validation::*;
use crate::presentation::components::datatable::DataTable;
use crate::presentation::components::filter::Filter;
use crate::presentation::components::form::Form;
use crate::presentation::components::promotion_form::PromotionForm;
use crate::presentation::components::searchbar::SearchBar;
use crate::presentation::my_app::{self, Columnas};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let todos_seleccionados = !estado.read().alumnos.is_empty()
        && estado
            .read()
            .alumnos
            .iter()
            .all(|a| estado.read().seleccionados.contains(&a.id));
    let texto_boton = if todos_seleccionados {
        "Deseleccionar todos"
    } else {
        "Seleccionar todos"
    };
    let alumnos_lista = use_signal(|| estado.read().alumnos.clone());

    rsx! {
        div { class: "flex flex-col h-full space-y-4 ",
            div { class: "relative flex items-center justify-center py-2",
                h2 { class: "text-3xl font-bold text-gray-800 text-center", "Consultar" }
                button {
                    class: "absolute right-0 px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors text-sm",
                    onclick: move |_| {
                        estado.write().toggle_all(alumnos_lista.read().clone());
                    },
                    "{texto_boton}"
                }
            }

            DataTable { alumnos_lista, estado, aplicar_color_seleccion: true }

            div { class: "flex  justify-between items-center ",
                div { class: "text-gray-500 text-xs ",
                    "Mostrando {estado.read().alumnos.len()} alumnos registrados"
                }
                div { class: "text-gray-500 text-xs ",
                    "alumnos seleccionados: {estado.read().seleccionados.len()}"
                }
            }

        }
    }
}

#[component]
pub fn Buscar() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    let mut filtro = use_signal(|| (my_app::Columnas::Nombre, String::new()));
    let alumnos_filtrados = use_signal(|| estado.read().alumnos.clone());
    let todos_seleccionados = !alumnos_filtrados.read().is_empty()
        && alumnos_filtrados
            .read()
            .iter()
            .all(|a| estado.read().seleccionados.contains(&a.id));
    let texto_boton = if todos_seleccionados {
        "Deseleccionar todos"
    } else {
        "Seleccionar todos"
    };

    {
        let filtro = filtro.clone();
        let estado = estado.clone();
        let mut alumnos_filtrados = alumnos_filtrados.clone();
        use_effect(move || {
            let app = estado.read();
            alumnos_filtrados.set(app.buscar_alumnos(filtro.read().0, &filtro.read().1));
        });
    }

    rsx! {
        div { class: "flex flex-col h-full space-y-4",
            div { class: "relative flex items-center justify-center py-2",
                h2 { class: "text-3xl font-bold text-gray-800 text-center", "Buscar" }
                button {
                    class: "absolute right-0 px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors text-sm",
                    onclick: move |_| {
                        estado.write().toggle_all(alumnos_filtrados.read().clone());
                    },
                    "{texto_boton}"
                }
            }

            SearchBar {
                on_input: move |data| filtro.set(data),
                options: vec![
                    ("Id".to_string(), my_app::Columnas::Id),
                    ("Nombre".to_string(), my_app::Columnas::Nombre),
                    ("Representante".to_string(), my_app::Columnas::Representante),
                    ("Teléfono".to_string(), my_app::Columnas::Telefono),
                ],
                placeholder: "Buscar alumno...".to_string(),
                initial_param: my_app::Columnas::Nombre,
            }
            DataTable { alumnos_lista: alumnos_filtrados, estado, aplicar_color_seleccion: true  }

            div { class: "flex  justify-between items-center ",
                div { class: "text-gray-500 text-xs ",
                    "Mostrando {alumnos_filtrados.read().len()} alumnos en pantalla"
                }
                div { class: "text-gray-500 text-xs ",
                    "alumnos seleccionados: {estado.read().seleccionados.len()}"
                }
            }
        }

    }
}

#[component]
pub fn Filtrar() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    // Iniciamos con Cinta por defecto para que coincida con el componente Filter
    let mut filtro = use_signal(|| (my_app::Columnas::Cinta, "Blanca".to_string(), false));
    let mut alumnos_filtrados = use_signal(|| estado.read().alumnos.clone());

    let todos_seleccionados = !alumnos_filtrados.read().is_empty()
        && alumnos_filtrados
            .read()
            .iter()
            .all(|a| estado.read().seleccionados.contains(&a.id));

    let texto_boton = if todos_seleccionados {
        "Deseleccionar todos"
    } else {
        "Seleccionar todos"
    };

    // Lógica de filtrado reactiva
    use_effect(move || {
        let app = estado.read();
        let (columna, valor, solo_rallita) = filtro.read().clone();

        let resultado = match columna {
            Columnas::Cinta => app.filtrar_cinta(valor, solo_rallita),
            Columnas::Edad => app.filtrar_edad(valor),
            _ => app.buscar_alumnos(columna, &valor),
        };

        alumnos_filtrados.set(resultado);
    });

    rsx! {
        div { class: "flex flex-col h-full space-y-4",
            div { class: "relative flex items-center justify-center py-2",
                h2 { class: "text-3xl font-bold text-gray-800 text-center", "Filtrar" }
                button {
                    class: "absolute right-0 px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors text-sm",
                    onclick: move |_| {
                        estado.write().toggle_all(alumnos_filtrados.read().clone());
                    },
                    "{texto_boton}"
                }
            }

            Filter {
                on_input: move |data| filtro.set(data),
                options: vec![
                    ("Cinta".to_string(), Columnas::Cinta),
                    ("Edad".to_string(), Columnas::Edad),
                ],
                placeholder: "Filtrar alumnos...".to_string(),
                initial_param: my_app::Columnas::Cinta,
            }

            DataTable { alumnos_lista: alumnos_filtrados, estado, aplicar_color_seleccion: true  }

            div { class: "flex  justify-between items-center ",
                div { class: "text-gray-500 text-xs ",
                    "Mostrando {alumnos_filtrados.read().len()} alumnos filtrados"
                }
                div { class: "text-gray-500 text-xs ",
                    "alumnos seleccionados: {estado.read().seleccionados.len()}"
                }
            }
        }
    }
}

#[component]
pub fn Agregar() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    // 1. Signals para manejar el estado del formulario
    let mut nombre = use_signal(|| "".to_string());
    let mut fecha_nac = use_signal(|| "".to_string());
    let mut rango = use_signal(|| 10i32); // Por defecto "Blanca" (valor 10)[cite: 2]
    let mut representante = use_signal(|| "".to_string());
    let mut contacto = use_signal(|| "".to_string());
    let mut rallita = use_signal(|| false);
    //let mut msg_error = use_signal(|| "".to_string());

    let contacto_valido = contacto_valido(&contacto.read());

    let fecha_valida = es_fecha_valida_form(&fecha_nac.read());

    let formulario_valido = (
        !nombre.read().is_empty(),
        fecha_valida,
        !representante.read().is_empty(),
        contacto_valido,
    );

    rsx! {

         div { class: "flex flex-col h-full space-y-4 max-w-2xl mx-auto",
                // Encabezado
                div { class: "text-center py-4",
                    h2 { class: "text-3xl font-bold text-gray-800", "Registrar Nuevo Alumno" },
                    p { class: "text-gray-500", "Ingresa los datos personales y de grado del karateka." }
                }
        Form {nombre: nombre, fecha_nac: fecha_nac, rango: rango, representante: representante, contacto: contacto, rallita: rallita, campos_validos: formulario_valido, on_click: move |_| {

            let datos = DatosAlumno {
                nombre: nombre.read().clone(),
                fecha_de_nacimiento: fecha_nac.read().clone(),
                numero_contacto: contacto.read().clone(),
                rango: *rango.read(),
                representante: representante.read().clone(),
                rallita: *rallita.read(),
            };

            let _ = estado.write().agregar_alumno(datos);
            nombre.set("".to_string());
            fecha_nac.set("".to_string());
            representante.set("".to_string());
            contacto.set("".to_string());
            rallita.set(false);
            rango.set(10);

        }, texto_boton: "Guardar"}


     }

    }
}

#[component]
pub fn Editar() -> Element {
    // 1. TODOS los hooks estrictamente en la raíz de la función
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    let mut nombre = use_signal(|| "".to_string());
    let mut fecha_nac = use_signal(|| "".to_string());
    let mut rango = use_signal(|| 10);
    let mut representante = use_signal(|| "".to_string());
    let mut contacto = use_signal(|| "".to_string());
    let mut rallita = use_signal(|| false);

    let mut lista_seleccionados = use_signal(|| vec![]);

    let seleccionados = estado.read().seleccionados.clone();
    let alum_seleccionados = seleccionados.len();

    // 2. Sincronización de datos mediante use_effect
    use_effect(move || {
        let mut lista = seleccionados.iter().copied();
        match alum_seleccionados {
            1 => {
                let id = lista.next().unwrap();
                let alumno = estado.read().get_alumno_by_id(id);
                nombre.set(alumno.nombre.clone());
                fecha_nac.set(alumno.fecha_de_nacimiento.clone());
                rango.set(alumno.rango.clone());
                representante.set(alumno.representante.clone());
                contacto.set(alumno.numero_contacto.clone());
                rallita.set(alumno.rallita);
            }
            2..=usize::MAX => {
                let mut write_lista = lista_seleccionados.write();
                write_lista.clear();
                for id in lista {
                    let alumno = estado.read().get_alumno_by_id(id);
                    write_lista.push(alumno);
                }
                rango.set(99);
            }
            _ => {}
        }
    });

    // 3. Renderizado principal
    rsx! {
        div { class: "flex flex-col h-screen max-h-screen min-h-0 space-y-4 w-full mx-auto overflow-hidden",
            // 💡 Cabecera fija unificada para todas las vistas
            div { class: "text-center py-4 flex-none",
                h2 { class: "text-3xl font-bold text-gray-800", "Editar Alumno" }
                p { class: "text-gray-500", "Modifica los datos personales y de grado del karateka." }
            }

            // El match ahora solo decide qué cuerpo de formulario inyectar abajo del título
            match alum_seleccionados {
                0 => {
                    rsx! {
                        div { class: "flex flex-col h-full space-y-4 max-w-2xl mx-auto justify-center items-center border-2 border-dashed border-gray-300 bg-gray-50/50 rounded-xl p-8",
                            svg {
                                class: "h-12 w-12 text-gray-400 mb-3",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke_width: "1.5",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
                                }
                            }
                            p { class: "text-gray-500 font-medium text-center text-balance max-w-sm",
                                "Seleccione al menos 1 estudiante para continuar con la edición."
                            }
                        }
                    }
                }
                1 => {
                    let id = *estado.read().seleccionados.iter().next().unwrap();
                    let contacto_valido = contacto_valido(&contacto.read());
                    let fecha_valida = es_fecha_valida_form(&fecha_nac.read());
                    let formulario_valido = (!nombre.read().is_empty(), fecha_valida, !representante.read().is_empty(), contacto_valido);

                    rsx! {
                        div { class: "flex flex-col h-full space-y-4 max-w-2xl mx-auto",
                            Form {
                                nombre: nombre, fecha_nac: fecha_nac, rango: rango,
                                representante: representante, contacto: contacto, rallita: rallita,
                                campos_validos: formulario_valido,
                                on_click: move |_| {
                                    let datos = DatosAlumno {
                                        nombre: nombre.read().clone(),
                                        fecha_de_nacimiento: fecha_nac.read().clone(),
                                        numero_contacto: contacto.read().clone(),
                                        rango: *rango.read(),
                                        representante: representante.read().clone(),
                                        rallita: *rallita.read(),
                                    };
                                    let _ = estado.write().actualizar_alumno(id, datos);
                                },
                                texto_boton: "Guardar"
                            }
                        }
                    }
                }
                2..=usize::MAX => {
                    rsx! {
                        // Quita "h-full" y agrega "flex-1 min-h-0" para que respete el título de arriba
                        div { class: "flex flex-col flex-1 min-h-0 space-y-6 w-full",

                            // El formulario se queda igual
                            div { class: "w-full max-w-xl mx-auto flex-none",
                                PromotionForm {
                                    rango: rango,
                                    rallita: rallita,
                                    texto_boton: "Aplicar cambios",
                                    on_click: move |_| {
                                        let _ = estado.write().promover_seleccionados(*rango.read(), *rallita.read());
                                    }
                                }
                            }

                            // La DataTable suelta, igual que en Consulta, pero envuelta en un contenedor elástico
                            //div { class: "w-full flex-1 min-h-0 overflow-auto",
                                DataTable { alumnos_lista: lista_seleccionados, estado: estado, aplicar_color_seleccion: false }

                                div { class: "flex justify-end items-center w-full",
                                    div { class: "text-gray-500 text-xs",
                                        "alumnos seleccionados: {estado.read().seleccionados.len()}"
                                    }
                                }

                        }
                    }
                }
                _ => { panic!("error"); }
            }
        }
    }
}

#[component]
pub fn Eliminar() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let mut lista_alumnos = use_signal(Vec::new);

    use_effect(move || {
        let mut temporal = vec![];

        for id in estado.read().seleccionados.iter() {
            temporal.push(estado.read().get_alumno_by_id(*id));
        }

        lista_alumnos.set(temporal);
    });

    rsx! {
        div { class: "flex flex-col h-full space-y-4 max-w-2xl mx-auto",
               // Encabezado
               div { class: "text-center py-4",
                   h2 { class: "text-3xl font-bold text-gray-800", "Eliminar Alumno" },
                   p { class: "text-gray-500", "Ingresa los datos personales y de grado del karateka." }
               }



               button {
                   class: "w-48 self-center py-3 bg-red-600 text-white rounded-xl font-bold hover:bg-red-700 active:scale-[0.98] transition-all cursor-pointer",
                   onclick: move |_| {
                       let _ = estado.write().eliminar_seleccionados();
                   },
                   "Eliminar"
               }

               DataTable { alumnos_lista: lista_alumnos, estado: estado, aplicar_color_seleccion: false }
               div { class: "flex justify-end items-center w-full",
                   div { class: "text-gray-500 text-xs",
                       "alumnos que serán afectados: {estado.read().seleccionados.len()}"
                   }
               }
        }
    }
}
