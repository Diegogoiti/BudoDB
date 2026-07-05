#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;

mod components;
mod my_app;
mod utils;
mod views;

use crate::components::sidebar::Sidebar;
//use crate::models::Alumno;
use crate::views::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

const CSS: &str = include_str!("../assets/tailwind.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Sidebar)]
    #[route("/")]
    Home {},
    #[route("/buscar")]
    Buscar {},
    #[route("/filtrar")]
    Filtrar {},
    #[route("/agregar")]
    Agregar {},
    #[route("/editar")]
    Editar {},
    #[route("/eliminar")]
    Eliminar {},
}

fn main() {
    let initial_size = dioxus::desktop::LogicalSize::new(1024.0, 720.0);

    let mut window = WindowBuilder::new()
        .with_title("BudoDB")
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0))
        .with_inner_size(initial_size);

    // Configuración condicional
    #[cfg(target_os = "windows")]
    {
        window = window.with_visible(false);
    }

    #[cfg(not(target_os = "windows"))]
    {
        window = window.with_transparent(true);
    }

    let config = Config::default()
        .with_window(window)
        .with_menu(None)
        .with_custom_head(format!("<style>{}</style>", CSS));

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    let estado_app = use_signal(|| my_app::MyApp::new());
    use_context_provider(|| estado_app);

    use_effect(|| {
        let window = dioxus::desktop::window();
        window.set_visible(true);
    });
    rsx! {

        Router::<Route> {}

    }
}
