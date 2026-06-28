import random
import sqlite3
from datetime import datetime, timedelta


def generar_fecha():
    # Genera fechas para alumnos entre 5 y 25 años
    start_date = datetime.now() - timedelta(days=25 * 365)
    end_date = datetime.now() - timedelta(days=5 * 365)
    random_date = start_date + (end_date - start_date) * random.random()
    return random_date.strftime("%Y-%m-%d")


def llenar_db():
    # Conexión a tu archivo (asegúrate de que la ruta sea correcta)
    try:
        conn = sqlite3.connect("./database/database.db")
    except sqlite3.Error as e:
        print(f"Error al conectar con la base de datos: {e}")

    cursor = conn.cursor()

    nombres = [
        "Juan",
        "Maria",
        "Pedro",
        "Ana",
        "Luis",
        "Carmen",
        "Diego",
        "Elena",
        "Carlos",
        "Sofia",
    ]
    apellidos = [
        "Garcia",
        "Rodriguez",
        "Perez",
        "Martinez",
        "Lopez",
        "Sanchez",
        "Gonzalez",
        "Diaz",
    ]
    rangos = [x for x in range(0, 11)]  # Tus niveles de cinta

    print("Insertando 100 alumnos de prueba...")

    cantidad_alumnos = 300
    for i in range(cantidad_alumnos):
        nombre_completo = f"{random.choice(nombres)} {random.choice(apellidos)}"
        fecha = generar_fecha()
        rango = int(random.choice(rangos))
        rep = f"Padre de {nombre_completo.split()[0]}"
        tlf = f"0414-{random.randint(1000000, 9999999)}"

        cursor.execute(
            """
            INSERT INTO alumnos (nombre, fecha_de_nacimiento, rango, representante, numero_contacto, rallita)
            VALUES (?, ?, ?, ?, ?, ?)
        """,
            (nombre_completo, fecha, rango, rep, tlf, random.choice([0, 1])),
        )

    conn.commit()
    conn.close()
    print(f"¡Listo! {cantidad_alumnos} alumnos agregados con éxito.")


if __name__ == "__main__":
    llenar_db()
