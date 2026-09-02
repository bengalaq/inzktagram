use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use feed_core::{Candidate, FeedInput, Params, UserConfig, ALG_ENGAGEMENT};
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{now_epoch, prover, AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(list_users))
        .route("/state/{user_id}", get(get_state))
        .route("/settings/{user_id}", put(update_settings))
        .route("/demo/malicious", put(set_malicious))
        .route("/posts", post(create_post))
        .route("/posts/{post_id}/like", post(like_post))
        .route("/feed/{user_id}", get(get_feed))
        .route("/proofs/{view_id}", get(proof_status))
        .route("/proofs/{view_id}/verify", post(verify_proof))
        .route("/proofs/{view_id}/receipt", get(download_receipt))
        .route("/audit/{user_id}", get(audit_dump))
}

// ---------------------------------------------------------------------------
// Manejo de errores
// ---------------------------------------------------------------------------

pub struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.into().to_string())
    }
}

fn not_found(what: &str) -> ApiError {
    ApiError(StatusCode::NOT_FOUND, format!("{what} no encontrado"))
}

fn bad_request(msg: &str) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, msg.to_string())
}

type ApiResult<T> = Result<T, ApiError>;

// ---------------------------------------------------------------------------
// Usuarios y configuración
// ---------------------------------------------------------------------------

async fn list_users(State(st): State<Arc<AppState>>) -> ApiResult<Json<Value>> {
    let db = st.db.lock().await;
    let mut stmt =
        db.prepare("SELECT id, username, display_name, avatar_color FROM users ORDER BY id")?;
    let users: Vec<Value> = stmt
        .query_map([], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "username": r.get::<_, String>(1)?,
                "display_name": r.get::<_, String>(2)?,
                "avatar_color": r.get::<_, String>(3)?,
            }))
        })?
        .collect::<Result<_, _>>()?;
    Ok(Json(json!(users)))
}

