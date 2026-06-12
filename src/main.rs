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
    #[route("/buscar")] // Aunque no tengan contenido aún, deben existir
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

    // 1. Configuramos la ventana (SIN el menú aquí)
    let window = WindowBuilder::new()
        .with_title("BudoDB")
        .with_background_color((17, 24, 39, 255))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0))
        .with_inner_size(initial_size)
        // QUITAMOS el .with_visible(true) y .with_focused(true) de aquí
        // Windows a veces se lía si se los das tan temprano
        .with_maximized(false)
        .with_visible(false);

    // 2. Creamos el config y AQUÍ le quitamos el menú
    let config = Config::default().with_window(window).with_menu(None); // <--- El .with_menu() va pegado al Config, no al WindowBuilder

    // 3. Lanzamos
    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    use_effect(move || {
        // Muestra la ventana solo cuando el componente esté montado
        dioxus::desktop::window().set_visible(true);
    });

    let estado_app = use_signal(|| my_app::MyApp::new());

    // 3. Proveemos el signal ya creado al contexto
    use_context_provider(|| estado_app);

    rsx! {

        style { "{CSS}" }
        Router::<Route> {}
    }
}
