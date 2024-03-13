use chrono::Utc;
use openssl::sha::{self, Sha256};
use openssl::symm::{Cipher, encrypt};
use rusqlite::Result;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use rustc_serialize::hex::{self, ToHex};
use uuid::Uuid;
use crate::database::{get_key_pair, insert_block, insert_chain, insert_shared_key};


// Define the structure for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub chain_id: String,
    pub id: i64,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub provider_key: String,
    pub shared_key_hash: String,
    pub data_hash: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub id: String,
    pub name: String,
}


pub fn create_chain(name: String) -> Result<Chain> {
    // Generate a new symmetric key for encryption
    let shared_key = generate_shared_key();
    let shared_key_hash = hash_shared_key(&shared_key);
    let my_key = get_key_pair().unwrap().expect("Expected KeyPair");

    // Generate global id for new chain
    let id = Uuid::new_v4().to_string();

    let data = "genesis";

    let genesis_block = Block{ 
        chain_id: id.clone(), 
        id: 0, 
        timestamp: Utc::now().timestamp(), 
        data: encrypt_data(data, &shared_key), 
        previous_hash: 0.to_string(), 
        hash: hash_data(data.to_string()).to_string(), 
        provider_key: my_key.public_key,
        shared_key_hash: shared_key_hash,
        data_hash: hash_data(data.to_string())};

    let new_chain = Chain { name: name, id: id.clone() };
    let _ = insert_chain(&new_chain);
    let _ = insert_block(&genesis_block);
    let _ = insert_shared_key(&shared_key, id);
    Ok(new_chain)
}

fn generate_shared_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

fn hash_shared_key(key: &[u8]) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(key);
    let result = sha256.finish();
    result.to_hex()
}

fn hash_data(data: String) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(data.as_bytes());
    let result = sha256.finish();
    result.to_hex()
}

fn encrypt_data(data: &str, key: &[u8]) -> String {
    let cipher = Cipher::aes_256_cbc();
    let iv = [0; 16]; // Initialization vector (IV) for CBC mode, must be securely random in a real application
    let ciphertext = encrypt(cipher, key, Some(&iv), data.as_bytes()).unwrap();
    let hex_encoded_ciphertext = ciphertext.to_hex();
    hex_encoded_ciphertext
}