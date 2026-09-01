//! Datos de demostración: dos mundos que no se mezclan.
//!
//! - Cuentas 1..=6 (nombres calmos): posts largos, pocos likes, horas/días.
//!   Todo el mundo las sigue → dominan el algoritmo Bienestar.
//! - Cuentas 7..=12 (nombres de loop): ganchos cortos, miles de likes,
//!   publicados hace minutos. Nadie las sigue → colonizan Engagement
//!   (recencia + viralidad + slots de "novedad").
//!
//! Así, cambiar de algoritmo en Ajustes no reordena lo mismo: cambia el
//! tipo de conversación que ves.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

/// Subir este número fuerza un reseed al arrancar (borra users/posts/follows).
const SEED_VERSION: i64 = 2;

/// Primeras `CALM_COUNT` cuentas son las que todo el mundo sigue.
const CALM_COUNT: usize = 6;

const USERS: &[(&str, &str, &str)] = &[
    // --- Cuentas de bienestar (seguidas) ---
    ("sofia.calma", "Sofía Calma", "#7fa08d"),
    ("mateo.rio", "Mateo Río", "#8fa9c4"),
    ("valen.bosque", "Valen Bosque", "#94b49f"),
    ("lucia.mar", "Lucía Mar", "#9fb8cf"),
    ("tomas.cumbre", "Tomás Cumbre", "#b8a6c9"),
    ("emma.luz", "Emma Luz", "#d4b483"),
    // --- Cuentas de engagement (nadie las sigue) ---
    ("loop.infinito", "Loop Infinito", "#e05a6a"),
    ("click.ahora", "Click Ahora", "#e07840"),
    ("hilo.viral", "Hilo Viral", "#c44d8a"),
    ("fomo.diario", "FOMO Diario", "#d94c6a"),
    ("scroll.eterno", "Scroll Eterno", "#c45c2e"),
    ("dopa.mina", "Dopa Mina", "#b03d7a"),
];

