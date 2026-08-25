use crate::application::dto::AlumnoVista;
use crate::presentation::my_app;
use chrono::Local;
use dioxus::prelude::*;

/// Virtualización: altura fija de cada fila en píxeles. Cada `tr` fuerza esta
/// altura con estilo inline, así el cálculo de posiciones es exacto.
///
/// Debe ser MAYOR o igual que la altura natural del contenido de la fila
/// (~49px con los badges de Cinta/Rango); si fuera menor, las filas crecen
/// solas y el cálculo de posiciones pierde precisión.
const ALTO_FILA_PX: f64 = 50.0;

/// Solo reaccionamos al scroll cuando cruzó medio renglón: evita re-renderizar
/// por cada píxel y rompe bucles de auto-scroll entre eventos y re-renders.
const PASO_MINIMO_PX: f64 = ALTO_FILA_PX / 2.0;

/// Filas renderizadas siempre presentes alrededor del viewport.
const SOBRE_MUESTRA: usize = 20;

/// Ventana mínima de filas renderizadas (cualquier alto de panel queda cubierto).
const FILAS_VENTANA: usize = 80;

///componente que recibe un contexto con una clase myapp y clona el vector de vistas
/// para dibujar la tabla de los datos en la ventana
///
/// La tabla está VIRTUALIZADA: sin importar cuántos alumnos existan, solo se
/// renderizan las filas cercanas a la posición de scroll (+sobremuestra). Dos
/// filas-relleno mantienen la altura total del scroll y la cabecera sticky.
///
/// Consume PROYECCIONES (`AlumnoVista`): el nombre y teléfono del representante
/// ya vienen resueltos desde la capa de aplicación.
#[component]
pub fn DataTable(
    alumnos_lista: Signal<Vec<AlumnoVista>>,
    estado: Signal<my_app::MyApp>,
    aplicar_color_seleccion: bool,
    on_doble_click: EventHandler<()>,
) -> Element {
    let alumnos = alumnos_lista.read().clone();
    let hoy = Local::now().date_naive();
    let mut scroll_y = use_signal(|| 0.0f64);

    let total = alumnos.len();

    // Primera fila a renderizar: según el scroll, con margen hacia arriba.
    let inicio = (((*scroll_y.read() / ALTO_FILA_PX) as usize).saturating_sub(SOBRE_MUESTRA))
        .min(total);
    // Última fila (exclusiva): ventana + sobremuestra hacia abajo.
    let fin = (inicio + FILAS_VENTANA + SOBRE_MUESTRA).min(total);

    // Rellenos que simulan las filas no renderizadas para conservar
    // la barra de desplazamiento y la inercia del scroll.
    let alto_relleno_superior = inicio as f64 * ALTO_FILA_PX;
    let alto_relleno_inferior = (total - fin) as f64 * ALTO_FILA_PX;

    // Ventana visible: pares (índice real en la lista completa, vista).
    // El índice real mantiene el rayado cebra estable aunque solo se
    // renderice una porción de la lista. Se clona porque el rsx captura
    // los valores en closures `move`.
    let ventana: Vec<(usize, AlumnoVista)> = alumnos[inicio..fin]
        .iter()
        .enumerate()
        .map(|(desplazamiento, vista)| (inicio + desplazamiento, vista.clone()))
        .collect();

    rsx! {

        div {
            class: "overflow-auto rounded-xl border border-gray-800 bg-gray-900 shadow-xl ",
            // Desactiva el scroll anchoring del navegador: cuando los rellenos
            // cambian de tamaño, Chromium no debe "compensar" moviendo el
            // scroll por su cuenta (era la causa del scroll desbocado).
            style: "overflow-anchor:none",
            onscroll: move |e| {
                let arriba = e.data().scroll_top();
                if (arriba - *scroll_y.read()).abs() >= PASO_MINIMO_PX {
                    scroll_y.set(arriba);
                }
            },
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
                    if alto_relleno_superior > 0.0 {
                        tr {
                            td {
                                colspan: 9,
                                style: "height:{alto_relleno_superior}px;padding:0;border:none",
                            }
                        }
                    }

                    for (i, vista) in ventana {
                        // key por ID: al deslizar la ventana, dioxus mueve los
                        // nodos existentes en vez de reciclarlos para otra
                        // fila (eso cambiaba el fondo de cada tr en pantalla).
                        tr {
                            key: "{vista.alumno.id}",
                            style: "height:{ALTO_FILA_PX}px",
                            // Sin transition-colors: al reciclarse nodos durante
                            // el scroll, la transición animaba el cambio de gris
                            // y hacía "parpadear" el rayado cebra. Rayado con
                            // clases que EXISTEN en el CSS compilado.
                            class: {
                                let es_seleccionado = estado.read().seleccionados.contains(&vista.alumno.id);
                                let base = if aplicar_color_seleccion && es_seleccionado {
                                    "bg-blue-500 hover:bg-blue-700"
                                } else {
                                    if i % 2 == 0 { "bg-gray-900 hover:bg-gray-700" } else { "bg-gray-800 hover:bg-gray-700" }
                                };
                                base
                                },
                            onclick: move |_| {
                                estado.write().toggle_seleccion(vista.alumno.id);
                            },
                            ondoubleclick: move |_| {
                            if !estado.read().seleccionados.contains(&vista.alumno.id) {
                                estado.write().toggle_seleccion(vista.alumno.id);
                            }
                            on_doble_click.call(());
                            },
                            td { class: "px-4 py-3",
                                input {
                                    r#type: "checkbox",
                                    class: "w-4 h-4 rounded border-gray-700 bg-gray-800 text-blue-600 focus:ring-blue-500",
                                    checked: estado.read().seleccionados.contains(&vista.alumno.id),

                                }
                            }
                            td { class: "px-4 py-3 font-mono text-gray-500", "#{vista.alumno.id}" }
                            td { class: "px-4 py-3 font-bold text-white whitespace-nowrap", "{vista.alumno.nombre}" }
                            td { class: "px-4 py-3",
                                span { class: "inline-flex items-center justify-center min-w-36 px-3 py-1.5 rounded bg-gray-700 text-[10px] uppercase font-bold text-gray-300 whitespace-nowrap",
                                    "{vista.alumno.cinta()}"
                                }
                            }
                                                        td { class: "px-4 py-3",
                                span { class: "inline-flex items-center justify-center min-w-20 px-2 py-1.5 rounded bg-gray-700 text-[10px] uppercase font-bold text-gray-300 whitespace-nowrap",
                                    "{vista.alumno.rango()}"
                                }
                            }
                            td { class: "px-4 py-3 whitespace-nowrap", "{vista.alumno.edad(hoy)}" }
                            td { class: "px-4 py-3", "{vista.alumno.fecha_de_nacimiento}" }
                            td { class: "px-4 py-3 whitespace-nowrap", "{vista.nombre_representante}" }
                            td { class: "px-4 py-3 text-blue-400 font-mono whitespace-nowrap",
                                "{vista.telefono_representante}"
                            }
                        }
                    }

                    if alto_relleno_inferior > 0.0 {
                        tr {
                            td {
                                colspan: 9,
                                style: "height:{alto_relleno_inferior}px;padding:0;border:none",
                            }
                        }
                    }
                }
            }
        }
    }
}
