use crate::presentation::my_app;
use dioxus::prelude::*;

pub struct HeaderColumn {
    pub header: &'static str,
    pub class: Option<&'static str>,
}

impl PartialEq for HeaderColumn {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header && self.class == other.class
    }
}

impl Clone for HeaderColumn {
    fn clone(&self) -> Self {
        Self {
            header: self.header,
            class: self.class,
        }
    }
}

const ALTO_FILA_PX: f64 = 50.0;
const PASO_MINIMO_PX: f64 = ALTO_FILA_PX / 2.0;
const SOBRE_MUESTRA: usize = 20;
const FILAS_VENTANA: usize = 80;

#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(
    data: Signal<Vec<T>>,
    header_columns: &'static [HeaderColumn],
    row_key: fn(&T) -> usize,
    render_row: fn(&T, Signal<my_app::MyApp>) -> Element,
    estado: Signal<my_app::MyApp>,
    aplicar_color_seleccion: bool,
    checkbox: bool,
    on_doble_click: EventHandler<()>,
) -> Element {
    let items = data.read().clone();
    let mut scroll_y = use_signal(|| 0.0f64);
    let total = items.len();

    let inicio = (((*scroll_y.read() / ALTO_FILA_PX) as usize).saturating_sub(SOBRE_MUESTRA))
        .min(total);
    let fin = (inicio + FILAS_VENTANA + SOBRE_MUESTRA).min(total);
    let alto_relleno_superior = inicio as f64 * ALTO_FILA_PX;
    let alto_relleno_inferior = (total - fin) as f64 * ALTO_FILA_PX;

    let columnas = header_columns;
    let num_cols = if checkbox { columnas.len() + 1 } else { columnas.len() };

    let ventana: Vec<(usize, T)> = items[inicio..fin]
        .iter()
        .enumerate()
        .map(|(desplazamiento, item)| (inicio + desplazamiento, item.clone()))
        .collect();

    rsx! {
        div {
            class: "overflow-auto rounded-xl border border-gray-800 bg-gray-900 shadow-xl",
            style: "overflow-anchor:none",
            onscroll: move |e| {
                let arriba = e.data().scroll_top();
                if (arriba - *scroll_y.read()).abs() >= PASO_MINIMO_PX {
                    scroll_y.set(arriba);
                }
            },
            table { class: "w-full border-collapse text-left text-xs md:text-sm table-auto",
                thead {
                    tr { class: "sticky top-0 text-white bg-gray-800",
                        if checkbox {
                            th { class: "z-20 px-4 py-3", "Sel." }
                        }
                        for col in columnas {
                            th {
                                class: {
                                    let base = "z-20 px-4 py-3";
                                    match col.class {
                                        Some(c) => format!("{} {}", base, c),
                                        None => base.to_string(),
                                    }
                                },
                                "{col.header}"
                            }
                        }
                    }
                }
                tbody { class: "divide-y divide-gray-800 text-gray-300",
                    if alto_relleno_superior > 0.0 {
                        tr {
                            td {
                                colspan: "{num_cols}",
                                style: "height:{alto_relleno_superior}px;padding:0;border:none",
                            }
                        }
                    }

                    for (i, item) in ventana {
                        {
                            let key = row_key(&item);
                            let es_seleccionado = estado.read().seleccionados.contains(&key);
                            let base = if aplicar_color_seleccion && es_seleccionado {
                                "bg-blue-500 hover:bg-blue-700"
                            } else if i % 2 == 0 {
                                "bg-gray-900 hover:bg-gray-700"
                            } else {
                                "bg-gray-800 hover:bg-gray-700"
                            };
                            rsx! {
                                tr {
                                    key: "{key}",
                                    style: "height:{ALTO_FILA_PX}px",
                                    class: { base },
                                    onclick: move |_| {
                                        estado.write().toggle_seleccion(key);
                                    },
                                    ondoubleclick: move |_| {
                                        if !estado.read().seleccionados.contains(&key) {
                                            estado.write().toggle_seleccion(key);
                                        }
                                        on_doble_click.call(());
                                    },
                                    if checkbox {
                                        td { class: "px-4 py-3",
                                            input {
                                                r#type: "checkbox",
                                                class: "w-4 h-4 rounded border-gray-700 bg-gray-800 text-blue-600 focus:ring-blue-500",
                                                checked: estado.read().seleccionados.contains(&key),
                                            }
                                        }
                                    }
                                    { render_row(&item, estado.clone()) }
                                }
                            }
                        }
                    }

                    if alto_relleno_inferior > 0.0 {
                        tr {
                            td {
                                colspan: "{num_cols}",
                                style: "height:{alto_relleno_inferior}px;padding:0;border:none",
                            }
                        }
                    }
                }
            }
        }
    }
}