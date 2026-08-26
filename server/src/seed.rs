//! Datos de demostración: 12 usuarios, una red de follows y ~40 posts en
//! español que mezclan contenido reflexivo largo (favorecido por el algoritmo
//! Bienestar) y contenido viral corto (favorecido por Engagement), para que
//! la diferencia entre algoritmos sea visible en la demo.

use anyhow::Result;
use rusqlite::{params, Connection};

const USERS: &[(&str, &str, &str)] = &[
    ("sofia.calma", "Sofía Calma", "#7fa08d"),
    ("mateo.rio", "Mateo Río", "#8fa9c4"),
    ("valen.bosque", "Valen Bosque", "#94b49f"),
    ("lucia.mar", "Lucía Mar", "#9fb8cf"),
    ("tomas.cumbre", "Tomás Cumbre", "#b8a6c9"),
    ("emma.luz", "Emma Luz", "#d4b483"),
    ("juli.brisa", "Juli Brisa", "#a3c4bc"),
    ("nico.sendero", "Nico Sendero", "#c9a689"),
    ("cata.aurora", "Cata Aurora", "#b5c9a6"),
    ("fede.raiz", "Fede Raíz", "#8d9f7f"),
    ("mia.niebla", "Mía Niebla", "#c4b3d4"),
    ("leo.faro", "Leo Faro", "#86b3a2"),
];

