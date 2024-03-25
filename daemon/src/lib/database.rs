use openssl::rsa::Rsa;
use rusqlite::{params, Connection, Result};
use crate::blockchain::{Chain, Block};

const DB_STRING: &str = "ehr.sqlite";

#[derive(Debug)]
pub struct KeyPair {
    pub public_key: String,
    private_key: String,
}

pub fn insert_chain(chain: &Chain) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute("INSERT INTO chains (id, first_name, last_name, date_of_birth) VALUES (?1, ?2, ?3, ?4)", params![chain.id, chain.first_name, chain.last_name, chain.date_of_birth])?;
    Ok(())
}

pub fn fetch_chains() -> Result<Vec<Chain>, rusqlite::Error> {
    let conn = Connection::open(DB_STRING)?;
    let query = "SELECT id, first_name, last_name, date_of_birth FROM chains";
    let mut stmt = conn.prepare(query)?;
    let chain_iter = stmt.query_map([], |row| {
        Ok(Chain {
            id: row.get(0)?,
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            date_of_birth: row.get(2)?
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
    conn.execute("INSERT INTO blocks (chain_id, id, timestamp, data, previous_hash, hash, provider_key, shared_key_hash, data_hash) 
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", 
                        params![block.chain_id, block.id, block.timestamp, block.data, block.previous_hash, block.hash, block.provider_key, block.shared_key_hash, block.data_hash])?;
    Ok(())
}

pub fn insert_shared_key(shared_key: &[u8], chain_id: String) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute(
        "INSERT INTO shared_keys (chain_id, value, active) VALUES (?, ?, ?)",
        params![chain_id, &shared_key, true],
    )?;
    Ok(())
}

pub fn bootstrap() -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    // Create tables if they don't exist
    create_tables(&conn)?;

    // Check if public/private key pair exist
    let key_pair_res = get_key_pair();

    // If key_pair doesn't exist, generate one and save
    if key_pair_res.is_ok() && key_pair_res.unwrap().is_none() {
        let new_key_pair = generate_key_pair();
        
        let _ = insert_key_pair(&conn, new_key_pair);
    }
    Ok(())
}

pub fn get_key_pair() -> Result<Option<KeyPair>>{
    let conn = Connection::open(DB_STRING)?;
    let mut stmt = conn.prepare("SELECT public_key, private_key FROM user_key_pairs LIMIT 1")?;
    let mut rows = stmt.query([])?;
    // Check if the count is greater than 0
    if let Some(row) = rows.next()? {
        
        // Extract the key pair from the row
        let key_pair = KeyPair {
            public_key: row.get(0)?,
            private_key: row.get(1)?,
        };

        // Return the key pair wrapped in Some
        Ok(Some(key_pair))
    } else {
        // If no rows are found, return None
        Ok(None)
    }
}

fn generate_key_pair() -> KeyPair {
    let rsa = Rsa::generate(2048).unwrap();

    // Extract public and private keys as strings
    let public_key = rsa.public_key_to_pem().unwrap();
    let private_key = rsa.private_key_to_pem().unwrap();

    KeyPair {
        public_key: String::from_utf8(public_key).unwrap(),
        private_key: String::from_utf8(private_key).unwrap(),
    }
}

fn insert_key_pair(conn: &Connection, key_pair: KeyPair) -> Result<()>{
    conn.execute(
        "INSERT INTO user_key_pairs (public_key, private_key) VALUES (?, ?)",
        params![&key_pair.public_key, &key_pair.private_key],
    )?;
    Ok(())
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chains (
            id TEXT PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            date_of_birth TEXT NOT NULL
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
            data_hash TEXT,
            FOREIGN KEY (chain_id) REFERENCES chains(id),
            PRIMARY KEY (chain_id, id)
         )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS shared_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chain_id INTEGER,
            value BLOB,
            active INTEGER
         )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_key_pairs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_key TEXT,
            private_key TEXT
         )",
        [],
    )?;

    Ok(())
}