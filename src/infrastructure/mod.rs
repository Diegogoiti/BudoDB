//! Capa de infraestructura: adaptadores que IMPLEMENTAN los puertos definidos
//! en `application` (persistencia, logging, configuración).

pub mod console_logger;
pub mod env_config;
pub mod sqlite_repository;
