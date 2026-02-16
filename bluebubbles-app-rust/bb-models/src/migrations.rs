//! Versioned database migrations.
//!
//! Migrations run sequentially from the current stored version to the latest.
//! Each migration is an idempotent SQL script.

use rusqlite::Connection;
use tracing::{info, warn};
use bb_core::error::{BbError, BbResult};
use bb_core::constants::DB_SCHEMA_VERSION;

/// Run all pending migrations on the database.
pub fn run_migrations(conn: &Connection) -> BbResult<()> {
    let current_version = get_schema_version(conn)?;

    if current_version >= DB_SCHEMA_VERSION {
        info!("database schema is up to date (version {current_version})");
        return Ok(());
    }

    info!("running migrations from version {current_version} to {DB_SCHEMA_VERSION}");

    // Run each migration in sequence
    for version in (current_version + 1)..=DB_SCHEMA_VERSION {
        run_migration(conn, version)?;
    }

    set_schema_version(conn, DB_SCHEMA_VERSION)?;
    info!("migrations complete, schema at version {DB_SCHEMA_VERSION}");
    Ok(())
}

/// Get the current schema version from the database.
fn get_schema_version(conn: &Connection) -> BbResult<i32> {
    // Check if the version table has any rows
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
        .map_err(|e| BbError::Database(e.to_string()))?;

    if count == 0 {
        // First run - set version to 0
        conn.execute("INSERT INTO schema_version (version) VALUES (0)", [])
            .map_err(|e| BbError::Database(e.to_string()))?;
        return Ok(0);
    }

    conn.query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
        row.get(0)
    })
    .map_err(|e| BbError::Database(e.to_string()))
}

/// Set the schema version in the database.
fn set_schema_version(conn: &Connection, version: i32) -> BbResult<()> {
    conn.execute("UPDATE schema_version SET version = ?1", [version])
        .map_err(|e| BbError::Database(e.to_string()))?;
    Ok(())
}

/// Run a specific migration version.
fn run_migration(conn: &Connection, version: i32) -> BbResult<()> {
    info!("applying migration version {version}");

    match version {
        1 => migration_v1(conn),
        _ => {
            warn!("unknown migration version {version}, skipping");
            Ok(())
        }
    }
}

/// Migration v1: Initial schema is created by schema::create_tables.
/// This migration seeds default themes.
fn migration_v1(conn: &Connection) -> BbResult<()> {
    // Seed default themes if none exist
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM themes", [], |row| row.get(0))
        .map_err(|e| BbError::Database(e.to_string()))?;

    if count == 0 {
        conn.execute(
            "INSERT INTO themes (name, theme_data) VALUES ('OLED Dark', ?1)",
            [DEFAULT_DARK_THEME],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO themes (name, theme_data) VALUES ('Bright White', ?1)",
            [DEFAULT_LIGHT_THEME],
        )
        .map_err(|e| BbError::Database(e.to_string()))?;

        info!("seeded default themes");
    }

    Ok(())
}

const DEFAULT_DARK_THEME: &str = r#"{"colorScheme":{"brightness":0,"primary":4278221567,"onPrimary":4294967295,"background":4278190080,"onBackground":4294967295,"surface":4278190080,"onSurface":4294967295},"textTheme":{"font":"Default"}}"#;

const DEFAULT_LIGHT_THEME: &str = r#"{"colorScheme":{"brightness":1,"primary":4278221567,"onPrimary":4294967295,"background":4294967295,"onBackground":4278190080,"surface":4294967295,"onSurface":4278190080},"textTheme":{"font":"Default"}}"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    #[test]
    fn test_migrations_on_fresh_db() {
        let conn = Connection::open_in_memory().unwrap();
        schema::create_tables(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, DB_SCHEMA_VERSION);
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        schema::create_tables(&conn).unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // Should be a no-op
    }

    #[test]
    fn test_default_themes_seeded() {
        let conn = Connection::open_in_memory().unwrap();
        schema::create_tables(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM themes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }
}
