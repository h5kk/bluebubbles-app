//! Database initialization, connection pooling, and lifecycle management.
//!
//! Uses SQLite in WAL mode with r2d2 connection pooling.
//! Runs integrity checks on startup and applies versioned migrations.

use std::path::Path;
use std::sync::Arc;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use tracing::{info, warn, error};

use bb_core::error::{BbError, BbResult};
use bb_core::config::DatabaseConfig;

use crate::schema;
use crate::migrations;

/// Type alias for the SQLite connection pool.
pub type DbPool = Pool<SqliteConnectionManager>;

/// Database wrapper providing initialization, pooling, and lifecycle management.
#[derive(Clone)]
pub struct Database {
    pool: Arc<DbPool>,
}

impl Database {
    /// Initialize the database at the given path with the provided configuration.
    ///
    /// This:
    /// 1. Creates the database file and parent directories if needed
    /// 2. Enables WAL mode for concurrent read/write
    /// 3. Sets up connection pooling
    /// 4. Runs integrity checks if configured
    /// 5. Creates the schema tables
    /// 6. Runs pending migrations
    pub fn init(db_path: &Path, config: &DatabaseConfig) -> BbResult<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("initializing database at {}", db_path.display());

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .max_size(config.pool_size)
            .connection_customizer(Box::new(ConnectionCustomizer {
                wal_mode: config.wal_mode,
            }))
            .build(manager)
            .map_err(|e| BbError::Pool(e.to_string()))?;

        let db = Self {
            pool: Arc::new(pool),
        };

        // Run integrity check if configured
        if config.integrity_check_on_startup {
            db.run_integrity_check()?;
        }

        // Create schema and run migrations
        {
            let conn = db.conn()?;
            schema::create_tables(&conn)?;
            migrations::run_migrations(&conn)?;
        }

        info!("database initialized successfully");
        Ok(db)
    }

    /// Get a connection from the pool.
    pub fn conn(&self) -> BbResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().map_err(|e| BbError::Pool(e.to_string()))
    }

    /// Get a reference to the underlying pool.
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// Run a SQLite integrity check.
    pub fn run_integrity_check(&self) -> BbResult<()> {
        let conn = self.conn()?;
        let result: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| BbError::Database(e.to_string()))?;

        if result != "ok" {
            error!("database integrity check failed: {result}");
            return Err(BbError::IntegrityCheck(result));
        }

        info!("database integrity check passed");
        Ok(())
    }

    /// Execute a function within a database transaction.
    pub fn transaction<T, F>(&self, f: F) -> BbResult<T>
    where
        F: FnOnce(&Connection) -> BbResult<T>,
    {
        let mut conn = self.conn()?;
        let tx = conn
            .transaction()
            .map_err(|e| BbError::Database(e.to_string()))?;

        let result = f(&tx)?;

        tx.commit()
            .map_err(|e| BbError::Database(e.to_string()))?;

        Ok(result)
    }

    /// Get database statistics (row counts per table).
    pub fn stats(&self) -> BbResult<DatabaseStats> {
        let conn = self.conn()?;

        let count = |table: &str| -> BbResult<i64> {
            let sql = format!("SELECT COUNT(*) FROM {table}");
            conn.query_row(&sql, [], |row| row.get(0))
                .map_err(|e| BbError::Database(e.to_string()))
        };

        Ok(DatabaseStats {
            chats: count("chats").unwrap_or(0),
            messages: count("messages").unwrap_or(0),
            handles: count("handles").unwrap_or(0),
            attachments: count("attachments").unwrap_or(0),
            contacts: count("contacts").unwrap_or(0),
            themes: count("themes").unwrap_or(0),
            settings: count("settings").unwrap_or(0),
        })
    }

    /// Reset the database by dropping and recreating all tables.
    pub fn reset(&self) -> BbResult<()> {
        warn!("resetting database - all data will be lost");
        let conn = self.conn()?;
        schema::drop_tables(&conn)?;
        schema::create_tables(&conn)?;
        migrations::run_migrations(&conn)?;
        info!("database reset complete");
        Ok(())
    }
}

/// Database row count statistics.
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub chats: i64,
    pub messages: i64,
    pub handles: i64,
    pub attachments: i64,
    pub contacts: i64,
    pub themes: i64,
    pub settings: i64,
}

impl std::fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chats={}, messages={}, handles={}, attachments={}, contacts={}, themes={}, settings={}",
            self.chats, self.messages, self.handles, self.attachments, self.contacts,
            self.themes, self.settings
        )
    }
}

/// r2d2 connection customizer that applies PRAGMA settings.
#[derive(Debug)]
struct ConnectionCustomizer {
    wal_mode: bool,
}

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for ConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // Enable WAL mode for better concurrent performance
        if self.wal_mode {
            conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        }

        // Performance pragmas
        conn.execute_batch(
            "PRAGMA synchronous=NORMAL;
             PRAGMA temp_store=MEMORY;
             PRAGMA mmap_size=268435456;
             PRAGMA cache_size=-64000;
             PRAGMA busy_timeout=5000;
             PRAGMA foreign_keys=ON;",
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_db() -> (Database, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let config = DatabaseConfig::default();
        let db = Database::init(&path, &config).unwrap();
        (db, dir)
    }

    #[test]
    fn test_database_init() {
        let (db, _dir) = test_db();
        let stats = db.stats().unwrap();
        assert_eq!(stats.chats, 0);
    }

    #[test]
    fn test_integrity_check() {
        let (db, _dir) = test_db();
        assert!(db.run_integrity_check().is_ok());
    }

    #[test]
    fn test_transaction() {
        let (db, _dir) = test_db();
        let result = db.transaction(|conn| {
            conn.execute(
                "INSERT INTO handles (address, service, unique_address_service) VALUES (?1, ?2, ?3)",
                rusqlite::params!["test@test.com", "iMessage", "test@test.com/iMessage"],
            ).map_err(|e| BbError::Database(e.to_string()))?;
            Ok(42)
        });
        assert_eq!(result.unwrap(), 42);
    }
}
