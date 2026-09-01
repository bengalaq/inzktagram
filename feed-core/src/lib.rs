//! Núcleo determinista de ranking de inZKtagram.
//!
//! Este crate es la única fuente de verdad de los 3 algoritmos de
//! recomendación. Se compila tanto en el servidor (para responder el feed al
//! instante) como dentro del guest de RISC Zero (para generar la prueba ZK),
//! garantizando que lo que se muestra y lo que se prueba es la misma función.
//!
//! Reglas de determinismo:
//! - Solo aritmética entera (`i64`/`u64`), nunca floats.
//! - Ordenamientos con desempate total por `post_id` (no hay empates ambiguos).
//! - El tiempo actual (`now`) es un input explícito, nunca se lee un reloj.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const ALG_ENGAGEMENT: u8 = 1;
pub const ALG_WELLBEING: u8 = 2;
pub const ALG_MIXED: u8 = 3;

pub fn algorithm_name(id: u8) -> &'static str {
    match id {
        ALG_ENGAGEMENT => "Engagement",
        ALG_WELLBEING => "Bienestar",
        ALG_MIXED => "Mixto",
        _ => "Desconocido",
    }
}

/// Un post candidato a entrar al feed, con las señales que usan los algoritmos.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub post_id: u64,
    pub author_id: u64,
    /// Epoch en segundos.
    pub created_at: u64,
    pub likes: u32,
    pub comments: u32,
    pub length_chars: u32,
    /// Si el usuario del feed sigue al autor.
    pub is_followed: bool,
}

/// Parámetros de los algoritmos. Su hash va al journal: quedan comprometidos.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Params {
    /// Peso de la recencia hiperbólica (engagement).
    pub w_recency: i64,
    /// Peso del log2 de interacciones (engagement).
    pub w_engagement: i64,
    /// Bonus fijo a contenido corto (engagement).
    pub w_short: i64,
    /// Peso del bonus por viralidad (engagement).
    pub w_viral: i64,
    /// Minutos que un post largo puede "adelantarse" (bienestar).
    pub w_long_minutes: i64,
    /// Cada cuántas posiciones el algoritmo engagement inyecta novedad
    /// (posts de cuentas no seguidas con alto engagement).
    pub novelty_interval: u64,
    /// Máximo de posts consecutivos del mismo autor (bienestar y mixto).
    pub max_consecutive_author: u64,
    /// Porcentaje del componente bienestar en el algoritmo mixto (0..=100).
    pub mix_wellbeing_pct: i64,
    /// Largo máximo del feed resultante.
    pub feed_len: u64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            w_recency: 40,
            w_engagement: 25,
            w_short: 180,
            w_viral: 18,
            w_long_minutes: 30,
            novelty_interval: 3,
            max_consecutive_author: 2,
            mix_wellbeing_pct: 60,
            feed_len: 30,
        }
    }
}

/// Configuración del usuario: es lo que el journal compromete como "elección".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfig {
    pub user_id: u64,
    pub algorithm_id: u8,
    /// Nonce regenerado en cada cambio de configuración (evita ambigüedad
    /// sobre *cuál* versión de la config se usó).
    pub nonce: u64,
}

/// Input completo del cómputo del feed. Es exactamente lo que recibe el guest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedInput {
    pub config: UserConfig,
    pub params: Params,
    /// Deben venir en orden canónico (post_id ascendente) para que
    /// `candidates_hash` sea reproducible por un auditor.
    pub candidates: Vec<Candidate>,
    /// Epoch en segundos usado para el ranking.
    pub now: u64,
}

/// Parte pública de la prueba: lo único que el verificador necesita leer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journal {
    pub algorithm_id: u8,
    pub config_hash: [u8; 32],
    pub params_hash: [u8; 32],
    pub candidates_hash: [u8; 32],
    pub feed_hash: [u8; 32],
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Hashes canónicos (dominio separado por prefijo, enteros little-endian)
// ---------------------------------------------------------------------------

pub fn hash_config(c: &UserConfig) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"inzktagram.config.v1");
    h.update(c.user_id.to_le_bytes());
    h.update([c.algorithm_id]);
    h.update(c.nonce.to_le_bytes());
    h.finalize().into()
}

