use anyhow::Result;
use rusqlite::Connection;

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            avatar_color TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS follows (
            follower_id INTEGER NOT NULL,
            followee_id INTEGER NOT NULL,
            PRIMARY KEY (follower_id, followee_id)
        );

        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            author_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            likes INTEGER NOT NULL DEFAULT 0,
            comments INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS settings (
            user_id INTEGER PRIMARY KEY,
            algorithm_id INTEGER NOT NULL DEFAULT 2,
            nonce INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS demo (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            malicious INTEGER NOT NULL DEFAULT 0
        );

        -- Cada vez que un usuario abre su feed se registra una "vista":
        -- el input exacto del cómputo (para el prover), el feed mostrado y,
        -- cuando el worker termina, el receipt de RISC Zero y su journal.
        CREATE TABLE IF NOT EXISTS feed_views (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            algorithm_claimed INTEGER NOT NULL,
            algorithm_served INTEGER NOT NULL,
            input_json TEXT NOT NULL,
            feed_json TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            receipt BLOB,
            journal_json TEXT,
            proving_ms INTEGER,
            user_cycles INTEGER,
            error TEXT
        );
        "#,
    )?;
    Ok(conn)
}
