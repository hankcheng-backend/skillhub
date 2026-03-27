use crate::error::AppError;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

pub mod models;

const MIGRATIONS: &[M<'static>] = &[
    // Migration 0: initial schema (existing DDL verbatim from v0.1.0)
    M::up(
        "CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            enabled INTEGER DEFAULT 0,
            skill_dir TEXT
        );

        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            folder_name TEXT NOT NULL,
            origin_agent TEXT NOT NULL,
            name TEXT,
            description TEXT,
            tags TEXT,
            notes TEXT,
            discovered_at INTEGER,
            updated_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS skill_syncs (
            skill_id TEXT NOT NULL,
            agent TEXT NOT NULL,
            symlink_path TEXT,
            PRIMARY KEY (skill_id, agent),
            FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE,
            FOREIGN KEY (agent) REFERENCES agents(id)
        );

        CREATE TABLE IF NOT EXISTS sources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            url TEXT,
            folder_id TEXT,
            keychain_key TEXT,
            refresh_token_key TEXT,
            added_at INTEGER
        );

        INSERT OR IGNORE INTO agents (id, enabled, skill_dir) VALUES
            ('claude', 0, NULL),
            ('codex', 0, NULL),
            ('gemini', 0, NULL);

        CREATE TABLE IF NOT EXISTS app_config (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO app_config (key, value) VALUES ('mcp_port', '9800');",
    ),
];

/// Returns (Connection, is_fresh) where is_fresh = true when DB was just created.
/// D-10: Backup existing DB before migration (before opening connection).
/// D-11: Restore from backup on failure; caller shows error dialog.
pub fn init_db(db_path: &Path) -> Result<(Connection, bool), AppError> {
    let is_fresh = !db_path.exists();

    // D-10: Backup existing DB before migration (before opening connection to avoid WAL issues)
    let bak_path = db_path.with_extension("db.bak");
    if !is_fresh {
        if let Err(e) = std::fs::copy(db_path, &bak_path) {
            log::warn!("Failed to backup DB before migration: {}", e);
        }
    }

    let mut conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;",
    )?;

    let migrations = Migrations::new(MIGRATIONS.to_vec());
    if let Err(e) = migrations.to_latest(&mut conn) {
        log::error!("Migration failed: {}", e);
        // D-11: Restore from backup on failure
        drop(conn);
        if bak_path.exists() {
            if let Err(restore_err) = std::fs::copy(&bak_path, db_path) {
                log::error!("Failed to restore DB from backup: {}", restore_err);
            }
        }
        // Re-open the pre-migration DB so the app can still run
        let fallback_conn = Connection::open(db_path)?;
        fallback_conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;",
        )?;
        return Err(AppError::Migration(format!(
            "Database migration failed: {}. The database has been restored from backup.",
            e
        )));
    }

    Ok((conn, is_fresh))
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use rusqlite::{Connection, OptionalExtension};

    fn test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;",
        )
        .unwrap();
        Migrations::new(MIGRATIONS.to_vec())
            .to_latest(&mut conn)
            .unwrap();
        conn
    }

    #[test]
    fn test_app_config_seed_mcp_port() {
        let conn = test_db();
        let val: String = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'mcp_port'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(val, "9800");
    }

    #[test]
    fn test_app_config_missing_key_returns_none() {
        let conn = test_db();
        let val: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'nonexistent'",
                [],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn test_app_config_migrate_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;",
        )
        .unwrap();
        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations.to_latest(&mut conn).unwrap();
        // Simulate user customizing the port
        conn.execute(
            "UPDATE app_config SET value = '9999' WHERE key = 'mcp_port'",
            [],
        )
        .unwrap();
        // Re-run migration (e.g. after app update) — library skips already-applied migrations
        migrations.to_latest(&mut conn).unwrap();
        // Custom value must be preserved
        let val: String = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'mcp_port'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(val, "9999");
    }

    #[test]
    fn migrations_validate() {
        assert!(Migrations::new(MIGRATIONS.to_vec()).validate().is_ok());
    }
}