pub fn hash_params(p: &Params) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"inzktagram.params.v1");
    for v in [
        p.w_recency,
        p.w_engagement,
        p.w_short,
        p.w_viral,
        p.w_long_minutes,
        p.mix_wellbeing_pct,
    ] {
        h.update(v.to_le_bytes());
    }
    for v in [p.novelty_interval, p.max_consecutive_author, p.feed_len] {
        h.update(v.to_le_bytes());
    }
    h.finalize().into()
}

pub fn hash_candidates(cands: &[Candidate]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"inzktagram.candidates.v1");
    h.update((cands.len() as u64).to_le_bytes());
    for c in cands {
        h.update(c.post_id.to_le_bytes());
        h.update(c.author_id.to_le_bytes());
        h.update(c.created_at.to_le_bytes());
        h.update(c.likes.to_le_bytes());
        h.update(c.comments.to_le_bytes());
        h.update(c.length_chars.to_le_bytes());
        h.update([c.is_followed as u8]);
    }
    h.finalize().into()
}

/// Hash del feed resultante (lista ordenada de post_ids). El cliente web lo
/// recomputa con WebCrypto sobre el feed que efectivamente renderizó.
pub fn hash_feed(post_ids: &[u64]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"inzktagram.feed.v1");
    for id in post_ids {
        h.update(id.to_le_bytes());
    }
    h.finalize().into()
}

// ---------------------------------------------------------------------------
// Scores
// ---------------------------------------------------------------------------

fn ilog2(x: u64) -> i64 {
    if x == 0 {
        0
    } else {
        (63 - x.leading_zeros()) as i64
    }
}

fn interactions(c: &Candidate) -> u64 {
    2 * c.likes as u64 + 3 * c.comments as u64
}

/// Algoritmo 1 — Engagement: recencia agresiva, premio a lo viral y lo corto.
/// Modela el "loop de dopamina" de las redes actuales.
pub fn engagement_score(c: &Candidate, now: u64, p: &Params) -> i64 {
    let age_secs = now.saturating_sub(c.created_at) as i64;
    // Decaimiento hiperbólico: un post recién publicado vale ~24x más que uno
    // de hace un día.
    let recency = p.w_recency * 86_400 / (age_secs + 3_600);
    let inter = interactions(c);
    let engagement = p.w_engagement * ilog2(inter + 1);
    let short_bonus = if c.length_chars < 120 { p.w_short } else { 0 };
    let viral_bonus = if inter > 100 { p.w_viral * ilog2(inter) } else { 0 };
    recency + engagement + short_bonus + viral_bonus
}

/// Algoritmo 2 — Bienestar: base cronológica; un post largo (>= 300 chars)
/// puede adelantarse hasta `w_long_minutes` minutos. Los ganchos cortos
/// (< 120 chars) se retrasan un día entero: aunque vengan de cuentas seguidas,
/// no colonizan el feed. Sin likes, comentarios ni viralidad.
pub fn wellbeing_score(c: &Candidate, _now: u64, p: &Params) -> i64 {
    let long_bonus = if c.length_chars >= 300 {
        p.w_long_minutes * 60
    } else {
        0
    };
    let short_penalty = if c.length_chars < 120 { 86_400 } else { 0 };
    c.created_at as i64 + long_bonus - short_penalty
}

/// Penalización usada solo por el algoritmo mixto para relegar (sin excluir)
/// los posts de cuentas no seguidas: equivale a ~10 años de antigüedad.
const NOT_FOLLOWED_PENALTY: i64 = 10 * 365 * 24 * 3_600;

// ---------------------------------------------------------------------------
// Rankings
// ---------------------------------------------------------------------------

/// Orden descendente por score, con desempate total por post_id descendente.
fn sorted_by_score<'a, F>(cands: &'a [Candidate], score: F) -> Vec<&'a Candidate>
where
    F: Fn(&Candidate) -> i64,
{
    let mut v: Vec<(i64, &Candidate)> = cands.iter().map(|c| (score(c), c)).collect();
    v.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.post_id.cmp(&a.1.post_id)));
    v.into_iter().map(|(_, c)| c).collect()
}

