use rusqlite::{Connection, Result};

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS blockchains (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
         )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS blocks (
            id INTEGER PRIMARY KEY,
            data TEXT NOT NULL
         )",
        [],
    )?;

    Ok(())
}

pub fn bootstrap() -> Result<()> {
    let conn = Connection::open("blockchain.db")?;

    create_tables(&conn)?;

    // Any other bootstrapping goes here (additional tables, etc)

    Ok(())
}