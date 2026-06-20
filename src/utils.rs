use chrono::NaiveDate;

pub fn es_fecha_valida(fecha: &str) -> bool {
    // Intenta parsear la fecha con el formato que definiste (año-mes-día)
    NaiveDate::parse_from_str(fecha, "%Y-%m-%d").is_ok()
}

pub fn contacto_valido(numero: String) -> bool {
    if numero.is_empty() || numero.len() < 12 {
        false
    } else {
        true
    }
}

pub fn es_fecha_valida2form(fecha: String) -> bool {
    if fecha.is_empty() {
        false
    } else {
        if !es_fecha_valida(fecha.as_str()) {
            false
        } else {
            true
        }
    }
}
