//! Catálogos de estados, métodos y tipos del sistema.
//! Cada enum mapea directamente al ID de su tabla `cat_*` correspondiente.
//! Son de solo lectura en la UI — se siembran al iniciar la BD.

// ─── Estados de deuda ───

/// Estado persistido de una deuda (NO derivado como antes).
/// Mapea a `cat_estados_deuda.id`.
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum EstadoDeuda {
    Pendiente = 1,
    Parcial = 2,
    Pagada = 3,
    Anticipada = 4,
    Anulada = 5,
}

impl EstadoDeuda {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Pendiente),
            2 => Some(Self::Parcial),
            3 => Some(Self::Pagada),
            4 => Some(Self::Anticipada),
            5 => Some(Self::Anulada),
            _ => None,
        }
    }

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Pendiente => "Pendiente",
            Self::Parcial => "Parcial",
            Self::Pagada => "Pagada",
            Self::Anticipada => "Anticipada",
            Self::Anulada => "Anulada",
        }
    }

    pub fn es_terminal(&self) -> bool {
        matches!(self, Self::Pagada | Self::Anulada)
    }

    pub fn es_cobrable(&self) -> bool {
        matches!(self, Self::Pendiente | Self::Parcial)
    }
}

// ─── Estados de pago ───

/// Estado de un registro de pago.
/// Mapea a `cat_estados_pago.id`.
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum EstadoPago {
    Completado = 1,
    Reversado = 2,
    PendienteConfirmar = 3,
}

impl EstadoPago {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Completado),
            2 => Some(Self::Reversado),
            3 => Some(Self::PendienteConfirmar),
            _ => None,
        }
    }

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Completado => "Completado",
            Self::Reversado => "Reversado",
            Self::PendienteConfirmar => "Pendiente",
        }
    }
}

// ─── Métodos de pago ───

/// Método por el que se realizó el pago.
/// Mapea a `cat_metodos_pago.id`.
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum MetodoPago {
    Efectivo = 1,
    Transferencia = 2,
    Tarjeta = 3,
    Cheque = 4,
}

impl MetodoPago {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Efectivo),
            2 => Some(Self::Transferencia),
            3 => Some(Self::Tarjeta),
            4 => Some(Self::Cheque),
            _ => None,
        }
    }

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Efectivo => "Efectivo",
            Self::Transferencia => "Transferencia",
            Self::Tarjeta => "Tarjeta",
            Self::Cheque => "Cheque",
        }
    }

    pub fn todos() -> &'static [MetodoPago] {
        &[Self::Efectivo, Self::Transferencia, Self::Tarjeta, Self::Cheque]
    }
}

// ─── Estados de alumno ───

/// Estado de actividad de un alumno.
/// Mapea a `cat_estados_alumno.id`.
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum EstadoAlumno {
    Activo = 1,
    Inactivo = 2,
    Suspendido = 3,
    Retirado = 4,
}

impl EstadoAlumno {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Activo),
            2 => Some(Self::Inactivo),
            3 => Some(Self::Suspendido),
            4 => Some(Self::Retirado),
            _ => None,
        }
    }

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Activo => "Activo",
            Self::Inactivo => "Inactivo",
            Self::Suspendido => "Suspendido",
            Self::Retirado => "Retirado",
        }
    }

    /// Genera deudas solo para alumnos activos.
    pub fn genera_deudas(&self) -> bool {
        matches!(self, Self::Activo)
    }
}

// ─── Estados de representante ───

/// Estado de actividad de un representante.
/// Mapea a `cat_estados_representante.id`.
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum EstadoRepresentante {
    Activo = 1,
    Inactivo = 2,
}

impl EstadoRepresentante {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Activo),
            2 => Some(Self::Inactivo),
            _ => None,
        }
    }

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Activo => "Activo",
            Self::Inactivo => "Inactivo",
        }
    }
}

// ─── Tipos de historial ───

/// Tipo de movimiento registrado en el historial.
/// Mapea a `cat_tipos_historial.id`.
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum TipoHistorial {
    DeudaCreada = 1,
    PagoRegistrado = 2,
    AbonoAplicado = 3,
    AjusteManual = 4,
    Anulacion = 5,
}

impl TipoHistorial {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::DeudaCreada),
            2 => Some(Self::PagoRegistrado),
            3 => Some(Self::AbonoAplicado),
            4 => Some(Self::AjusteManual),
            5 => Some(Self::Anulacion),
            _ => None,
        }
    }

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::DeudaCreada => "Deuda Creada",
            Self::PagoRegistrado => "Pago Registrado",
            Self::AbonoAplicado => "Abono Aplicado",
            Self::AjusteManual => "Ajuste Manual",
            Self::Anulacion => "Anulación",
        }
    }
}

// ─── Tipos de evento ───

/// Tipo de evento deportivo.
/// Mapea a `cat_tipos_evento.id`.
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum TipoEvento {
    Competencia = 1,
    Examen = 2,
    Graduacion = 3,
    ClaseEspecial = 4,
}

impl TipoEvento {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Competencia),
            2 => Some(Self::Examen),
            3 => Some(Self::Graduacion),
            4 => Some(Self::ClaseEspecial),
            _ => None,
        }
    }

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Competencia => "Competencia",
            Self::Examen => "Examen",
            Self::Graduacion => "Graduación",
            Self::ClaseEspecial => "Clase Especial",
        }
    }

    pub fn todos() -> &'static [TipoEvento] {
        &[Self::Competencia, Self::Examen, Self::Graduacion, Self::ClaseEspecial]
    }
}