/// Impone el tope de posts consecutivos del mismo autor, preservando el orden
/// dado tanto como sea posible (greedy determinista).
fn apply_author_cap(mut remaining: Vec<Candidate>, p: &Params) -> Vec<Candidate> {
    let k = p.max_consecutive_author.max(1) as usize;
    let mut feed: Vec<Candidate> = Vec::new();
    while !remaining.is_empty() && (feed.len() as u64) < p.feed_len {
        let violates = |c: &Candidate| {
            feed.len() >= k && feed[feed.len() - k..].iter().all(|f| f.author_id == c.author_id)
        };
        let idx = remaining.iter().position(|c| !violates(c)).unwrap_or(0);
        feed.push(remaining.remove(idx));
    }
    feed
}

fn rank_engagement(cands: &[Candidate], now: u64, p: &Params) -> Vec<Candidate> {
    let sorted = sorted_by_score(cands, |c| engagement_score(c, now, p));
    let interval = p.novelty_interval.max(2);
    let mut used: Vec<u64> = Vec::new();
    let mut feed: Vec<Candidate> = Vec::new();
    let target = (p.feed_len as usize).min(sorted.len());
    for slot in 1..=target as u64 {
        let unused = |c: &&&Candidate| !used.contains(&c.post_id);
        // Cada `interval` posiciones se inyecta "novedad": el mejor post de
        // una cuenta NO seguida. Es el mecanismo de descubrimiento/dopamina.
        let pick = if slot % interval == 0 {
            sorted
                .iter()
                .filter(unused)
                .find(|c| !c.is_followed)
                .or_else(|| sorted.iter().find(unused))
        } else {
            sorted.iter().find(unused)
        };
        if let Some(c) = pick {
            used.push(c.post_id);
            feed.push((*c).clone());
        }
    }
    feed
}

fn rank_wellbeing(cands: &[Candidate], now: u64, p: &Params) -> Vec<Candidate> {
    let followed: Vec<Candidate> = cands.iter().filter(|c| c.is_followed).cloned().collect();
    let sorted: Vec<Candidate> = sorted_by_score(&followed, |c| wellbeing_score(c, now, p))
        .into_iter()
        .cloned()
        .collect();
    apply_author_cap(sorted, p)
}

fn rank_mixed(cands: &[Candidate], now: u64, p: &Params) -> Vec<Candidate> {
    // Combinación estilo Borda: se rankea bajo cada algoritmo y se combinan
    // las posiciones. Evita mezclar unidades de score incompatibles.
    let e_order = sorted_by_score(cands, |c| engagement_score(c, now, p));
    let w_order = sorted_by_score(cands, |c| {
        let mut s = wellbeing_score(c, now, p);
        if !c.is_followed {
            s -= NOT_FOLLOWED_PENALTY;
        }
        s
    });
    let position = |order: &[&Candidate]| -> Vec<(u64, i64)> {
        order
            .iter()
            .enumerate()
            .map(|(i, c)| (c.post_id, i as i64))
            .collect()
    };
    let pe = position(&e_order);
    let pw = position(&w_order);
    let pos_of = |table: &[(u64, i64)], id: u64| -> i64 {
        table
            .iter()
            .find(|(pid, _)| *pid == id)
            .map(|(_, p)| *p)
            .unwrap_or(i64::MAX / 200)
    };
    let we = 100 - p.mix_wellbeing_pct;
    let combined: Vec<Candidate> = sorted_by_score(cands, |c| {
        -(p.mix_wellbeing_pct * pos_of(&pw, c.post_id) + we * pos_of(&pe, c.post_id))
    })
    .into_iter()
    .cloned()
    .collect();
    apply_author_cap(combined, p)
}

/// Punto de entrada único: rankea según el algoritmo elegido en la config.
/// Esta función es la que ejecuta tanto el servidor como el guest zkVM.
pub fn rank(input: &FeedInput) -> Vec<u64> {
    let feed = match input.config.algorithm_id {
        ALG_WELLBEING => rank_wellbeing(&input.candidates, input.now, &input.params),
        ALG_MIXED => rank_mixed(&input.candidates, input.now, &input.params),
        _ => rank_engagement(&input.candidates, input.now, &input.params),
    };
    feed.into_iter().map(|c| c.post_id).collect()
}