// (índice de autor 0-based, minutos de antigüedad, likes, comments, contenido)
const POSTS: &[(usize, u64, u32, u32, &str)] = &[
    // --- Bienestar: reflexivo, >= 300 chars, likes modestos, horas/días ---
    (0, 95, 34, 6, "Esta mañana salí a caminar sin el teléfono y noté algo curioso: los primeros diez minutos fueron incómodos, como si me faltara algo en la mano. Después llegó una calma extraña. Escuché a los pájaros, el ruido de mis pasos, y pensé con claridad en un problema que venía arrastrando hace semanas. La solución apareció sola, sin buscarla. Tal vez la atención no se entrena consumiendo más, sino dejando espacio para que la mente respire."),
    (2, 210, 41, 12, "Terminé de leer un libro que me llevó un mes entero y quiero compartir una idea que me quedó dando vueltas: la profundidad no es un lujo, es una habilidad. Cada vez que elegimos el resumen en vez del texto, el video de treinta segundos en vez de la conversación larga, estamos entrenando algo. La pregunta que me hago es qué estamos entrenando exactamente, y si dentro de diez años vamos a poder sostener un pensamiento complejo el tiempo suficiente como para terminarlo."),
    (4, 340, 28, 9, "Hace un año empecé a escalar y hoy por fin llegué a la cumbre que veía desde mi ventana. No fue el paisaje lo que me emocionó, fue darme cuenta de que hay cosas que solo se consiguen despacio. Nadie puede escalar una montaña en un scroll. Hay que poner un pie, después el otro, respirar, seguir. La montaña no negocia atajos y creo que por eso me hace tan bien: es un recordatorio de que lo valioso tiene otro ritmo, uno que ninguna notificación puede apurar."),
    (1, 150, 22, 4, "Receta de domingo lento: pan casero de masa madre. Mezclar harina, agua y paciencia. Dejar reposar la masa toda la noche, como a las buenas ideas. A la mañana, hornear y compartir con alguien querido. El secreto no está en los ingredientes sino en no apurar ninguno de los pasos. Llevo tres meses haciendo este ritual cada domingo y puedo decir que amasar es la meditación más subestimada que existe. El pan sale distinto cada vez, y está bien que así sea."),
    (3, 480, 37, 11, "Anoche me quedé mirando el cielo cuarenta minutos esperando ver la aurora que anunciaban. No apareció. Y sin embargo fue una de las mejores noches del mes: el frío en la cara, las estrellas de siempre, la conversación con mi hermana en voz baja para no romper nada. A veces el plan falla y el momento sale mejor. Me pregunto cuántas noches así me perdí por quedarme adentro mirando las auroras de otros en una pantalla, esperando una vida en diferido."),
    (5, 600, 19, 7, "Hoy la niebla tapó toda la ciudad y los edificios desaparecieron durante horas. Me senté a dibujarla desde el balcón, aunque dibujar niebla es básicamente dibujar nada. Mi abuela decía que la niebla no esconde el paisaje, lo descansa. Me gusta pensar que la atención funciona igual: necesita sus días de niebla, sus ratos sin estímulo, para que cuando el cielo se despeje podamos ver de verdad lo que tenemos enfrente, y no solo mirarlo de pasada."),
    (1, 720, 45, 14, "Tres horas remando en el río al amanecer. Sin música, sin podcast, sin nada. Solo el ritmo de los remos y la respiración. Pensé en por qué nos cuesta tanto estar en silencio, y creo que es porque el silencio nos presenta con nosotros mismos, y a veces no sabemos qué decirnos. Pero como cualquier conversación incómoda, mejora con la práctica. Hoy el río estaba planchado como un espejo y por primera vez en meses sentí que el tiempo alcanzaba para todo."),
    (3, 890, 31, 8, "El mar me enseñó algo sobre las redes sociales, aunque suene raro. Las olas llegan sin parar, una tras otra, y si intentás prestarle atención a todas terminás mareado. Los surfistas lo saben: el arte está en dejar pasar casi todas y elegir una. Creo que con la información pasa igual. No hay que leer todo, ni opinar de todo, ni estar en todo. Hay que aprender a flotar, mirar el horizonte con paciencia y remar fuerte solo cuando viene la ola que importa."),
    (5, 1100, 26, 5, "Apagué las notificaciones hace exactamente un mes y esto es lo que cambió: leo unos veinte minutos más por día, llego menos cansada a la noche y, lo más raro, no me perdí absolutamente nada importante. Todo lo urgente encontró la manera de llegar. Lo que desapareció fue el ruido, esa sensación de estar siempre en deuda con el teléfono. No digo que sea la solución para todos, pero me hizo pensar cuántas de nuestras urgencias son en realidad de otros."),
    (0, 1350, 33, 10, "Caminé el mismo sendero que camino hace años y encontré un árbol caído que no estaba. Me senté encima a tomar mate y pensé: este árbol tardó ochenta años en crecer y una tormenta de veinte minutos lo bajó. Pero ya tenía musgo encima, hongos, bichos, vida nueva. La naturaleza no conoce el desperdicio, solo la transformación. Volví a casa despacio, sin apuro, con esa sensación rara de haber entendido algo que todavía no sé poner del todo en palabras."),
    (2, 1600, 24, 6, "Planté tomates en el balcón hace cuatro meses y hoy coseché el primero. Un solo tomate. Las cuentas no cierran: meses de riego, tierra, macetas, para un tomate que en la verdulería sale monedas. Y sin embargo no recuerdo la última vez que algo me dio tanta satisfacción. Creo que es porque el balcón no tiene botón de acelerar. La planta crece a su ritmo y a vos no te queda otra que acompañar. Eso, que parece una limitación, es exactamente el regalo."),
    (4, 1900, 29, 9, "Trabajo en un faro imaginario: todas las mañanas escribo una página antes de mirar el teléfono. Es mi manera de encender la luz propia antes de que lleguen las luces de los demás. Algunas páginas son malas, la mayoría son del montón, pero cada tanto aparece una frase que me justifica la semana. Si esperara a tener tiempo libre para escribir, no escribiría nunca. La atención de la mañana es la más limpia del día y decidí dejar de regalarla tan barata."),
    (0, 260, 74, 15, "Encontré una librería de barrio que no conocía. Me atendió el dueño, hablamos veinte minutos de novelas que no estaban en ningún ranking. Salí con tres libros que no buscaba y con la sensación de haber hablado con una persona de verdad, no con un perfil. Quiero volver el sábado, sin apuro, y preguntarle qué está leyendo él."),
    (1, 380, 88, 18, "El río hoy a las 7am. Nadie en la costanera. El agua estaba tan quieta que se veían las piedras del fondo. Estas mañanas no dan likes ni se pueden acelerar: duran exactamente lo que duran, y por eso valen la semana entera."),
    (2, 640, 59, 9, "Taller de cerámica, clase 3: mi bowl sigue saliendo torcido pero ya tiene personalidad. La profe dijo que el barro enseña a aceptar lo imperfecto porque no hay Ctrl+Z. Me quedé pensando en cuántas cosas de la vida estoy editando en vez de habitar."),
    (3, 800, 65, 12, "Playlist para cocinar despacio un día de lluvia: jazz viejo, alguna voz que no apure. La dejo en comentarios por si a alguien le sirve acompañar una olla sin mirar la pantalla. Hoy hice lentejas y la casa huele a domingo de verdad."),
    (4, 1150, 71, 13, "Entrenamiento de hoy: 12 km por el cerro. Las piernas protestan, la cabeza agradece. Arriba no había señal y por un rato el mundo se redujo a respirar y poner un pie. Bajé más liviano de lo que subí, que es una métrica que ninguna app registra."),
    (5, 1400, 55, 8, "Volvieron las golondrinas al techo de casa. Puntuales como todos los años, sin anunciarse. Me hice un té y las miré un rato largo. No hay hilo, no hay parte 2. Solo pájaros que saben cuándo volver, y una tarde que no pedía nada de mí."),

    // --- Engagement: carnada corta, miles de interacciones, minutos ---
    (6, 2, 48_200, 3_410, "PARÁ. No scrollees. Esto te va a CAMBIAR el cerebro 🧠🔥"),
    (7, 4, 39_800, 2_870, "El 99% se va antes del final. ¿Vos aguantás? 👇💀"),
    (8, 6, 52_100, 4_120, "Me banearon por decir ESTO (parte 7) y lo vuelvo a subir 😤"),
    (9, 3, 44_600, 3_050, "Si no das like en 3 segundos tu crush te bloquea. PROBADO."),
    (10, 5, 61_400, 5_330, "POV: son las 3:17am y ESTO es lo único que te mantiene vivo"),
    (11, 7, 36_900, 2_440, "Nadie te va a avisar: tu algoritmo ya decidió por vos 👁️"),
    (6, 9, 28_700, 1_980, "Hilo 🧵 las 7 señales de que SOS adicto y no lo sabés (la 4 duele)"),
    (7, 11, 33_500, 2_210, "TEST imposible: tu color revela el trauma que escondés 🤯"),
    (8, 8, 41_200, 2_760, "esto es TODO lo que está mal con la gente. RT si estás de acuerdo"),
    (9, 13, 24_800, 1_640, "Gané una apuesta IMPOSIBLE. Mirá hasta el final o no sirve 🎥"),
    (10, 10, 31_600, 2_080, "día 63 comiendo lo mismo. like si seguís acá. no pregunten 💀"),
    (11, 15, 27_400, 1_770, "Una IA me dijo mi futuro y ahora no puedo dormir 😳👉"),
    (6, 18, 22_100, 1_450, "El truco que NO quieren que sepas. Lo borro en 24h ⏳"),
    (7, 12, 35_900, 2_390, "si esto llega a 10k likes me tatuo el logo en la cara 🟢"),
    (8, 21, 19_800, 1_280, "5 señales de que tu ex todavía te stalkea (la 3 me destrozó)"),
    (9, 16, 26_300, 1_810, "No vas a creer lo que pasó cuando toqué ESTO 😱 hilo"),
    (10, 24, 18_400, 1_120, "Ranking DEFINITIVO. Si no estás de acuerdo, bloqueame."),
    (11, 19, 23_700, 1_560, "Abrí la app «solo un minuto». Han pasado 4 horas. OTRA VEZ."),
    (6, 28, 16_900, 990, "Te reto a no dar like. Imposible. El cerebro no puede 🧠"),
    (7, 22, 21_500, 1_340, "ÚLTIMA PARTE. La que no te mostraron. Entrá AHORA 🚨"),
];

