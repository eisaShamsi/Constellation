# Motor arabe

Constellation analiza el texto en arabe con un motor morfologico de cinco capas, construido desde cero para esta aplicacion. No es el port de un stemmer existente: es un instrumento nativo que entiende raices, patrones, nombres propios, prestamos y tu propia terminologia. Nunca configuras el motor en si mismo; corre en silencio bajo cada busqueda, cada enlace, cada entrada del indice. Lo que si *puedes* configurar — y lo que cubre este tema de ayuda — es el unico lugar donde el motor invita a tu criterio: el panel **Anulaciones del motor arabe** en Configuracion.

---

## Por que existe el motor

El arabe es una lengua con patrones. Una sola raiz como ك‑ت‑ب ("escribir") genera decenas de formas superficiales — كاتب (escritor), مكتوب (escrito), كتاب (libro), يكتب (el escribe), كتبنا (escribimos) — que al buscar deben colapsar al mismo nucleo semantico. Un stemmer ingenuo o bien destroza estas formas (recortando en exceso وائل a ائل, por ejemplo) o bien pierde por completo la conexion entre ellas. El motor de Constellation evita ambos fallos haciendo pasar cada palabra arabe por cinco capas en estricto orden de prioridad:

1. **Capa 0 — Anulaciones del usuario** (esta es la que tu controlas)
2. **Capa 2 — Lista protegida** (unos 1.200 nombres propios, lugares, prestamos y palabras funcionales curados a mano que no deben tocarse nunca)
3. **Capa 3 — FST generativo** (un transductor de estados finitos compilado que mapea unas 7.000 raices x 158 patrones a todo su vocabulario superficial)
4. **Capa 3b — Cascada** (reparaciones fonologicas: asimilacion, raices debiles, colocacion de la hamza)
5. **Capa 5 — Heuristica** (el respaldo indulgente — un recortador de afijos conservador que solo actua cuando todas las demas capas han declinado responder)

Un paso de clasificacion (Capa 4) elige el mejor analisis cuando mas de una capa produce una lectura. La clasificacion coloca tus anulaciones por encima de todo lo demas.

---

## Funcion: Anulaciones del motor arabe

### Que es

El panel de Anulaciones es una pequena tabla en Configuracion donde le dices al motor, con tus propias palabras, como analizar superficies arabes concretas. Cada anulacion tiene:

- **Forma superficial** — la palabra arabe exactamente como la escribes (p. ej. وائل).
- **Lema** — la forma canonica que debe devolver el motor (p. ej. وائل).
- **Raiz** — opcional. Tres o cuatro consonantes si la palabra tiene raiz clasica.
- **Patron** — opcional. Una etiqueta libre (p. ej. `فاعل`) si quieres registrar la plantilla morfologica.
- **Categoria** — Nombre propio / Sustantivo / Adjetivo / Adverbio / Verbo / Particula / Extranjero / Desconocido.
- **Nota** — opcional. Una linea de contexto para ti mismo en el futuro.

### Por que importa

¿Toda red de conocimiento contiene terminos que el motor no puede conocer por un diccionario? Si: tus propias acunaciones, nombres de tu pueblo, siglas de tu campo, prestamos que tus colegas prefieren escritos de una forma concreta. Sin anulaciones, el motor aplicaria su analisis generico a esas superficies y tus resultados de busqueda se fragmentarian en torno a ligeras variaciones. Una anulacion es la respuesta soberana — vence al FST generativo, a la cascada y al respaldo heuristico. La clasificacion de la Capa 4 otorga a las anulaciones el origen mas alto y una confianza de 1,0, por lo que nunca se descartan en favor de otro analisis.

Las anulaciones viven en un unico archivo JSON en `<tu Universo>/.constellation/arabic-overrides.json`. El archivo es texto plano, ordenado alfabeticamente y escrito de forma atomica (mediante un par `.tmp`+renombrar), por lo que un corte de corriente durante una edicion no puede corromperlo. Es tuyo — puedes ponerlo bajo control de versiones, diferenciarlo o compartirlo entre dispositivos.

### Como usarlo

**Paso 1: abrir el panel**

Haz clic en el icono del engranaje de la barra superior derecha (o pulsa `Ctrl + ,` / `Cmd + ,`) para abrir Configuracion. En la barra lateral izquierda, selecciona **Anulaciones del arabe** — esta junto a **Idioma**. Si no la ves, desplaza la barra lateral.

