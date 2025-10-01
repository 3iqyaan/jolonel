use rusqlite::Connection;

pub fn create_tables() -> rusqlite::Result<()> {
    let conn = Connection::open("jnel.sqlite")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;").expect("Unable to enable foreign keys");
    conn.execute_batch(include_str!("create_tables.sql"))?;
    Ok(())
}