async fn get_state(
    State(st): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
) -> ApiResult<Json<Value>> {
    let db = st.db.lock().await;
    let user = db
        .query_row(
            "SELECT id, username, display_name, avatar_color FROM users WHERE id = ?1",
            params![user_id as i64],
            |r| {
                Ok(json!({
                    "id": r.get::<_, i64>(0)?,
                    "username": r.get::<_, String>(1)?,
                    "display_name": r.get::<_, String>(2)?,
                    "avatar_color": r.get::<_, String>(3)?,
                }))
            },
        )
        .map_err(|_| not_found("usuario"))?;
    let (algorithm_id, nonce): (i64, i64) = db.query_row(
        "SELECT algorithm_id, nonce FROM settings WHERE user_id = ?1",
        params![user_id as i64],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let malicious: i64 = db.query_row("SELECT malicious FROM demo WHERE id = 1", [], |r| r.get(0))?;
    Ok(Json(json!({
        "user": user,
        "algorithm_id": algorithm_id,
        "nonce": nonce,
        "malicious": malicious != 0,
        "image_id": prover::image_id_hex(),
    })))
}

#[derive(Deserialize)]
struct SettingsBody {
    algorithm_id: u8,
}

async fn update_settings(
    State(st): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
    Json(body): Json<SettingsBody>,
) -> ApiResult<Json<Value>> {
    if !(1..=3).contains(&body.algorithm_id) {
        return Err(bad_request("algorithm_id debe ser 1, 2 o 3"));
    }
    let db = st.db.lock().await;
    // El nonce se incrementa en cada cambio: el journal compromete la versión
    // exacta de la configuración usada.
    db.execute(
        "UPDATE settings SET algorithm_id = ?1, nonce = nonce + 1 WHERE user_id = ?2",
        params![body.algorithm_id as i64, user_id as i64],
    )?;
    let nonce: i64 = db.query_row(
        "SELECT nonce FROM settings WHERE user_id = ?1",
        params![user_id as i64],
        |r| r.get(0),
    )?;
    Ok(Json(json!({ "algorithm_id": body.algorithm_id, "nonce": nonce })))
}

#[derive(Deserialize)]
struct MaliciousBody {
    enabled: bool,
}

async fn set_malicious(
    State(st): State<Arc<AppState>>,
    Json(body): Json<MaliciousBody>,
) -> ApiResult<Json<Value>> {
    let db = st.db.lock().await;
    db.execute(
        "UPDATE demo SET malicious = ?1 WHERE id = 1",
        params![body.enabled as i64],
    )?;
    Ok(Json(json!({ "malicious": body.enabled })))
}

// ---------------------------------------------------------------------------
// Posts
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct NewPostBody {
    author_id: u64,
    content: String,
}

async fn create_post(
    State(st): State<Arc<AppState>>,
    Json(body): Json<NewPostBody>,
) -> ApiResult<Json<Value>> {
    let content = body.content.trim();
    if content.is_empty() || content.chars().count() > 2000 {
        return Err(bad_request("el post debe tener entre 1 y 2000 caracteres"));
    }
    let db = st.db.lock().await;
    db.execute(
        "INSERT INTO posts (author_id, content, created_at, likes, comments) VALUES (?1, ?2, ?3, 0, 0)",
        params![body.author_id as i64, content, now_epoch() as i64],
    )?;
    let id = db.last_insert_rowid();
    Ok(Json(json!({ "id": id })))
}

async fn like_post(
    State(st): State<Arc<AppState>>,
    Path(post_id): Path<u64>,
) -> ApiResult<Json<Value>> {
    let db = st.db.lock().await;
    db.execute(
        "UPDATE posts SET likes = likes + 1 WHERE id = ?1",
        params![post_id as i64],
    )?;
    let likes: i64 = db.query_row(
        "SELECT likes FROM posts WHERE id = ?1",
        params![post_id as i64],
        |r| r.get(0),
    )?;
    Ok(Json(json!({ "likes": likes })))
}

// ---------------------------------------------------------------------------
// Feed: cada vista crea un registro con el input exacto del cómputo,
// que el worker luego prueba en la zkVM.
// ---------------------------------------------------------------------------

async fn get_feed(
    State(st): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
) -> ApiResult<Json<Value>> {
    let now = now_epoch();
    let db = st.db.lock().await;

    let (algorithm_chosen, nonce): (i64, i64) = db
        .query_row(
            "SELECT algorithm_id, nonce FROM settings WHERE user_id = ?1",
            params![user_id as i64],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| not_found("usuario"))?;
    let malicious: i64 = db.query_row("SELECT malicious FROM demo WHERE id = 1", [], |r| r.get(0))?;
    let malicious = malicious != 0;

    // Conjunto de candidatos en orden canónico (post_id ascendente), para que
    // candidates_hash sea reproducible por cualquier auditor.
    let mut stmt = db.prepare(
        "SELECT p.id, p.author_id, p.created_at, p.likes, p.comments, LENGTH(p.content),
                EXISTS(SELECT 1 FROM follows f WHERE f.follower_id = ?1 AND f.followee_id = p.author_id)
         FROM posts p WHERE p.author_id != ?1 ORDER BY p.id",
    )?;
    let candidates: Vec<Candidate> = stmt
        .query_map(params![user_id as i64], |r| {
            Ok(Candidate {
                post_id: r.get::<_, i64>(0)? as u64,
                author_id: r.get::<_, i64>(1)? as u64,
                created_at: r.get::<_, i64>(2)? as u64,
                likes: r.get::<_, i64>(3)? as u32,
                comments: r.get::<_, i64>(4)? as u32,
                length_chars: r.get::<_, i64>(5)? as u32,
                is_followed: r.get::<_, i64>(6)? != 0,
            })
        })?
        .collect::<Result<_, _>>()?;
    drop(stmt);

    // Lo que el servidor AFIRMA usar (y lo que el guest probará): la elección
    // del usuario. En modo malicioso el servidor sirve otra cosa: la prueba
    // seguirá siendo válida pero su feed_hash no coincidirá con lo mostrado.
    let proof_input = FeedInput {
        config: UserConfig {
            user_id,
            algorithm_id: algorithm_chosen as u8,
            nonce: nonce as u64,
        },
        params: Params::default(),
        candidates,
        now,
    };
    feed_core::check_candidacy(&proof_input)
        .map_err(|e| bad_request(&format!("candidacy rule violated: {e}")))?;
    let algorithm_served = if malicious {
        ALG_ENGAGEMENT
    } else {
        algorithm_chosen as u8
    };
    let served_ids: Vec<u64> = if malicious {
        let mut lied = proof_input.clone();
        lied.config.algorithm_id = algorithm_served;
        feed_core::rank(&lied)
    } else {
        feed_core::rank(&proof_input)
    };

    db.execute(
        "INSERT INTO feed_views
            (user_id, created_at, algorithm_claimed, algorithm_served, input_json, feed_json, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')",
        params![
            user_id as i64,
            now as i64,
            algorithm_chosen,
            algorithm_served as i64,
            serde_json::to_string(&proof_input)?,
            serde_json::to_string(&served_ids)?,
        ],
    )?;
    let view_id = db.last_insert_rowid();

    // Posts del feed en el orden servido, con datos del autor.
    let mut post_stmt = db.prepare(
        "SELECT p.id, p.content, p.content_en, p.created_at, p.likes, p.comments,
                u.username, u.display_name, u.avatar_color,
                EXISTS(SELECT 1 FROM follows f WHERE f.follower_id = ?1 AND f.followee_id = p.author_id)
         FROM posts p JOIN users u ON u.id = p.author_id WHERE p.id = ?2",
    )?;
    let mut posts: Vec<Value> = Vec::new();
    for id in &served_ids {
        let post = post_stmt.query_row(params![user_id as i64, *id as i64], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "content": r.get::<_, String>(1)?,
                "content_en": r.get::<_, Option<String>>(2)?,
                "created_at": r.get::<_, i64>(3)?,
                "likes": r.get::<_, i64>(4)?,
                "comments": r.get::<_, i64>(5)?,
                "username": r.get::<_, String>(6)?,
                "display_name": r.get::<_, String>(7)?,
                "avatar_color": r.get::<_, String>(8)?,
                "is_followed": r.get::<_, i64>(9)? != 0,
            }))
        })?;
        posts.push(post);
    }

    Ok(Json(json!({
        "view_id": view_id,
        "algorithm_id": algorithm_chosen,
        "generated_at": now,
        "proof_status": "pending",
        "posts": posts,
    })))
}

