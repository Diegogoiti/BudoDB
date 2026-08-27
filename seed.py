import sqlite3
import os
import random

DB_PATH = os.path.join(os.path.dirname(__file__), "database", "database.db")

# Ensure data directory exists
os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)

# Remove old DB for a fresh start
if os.path.exists(DB_PATH):
    os.remove(DB_PATH)

conn = sqlite3.connect(DB_PATH)
conn.execute("PRAGMA foreign_keys = ON")

# Create tables
conn.executescript("""
CREATE TABLE representantes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    numero_contacto TEXT NOT NULL,
    eliminado BOOLEAN NOT NULL DEFAULT 0
);
CREATE TABLE alumnos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    fecha_de_nacimiento TEXT NOT NULL,
    rango INTEGER NOT NULL,
    representante_id INTEGER NOT NULL DEFAULT 0,
    rallita BOOLEAN NOT NULL DEFAULT 0,
    eliminado BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (representante_id) REFERENCES representantes(id)
);
CREATE TABLE pagos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    representante_id INTEGER NOT NULL,
    monto REAL NOT NULL,
    periodo TEXT NOT NULL,
    fecha TEXT NOT NULL,
    observacion TEXT NOT NULL DEFAULT '',
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
CREATE TABLE abonos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deuda_id INTEGER NOT NULL,
    monto REAL NOT NULL,
    fecha TEXT NOT NULL,
    observacion TEXT NOT NULL DEFAULT '',
    eliminado BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (deuda_id) REFERENCES deudas(id)
);
CREATE TABLE ajustes (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL
);
CREATE TABLE historial_pagos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    representante_id INTEGER NOT NULL,
    tipo TEXT NOT NULL,
    monto REAL NOT NULL DEFAULT 0,
    periodo TEXT NOT NULL,
    fecha TEXT NOT NULL,
    observacion TEXT NOT NULL DEFAULT '',
    eliminado BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (representante_id) REFERENCES representantes(id)
);
""")

# Seed representantes
representantes = [
    ("Maria Garcia", "0412-1234567"),
    ("Carlos Rodriguez", "0424-7654321"),
    ("Ana Martinez", "0416-9876543"),
    ("Pedro Lopez", "0412-5551234"),
    ("Laura Fernandez", "0424-3334567"),
    ("Jose Hernandez", "0416-7778901"),
    ("Carmen Diaz", "0412-2223456"),
    ("Roberto Sanchez", "0424-8889012"),
]

for nombre, telefono in representantes:
    conn.execute("INSERT INTO representantes (nombre, numero_contacto) VALUES (?, ?)", (nombre, telefono))

# Seed alumnos
alumnos_data = [
    ("Juan Perez", "2012-03-15", 6, 1, False),
    ("Sofia Garcia", "2010-07-22", 8, 1, False),
    ("Diego Rodriguez", "2014-11-10", 4, 2, True),
    ("Valentina Martinez", "2008-01-30", 10, 2, False),
    ("Mateo Lopez", "2011-06-18", 7, 3, False),
    ("Isabella Fernandez", "2013-09-05", 5, 3, True),
    ("Santiago Hernandez", "2009-04-12", 9, 4, False),
    ("Camila Diaz", "2015-02-28", 3, 4, True),
    ("Sebastian Sanchez", "2007-12-01", -1, 5, False),
    ("Mariana Torres", "2010-08-17", 8, 5, False),
    ("Adrian Reyes", "2012-05-09", 6, 6, False),
    ("Lucia Morales", "2014-10-25", 4, 6, True),
    ("Andres Gutierrez", "2008-06-14", 10, 7, False),
    ("Daniela Castillo", "2011-01-20", 7, 7, False),
    ("Pablo Vargas", "2013-03-03", 5, 8, True),
    ("Elena Mendoza", "2009-11-11", 9, 8, False),
    ("Nicolas Romero", "2015-07-07", 2, 1, True),
    ("Gabriela Ramos", "2010-04-04", 8, 2, False),
    ("Fernando Silva", "2012-09-16", 6, 3, False),
    ("Alejandra Cruz", "2014-02-14", 4, 4, False),
]

for nombre, fecha, rango, rep_id, rallita in alumnos_data:
    conn.execute(
        "INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante_id, rallita) VALUES (?, ?, ?, ?, ?)",
        (nombre, fecha, rango, rep_id, rallita)
    )

# Seed deudas para agosto 2026 (representantes activos: 1-4)
for rep_id in range(1, 5):
    conn.execute(
        "INSERT INTO deudas (representante_id, monto_total, monto_pendiente, periodo, fecha_vencimiento, estado_id) VALUES (?, 1500, 1500, '2026-08', '2026-08-10', 1)",
        (rep_id,)
    )

# Set mensualidad default
conn.execute("INSERT OR REPLACE INTO ajustes (clave, valor) VALUES ('monto_mensualidad', '1500')")

conn.commit()
conn.close()

print(f"Database seeded at: {DB_PATH}")
print(f"  - {len(representantes)} representantes")
print(f"  - {len(alumnos_data)} alumnos")
print(f"  - 4 deudas agosto 2026")
print(f"  - 1 ajuste (mensualidad: 1500)")
print()
print("Run 'cargo run' to start the application.")