pub fn seed_if_empty(conn: &Connection, now: u64) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS seed_meta (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL
        );",
    )?;
    let current: Option<i64> = conn
        .query_row("SELECT version FROM seed_meta WHERE id = 1", [], |r| r.get(0))
        .optional()?;
    if current == Some(SEED_VERSION) {
        return Ok(());
    }

    tracing::info!(
        anterior = current,
        nueva = SEED_VERSION,
        "reseed de demo: cuentas calmas vs. loop de atención"
    );

    conn.execute_batch(
        "DELETE FROM feed_views;
         DELETE FROM posts;
         DELETE FROM follows;
         DELETE FROM settings;
         DELETE FROM users;
         DELETE FROM demo;
         DELETE FROM seed_meta;",
    )?;

    for (i, (username, display, color)) in USERS.iter().enumerate() {
        conn.execute(
            "INSERT INTO users (id, username, display_name, avatar_color) VALUES (?1, ?2, ?3, ?4)",
            params![(i + 1) as i64, username, display, color],
        )?;
        conn.execute(
            "INSERT INTO settings (user_id, algorithm_id, nonce) VALUES (?1, 2, 1)",
            params![(i + 1) as i64],
        )?;
    }

    // Todos siguen a las cuentas calmas (excepto a sí mismos). Nadie sigue
    // a las cuentas virales: Engagement las inyecta igual por "novedad".
    let n = USERS.len() as i64;
    let calm = CALM_COUNT as i64;
    for follower in 1..=n {
        for followee in 1..=calm {
            if follower != followee {
                conn.execute(
                    "INSERT INTO follows (follower_id, followee_id) VALUES (?1, ?2)",
                    params![follower, followee],
                )?;
            }
        }
    }

    for (author_idx, mins_ago, likes, comments, content) in POSTS {
        conn.execute(
            "INSERT INTO posts (author_id, content, created_at, likes, comments)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                (*author_idx + 1) as i64,
                content,
                (now - mins_ago * 60) as i64,
                likes,
                comments
            ],
        )?;
    }

    conn.execute("INSERT INTO demo (id, malicious) VALUES (1, 0)", [])?;
    conn.execute(
        "INSERT INTO seed_meta (id, version) VALUES (1, ?1)",
        params![SEED_VERSION],
    )?;
    Ok(())
}
