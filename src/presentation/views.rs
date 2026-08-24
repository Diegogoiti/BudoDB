//! se encarga de dibujar las vistas segun las rutas seleccionadas
//! contiene las funciones con el codigo especifico de cada vista

//use std::string;

//use std::intrinsics::fabs;
use std::{usize, vec};

use crate::application::dto::{DatosAbono, DatosAlumno, DatosRepresentante};
use crate::application::validation::*;
use crate::domain::EstadoDeuda;
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

    let mut lista_seleccionados = use_signal(Vec::new);

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

/// Texto compacto para montos: sin decimales cuando son enteros
/// ("1500"), con ellos cuando no ("1500.5"). Evita "1500.00" ruidoso.
fn fmt_monto(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if (r - r.trunc()).abs() < f64::EPSILON {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// (etiqueta, clases del badge) según el estado de la deuda.
fn badge_estado(estado: &EstadoDeuda) -> (&'static str, &'static str) {
    match estado {
        EstadoDeuda::Pagado => ("Pagado", "bg-emerald-900 text-emerald-300"),
        EstadoDeuda::Parcial => ("Parcial", "bg-amber-900 text-amber-300"),
        EstadoDeuda::Pendiente => ("Pendiente", "bg-red-900 text-red-300"),
    }
}

/// Color de la barra de progreso según el estado.
fn clase_barra(estado: &EstadoDeuda) -> &'static str {
    match estado {
        EstadoDeuda::Pagado => "bg-emerald-500",
        EstadoDeuda::Parcial => "bg-amber-500",
        EstadoDeuda::Pendiente => "bg-red-500",
    }
}

/// Panel de pagos: el ciclo completo de deudas y abonos del mes en UNA
/// pantalla — estadísticas arriba, acciones al centro, tabla con progreso
/// por representante, gestión de representantes abajo y modal de abono.
/// Solo pinta datos ya cargados por los casos de uso en el ViewModel.
#[component]
pub fn Pagos() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    // Modal de registro de abono
    let mut modal_abono = use_signal(|| false);
    let mut abono_deuda_id = use_signal(|| 0usize);
    let mut abono_monto = use_signal(String::new);
    let mut abono_observacion = use_signal(String::new);
    let mut abono_msg = use_signal(|| (String::new(), false));

    // Formulario compacto de representantes
    let mut rep_nombre = use_signal(String::new);
    let mut rep_contacto = use_signal(String::new);

    // Mensaje de la acción "crear deudas"
    let mut msg_deudas = use_signal(|| (String::new(), false));

    // Una sola pasada de lectura sobre el contexto
    let (etiqueta_mes, total_deudas, total_abonado, pagados, parciales, pendientes) = {
        let e = estado.read();
        (
            e.etiqueta_periodo_actual(),
            e.total_deudas_periodo(),
            e.total_abonado_periodo(),
            e.reps_pagados(),
            e.reps_parciales(),
            e.reps_pendientes(),
        )
    };
    let por_cobrar = (total_deudas - total_abonado).max(0.0);

    let deudas = estado.read().deudas.clone();
    let representantes = estado.read().representantes.clone();
    let hay_pendientes = deudas.iter().any(|v| v.estado != EstadoDeuda::Pagado);

    let rep_formulario_ok =
        nombre_valido(&rep_nombre.read()) && contacto_valido(&rep_contacto.read());

    // Deuda actualmente elegida en el modal (para el helper de saldo)
    let deuda_elegida = estado
        .read()
        .deudas
        .iter()
        .find(|v| v.deuda.id == *abono_deuda_id.read())
        .cloned();

    // Avance global del mes (para la tarjeta de progreso), precalculado
    // para mantener el rsx libre de expresiones con llaves anidadas.
    let pct_global = if total_deudas > 0.0 {
        (total_abonado / total_deudas * 100.0).min(100.0)
    } else {
        0.0
    };
    let texto_avance = if total_deudas > 0.0 {
        format!("{pct_global:.0}% recaudado")
    } else {
        "Sin deudas aún".to_string()
    };
    let clase_avance = if total_deudas > 0.0 && pendientes == 0 {
        clase_barra(&EstadoDeuda::Pagado)
    } else if total_deudas > 0.0 && pagados == 0 {
        clase_barra(&EstadoDeuda::Pendiente)
    } else {
        clase_barra(&EstadoDeuda::Parcial)
    };

    // Abre el modal apuntando a una deuda concreta, con el saldo prellenado.
    let mut abrir_modal = move |id: usize, saldo: f64| {
        abono_deuda_id.set(id);
        abono_monto.set(fmt_monto(saldo));
        abono_observacion.set(String::new());
        abono_msg.set((String::new(), false));
        modal_abono.set(true);
    };

    rsx! {
        div { class: "flex flex-col h-full space-y-5 overflow-auto pr-1",

            // ── Encabezado: título + chip del periodo ──
            div { class: "flex items-end justify-between py-1",
                div {
                    h2 { class: "text-3xl font-bold text-gray-800", "💳 Panel de Pagos" }
                    p { class: "text-gray-500 text-sm mt-1",
                        "Deudas y abonos de mensualidad — vista del mes completo."
                    }
                }
                span { class: "px-3 py-1 rounded-full bg-gray-200 text-gray-700 text-xs font-bold tracking-widest uppercase whitespace-nowrap",
                    "{etiqueta_mes}"
                }
            }

            // ── Estadísticas del mes (4 tarjetas oscuras) ──
            div { class: "grid grid-cols-4 gap-4",
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "💰 Deudas del mes" }
                    p { class: "text-2xl font-bold text-gray-100 mt-1", {fmt_monto(total_deudas)} }
                    p { class: "text-[11px] text-gray-500 mt-1", "{deudas.len()} representantes" }
                }
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "✅ Recaudado" }
                    p { class: "text-2xl font-bold text-emerald-400 mt-1", {fmt_monto(total_abonado)} }
                    p { class: "text-[11px] text-gray-500 mt-1", "{pagados} pagaron completo" }
                }
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "⏳ Por cobrar" }
                    p { class: "text-2xl font-bold text-amber-400 mt-1", {fmt_monto(por_cobrar)} }
                    p { class: "text-[11px] text-gray-500 mt-1", "{parciales} abonaron parcial · {pendientes} sin abonos" }
                }
                div { class: "bg-gray-800 rounded-xl p-4 shadow-lg border border-gray-700",
                    p { class: "text-xs uppercase tracking-widest text-gray-400", "📊 Avance" }
                    div { class: "mt-3 h-2 w-full bg-gray-700 rounded-full overflow-hidden",
                        div {
                            class: "h-full rounded-full {clase_avance}",
                            style: "width:{pct_global:.0}%",
                        }
                    }
                    p { class: "text-[11px] text-gray-500 mt-1", "{texto_avance}" }
                }
            }

            // ── Barra de acciones + leyenda ──
            div { class: "flex items-center justify-between gap-3 flex-wrap",
                div { class: "flex items-center gap-3",
                    button {
                        class: "px-5 py-2.5 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                        onclick: move |_| {
                            match estado.write().crear_deudas_del_mes() {
                                Ok(creadas) => {
                                    if creadas == 0 {
                                        msg_deudas.set(("Todos los representantes ya tienen su deuda este mes.".to_string(), false));
                                    } else {
                                        msg_deudas.set((format!("Se crearon {creadas} deudas."), false));
                                    }
                                }
                                Err(error) => msg_deudas.set((error.to_string(), true)),
                            }
                        },
                        "＋ Crear deudas del mes"
                    }
                    button {
                        class: if hay_pendientes {
                            "px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                        } else {
                            "px-5 py-2.5 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed text-sm"
                        },
                        disabled: !hay_pendientes,
                        title: if hay_pendientes { "Registrar un abono" } else { "Todas las deudas están saldadas" },
                        onclick: move |_| {
                            // Preselecciona la primera deuda con saldo pendiente.
                            if let Some(v) = estado.read().deudas.iter().find(|v| v.estado != EstadoDeuda::Pagado) {
                                abrir_modal(v.deuda.id, v.saldo);
                            }
                        },
                        "💵 Registrar abono"
                    }
                    if !msg_deudas.read().0.is_empty() {
                        span {
                            class: if msg_deudas.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" },
                            "{msg_deudas.read().0}"
                        }
                    }
                }
                // Leyenda de colores (ayuda rápida)
                div { class: "flex items-center gap-3 text-[11px] text-gray-500",
                    span { class: "flex items-center gap-1",
                        i { class: "inline-block w-2 h-2 rounded-full bg-emerald-500" }
                        "Pagado"
                    }
                    span { class: "flex items-center gap-1",
                        i { class: "inline-block w-2 h-2 rounded-full bg-amber-500" }
                        "Abono parcial"
                    }
                    span { class: "flex items-center gap-1",
                        i { class: "inline-block w-2 h-2 rounded-full bg-red-500" }
                        "Pendiente"
                    }
                }
            }

            // ── Tabla de deudas del mes ──
            div { class: "overflow-auto rounded-xl border border-gray-800 bg-gray-900 shadow-xl max-h-80",
                table { class: "w-full border-collapse text-left text-xs md:text-sm",
                    thead {
                        tr { class: "sticky top-0 text-white bg-gray-800 z-10",
                            th { class: "px-4 py-3", "Representante" }
                            th { class: "px-4 py-3 text-right", "Mensualidad" }
                            th { class: "px-4 py-3 text-right", "Abonado" }
                            th { class: "px-4 py-3 w-40", "Progreso" }
                            th { class: "px-4 py-3 text-right", "Saldo" }
                            th { class: "px-4 py-3 text-center", "Estado" }
                            th { class: "px-4 py-3", "" }
                        }
                    }
                    tbody { class: "divide-y divide-gray-800 text-gray-300",
                        if deudas.is_empty() {
                            tr {
                                td { colspan: 7, class: "px-4 py-10 text-center",
                                    p { class: "text-3xl mb-2", "🗓️" }
                                    p { class: "text-gray-400 font-medium", "Este mes aún no tiene deudas" }
                                    p { class: "text-gray-500 text-xs mt-1",
                                        "Pulsa \"＋ Crear deudas del mes\" para generarlas automáticamente."
                                    }
                                }
                            }
                        }
                        {(deudas.iter()).map(|vista| {
                            let id_deuda = vista.deuda.id;
                            let saldo_texto = fmt_monto(vista.saldo);
                            let pct = if vista.deuda.monto > 0.0 {
                                (vista.total_abonado / vista.deuda.monto * 100.0).min(100.0)
                            } else {
                                0.0
                            };
                            let (etiqueta, clases) = badge_estado(&vista.estado);
                            let abrible = vista.estado != EstadoDeuda::Pagado;
                            let saldo_fila = vista.saldo;
                            rsx! {
                            tr {
                                key: "{id_deuda}",
                                class: "hover:bg-gray-700/50",
                                onclick: move |_| {
                                    if abrible {
                                        abrir_modal(id_deuda, saldo_fila);
                                    }
                                },
                                td { class: "px-4 py-2.5",
                                    p { class: "font-medium text-white truncate max-w-48", "{vista.nombre_representante}" }
                                    p { class: "text-[11px] text-gray-500 font-mono", "{vista.telefono_representante}" }
                                }
                                td { class: "px-4 py-2.5 text-right font-mono text-gray-300",
                                    "{fmt_monto(vista.deuda.monto)}"
                                }
                                td { class: "px-4 py-2.5 text-right font-mono text-emerald-400",
                                    "{fmt_monto(vista.total_abonado)}"
                                }
                                td { class: "px-4 py-2.5",
                                    div { class: "h-2 w-full bg-gray-700 rounded-full overflow-hidden",
                                        div {
                                            class: "h-full rounded-full {clase_barra(&vista.estado)}",
                                            style: "width:{pct:.0}%",
                                        }
                                    }
                                    p { class: "text-[10px] text-gray-500 mt-1", "{pct:.0}%" }
                                }
                                td { class: "px-4 py-2.5 text-right font-mono font-bold text-amber-400",
                                    "{saldo_texto}"
                                }
                                td { class: "px-4 py-2.5 text-center",
                                    span { class: "inline-block px-2 py-0.5 rounded-full text-[10px] font-bold {clases}",
                                        "{etiqueta}"
                                    }
                                }
                                td { class: "px-4 py-2.5 text-right",
                                    if abrible {
                                        button {
                                            class: "px-2 py-1 rounded bg-gray-800 border border-gray-700 text-emerald-400 hover:text-white hover:border-emerald-500 font-bold text-xs transition-colors cursor-pointer",
                                            onclick: move |_| abrir_modal(id_deuda, saldo_fila),
                                            "+ Abono"
                                        }
                                    }
                                }
                            }
                            }
                        })}
                    }
                }
            }

            // ── Gestión de representantes (compacta, misma pantalla) ──
            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-3",
                div { class: "flex items-center justify-between",
                    h3 { class: "font-bold text-white", "👥 Representantes ({representantes.len()})" }
                }
                // Alta rápida en línea
                div { class: "flex items-end gap-2",
                    div { class: "flex flex-col space-y-1 flex-1",
                        label { class: "text-xs font-semibold text-gray-400", "Nombre" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                            placeholder: "Ej: Pedro Pérez",
                            value: "{rep_nombre}",
                            oninput: move |e| rep_nombre.set(e.value())
                        }
                    }
                    div { class: "flex flex-col space-y-1 w-44",
                        label { class: "text-xs font-semibold text-gray-400", "Teléfono" }
                        input {
                            r#type: "tel",
                            maxlength: "12",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
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
                        class: if rep_formulario_ok {
                            "px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm"
                        } else {
                            "px-4 py-2 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed text-sm"
                        },
                        disabled: !rep_formulario_ok,
                        onclick: move |_| {
                            if !rep_formulario_ok { return; }
                            let datos = DatosRepresentante {
                                nombre: rep_nombre.read().clone(),
                                numero_contacto: rep_contacto.read().clone(),
                            };
                            if estado.write().agregar_representante(datos).is_ok() {
                                rep_nombre.set(String::new());
                                rep_contacto.set(String::new());
                            }
                        },
                        "＋ Agregar"
                    }
                }
                // Listado en grilla de 2 columnas con avatar-inicial
                div { class: "grid grid-cols-2 gap-2 mt-2",
                    {representantes.iter().map(|r| rsx! {
                        div {
                            key: "{r.id}",
                            class: "flex items-center gap-3 px-3 py-2 bg-gray-900 rounded-lg border border-gray-800",
                            span { class: "w-8 h-8 rounded-full bg-blue-600 text-white flex items-center justify-center text-xs font-bold flex-none",
                                {r.nombre.chars().next().unwrap_or('?').to_string()}
                            }
                            div { class: "min-w-0",
                                p { class: "text-sm text-gray-100 truncate", "{r.nombre}" }
                                p { class: "text-[11px] text-gray-500 font-mono", "{r.numero_contacto}" }
                            }
                        }
                    })}
                }
            }
        }

        // ═══ Modal: registrar abono ═══
        if *modal_abono.read() {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
                onclick: move |_| modal_abono.set(false),
                div {
                    class: "bg-gray-800 rounded-xl shadow-2xl border border-gray-700 p-6 w-full max-w-sm space-y-4 mx-4",
                    onclick: move |e| e.stop_propagation(),

                    // Título + cerrar
                    div { class: "flex items-center justify-between",
                        h3 { class: "text-lg font-bold text-white", "💵 Registrar abono" }
                        button {
                            class: "text-gray-400 hover:text-white text-xl leading-none cursor-pointer",
                            onclick: move |_| modal_abono.set(false),
                            "✕"
                        }
                    }

                    // Selector de deuda pendiente
                    div { class: "flex flex-col space-y-1",
                        div { class: "flex items-center justify-between",
                            label { class: "text-xs font-semibold text-gray-400", "Deuda" }
                            if let Some(v) = &deuda_elegida {
                                span { class: "text-xs text-gray-500",
                                    "Saldo: "
                                    span { class: "text-amber-400 font-bold", "{fmt_monto(v.saldo)}" }
                                }
                            }
                        }
                        select {
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/50",
                            value: "{abono_deuda_id}",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<usize>() {
                                    abono_deuda_id.set(id);
                                    // Autocompleta el monto con el saldo restante.
                                    if let Some(v) = estado.read().deudas.iter().find(|d| d.deuda.id == id) {
                                        abono_monto.set(fmt_monto(v.saldo));
                                    }
                                    abono_msg.set((String::new(), false));
                                }
                            },
                            option { class: "bg-gray-900", value: "0", "-- Seleccione --" }
                            {estado.read().deudas.iter().filter(|v| v.estado != EstadoDeuda::Pagado).map(|v| rsx! {
                                option {
                                    key: "{v.deuda.id}",
                                    class: "bg-gray-900", value: "{v.deuda.id}",
                                    selected: *abono_deuda_id.read() == v.deuda.id,
                                    "{v.nombre_representante} — saldo {fmt_monto(v.saldo)}"
                                }
                            })}
                        }
                    }

                    // Monto
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Monto" }
                        input {
                            r#type: "text",
                            class: if abono_monto.read().trim().is_empty() {
                                "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50"
                            } else if abono_monto.read().trim().replace(',', ".").parse::<f64>().map(|m| m > 0.0).unwrap_or(false) {
                                "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50"
                            } else {
                                "p-2 rounded-lg bg-gray-900 text-gray-100 border border-red-500 outline-none ring-2 ring-red-500/30"
                            },
                            placeholder: "Ej: 500",
                            value: "{abono_monto}",
                            oninput: move |e| {
                                abono_monto.set(e.value());
                                abono_msg.set((String::new(), false));
                            }
                        }
                    }

                    // Observación
                    div { class: "flex flex-col space-y-1",
                        label { class: "text-xs font-semibold text-gray-400", "Observación (opcional)" }
                        input {
                            r#type: "text",
                            class: "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50",
                            placeholder: "Ej: efectivo, pago parcial…",
                            value: "{abono_observacion}",
                            oninput: move |e| abono_observacion.set(e.value())
                        }
                    }

                    if !abono_msg.read().0.is_empty() {
                        p {
                            class: if abono_msg.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" },
                            "{abono_msg.read().0}"
                        }
                    }

                    // Acciones del modal
                    div { class: "flex gap-2 justify-end pt-1",
                        button {
                            class: "px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors cursor-pointer",
                            onclick: move |_| modal_abono.set(false),
                            "Cancelar"
                        }
                        button {
                            class: "px-5 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm",
                            onclick: move |_| {
                                let deuda_id = *abono_deuda_id.read();
                                if deuda_id == 0 {
                                    abono_msg.set(("Seleccione una deuda.".to_string(), true));
                                    return;
                                }
                                let monto = abono_monto.read().trim().replace(',', ".").parse::<f64>().unwrap_or(0.0);
                                if monto <= 0.0 {
                                    abono_msg.set(("El monto debe ser mayor a cero.".to_string(), true));
                                    return;
                                }
                                if let Some(v) = estado.read().deudas.iter().find(|d| d.deuda.id == deuda_id) {
                                    if monto > v.saldo + 0.009 {
                                        abono_msg.set((format!("El monto excede el saldo ({saldo}).", saldo = fmt_monto(v.saldo)), true));
                                        return;
                                    }
                                }
                                let datos = DatosAbono {
                                    deuda_id,
                                    monto,
                                    fecha: Local::now().format(crate::domain::alumno::FORMATO_FECHA).to_string(),
                                    observacion: abono_observacion.read().clone(),
                                };
                                match estado.write().registrar_abono(datos) {
                                    Ok(()) => {
                                        modal_abono.set(false);
                                        abono_monto.set(String::new());
                                        abono_observacion.set(String::new());
                                        abono_msg.set((String::new(), false));
                                    }
                                    Err(error) => abono_msg.set((error.to_string(), true)),
                                }
                            },
                            "Guardar abono"
                        }
                    }
                }
            }
        }
    }
}

