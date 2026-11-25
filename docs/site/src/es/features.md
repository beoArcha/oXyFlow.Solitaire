[English](../features.md) · [Polski](../pl/features.md) · [Deutsch](../de/features.md) · **Español** · [Français](../fr/features.md) · [Italiano](../it/features.md) · [Nederlands](../nl/features.md) · [Português (Brasil)](../pt-br/features.md) · [Українська](../uk/features.md)

# Funciones

## Juegos

Catorce juegos en seis familias, un solo programa, un solo conjunto de teclas:

- **Klondike**: robar 1 o robar 3.
- **Spider**: 1, 2 o 4 palos.
- **FreeCell**.
- **Pirámide**: relajada o clásica.
- **Familia Yukon**: Yukon, Solitario ruso, Alaska, Moosehide y Australiano.
- **Golf**: despejar siete columnas sobre el descarte, un rango más alto o más bajo cada vez.

El menú de juegos agrupa los juegos bajo encabezados de familia, marca con una estrella los populares, guarda tus favoritos en su propia lista y encuentra un juego a medida que escribes su nombre. Las reglas de cada juego están en la ventana de ayuda (`i`, "Cómo jugar").

## Repartos: semillas, repartos ganables y dificultad

Cada reparto tiene una **semilla**. Cópiala con `c` para repetir el reparto o compartirlo.

- **Solo repartos ganables.** Una partida nueva puede esperar a un reparto que el solucionador integrado haya demostrado ganable. El solucionador conoce cada carta, así que "ganable" significa que el reparto puede ganarse, no que sea fácil.
- **Nueva partida desde una semilla.** Elige primero el juego y después de dónde sale la semilla: escribir una, elegir uno de los repartos que ya has jugado (el más reciente primero, con el resultado y el tiempo), dejar que el juego escoja uno de ellos al azar o tomar uno de los **repartos seleccionados**.
- **Cinco niveles de dificultad.** Cada juego ofrece tres repartos seleccionados por nivel, de muy fácil a muy difícil. Todos están demostrados ganables. Los niveles salen de mediciones: cuánto trabajo necesitó el solucionador, cuánto dura la línea ganadora y si seguir solo las pistas basta para ganar el reparto.

## Ayuda mientras juegas

- **Panel de guía** (`g`): los datos de la partida, el objetivo del juego, las teclas (marcando las que funcionan ahora) y las acciones del ratón. A 130 columnas o más se sitúa junto al tablero, con un botón **Nueva partida** arriba; en terminales más estrechas se abre en una ventana.
- **Panel de jugadas posibles** (`v`): cada jugada que puedes hacer ahora, con su tecla, y el motivo cuando una jugada no es posible.
- **Pistas** (`h`): el juego marca una jugada en el tablero y la nombra. Las pistas solo usan lo que puedes ver: nunca miran las cartas boca abajo ni dentro del mazo.
- **Jugadas rápidas**: `a` envía las cartas bajo el cursor a su mejor destino, `e` hace lo mismo con la carta de arriba del descarte desde cualquier punto del tablero, y un doble clic funciona como `a`. Las jugadas rápidas se pueden desactivar en Ajustes o en las opciones rápidas (`o`) si prefieres colocar cada carta tú mismo.
- **Autoterminar**: cuando la victoria es segura, las jugadas restantes se hacen por ti. Se puede desactivar.
- **Deshacer y rehacer** (`u`, `r`), con una opción para jugar sin deshacer.

## Teclado y ratón

Cada acción funciona con el teclado y con el ratón. Haz clic en una carta para cogerla y en un destino para soltarla, haz doble clic para una jugada rápida, arrastra una carta a su destino y haz clic derecho para devolverla. Las barras inferiores y botones como Nueva partida o Cerrar muestran su tecla, y la pestaña activa de una ventana nunca parece el botón enfocado. La ventana de ayuda (`?`) lista cada tecla; también puede quedarse anclada al lado de la pantalla (`d`).

## Ajustes

Los ajustes tienen tres pestañas:

