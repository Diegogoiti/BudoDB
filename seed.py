import sqlite3
import os
import random
import sys
from datetime import date, timedelta

# ── Configuración ──────────────────────────────────────────────
NUM_REPS = int(sys.argv[1]) if len(sys.argv) > 1 else 8
PERIODOS = ["2026-06", "2026-07", "2026-08", "2026-09"]
MESUALIDAD = 1500.0

DB_PATH = os.path.join(os.path.dirname(__file__), "database", "database.db")

# ── Nombres y datos aleatorios ─────────────────────────────────
NOMBRES_M = [
    "Juan", "Diego", "Mateo", "Santiago", "Sebastian", "Adrian", "Nicolas",
    "Pablo", "Andres", "Fernando", "Carlos", "Pedro", "Jose", "Luis",
    "Miguel", "Roberto", "Daniel", "Antonio", "Manuel", "Javier",
    "Alejandro", "Ricardo", "Eduardo", "Sergio", "Raul", "Oscar",
    "Tomas", "Lucas", "Mateo", "Emiliano", "Santiago", "Leonardo",
]
NOMBRES_F = [
    "Maria", "Sofia", "Valentina", "Isabella", "Camila", "Lucia",
    "Mariana", "Daniela", "Elena", "Gabriela", "Alejandra", "Ana",
    "Laura", "Carmen", "Rosa", "Teresa", "Patricia", "Claudia",
    "Adriana", "Beatriz", "Carolina", "Diana", "Fernanda", "Gloria",
    "Irene", "Julia", "Karla", "Lorena", "Monica", "Natalia",
]
APELLIDOS = [
    "Garcia", "Rodriguez", "Martinez", "Lopez", "Fernandez", "Hernandez",
    "Diaz", "Sanchez", "Torres", "Reyes", "Morales", "Gutierrez",
    "Castillo", "Vargas", "Mendoza", "Romero", "Ramos", "Silva",
    "Cruz", "Ortiz", "Gomez", "Flores", "Ruiz", "Herrera",
    "Medina", "Aguilar", "Vega", "Castro", "Jimenez", "Moreno",
    "Rojas", "Dominguez", "Munoz", "Alvarez", "Romero", "Navarro",
]

RANGOS_KYU = [1, 2, 3, 4, 5, 6, 7, 10]
RANGOS_DAN = [0, -1, -2, -3, -4, -5]


def nombre_aleatorio():
    nombre = random.choice(NOMBRES_M if random.random() < 0.5 else NOMBRES_F)
    apellido = random.choice(APELLIDOS)
    return f"{nombre} {apellido}"


def telefono_aleatorio():
    prefijos = ["0412", "0414", "0416", "0424"]
    return f"{random.choice(prefijos)}-{random.randint(1000000, 9999999)}"


def fecha_nacimiento_aleatoria():
    hoy = date(2026, 8, 1)
    min_edad, max_edad = 5, 18
    inicio = hoy - timedelta(days=max_edad * 365)
    fin = hoy - timedelta(days=min_edad * 365)
    delta = (fin - inicio).days
    return inicio + timedelta(days=random.randint(0, delta))


def fecha_en_periodo(periodo):
    anio, mes = map(int, periodo.split("-"))
    dia = random.randint(1, 28)
    return f"{anio}-{mes:02d}-{dia:02d}"


# ── Crear base de datos ────────────────────────────────────────
os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
if os.path.exists(DB_PATH):
    os.remove(DB_PATH)

conn = sqlite3.connect(DB_PATH)
conn.execute("PRAGMA foreign_keys = ON")

