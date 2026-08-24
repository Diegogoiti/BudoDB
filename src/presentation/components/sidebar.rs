use crate::presentation::app::Route;
use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        div { class: "flex h-screen w-screen bg-gray-100 overflow-hidden",
            nav { class: "w-64 bg-gray-900 text-white flex flex-col shadow-xl flex-none",
                div { class: "p-6 border-b border-gray-800 flex justify-center",
                    h1 { class: "text-2xl font-bold text-white", "BudoDB" }
                }
                div { class: "flex-1 px-4 py-6 space-y-2",
                    SidebarItem { to: Route::Home {}, icon: "📋", label: "Consulta" }
                    SidebarItem { to: Route::Buscar {}, icon: "🔍", label: "Buscar" }
                    SidebarItem {
                        to: Route::Filtrar {},
                        icon: "📊",
                        label: "Filtrar",
                    }
                    hr { class: "my-6 border-gray-800" }
                    SidebarItem {
                        to: Route::Agregar {},
                        icon: "➕",
                        label: "Agregar",
                    }
                    SidebarItem {
                        to: Route::Editar {},
                        icon: "✏️",
                        label: "Editar",
                    }
                    SidebarItem {
                        to: Route::Eliminar {},
                        icon: "🗑️",
                        label: "Eliminar",
                    }
                }
                div { class: "p-4 bg-gray-950 text-center border-t border-gray-800",
                    p { class: "text-[10px] text-gray-500 tracking-widest",
                        "BudoDB V0.0.1-BETA"
                    }
                    p { class: "text-[10px] text-gray-500 tracking-widest",
                        "© 2026 Diego Andrés Goitia Márquez. All rights reserved."
                    }
                }



            }
            // El "hueco" donde se verán las vistas
            main { class: "flex-1 h-full flex flex-col bg-gray-100 overflow-hidden",
                div { class: "p-8 w-full h-full flex flex-col", Outlet::<Route> {} }
            }
        }
    }
}

#[component]
fn SidebarItem(to: Route, icon: &'static str, label: &'static str) -> Element {
    rsx! {
        Link {
            to,
            class: "flex items-center space-x-3 p-2 rounded hover:bg-gray-800",
            span { "{icon}" }
            span { "{label}" }
        }
    }
}
