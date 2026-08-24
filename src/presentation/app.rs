use crate::composition_root;
use crate::presentation::components::sidebar::Sidebar;
use crate::presentation::views::*;
use dioxus::prelude::*;

pub const CSS: &str = include_str!("../../assets/tailwind.css");

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
    #[route("/administrador")]
    Administrador {},
}

/// Componente raíz. El estado inicial se construye a través del
/// composition root (único punto de construcción de objetos).
#[component]
pub fn App() -> Element {
    let estado_app = use_signal(composition_root::construir_estado_aplicacion);
    use_context_provider(|| estado_app);

    use_effect(|| {
        let window = dioxus::desktop::window();
        window.set_visible(true);
    });
    rsx! {

        Router::<Route> {}

    }
}