**Paso 2: anadir tu primera anulacion**

Haz clic en **Anadir anulacion**. Aparece un formulario con seis campos (forma superficial, lema, raiz, patron, categoria, nota). Escribe la forma superficial tal como la tecleas en tus notas — el motor normaliza diacriticos y variantes de alif internamente, asi que no tienes que reproducirlas al detalle. Rellena el lema que deseas que devuelva. Deja raiz y patron en blanco si no los sabes; el motor seguira usando la anulacion. Elige una categoria del desplegable o dejala en **Desconocido**. Haz clic en **Guardar**.

**Paso 3: observar la franja de reindexacion**

En cuanto guardas, el panel muestra **Reindexando…** y el motor barre cada nota del Universo activo cuyo texto contiene esa superficie. Cada nota coincidente se vuelve a tokenizar bajo el nuevo veredicto de la anulacion. Cuando el barrido termina — normalmente en menos de un segundo en un Universo tipico — la franja cambia a **Se reindexaron N nota(s)** y se oculta sola tras tres segundos. No necesitas reiniciar la aplicacion y no necesitas reconstruir ningun indice.

**Paso 4: verificar en la busqueda**

Abre el hub de Busqueda (`Ctrl + K` / `Cmd + K`) y teclea la superficie. Las coincidencias deben reflejar ahora el lema que especificaste: las consultas por el lema encuentran la superficie, y las consultas por la superficie encuentran otras flexiones del lema.

**Paso 5: eliminar una anulacion**

Haz clic en el boton **x** de la fila de la anulacion. La entrada se elimina del disco al instante y el mismo barrido de reindexacion corre al reves — las notas que contenian la superficie se retokenizan bajo el analisis generico del motor. La franja informa cuantas notas se tocaron.

### Interaccion con la Lista protegida

La Lista protegida (Capa 2) ya contiene unas 1.200 superficies comunes que no deben recortarse nunca — nombres como وائل, lugares como فلسطين, prestamos como إنترنت. No necesitas anadirlos tu; vienen con el motor. Usa el panel de Anulaciones para superficies *personales* de tu Universo — tu propia terminologia, nombres locales, prestamos especificos de tu area o casos en los que no coincides con la lectura automatica del motor.

### Interaccion entre Universos

Cada Universo tiene su propio archivo de anulaciones. Al cambiar de Universo se intercambia el conjunto de anulaciones activo en memoria — el motor recarga el JSON desde la carpeta `.constellation/` del nuevo Universo. Si falta el archivo (Universo recien creado), el motor trata el conjunto como vacio. Si el archivo esta mal formado, el motor registra un aviso y cae a un conjunto vacio en lugar de negarse a cargar.

### Que ocurre si editas el archivo a mano

Puedes hacerlo. El formato del archivo es:

```json
[
  {
    "surface": "وائل",
    "lemma": "وائل",
    "root": null,
    "pattern": null,
    "pos": "ProperNoun",
    "note": "Nombre propio — no recortar nunca"
  }
]
```

Mantén las entradas ordenadas alfabeticamente por superficie para diffs amigables con git. El motor reordena en cada guardado, asi que las reordenaciones manuales no sobreviven a una edicion desde la interfaz.

---

## Glosario

- **Forma superficial** — una palabra arabe tal como se escribe, incluidos los cliticos unidos (p. ej. الكتاب, بالكتاب, كتبنا).
- **Lema** — la forma de cita de una palabra, sin flexion (p. ej. كتاب).
- **Raiz** — el nucleo semantico de 3 o 4 consonantes compartido por una familia de palabras (p. ej. ك‑ت‑ب).
- **Patron** — la plantilla de vocales y afijos que combinada con una raiz produce una superficie (p. ej. فاعل → كاتب).
- **FST** — un transductor de estados finitos. El motor usa uno para mapear raices x patrones a todo su vocabulario superficial de forma eficiente.
- **Cascada** — la capa de reparaciones fonologicas que gestiona asimilacion, consonantes debiles y colocacion de la hamza.
- **Anulacion** — tu propio veredicto sobre como se debe analizar una superficie concreta; vence a cualquier otra capa.
