//! se encarga de dibujar las vistas segun las rutas seleccionadas
//! contiene las funciones con el codigo especifico de cada vista

//use std::string;

//use std::intrinsics::fabs;
use std::{usize, vec};

use crate::application::dto::{DatosAlumno, DatosPago, DatosRepresentante};
use crate::application::validation::*;
use crate::presentation::components::datatable::DataTable;
use crate::presentation::components::filter::Filter;
use crate::presentation::components::form::Form;
use crate::presentation::components::promotion_form::PromotionForm;
use crate::presentation::components::searchbar::SearchBar;
use crate::presentation::my_app::{self, Columnas};
use chrono::Local;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();
    let todos_seleccionados = !estado.read().alumnos.is_empty()
        && estado
            .read()
            .alumnos
            .iter()
            .all(|v| estado.read().seleccionados.contains(&v.alumno.id));
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
            .all(|v| estado.read().seleccionados.contains(&v.alumno.id));
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
            .all(|v| estado.read().seleccionados.contains(&v.alumno.id));

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
    let mut representante_id = use_signal(|| 0usize);
    let mut rallita = use_signal(|| false);
    //let mut msg_error = use_signal(|| "".to_string());

    let representantes = estado.read().representantes.clone();

    let fecha_valida = es_fecha_valida_form(&fecha_nac.read());

    let formulario_valido = (
        !nombre.read().is_empty(),
        fecha_valida,
        representante_asignado(*representante_id.read()),
    );

    rsx! {

         div { class: "flex flex-col h-full space-y-4 max-w-2xl mx-auto",
                // Encabezado
                div { class: "text-center py-4",
                    h2 { class: "text-3xl font-bold text-gray-800", "Registrar Nuevo Alumno" },
                    p { class: "text-gray-500", "Ingresa los datos personales y de grado del karateka." }
                }
        Form {nombre: nombre, fecha_nac: fecha_nac, rango: rango, representantes: representantes, representante_id: representante_id, rallita: rallita, campos_validos: formulario_valido, on_click: move |_| {

            let datos = DatosAlumno {
                nombre: nombre.read().clone(),
                fecha_de_nacimiento: fecha_nac.read().clone(),
                rango: *rango.read(),
                representante_id: *representante_id.read(),
                rallita: *rallita.read(),
            };

            let _ = estado.write().agregar_alumno(datos);
            nombre.set("".to_string());
            fecha_nac.set("".to_string());
            representante_id.set(0);
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
    let mut representante_id = use_signal(|| 0usize);
    let mut rallita = use_signal(|| false);

    let mut lista_seleccionados = use_signal(|| vec![]);

    let seleccionados = estado.read().seleccionados.clone();
    let alum_seleccionados = seleccionados.len();
    let representantes = estado.read().representantes.clone();

    // 2. Sincronización de datos mediante use_effect
    use_effect(move || {
        let mut lista = seleccionados.iter().copied();
        match alum_seleccionados {
            1 => {
                let id = lista.next().unwrap();
                let alumno = estado.read().get_alumno_by_id(id);
                nombre.set(alumno.nombre.clone());
                fecha_nac.set(alumno.fecha_de_nacimiento.clone());
                rango.set(alumno.rango);
                representante_id.set(alumno.representante_id);
                rallita.set(alumno.rallita);
            }
            2..=usize::MAX => {
                let mut write_lista = lista_seleccionados.write();
                write_lista.clear();
                for id in lista {
                    let vista = estado
                        .read()
                        .alumnos
                        .iter()
                        .find(|v| v.alumno.id == id)
                        .cloned();
                    if let Some(vista) = vista {
                        write_lista.push(vista);
                    }
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
                    let fecha_valida = es_fecha_valida_form(&fecha_nac.read());
                    let formulario_valido = (!nombre.read().is_empty(), fecha_valida, representante_asignado(*representante_id.read()));

                    rsx! {
                        div { class: "flex flex-col h-full space-y-4 max-w-2xl mx-auto",
                            Form {
                                nombre: nombre, fecha_nac: fecha_nac, rango: rango,
                                representantes: representantes, representante_id: representante_id,
                                rallita: rallita,
                                campos_validos: formulario_valido,
                                on_click: move |_| {
                                    let datos = DatosAlumno {
                                        nombre: nombre.read().clone(),
                                        fecha_de_nacimiento: fecha_nac.read().clone(),
                                        rango: *rango.read(),
                                        representante_id: *representante_id.read(),
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
        // Proyectamos directamente desde la caché de vistas del ViewModel.
        let temporal: Vec<_> = estado
            .read()
            .alumnos
            .iter()
            .filter(|v| estado.read().seleccionados.contains(&v.alumno.id))
            .cloned()
            .collect();

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

/// Panel de administrador: resumen del mes, registro/anulación de pagos de
/// mensualidad y catálogo de representantes. Solo pinta datos ya cargados por
/// los casos de uso en el ViewModel; toda validación vive en `application`.
#[component]
pub fn Administrador() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    // Estado del formulario de pago
    let mut pago_rep_id = use_signal(|| 0usize);
    let mut monto_texto = use_signal(|| "".to_string());
    let mut observacion = use_signal(|| "".to_string());
    let mut mensaje = use_signal(|| (String::new(), false)); // (texto, es_error)

    // Estado del formulario de representante
    let mut rep_nombre = use_signal(|| "".to_string());
    let mut rep_contacto = use_signal(|| "".to_string());

    // Lecturas derivadas de la caché del ViewModel
    let etiqueta_mes = estado.read().etiqueta_periodo_actual();
    let total = estado.read().total_del_mes();
    let cantidad_pagos = estado.read().pagos.len();
    let morosos = estado.read().morosos.clone();
    let representantes = estado.read().representantes.clone();
    let pagos = estado.read().pagos.clone();

    let rep_elegido = *pago_rep_id.read() > 0;
    let monto_ok = monto_texto
        .read()
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map(monto_valido)
        .unwrap_or(false);
    let pago_listo = rep_elegido && monto_ok;
    let rep_formulario_ok =
        nombre_valido(&rep_nombre.read()) && contacto_valido(&rep_contacto.read());

    rsx! {
        div { class: "flex flex-col h-full space-y-6 overflow-auto pr-1",

            // Encabezado
            div { class: "text-center py-2",
                h2 { class: "text-3xl font-bold text-gray-800", "Panel de Administrador" }
                p { class: "text-gray-500", "Mensualidades y representantes — {etiqueta_mes}" }
            }

            // Resumen del mes
            div { class: "grid grid-cols-3 gap-4",
                div { class: "bg-gray-800 rounded-xl p-4 text-center shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "Recaudado" }
                    p { class: "text-2xl font-bold text-emerald-400", {format!("{total:.2}")} }
                }
                div { class: "bg-gray-800 rounded-xl p-4 text-center shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "Pagos del mes" }
                    p { class: "text-2xl font-bold text-blue-400", "{cantidad_pagos}" }
                }
                div { class: "bg-gray-800 rounded-xl p-4 text-center shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "Morosos" }
                    p { class: if morosos.is_empty() { "text-2xl font-bold text-emerald-400" } else { "text-2xl font-bold text-red-400" },
                        "{morosos.len()}"
                    }
                    if !morosos.is_empty() {
                        p { class: "text-[10px] text-gray-500 mt-1 truncate",
                            {morosos.iter().map(|r| r.nombre.as_str()).collect::<Vec<_>>().join(", ")}
                        }
                    }
                }
            }

            // Registrar pago
            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-3",
                h3 { class: "font-bold text-white", "Registrar mensualidad" }
                div { class: "grid grid-cols-[1fr_140px_1fr_auto] gap-3 items-end",
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Representante" }
                        select {
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer",
                            style: "-webkit-appearance:none; appearance:none; background-color:#111827;",
                            value: "{pago_rep_id}",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<usize>() { pago_rep_id.set(id); }
                            },
                            option {
                                class: "bg-gray-900", value: "0",
                                selected: !rep_elegido, disabled: true,
                                "-- Seleccione --"
                            }
                            {representantes.iter().map(|r| rsx! {
                                option {
                                    class: "bg-gray-900", value: "{r.id}",
                                    selected: *pago_rep_id.read() == r.id,
                                    "{r.nombre}"
                                }
                            })}
                        }
                    }
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Monto" }
                        input {
                            r#type: "text",
                            class: if monto_ok || monto_texto.read().is_empty() { "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none" } else { "p-2 rounded-lg bg-gray-900 text-gray-100 border border-red-500 outline-none" },
                            placeholder: "1500.00",
                            value: "{monto_texto}",
                            oninput: move |e| monto_texto.set(e.value())
                        }
                    }
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Observación (opcional)" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none",
                            placeholder: "Ej: incluye hermano",
                            value: "{observacion}",
                            oninput: move |e| observacion.set(e.value())
                        }
                    }
                    button {
                        class: if pago_listo { "px-5 py-2 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg transition-colors cursor-pointer" } else { "px-5 py-2 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed" },
                        disabled: !pago_listo,
                        onclick: move |_| {
                            if !pago_listo { return; }
                            let datos = DatosPago {
                                representante_id: *pago_rep_id.read(),
                                monto: monto_texto.read().trim().replace(',', ".").parse::<f64>().unwrap_or(0.0),
                                periodo: estado.read().periodo_actual.clone(),
                                fecha: Local::now().format(crate::domain::alumno::FORMATO_FECHA).to_string(),
                                observacion: observacion.read().clone(),
                            };
                            match estado.write().registrar_pago(datos) {
                                Ok(()) => {
                                    mensaje.set(("Mensualidad registrada".to_string(), false));
                                    monto_texto.set("".to_string());
                                    observacion.set("".to_string());
                                }
                                Err(error) => mensaje.set((error.to_string(), true)),
                            }
                        },
                        "Registrar"
                    }
                }
                if !mensaje.read().0.is_empty() {
                    p { class: if mensaje.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" }, "{mensaje.read().0}" }
                }
            }

            // Tabla de pagos del mes
            div { class: "overflow-auto rounded-xl border border-gray-800 bg-gray-900 shadow-xl max-h-72",
                table { class: "w-full border-collapse text-left text-xs md:text-sm",
                    thead {
                        tr { class: "sticky top-0 text-white bg-gray-800 z-10",
                            th { class: "px-4 py-3", "ID" }
                            th { class: "px-4 py-3", "Representante" }
                            th { class: "px-4 py-3", "Monto" }
                            th { class: "px-4 py-3", "Registrado" }
                            th { class: "px-4 py-3", "Observación" }
                            th { class: "px-4 py-3", "" }
                        }
                    }
                    tbody { class: "divide-y divide-gray-800 text-gray-300",
                        if pagos.is_empty() {
                            tr {
                                td { colspan: 6, class: "px-4 py-6 text-center text-gray-500 italic",
                                    "Sin pagos registrados este mes"
                                }
                            }
                        }
                        for vista in pagos {
                            tr {
                                key: "{vista.pago.id}",
                                class: "hover:bg-gray-700/50",
                                td { class: "px-4 py-2.5 font-mono text-gray-500", "#{vista.pago.id}" }
                                td { class: "px-4 py-2.5 font-medium text-white whitespace-nowrap", "{vista.nombre_representante}" }
                                td { class: "px-4 py-2.5 text-emerald-400 font-mono", {format!("{:.2}", vista.pago.monto)} }
                                td { class: "px-4 py-2.5 whitespace-nowrap", "{vista.pago.fecha}" }
                                td { class: "px-4 py-2.5 text-gray-400", "{vista.pago.observacion}" }
                                td { class: "px-4 py-2.5 text-right",
                                    button {
                                        class: "text-red-400 hover:text-red-300 font-bold text-xs cursor-pointer",
                                        title: "Anular pago",
                                        onclick: move |_| {
                                            let _ = estado.write().anular_pago(vista.pago.id);
                                        },
                                        "✕ Anular"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Catálogo de representantes
            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-3",
                h3 { class: "font-bold text-white", "Representantes ({representantes.len()})" }
                div { class: "grid grid-cols-[1fr_200px_auto] gap-3 items-end",
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Nombre" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none",
                            placeholder: "Ej: Pedro Pérez",
                            value: "{rep_nombre}",
                            oninput: move |e| rep_nombre.set(e.value())
                        }
                    }
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Teléfono" }
                        input {
                            r#type: "tel",
                            maxlength: "12",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none",
                            placeholder: "0412-0000000",
                            value: "{rep_contacto}",
                            oninput: move |e| {
                                let mut val = e.value();
                                val.retain(|c| c.is_ascii_digit());
                                if val.len() > 4 {
                                    val.insert(4, '-');
                                }
                                val.truncate(12);
                                rep_contacto.set(val);
                            }
                        }
                    }
                    button {
                        class: if rep_formulario_ok { "px-5 py-2 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg transition-colors cursor-pointer" } else { "px-5 py-2 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed" },
                        disabled: !rep_formulario_ok,
                        onclick: move |_| {
                            if !rep_formulario_ok { return; }
                            let datos = DatosRepresentante {
                                nombre: rep_nombre.read().clone(),
                                numero_contacto: rep_contacto.read().clone(),
                            };
                            match estado.write().agregar_representante(datos) {
                                Ok(()) => {
                                    rep_nombre.set("".to_string());
                                    rep_contacto.set("".to_string());
                                }
                                Err(_) => {}
                            }
                        },
                        "+ Agregar"
                    }
                }
                ul { class: "divide-y divide-gray-700/60 mt-2",
                    {representantes.iter().map(|r| rsx! {
                        li {
                            key: "{r.id}",
                            class: "flex justify-between items-center px-1 py-2 text-sm",
                            span { class: "text-gray-200", "{r.nombre}" }
                            span { class: "text-blue-400 font-mono text-xs", "{r.numero_contacto}" }
                        }
                    })}
                }
            }
        }
    }
}