/// Panel de ajustes: configuraciones de la aplicación persistidas vía su
/// puerto propio, más información de solo lectura sobre el sistema.
#[component]
pub fn Ajustes() -> Element {
    let mut estado = use_context::<Signal<my_app::MyApp>>();

    let mut monto_texto = use_signal(|| {
        let monto = estado.read().monto_predeterminado;
        if monto > 0.0 { fmt_monto(monto) } else { String::new() }
    });
    let mut mensaje = use_signal(|| (String::new(), false)); // (texto, es_error)

    let ruta_bd = estado.read().ruta_bd.clone();
    let periodo = estado.read().etiqueta_periodo_actual();
    let version = env!("CARGO_PKG_VERSION");

    // Feedback en vivo con la MISMA regla que aplica el caso de uso.
    let monto_ok = monto_texto
        .read()
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map(monto_valido)
        .unwrap_or(false);

    rsx! {
        div { class: "flex flex-col h-full space-y-6 overflow-auto pr-1 max-w-3xl mx-auto",

            // Encabezado
            div { class: "flex items-end justify-between py-1",
                div {
                    h2 { class: "text-3xl font-bold text-gray-800", "⚙️ Panel de Ajustes" }
                    p { class: "text-gray-500 text-sm mt-1", "Configuración de la aplicación." }
                }
                span { class: "px-3 py-1 rounded-full bg-gray-200 text-gray-700 text-xs font-bold tracking-widest uppercase whitespace-nowrap",
                    "{periodo}"
                }
            }

            // Configuración de mensualidad
            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-3",
                h3 { class: "font-bold text-white", "💵 Mensualidad" }
                p { class: "text-xs text-gray-400",
                    "Monto predeterminado: prellena la generación de deudas del panel de Pagos cada mes."
                }
                div { class: "flex items-end gap-2",
                    div { class: "flex flex-col space-y-1 w-56",
                        label { class: "text-xs font-semibold text-gray-400", "Monto predeterminado" }
                        input {
                            r#type: "text",
                            class: if monto_ok || monto_texto.read().is_empty() { "p-2 rounded-lg bg-gray-900 text-gray-100 border border-gray-700 outline-none focus:ring-2 focus:ring-blue-500/50" } else { "p-2 rounded-lg bg-gray-900 text-gray-100 border border-red-500 outline-none ring-2 ring-red-500/30" },
                            placeholder: "Ej: 1500",
                            value: "{monto_texto}",
                            oninput: move |e| monto_texto.set(e.value())
                        }
                    }
                    button {
                        class: if monto_ok { "px-5 py-2 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors active:scale-[0.98] cursor-pointer text-sm" } else { "px-5 py-2 bg-gray-700 text-gray-400 font-bold rounded-lg cursor-not-allowed text-sm" },
                        disabled: !monto_ok,
                        onclick: move |_| {
                            if !monto_ok { return; }
                            match estado.write().cambiar_monto_predeterminado(monto_texto.read().clone()) {
                                Ok(()) => mensaje.set(("Ajuste guardado".to_string(), false)),
                                Err(error) => mensaje.set((error.to_string(), true)),
                            }
                        },
                        "Guardar"
                    }
                }
                if !mensaje.read().0.is_empty() {
                    p { class: if mensaje.read().1 { "text-xs text-red-400" } else { "text-xs text-emerald-400" }, "{mensaje.read().0}" }
                }
            }

            // Información del sistema (solo lectura)
            div { class: "bg-gray-800 rounded-xl shadow-lg border border-gray-700 p-5 space-y-2",
                h3 { class: "font-bold text-white", "ℹ️ Información del sistema" }
                div { class: "flex justify-between items-center px-1 py-2 text-sm border-b border-gray-700",
                    span { class: "text-gray-400", "Versión" }
                    span { class: "text-gray-200 font-mono", "BudoDB v{version}" }
                }
                div { class: "flex justify-between items-center px-1 py-2 text-sm border-b border-gray-700/60",
                    span { class: "text-gray-400", "Periodo administrado" }
                    span { class: "text-gray-200", "{periodo}" }
                }
                div { class: "flex flex-col space-y-1 px-1 py-2 text-sm",
                    span { class: "text-gray-400", "Base de datos" }
                    span { class: "text-blue-400 font-mono text-xs break-all", "{ruta_bd}" }
                    p { class: "text-[10px] text-gray-500 mt-1",
                        "Se configura con la variable de entorno BUDODB_DB_PATH antes de abrir la aplicación."
                    }
                }
            }
        }
    }
}
