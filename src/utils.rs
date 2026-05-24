use chrono::NaiveDate;

pub fn es_fecha_valida(fecha: &str) -> bool {
    // Intenta parsear la fecha con el formato que definiste (año-mes-día)
    NaiveDate::parse_from_str(fecha, "%Y-%m-%d").is_ok()
}