conn.executescript("""
-- Catálogos
CREATE TABLE IF NOT EXISTS cat_estados_deuda (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS cat_estados_pago (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS cat_metodos_pago (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS cat_estados_alumno (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS cat_estados_representante (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS cat_tipos_historial (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS cat_tipos_evento (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL);

INSERT OR IGNORE INTO cat_estados_deuda VALUES (1,'Pendiente'),(2,'Parcial'),(3,'Pagada'),(4,'Anticipada'),(5,'Anulada');
INSERT OR IGNORE INTO cat_estados_pago VALUES (1,'Completado'),(2,'Reversado'),(3,'Pendiente');
INSERT OR IGNORE INTO cat_metodos_pago VALUES (1,'Efectivo'),(2,'Transferencia'),(3,'Tarjeta'),(4,'Cheque');
INSERT OR IGNORE INTO cat_estados_alumno VALUES (1,'Activo'),(2,'Inactivo'),(3,'Suspendido'),(4,'Retirado');
INSERT OR IGNORE INTO cat_estados_representante VALUES (1,'Activo'),(2,'Inactivo');
INSERT OR IGNORE INTO cat_tipos_historial VALUES (1,'Deuda Creada'),(2,'Pago Registrado'),(3,'Abono Aplicado'),(4,'Ajuste Manual'),(5,'Anulación');
INSERT OR IGNORE INTO cat_tipos_evento VALUES (1,'Competencia'),(2,'Examen'),(3,'Graduación'),(4,'Clase Especial');

-- Tablas de negocio
CREATE TABLE representantes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    numero_contacto TEXT NOT NULL,
    estado_id INTEGER NOT NULL DEFAULT 1,
    eliminado BOOLEAN NOT NULL DEFAULT 0
);
CREATE TABLE alumnos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    fecha_de_nacimiento TEXT NOT NULL,
    rango INTEGER NOT NULL,
    representante_id INTEGER NOT NULL DEFAULT 0,
    rallita BOOLEAN NOT NULL DEFAULT 0,
    estado_id INTEGER NOT NULL DEFAULT 1,
    eliminado BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (representante_id) REFERENCES representantes(id)
);
CREATE TABLE deudas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    representante_id INTEGER NOT NULL,
    monto_total REAL NOT NULL,
    monto_pendiente REAL NOT NULL,
    periodo TEXT NOT NULL,
    fecha_vencimiento TEXT NOT NULL,
    estado_id INTEGER NOT NULL DEFAULT 1,
    alumno_id INTEGER,
    eliminado BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (representante_id) REFERENCES representantes(id)
);
CREATE TABLE pagos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    representante_id INTEGER NOT NULL,
    monto_recibido REAL NOT NULL,
    estado_id INTEGER NOT NULL DEFAULT 1,
    metodo_id INTEGER NOT NULL DEFAULT 1,
    fecha_pago TEXT NOT NULL,
    eliminado BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (representante_id) REFERENCES representantes(id)
);
CREATE TABLE aplicaciones_pago (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pago_id INTEGER NOT NULL,
    deuda_id INTEGER NOT NULL,
    monto_aplicado REAL NOT NULL,
    fecha TEXT NOT NULL,
    FOREIGN KEY (pago_id) REFERENCES pagos(id),
    FOREIGN KEY (deuda_id) REFERENCES deudas(id)
);
CREATE TABLE historial_pagos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    representante_id INTEGER NOT NULL,
    tipo_id INTEGER NOT NULL,
    monto REAL NOT NULL DEFAULT 0,
    periodo TEXT NOT NULL,
    fecha TEXT NOT NULL,
    observacion TEXT NOT NULL DEFAULT '',
    eliminado BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (representante_id) REFERENCES representantes(id)
);
CREATE TABLE ajustes (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_alumnos_rep_estado ON alumnos(representante_id, estado_id);
CREATE INDEX IF NOT EXISTS idx_deudas_rep_estado_fecha ON deudas(representante_id, estado_id, fecha_vencimiento);
CREATE INDEX IF NOT EXISTS idx_deudas_periodo ON deudas(periodo);
CREATE INDEX IF NOT EXISTS idx_pagos_rep_fecha ON pagos(representante_id, fecha_pago);
CREATE INDEX IF NOT EXISTS idx_aplicaciones_pago_id ON aplicaciones_pago(pago_id);
CREATE INDEX IF NOT EXISTS idx_aplicaciones_deuda_id ON aplicaciones_pago(deuda_id);
CREATE INDEX IF NOT EXISTS idx_historial_rep_fecha ON historial_pagos(representante_id, fecha);
""")

# ── Seed representantes ────────────────────────────────────────
reps_creados = []
for i in range(NUM_REPS):
    nombre = nombre_aleatorio()
    telefono = telefono_aleatorio()
    cur = conn.execute(
        "INSERT INTO representantes (nombre, numero_contacto) VALUES (?, ?)",
        (nombre, telefono),
    )
    reps_creados.append({"id": cur.lastrowid, "nombre": nombre})

# ── Seed alumnos: 1-3 por representante ────────────────────────
total_alumnos = 0
for rep in reps_creados:
    num_alumnos = random.randint(1, 3)
    for _ in range(num_alumnos):
        nombre = nombre_aleatorio()
        fecha = fecha_nacimiento_aleatoria().isoformat()
        if random.random() < 0.85:
            rango = random.choice(RANGOS_KYU)
            rallita = random.random() < 0.25 if rango > 0 else False
        else:
            rango = random.choice(RANGOS_DAN)
            rallita = False
        conn.execute(
            "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante_id, rallita) VALUES (?, ?, ?, ?, ?)",
            (nombre, fecha, rango, rep["id"], rallita),
        )
        total_alumnos += 1