- **Populares** se abre primero y reúne los ajustes que más se cambian: idioma, baja visión, tema, tamaño de cartas, reverso de carta, volumen, aura y autoterminar.
- **Audio y visual** contiene todo lo relativo al aspecto y al sonido: colores, pantalla completa y tamaño de ventana, tema, visión del color, estilos de cartas y tonos, la mesa, el aura, la fuente de la consola, el sonido y la prioridad del proceso.
- **Juego** contiene el idioma y las ayudas: aura, marcas de destino, los paneles de guía y de jugadas, autoterminar y jugadas rápidas.

Una vista previa junto a la lista muestra un cambio antes de que lo conserves.

## Cartas y mesa

- **Seis estilos de caras**: Internacional, Francés, Alemán, Italiano, Español y Suizo, cada uno con sus propios signos de palo o letras de figuras.
- **Dos estilos de reverso** (Celosía y Ornamento) y colores de reverso del tema, Monokai o Dracula.
- **Tono de la cara de la carta**: el del propio tema, blanco, marfil, gris claro u oscuro.
- **Mesa**: el color del tema, fieltro verde, azul, rojo, gris, marrón o negro, y la textura lisa, de fieltro o rayada.
- **Tamaño de cartas**: haz las cartas más grandes con `]` y más pequeñas con `[`; `0` vuelve al tamaño automático que cabe en la ventana. Las columnas largas se juntan para que su última carta siempre se vea entera, y las cartas solo se reducen cuando juntarlas no basta.

## Temas y accesibilidad

Cinco temas: **Oscuro** y **Claro** para el día a día, **Alto contraste** para baja visión, **Monokai** y **Dracula**. Cada uno se muestra aquí en la misma posición.

![El tema Oscuro](../img/es/theme-dark.svg)

![El tema Claro](../img/es/theme-light.svg)

![El tema Alto contraste](../img/es/theme-high-contrast.svg)

![El tema Monokai](../img/es/theme-monokai.svg)

![El tema Dracula](../img/es/theme-dracula.svg)

- **Baja visión.** Un único interruptor, ofrecido en el primer arranque y en Ajustes: contraste de texto de al menos 7:1 en cualquier tema, las cartas más grandes que quepan, un aura gruesa, rango y palo en negrita, marcas de foco y de destino más grandes y, en la consola clásica de Windows, una fuente más grande. Cada parte sigue siendo ajustable, y al desactivar el modo se recuperan tus ajustes anteriores.
- **Visión del color.** Modos para protanopía, deuteranopía, tritanopía y monocromo que cambian los colores para que lo que debes distinguir siga distinguiéndose. La **baraja de cuatro colores** da a cada palo su propio color: picas negras, corazones rojos, diamantes azules y tréboles verdes.
- **El color nunca es la única señal.** Los palos son signos (♠ ♥ ♦ ♣) junto al rango, el elemento enfocado lleva un símbolo, las cartas cogidas y los destinos tienen sus propias marcas, los resultados de ganabilidad llevan ✓ o ✗, y una línea bajo el tablero dice con palabras dónde está el cursor y qué puede hacer.

## Estadísticas

La pantalla de estadísticas muestra los resultados de cada juego (victorias, rachas, mejor tiempo y menos movimientos) y un historial de cada reparto, el más reciente primero y en tu hora local. Desde el historial puedes volver a jugar una semilla, copiarla o pedir al solucionador que compruebe si un reparto era ganable.

## Consola de Windows

En Windows, un juego iniciado con doble clic, o con `oxysol -n`, es dueño de su ventana de consola. Se abre a pantalla completa de forma predeterminada, y en modo ventana puedes elegir su tamaño en Ajustes: 130 × 40, 195 × 60 o 260 × 80. Un juego iniciado en una consola que ya está abierta deja esa consola como está. En la consola clásica de Windows, Ctrl+más, Ctrl+menos y Ctrl+0 cambian el tamaño de la fuente de la consola; otros terminales hacen zoom con sus propias teclas.

## Idiomas

El juego está disponible en nueve idiomas: inglés, polaco, alemán, español, francés, italiano, neerlandés, portugués de Brasil y ucraniano. Elige uno con el selector de banderas de la esquina superior izquierda del menú principal, o en Ajustes; se aplica al instante. Este sitio web está disponible en los mismos nueve idiomas.

## Lo que aún no es

oXySol es una versión previa. Espera asperezas, y cuéntanos sobre ellas en el
[gestor de incidencias](https://github.com/beoArcha/oXyFlow.Solitaire/issues).
