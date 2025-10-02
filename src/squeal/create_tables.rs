use rusqlite::{ffi::SQLITE_NULL, params, Connection};

pub fn create_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;").expect("Unable to enable foreign keys");
    conn.execute_batch(include_str!("create_tables.sql"))?;
    Ok(())
}