# ── Seed deudas + pagos + aplicaciones + historial ──────────────
#
# Estrategia:
#   - Periodos pasados (2026-06..2026-08): deudas YA Pagadas, con pagos y
#     aplicaciones completas. FIFO las salta porque estado_id=3.
#   - Periodo actual (2026-09): mix de Pagada / Parcial / Pendiente para
#     tener datos de prueba realistas en la ConsultaTab.
#

total_deudas = 0
total_pagos = 0
total_aplicaciones = 0
total_historial = 0
periodo_actual = PERIODOS[-1]  # "2026-09"

for rep in reps_creados:
    rep_id = rep["id"]

    for periodo in PERIODOS:
        monto_total = MESUALIDAD
        fecha_venc = fecha_en_periodo(periodo)
        es_actual = periodo == periodo_actual

        if es_actual:
            # Periodo actual: mix realista
            roll = random.random()
            if roll < 0.30:
                estado_id = 3   # Pagada
                monto_pendiente = 0.0
            elif roll < 0.55:
                estado_id = 2   # Parcial
                monto_pendiente = round(monto_total * random.uniform(0.2, 0.7), 2)
            else:
                estado_id = 1   # Pendiente
                monto_pendiente = monto_total
        else:
            # Periodos pasados: SIEMPRE Pagada
            estado_id = 3
            monto_pendiente = 0.0

        cur = conn.execute(
            "INSERT INTO deudas (representante_id, monto_total, monto_pendiente, periodo, fecha_vencimiento, estado_id) VALUES (?, ?, ?, ?, ?, ?)",
            (rep_id, monto_total, monto_pendiente, periodo, fecha_venc, estado_id),
        )
        deuda_id = cur.lastrowid
        total_deudas += 1

        # Historial: DeudaCreada (tipo 1)
        conn.execute(
            "INSERT INTO historial_pagos (representante_id, tipo_id, monto, periodo, fecha, observacion) VALUES (?, 1, ?, ?, ?, ?)",
            (rep_id, monto_total, periodo, fecha_venc, f"Deuda creada para {periodo}"),
        )
        total_historial += 1

        monto_abonado = monto_total - monto_pendiente

        if monto_abonado > 0.0:
            fecha_pago = fecha_en_periodo(periodo)
            metodo = random.choice([1, 2, 3, 4])

            # Pago
            cur_pago = conn.execute(
                "INSERT INTO pagos (representante_id, monto_recibido, estado_id, metodo_id, fecha_pago) VALUES (?, ?, 1, ?, ?)",
                (rep_id, monto_abonado, metodo, fecha_pago),
            )
            pago_id = cur_pago.lastrowid
            total_pagos += 1

            # Aplicación del pago a la deuda
            conn.execute(
                "INSERT INTO aplicaciones_pago (pago_id, deuda_id, monto_aplicado, fecha) VALUES (?, ?, ?, ?)",
                (pago_id, deuda_id, monto_abonado, fecha_pago),
            )
            total_aplicaciones += 1

            # Historial: PagoRegistrado (tipo 2)
            conn.execute(
                "INSERT INTO historial_pagos (representante_id, tipo_id, monto, periodo, fecha, observacion) VALUES (?, 2, ?, ?, ?, ?)",
                (rep_id, monto_abonado, periodo, fecha_pago, f"Pago recibido {periodo}"),
            )
            total_historial += 1

            # Historial: AbonoAplicado (tipo 3)
            conn.execute(
                "INSERT INTO historial_pagos (representante_id, tipo_id, monto, periodo, fecha, observacion) VALUES (?, 3, ?, ?, ?, ?)",
                (rep_id, monto_abonado, periodo, fecha_pago, f"Abono a deuda {deuda_id}"),
            )
            total_historial += 1

# ── Ajustes ────────────────────────────────────────────────────
conn.execute("INSERT OR REPLACE INTO ajustes (clave, valor) VALUES ('monto_mensualidad', ?)", (str(int(MESUALIDAD)),))

conn.commit()
conn.close()

print(f"Database seeded at: {DB_PATH}")
print(f"  - {len(reps_creados)} representantes")
print(f"  - {total_alumnos} alumnos ({NUM_REPS} reps x 1-3 cada uno)")
print(f"  - {total_deudas} deudas en {len(PERIODOS)} periodos")
print(f"    - Periodos pasados: todas Pagadas (FIFO las salta)")
print(f"    - Periodo actual: mix Pagada/Parcial/Pendiente")
print(f"  - {total_pagos} pagos")
print(f"  - {total_aplicaciones} aplicaciones de pago")
print(f"  - {total_historial} registros de historial")
print(f"  - 1 ajuste (mensualidad: {MESUALIDAD})")
print()
print("Run 'cargo run' to start the application.")