// ---------------------------------------------------------------------------
// Pruebas ZK
// ---------------------------------------------------------------------------

async fn proof_status(
    State(st): State<Arc<AppState>>,
    Path(view_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let db = st.db.lock().await;
    let row = db
        .query_row(
            "SELECT status, algorithm_claimed, journal_json, proving_ms, user_cycles, error
             FROM feed_views WHERE id = ?1",
            params![view_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                ))
            },
        )
        .map_err(|_| not_found("feed view"))?;
    let journal: Option<Value> = match row.2 {
        Some(s) => Some(serde_json::from_str(&s)?),
        None => None,
    };
    Ok(Json(json!({
        "view_id": view_id,
        "status": row.0,
        "algorithm_claimed": row.1,
        "journal": journal,
        "proving_ms": row.3,
        "user_cycles": row.4,
        "error": row.5,
        "image_id": prover::image_id_hex(),
        "dev_mode": crate::prover_dev_mode(),
    })))
}

/// Verificación "de cortesía" del lado del servidor: útil para la UI, pero el
/// usuario no necesita confiar en ella. Puede descargar el receipt y correr
/// `verifier-cli` por su cuenta; además el cliente web recomputa el hash del
/// feed mostrado con WebCrypto y lo compara contra el journal.
async fn verify_proof(
    State(st): State<Arc<AppState>>,
    Path(view_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let db = st.db.lock().await;
    let (receipt_blob, feed_json, algorithm_claimed, algorithm_served): (
        Option<Vec<u8>>,
        String,
        i64,
        i64,
    ) = db
        .query_row(
            "SELECT receipt, feed_json, algorithm_claimed, algorithm_served FROM feed_views WHERE id = ?1",
            params![view_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|_| not_found("feed view"))?;
    drop(db);

    let Some(blob) = receipt_blob else {
        return Err(bad_request("la prueba todavía no está lista"));
    };
    let receipt: risc0_zkvm::Receipt = bincode::deserialize(&blob)?;

    let t = std::time::Instant::now();
    let verified = prover::verify_receipt(&receipt);
    let verify_ms = t.elapsed().as_millis() as u64;

    let served_ids: Vec<u64> = serde_json::from_str(&feed_json)?;
    let displayed_feed_hash = feed_core::hash_feed(&served_ids);

    match verified {
        Ok(journal) => {
            let algorithm_matches = journal.algorithm_id as i64 == algorithm_claimed;
            let feed_matches = journal.feed_hash == displayed_feed_hash;
            Ok(Json(json!({
                "proof_valid": true,
                "verify_ms": verify_ms,
                "image_id": prover::image_id_hex(),
                "journal": prover::journal_to_json(&journal),
                "algorithm_claimed": algorithm_claimed,
                "algorithm_served": algorithm_served,
                "displayed_feed_hash": hex::encode(displayed_feed_hash),
                "checks": {
                    "proof_valid": true,
                    "algorithm_matches": algorithm_matches,
                    "feed_matches": feed_matches,
                },
                "dev_mode": crate::prover_dev_mode(),
            })))
        }
        Err(e) => Ok(Json(json!({
            "proof_valid": false,
            "verify_ms": verify_ms,
            "image_id": prover::image_id_hex(),
            "error": e.to_string(),
            "algorithm_claimed": algorithm_claimed,
            "algorithm_served": algorithm_served,
            "displayed_feed_hash": hex::encode(displayed_feed_hash),
            "checks": {
                "proof_valid": false,
                "algorithm_matches": false,
                "feed_matches": false,
            },
            "dev_mode": crate::prover_dev_mode(),
        }))),
    }
}

async fn download_receipt(
    State(st): State<Arc<AppState>>,
    Path(view_id): Path<i64>,
) -> ApiResult<Response> {
    let db = st.db.lock().await;
    let blob: Option<Vec<u8>> = db
        .query_row(
            "SELECT receipt FROM feed_views WHERE id = ?1",
            params![view_id],
            |r| r.get(0),
        )
        .map_err(|_| not_found("feed view"))?;
    let Some(bytes) = blob else {
        return Err(bad_request("la prueba todavía no está lista"));
    };
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"inzktagram_view_{view_id}.receipt\""),
            ),
            (
                header::HeaderName::from_static("x-image-id"),
                prover::image_id_hex(),
            ),
        ],
        bytes,
    )
        .into_response())
}

