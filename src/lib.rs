//! BudoDB: gestión de alumnos de artes marciales.
//!
//! Arquitectura en capas:
//! - [`domain`]: entidades y reglas de negocio puras.
//! - [`application`]: casos de uso y puertos hacia el exterior.
//! - [`infrastructure`]: adaptadores concretos (SQLite, logging, configuración).
//! - [`presentation`]: interfaz Dioxus (vistas y componentes).
//! - [`composition_root`]: construcción del grafo de objetos y arranque.

pub mod application;
pub mod composition_root;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
