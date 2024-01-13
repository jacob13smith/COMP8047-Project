use rusqlite::{Connection, Result, params};

use crate::blockchain::{Chain, Block};

const DB_STRING: &str = "ehr.sqlite";

pub fn insert_chain(chain: &Chain) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute("INSERT INTO chains (id, name) VALUES (?1, ?2)", params![chain.id, chain.name])?;
    Ok(())
}

pub fn fetch_chains() -> Result<Vec<Chain>, rusqlite::Error> {
    let conn = Connection::open(DB_STRING)?;
    let query = "SELECT id, name FROM chains";
    let mut stmt = conn.prepare(query)?;
    let chain_iter = stmt.query_map([], |row| {
        Ok(Chain {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    let chains: Result<Vec<Chain>, _> = chain_iter.collect();
    chains
}

pub fn get_next_chain_id() -> Result<i64> {
    let conn = Connection::open(DB_STRING)?;
    let query = format!(
        "SELECT COALESCE(MAX({}), -1) + 1 AS next_id FROM {}",
        "id", "chains"
    );

    // Execute the query and retrieve the result
    let next_id: i64 = conn.query_row(&query, [], |row| row.get(0))?;

    Ok(next_id)
}

pub fn insert_block(block: &Block) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute("INSERT INTO blocks (chain_id, id, timestamp, data, previous_hash, hash, provider_key, shared_key_hash, unencrypted_data_hash) 
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", 
                        params![block.chain_id, block.id, block.timestamp, block.data, block.previous_hash, block.hash, block.provider_key, block.shared_key_hash, block.unencrypted_data_hash])?;
    Ok(())
}

pub fn bootstrap() -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    create_tables(&conn)?;
    Ok(())
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chains (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
         )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS blocks (
            chain_id INTEGER,
            id INTEGER,
            timestamp INTEGER,
            data TEXT NOT NULL,
            previous_hash TEXT,
            hash TEXT,
            provider_key TEXT,
            shared_key_hash TEXT,
            unencrypted_data_hash TEXT,
            FOREIGN KEY (chain_id) REFERENCES chains(id),
            PRIMARY KEY (chain_id, id)
         )",
        [],
    )?;

    Ok(())
}