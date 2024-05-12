use openssl::{pkey::PKey, rsa::Rsa};
use rusqlite::{params, Connection, Result};
use crate::blockchain::{Chain, Block};

const DB_STRING: &str = "ehr.sqlite";

#[derive(Debug)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key: Vec<u8>,
}

pub fn insert_chain(chain: &Chain) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute("INSERT INTO chains (id, first_name, last_name, date_of_birth, active) VALUES (?1, ?2, ?3, ?4, ?5)", params![chain.id, chain.first_name, chain.last_name, chain.date_of_birth, 1])?;
    Ok(())
}

pub fn fetch_chains() -> Result<Vec<Chain>, rusqlite::Error> {
    let conn = Connection::open(DB_STRING)?;
    let query = "SELECT id, first_name, last_name, date_of_birth FROM chains WHERE active = 1";
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

pub fn set_chain_inactive(chain_id: String) -> Result<()>{
    let conn = Connection::open(DB_STRING)?;
    conn.execute("UPDATE chains SET active = 0 WHERE id = ?", params![chain_id])?;
    Ok(())
}

pub fn fetch_all_transactions(id: String) -> Result<Vec<(i64, i64, String)>> {
    let conn = Connection::open(DB_STRING)?;

    let mut statement = conn.prepare("SELECT timestamp, id, data FROM blocks WHERE chain_id = ? ORDER BY timestamp ASC").unwrap();
    let blocks = statement.query_map(params![id], |row| {
        Ok((
            row.get::<usize, i64>(0)?,
            row.get::<usize, i64>(1)?,
            row.get::<usize, String>(2)?,
        ))
    })?;

    let mut result = Vec::new();

    for block in blocks {
        result.push(block?);
    }

    Ok(result)
}

pub fn fetch_all_blocks(id: String) -> Result<Vec<Block>> {
    let conn = Connection::open(DB_STRING)?;

    let mut statement = conn.prepare("SELECT chain_id, id, timestamp, data, previous_hash, hash, provider_key, data_hash FROM blocks WHERE chain_id = ? ORDER BY timestamp ASC").unwrap();
    let block_tuples = statement.query_map(params![id], |row| {
        Ok((
            row.get::<usize, String>(0)?,
            row.get::<usize, i64>(1)?,
            row.get::<usize, i64>(2)?,
            row.get::<usize, String>(3)?,
            row.get::<usize, String>(4)?,
            row.get::<usize, String>(5)?,
            row.get::<usize, String>(6)?,
            row.get::<usize, String>(7)?,
        ))
    })?;

    let mut result = Vec::new();

    for block_tuple_result in block_tuples {
        let block_tuple = block_tuple_result.unwrap();
        let block = Block{
            chain_id: block_tuple.0,
            id: block_tuple.1,
            timestamp: block_tuple.2,
            data: block_tuple.3,
            previous_hash: block_tuple.4,
            hash: block_tuple.5, 
            provider_key: block_tuple.6,
            data_hash: block_tuple.7
        };
        result.push(block);
    }

    Ok(result)
}

// Returns a tuple (timestamp, data)
pub fn fetch_record(chain_id: String, block_id: i64) -> Result<(i64, String)> {
    let conn = Connection::open(DB_STRING)?;

    let mut statement = conn.prepare("SELECT timestamp, data FROM blocks WHERE chain_id = ? AND id = ?").unwrap();

    let record = statement.query_row(params![chain_id, block_id], |row| {
        Ok((
            row.get::<usize, i64>(0)?,
            row.get::<usize, String>(1)?,
        ))
    })?;
    
    Ok(record)
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
    conn.execute("INSERT INTO blocks (chain_id, id, timestamp, data, previous_hash, hash, provider_key, data_hash) 
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", 
                        params![block.chain_id, block.id, block.timestamp, block.data, block.previous_hash, block.hash, block.provider_key, block.data_hash])?;
    Ok(())
}

pub fn update_block(block: &Block) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute("UPDATE blocks SET (data) = ? WHERE chain_id = ? and id = ?", 
                        params![block.data, block.chain_id, block.id])?;
    Ok(())
}

pub fn fetch_last_block(chain_id: String) -> Result<Block> {
    let conn = Connection::open(DB_STRING)?;

    let query = format!(
        "SELECT chain_id, id, timestamp, data, previous_hash, hash, provider_key, data_hash FROM blocks WHERE chain_id = ? AND id = (SELECT MAX(id) FROM blocks WHERE chain_id = ?)"
    );

    let mut statement = conn.prepare(&query)?;
    let mut rows = statement.query(params![chain_id, chain_id])?;

    if let Some(row) = rows.next()? {
        Ok(Block{
            chain_id: row.get(0)?,
            id: row.get(1)?,
            timestamp: row.get(2)?,
            data: row.get(3)?,
            previous_hash: row.get(4)?,
            hash: row.get(5)?,
            provider_key: row.get(6)?,
            data_hash: row.get(7)?,
        })
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows.into())
    }
}

pub fn insert_shared_key(shared_key: &[u8], chain_id: String) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute(
        "INSERT INTO shared_keys (chain_id, value, active) VALUES (?, ?, ?)",
        params![chain_id, &shared_key, true],
    )?;
    Ok(())
}

pub fn insert_new_shared_key(shared_key: &[u8], chain_id: String) -> Result<()> {
    let conn = Connection::open(DB_STRING)?;
    conn.execute("UPDATE shared_keys SET active = 0 WHERE chain_id = ?", params![chain_id])?;
    conn.execute(
        "INSERT INTO shared_keys (chain_id, value, active) VALUES (?, ?, ?)",
        params![chain_id, &shared_key, true],
    )?;
    Ok(())
}

pub fn get_shared_key(id: String) -> Result<Vec<u8>> {
    let conn = Connection::open(DB_STRING)?;

    let mut statement = conn.prepare("SELECT value FROM shared_keys WHERE chain_id = ? AND active = 1")?;
    let mut rows = statement.query(params![id])?;

    if let Some(row) = rows.next()? {
        let shared_key: Vec<u8> = row.get(0)?;
        Ok(shared_key)
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
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
        
        let public_key = row.get(0).unwrap();
        let private_key = row.get(1).unwrap();

        // Extract the key pair from the row
        let key_pair = KeyPair {
            public_key,
            private_key
        };

        // Return the key pair wrapped in Some
        Ok(Some(key_pair))
    } else {
        // If no rows are found, return None
        Ok(None)
    }
}

fn generate_key_pair() -> KeyPair {
    // Generate RSA key pair
    let rsa = Rsa::generate(2048).unwrap();

    // Extract public key as PEM string
    let public_key_pem = rsa.public_key_to_pem().unwrap();

    // Extract private key as PKCS#8 PEM string
    let private_key_der = {
        let pkey = PKey::from_rsa(rsa).unwrap();
        pkey.private_key_to_pkcs8().unwrap()
    };

    KeyPair {
        public_key: String::from_utf8(public_key_pem).unwrap(),
        private_key: private_key_der,
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
            date_of_birth TEXT NOT NULL,
            active INTEGER
         )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS blocks (
            chain_id TEXT,
            id INTEGER,
            timestamp INTEGER,
            data TEXT NOT NULL,
            previous_hash TEXT,
            hash TEXT,
            provider_key TEXT,
            data_hash TEXT,
            FOREIGN KEY (chain_id) REFERENCES chains(id),
            PRIMARY KEY (chain_id, id)
         )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS shared_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chain_id TEXT,
            value BLOB,
            active INTEGER
         )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_key_pairs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_key TEXT,
            private_key BLOB
         )",
        [],
    )?;

    Ok(())
}