// (índice de autor 0-based, minutos de antigüedad, likes, comments, contenido)
const POSTS: &[(usize, u64, u32, u32, &str)] = &[
    // --- Contenido reflexivo largo (>= 300 chars: bonus del algoritmo Bienestar) ---
    (0, 95, 34, 6, "Esta mañana salí a caminar sin el teléfono y noté algo curioso: los primeros diez minutos fueron incómodos, como si me faltara algo en la mano. Después llegó una calma extraña. Escuché a los pájaros, el ruido de mis pasos, y pensé con claridad en un problema que venía arrastrando hace semanas. La solución apareció sola, sin buscarla. Tal vez la atención no se entrena consumiendo más, sino dejando espacio para que la mente respire."),
    (2, 210, 41, 12, "Terminé de leer un libro que me llevó un mes entero y quiero compartir una idea que me quedó dando vueltas: la profundidad no es un lujo, es una habilidad. Cada vez que elegimos el resumen en vez del texto, el video de treinta segundos en vez de la conversación larga, estamos entrenando algo. La pregunta que me hago es qué estamos entrenando exactamente, y si dentro de diez años vamos a poder sostener un pensamiento complejo el tiempo suficiente como para terminarlo."),
    (4, 340, 28, 9, "Hace un año empecé a escalar y hoy por fin llegué a la cumbre que veía desde mi ventana. No fue el paisaje lo que me emocionó, fue darme cuenta de que hay cosas que solo se consiguen despacio. Nadie puede escalar una montaña en un scroll. Hay que poner un pie, después el otro, respirar, seguir. La montaña no negocia atajos y creo que por eso me hace tan bien: es un recordatorio de que lo valioso tiene otro ritmo, uno que ninguna notificación puede apurar."),
    (6, 150, 22, 4, "Receta de domingo lento: pan casero de masa madre. Mezclar harina, agua y paciencia. Dejar reposar la masa toda la noche, como a las buenas ideas. A la mañana, hornear y compartir con alguien querido. El secreto no está en los ingredientes sino en no apurar ninguno de los pasos. Llevo tres meses haciendo este ritual cada domingo y puedo decir que amasar es la meditación más subestimada que existe. El pan sale distinto cada vez, y está bien que así sea."),
    (8, 480, 37, 11, "Anoche me quedé mirando el cielo cuarenta minutos esperando ver la aurora que anunciaban. No apareció. Y sin embargo fue una de las mejores noches del mes: el frío en la cara, las estrellas de siempre, la conversación con mi hermana en voz baja para no romper nada. A veces el plan falla y el momento sale mejor. Me pregunto cuántas noches así me perdí por quedarme adentro mirando las auroras de otros en una pantalla, esperando una vida en diferido."),
    (10, 600, 19, 7, "Hoy la niebla tapó toda la ciudad y los edificios desaparecieron durante horas. Me senté a dibujarla desde el balcón, aunque dibujar niebla es básicamente dibujar nada. Mi abuela decía que la niebla no esconde el paisaje, lo descansa. Me gusta pensar que la atención funciona igual: necesita sus días de niebla, sus ratos sin estímulo, para que cuando el cielo se despeje podamos ver de verdad lo que tenemos enfrente, y no solo mirarlo de pasada."),
    (1, 720, 45, 14, "Tres horas remando en el río al amanecer. Sin música, sin podcast, sin nada. Solo el ritmo de los remos y la respiración. Pensé en por qué nos cuesta tanto estar en silencio, y creo que es porque el silencio nos presenta con nosotros mismos, y a veces no sabemos qué decirnos. Pero como cualquier conversación incómoda, mejora con la práctica. Hoy el río estaba planchado como un espejo y por primera vez en meses sentí que el tiempo alcanzaba para todo."),
    (3, 890, 31, 8, "El mar me enseñó algo sobre las redes sociales, aunque suene raro. Las olas llegan sin parar, una tras otra, y si intentás prestarle atención a todas terminás mareado. Los surfistas lo saben: el arte está en dejar pasar casi todas y elegir una. Creo que con la información pasa igual. No hay que leer todo, ni opinar de todo, ni estar en todo. Hay que aprender a flotar, mirar el horizonte con paciencia y remar fuerte solo cuando viene la ola que importa."),
    (5, 1100, 26, 5, "Apagué las notificaciones hace exactamente un mes y esto es lo que cambió: leo unos veinte minutos más por día, llego menos cansada a la noche y, lo más raro, no me perdí absolutamente nada importante. Todo lo urgente encontró la manera de llegar. Lo que desapareció fue el ruido, esa sensación de estar siempre en deuda con el teléfono. No digo que sea la solución para todos, pero me hizo pensar cuántas de nuestras urgencias son en realidad de otros."),
    (7, 1350, 33, 10, "Caminé el mismo sendero que camino hace años y encontré un árbol caído que no estaba. Me senté encima a tomar mate y pensé: este árbol tardó ochenta años en crecer y una tormenta de veinte minutos lo bajó. Pero ya tenía musgo encima, hongos, bichos, vida nueva. La naturaleza no conoce el desperdicio, solo la transformación. Volví a casa despacio, sin apuro, con esa sensación rara de haber entendido algo que todavía no sé poner del todo en palabras."),
    (9, 1600, 24, 6, "Planté tomates en el balcón hace cuatro meses y hoy coseché el primero. Un solo tomate. Las cuentas no cierran: meses de riego, tierra, macetas, para un tomate que en la verdulería sale monedas. Y sin embargo no recuerdo la última vez que algo me dio tanta satisfacción. Creo que es porque el balcón no tiene botón de acelerar. La planta crece a su ritmo y a vos no te queda otra que acompañar. Eso, que parece una limitación, es exactamente el regalo."),
    (11, 1900, 29, 9, "Trabajo en un faro imaginario: todas las mañanas escribo una página antes de mirar el teléfono. Es mi manera de encender la luz propia antes de que lleguen las luces de los demás. Algunas páginas son malas, la mayoría son del montón, pero cada tanto aparece una frase que me justifica la semana. Si esperara a tener tiempo libre para escribir, no escribiría nunca. La atención de la mañana es la más limpia del día y decidí dejar de regalarla tan barata."),

    // --- Contenido viral corto (carnada del algoritmo Engagement) ---
    (1, 12, 842, 130, "No vas a creer lo que pasó cuando abrí la heladera 😱 hilo 🧵"),
    (5, 25, 1240, 210, "TEST: tu color favorito revela tu personalidad oculta. El mío dio IMPRESIONANTE 🤯"),
    (7, 8, 933, 154, "esto es TODO lo que está mal con la gente hoy en día. RT si estás de acuerdo 🔥"),
    (3, 40, 1510, 260, "El truco que los dentistas NO quieren que sepas 🦷✨"),
    (9, 18, 780, 95, "día 47 comiendo lo mismo. no pregunten. like si seguís acá 💀"),
    (11, 55, 1105, 178, "POV: son las 3am y seguís scrolleando. sos vos. dale like."),
    (0, 33, 690, 88, "5 señales de que tu gato planea algo 🐈 la 3 me dejó helada"),
    (2, 70, 950, 142, "nadie habla de ESTO y es un escándalo. abro debate 👇"),
    (4, 90, 1320, 205, "Gané una apuesta imposible y lo grabé todo 🎥 MIRÁ hasta el final"),
    (6, 15, 875, 120, "si esto llega a 1000 likes me tiño el pelo de verde 🟢"),
    (8, 47, 1030, 165, "ranking DEFINITIVO de facturas. no acepto discusiones. medialunas último."),
    (10, 62, 720, 99, "una IA me dijo mi futuro y ahora no puedo dormir 😳"),

    // --- Contenido medio (ni viral ni largo) ---
    (0, 130, 96, 21, "Encontré una librería de barrio que no conocía. Me atendió el dueño, hablamos veinte minutos de novelas. Salí con tres libros que no buscaba."),
    (1, 260, 74, 15, "El río hoy a las 7am. Nadie en la costanera. Estas mañanas valen la semana entera."),
    (2, 380, 88, 18, "Taller de cerámica, clase 3: mi bowl sigue saliendo torcido pero ya tiene personalidad. Progreso."),
    (3, 500, 65, 12, "Playlist para cocinar despacio un día de lluvia. La dejo en comentarios."),
    (4, 640, 102, 24, "Entrenamiento de hoy: 12km por el cerro. Las piernas protestan, la cabeza agradece."),
    (5, 800, 59, 9, "Volvieron las golondrinas al techo de casa. Puntuales como todos los años."),
    (6, 950, 83, 17, "Hice la receta de ñoquis de mi abuela y la cocina quedó hecha un desastre hermoso."),
    (7, 1150, 71, 13, "Mapa de senderos nuevos del parque provincial. Este finde se estrena. ¿Alguien se suma?"),
    (8, 1400, 90, 20, "Foto de la luna de anoche con el telescopio nuevo. Todavía no lo puedo creer."),
    (9, 1700, 55, 8, "El compost ya tiene lombrices. Sé que a nadie le importa pero estoy orgulloso."),
    (10, 2000, 68, 14, "Terminé mi primera acuarela decente después de treinta intentos. La cuelgo igual."),
    (11, 2300, 77, 16, "Cambié el celular por un libro en la mesa de luz. Semana 2: duermo notablemente mejor."),
    (2, 45, 120, 30, "Hoy el bosque olía a lluvia. Dejo esta foto mental porque no llevé cámara, y está bien así."),
    (4, 2600, 49, 7, "Recordatorio amable: el descanso también es productividad."),
    (6, 2750, 62, 11, "Sopa de calabaza, manta y una serie lenta. Plan imbatible."),
    (8, 175, 85, 19, "Las nubes de hoy parecían pintadas. Tres personas en la plaza mirando el cielo, desconocidas, sonriendo."),
];

pub fn seed_if_empty(conn: &Connection, now: u64) -> Result<()> {
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
    if user_count > 0 {
        return Ok(());
    }

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

    // Cada usuario sigue a los 6 siguientes (red determinista, ~mitad seguidos).
    let n = USERS.len() as i64;
    for i in 1..=n {
        for k in 1..=6 {
            let followee = ((i - 1 + k) % n) + 1;
            conn.execute(
                "INSERT INTO follows (follower_id, followee_id) VALUES (?1, ?2)",
                params![i, followee],
            )?;
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
    Ok(())
}
