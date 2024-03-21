use tokio::sync::mpsc::{Receiver, Sender};
use chrono::Utc;
use openssl::sha::Sha256;
use openssl::symm::{Cipher, encrypt};
use rusqlite::Result;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use rustc_serialize::hex::ToHex;
use uuid::Uuid;
use crate::database::{get_key_pair, insert_block, insert_chain, insert_shared_key, fetch_chains};

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

#[derive(Serialize, Deserialize)]
pub struct CreateChainParams {
    pub chain_name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainRequest {
    pub action: String,
    pub parameters: serde_json::Map<String, serde_json::Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainResponse {
    pub ok: bool,
    pub data: serde_json::Value,
}

pub async fn initialize_blockchain_thread(mut receiver_from_socket: Receiver<String>, sender_to_socket: Sender<String>){
    // Receive messages from the socket thread
    loop {
        if let Some(msg) = receiver_from_socket.recv().await {
            let blockchain_request: BlockchainRequest = serde_json::from_str(&msg).unwrap();
            match blockchain_request.action.as_str() {
                "get_chains" => {
                    let response = get_chains();
                    sender_to_socket.send(serde_json::to_string(&response).unwrap()).await.unwrap();
                },
                "create_chain" => {
                    // Need to get parameters from socket message here and use them
                    let name = &blockchain_request.parameters["name"];
                    let response = create_chain(name.as_str().unwrap().to_string());
                    sender_to_socket.send(serde_json::to_string(&response).unwrap()).await.unwrap();
                },
                _ => {}
            }
        }
    }
}

pub fn get_chains() -> BlockchainResponse {
    match fetch_chains() {
        Ok(chains) => {
            let chains_json_string = serde_json::to_value(&chains).unwrap();
            BlockchainResponse{ok: true, data: chains_json_string}
        },
        Err(_) => {BlockchainResponse{ok: false, data: serde_json::Value::Null}}
    }
}

pub fn create_chain(name: String) -> BlockchainResponse {
    // Generate a new symmetric key for encryption
    let shared_key = generate_shared_key();
    let shared_key_hash = hash_shared_key(&shared_key);
    let my_key = get_key_pair().unwrap().expect("Expected KeyPair");

    // Generate global id for new chain
    let id = Uuid::new_v4().to_string();

    // TODO: figure out data formats for different types of transactions
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
    
    BlockchainResponse{ok: true, data: serde_json::Value::Null}
}

// TODO: Grant access for a given chain to a given IP address
pub fn grant_access(chain_id: String, remote_ip: String) -> Result<()> {
    
    Ok(())
}

// TODO: Revoke access for a given chain from given IP address
pub fn revoke_access(chain_id: String, remote_ip: String) -> Result<()> {
    
    Ok(())
}

// TODO: Add block to chain and propagate network
pub fn add_block(chain_id: String, ) -> Result<()> {

    Ok(())
}

fn generate_shared_key() -> [u8; 32] {
    let mut key: [u8; 32] = [0u8; 32];
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