/// Construye el journal (la parte pública de la prueba) para un input y su
/// feed resultante.
pub fn make_journal(input: &FeedInput, feed_ids: &[u64]) -> Journal {
    Journal {
        algorithm_id: input.config.algorithm_id,
        config_hash: hash_config(&input.config),
        params_hash: hash_params(&input.params),
        candidates_hash: hash_candidates(&input.candidates),
        feed_hash: hash_feed(feed_ids),
        timestamp: input.now,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_756_000_000;

    fn cand(
        post_id: u64,
        author_id: u64,
        mins_ago: u64,
        likes: u32,
        comments: u32,
        length: u32,
        followed: bool,
    ) -> Candidate {
        Candidate {
            post_id,
            author_id,
            created_at: NOW - mins_ago * 60,
            likes,
            comments,
            length_chars: length,
            is_followed: followed,
        }
    }

    /// Dataset sintético determinista con mezcla de virales, largos y recientes.
    fn dataset(n: u64) -> Vec<Candidate> {
        let mut v = Vec::new();
        for i in 0..n {
            // Generador lineal congruente para variedad reproducible.
            let r = (i.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(144_115_188)) >> 33;
            v.push(cand(
                i + 1,
                (i % 7) + 1,
                (r % 2_880) + 1,             // hasta 48 h de antigüedad
                (r % 900) as u32,            // likes
                (r % 90) as u32,             // comments
                80 + ((r % 11) * 60) as u32, // 80..680 chars
                i % 3 != 0,                  // 2/3 seguidos
            ));
        }
        v
    }

    fn input(alg: u8, cands: Vec<Candidate>) -> FeedInput {
        FeedInput {
            config: UserConfig {
                user_id: 42,
                algorithm_id: alg,
                nonce: 7,
            },
            params: Params::default(),
            candidates: cands,
            now: NOW,
        }
    }

    #[test]
    fn deterministic_same_input_same_output() {
        for alg in [ALG_ENGAGEMENT, ALG_WELLBEING, ALG_MIXED] {
            let i = input(alg, dataset(120));
            assert_eq!(rank(&i), rank(&i), "algoritmo {alg} no es determinista");
        }
    }

    #[test]
    fn wellbeing_only_followed_accounts() {
        let i = input(ALG_WELLBEING, dataset(120));
        let not_followed: Vec<u64> = i
            .candidates
            .iter()
            .filter(|c| !c.is_followed)
            .map(|c| c.post_id)
            .collect();
        for id in rank(&i) {
            assert!(!not_followed.contains(&id), "post no seguido {id} en feed bienestar");
        }
    }

    #[test]
    fn wellbeing_is_mostly_chronological() {
        // Posts cortos de autores distintos: el orden debe ser cronológico puro.
        let cands = vec![
            cand(1, 1, 300, 999, 99, 100, true),
            cand(2, 2, 10, 0, 0, 100, true),
            cand(3, 3, 100, 500, 50, 100, true),
        ];
        assert_eq!(rank(&input(ALG_WELLBEING, cands)), vec![2, 3, 1]);
    }

    #[test]
    fn wellbeing_long_post_jumps_ahead() {
        // El post largo (>=300 chars) de hace 20 min puede saltar por encima
        // del corto de hace 5 min (bonus de 30 min).
        let cands = vec![
            cand(1, 1, 5, 0, 0, 100, true),
            cand(2, 2, 20, 0, 0, 500, true),
        ];
        assert_eq!(rank(&input(ALG_WELLBEING, cands))[0], 2);
    }

    #[test]
    fn wellbeing_caps_consecutive_author() {
        let mut cands: Vec<Candidate> = (0..6).map(|i| cand(i + 1, 1, i + 1, 0, 0, 100, true)).collect();
        cands.push(cand(7, 2, 60, 0, 0, 100, true));
        let feed = rank(&input(ALG_WELLBEING, cands));
        // Con cap = 2, el autor 2 debe aparecer en la posición 3 (índice 2).
        assert_eq!(feed[2], 7, "el cap de autor consecutivo no se aplicó: {feed:?}");
    }

    #[test]
    fn engagement_prefers_viral_recent() {
        let cands = vec![
            cand(1, 1, 2_000, 2, 0, 400, true), // viejo, largo, sin likes
            cand(2, 2, 5, 800, 120, 60, true),  // reciente, viral, corto
        ];
        assert_eq!(rank(&input(ALG_ENGAGEMENT, cands))[0], 2);
    }

    #[test]
    fn engagement_injects_novelty_slot() {
        // 10 posts seguidos con mucho score + 1 no seguido mediocre:
        // el no seguido debe aparecer en la posición novelty_interval (3ª).
        let mut cands: Vec<Candidate> =
            (0..10).map(|i| cand(i + 1, i + 1, 5 + i, 500, 50, 60, true)).collect();
        cands.push(cand(99, 42, 2_000, 5, 0, 600, false));
        let feed = rank(&input(ALG_ENGAGEMENT, cands));
        assert_eq!(feed[2], 99, "la novedad no se inyectó en el slot 3: {feed:?}");
    }

    #[test]
    fn wellbeing_buries_short_bait_from_followed() {
        // Un gancho viral reciente de una cuenta seguida no le gana a un
        // texto largo un poco más viejo: bienestar no usa likes.
        let cands = vec![
            cand(1, 1, 8, 9_000, 400, 70, true),
            cand(2, 2, 90, 12, 2, 420, true),
        ];
        assert_eq!(rank(&input(ALG_WELLBEING, cands))[0], 2);
    }

    #[test]
    fn demo_shape_splits_feeds() {
        // Forma de la demo: cuentas seguidas publican largo y despacio;
        // las no seguidas, carnada reciente con miles de likes.
        let mut cands = Vec::new();
        for i in 0..8 {
            cands.push(cand(i + 1, 1, 200 + i * 30, 20, 4, 420, true));
        }
        for i in 0..12 {
            cands.push(cand(100 + i, 50, 3 + i, 4_000 + i as u32 * 100, 300, 55, false));
        }
        let e = rank(&input(ALG_ENGAGEMENT, cands.clone()));
        let w = rank(&input(ALG_WELLBEING, cands));
        assert!(
            e.iter().take(8).all(|id| *id >= 100),
            "engagement debería abrir con carnada: {e:?}"
        );
        assert!(
            w.iter().all(|id| *id < 100),
            "bienestar no debería mostrar cuentas no seguidas: {w:?}"
        );
        assert_ne!(e, w);
    }

    #[test]
    fn algorithms_produce_different_feeds() {
        let data = dataset(120);
        let e = rank(&input(ALG_ENGAGEMENT, data.clone()));
        let w = rank(&input(ALG_WELLBEING, data.clone()));
        let m = rank(&input(ALG_MIXED, data));
        assert_ne!(e, w);
        assert_ne!(e, m);
        assert_ne!(w, m);
    }

    #[test]
    fn feed_len_is_respected() {
        for alg in [ALG_ENGAGEMENT, ALG_WELLBEING, ALG_MIXED] {
            let i = input(alg, dataset(200));
            assert!(rank(&i).len() as u64 <= i.params.feed_len);
        }
    }

    #[test]
    fn hashes_are_stable_and_sensitive() {
        let i1 = input(ALG_WELLBEING, dataset(50));
        let mut i2 = i1.clone();
        assert_eq!(hash_candidates(&i1.candidates), hash_candidates(&i2.candidates));
        i2.candidates[0].likes += 1;
        assert_ne!(hash_candidates(&i1.candidates), hash_candidates(&i2.candidates));

        let c1 = UserConfig { user_id: 1, algorithm_id: 2, nonce: 3 };
        let c2 = UserConfig { algorithm_id: 3, ..c1.clone() };
        assert_ne!(hash_config(&c1), hash_config(&c2));
    }

    #[test]
    fn journal_commits_to_feed_and_choice() {
        let i = input(ALG_MIXED, dataset(80));
        let feed = rank(&i);
        let j = make_journal(&i, &feed);
        assert_eq!(j.algorithm_id, ALG_MIXED);
        assert_eq!(j.feed_hash, hash_feed(&feed));
        assert_eq!(j.candidates_hash, hash_candidates(&i.candidates));
        assert_eq!(j.timestamp, NOW);
    }
}