/// Dump público para que un auditor reconstruya `candidates_hash` en local.
/// No es una fuente de verdad: el servidor podría omitir posts. El valor
/// está en el *método* (recomputar el hash y compararlo con el journal).
async fn audit_dump(
    State(st): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
) -> ApiResult<Json<Value>> {
    use feed_core::{assemble_candidates, hash_candidates, FollowEdge, PublicPost};

    let db = st.db.lock().await;
    let exists: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM users WHERE id = ?1",
            params![user_id as i64],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists == 0 {
        return Err(not_found("usuario"));
    }

    let mut post_stmt = db.prepare(
        "SELECT id, author_id, created_at, likes, comments, LENGTH(content) FROM posts ORDER BY id",
    )?;
    let posts: Vec<PublicPost> = post_stmt
        .query_map([], |r| {
            Ok(PublicPost {
                post_id: r.get::<_, i64>(0)? as u64,
                author_id: r.get::<_, i64>(1)? as u64,
                created_at: r.get::<_, i64>(2)? as u64,
                likes: r.get::<_, i64>(3)? as u32,
                comments: r.get::<_, i64>(4)? as u32,
                length_chars: r.get::<_, i64>(5)? as u32,
            })
        })?
        .collect::<Result<_, _>>()?;
    drop(post_stmt);

    let mut follow_stmt = db.prepare("SELECT follower_id, followee_id FROM follows")?;
    let follows: Vec<FollowEdge> = follow_stmt
        .query_map([], |r| {
            Ok(FollowEdge {
                follower_id: r.get::<_, i64>(0)? as u64,
                followee_id: r.get::<_, i64>(1)? as u64,
            })
        })?
        .collect::<Result<_, _>>()?;
    drop(follow_stmt);

    let assembled = assemble_candidates(user_id, &posts, &follows);

    Ok(Json(json!({
        "untrusted": true,
        "note": "Recompute candidates_hash locally and compare it to the receipt journal. This dump is a convenience, not a transparency log.",
        "user_id": user_id,
        "posts": posts,
        "follows": follows,
        "candidates_hash": hex::encode(hash_candidates(&assembled)),
        "n_candidates": assembled.len(),
    })))
}
