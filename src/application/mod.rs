//! Capa de aplicacion: casos de uso y puertos hacia el exterior.
//! NO referencia infraestructura concreta (regla 1).

pub mod dto;
pub mod error;
pub mod ports;
pub mod service;
pub mod service_abonos;
pub mod service_ajustes;
pub mod service_deudas;
pub mod service_historial;
pub mod service_pagos;
pub mod service_representantes;
pub mod validation;
