use serde_json::{from_str, to_string, to_value, Map, Value};
use tokio::sync::mpsc::{Receiver, Sender};
use chrono::Utc;
use openssl::sha::Sha256;
use openssl::symm::{Cipher, encrypt, decrypt};
use rusqlite::Result;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use rustc_serialize::hex::{ToHex, FromHex};
use uuid::Uuid;
use crate::database::{fetch_all_blocks, fetch_chains, get_key_pair, get_shared_key, insert_block, insert_chain, insert_shared_key};

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
pub struct BlockData {
    pub action: String,
    pub fields: serde_json::Map<String, Value>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateChainParams {
    pub chain_name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainRequest {
    pub action: String,
    pub parameters: Map<String, Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainResponse {
    pub ok: bool,
    pub data: Value,
}

pub async fn initialize_blockchain_thread(mut receiver_from_socket: Receiver<String>, sender_to_socket: Sender<String>){
    // Receive messages from the socket thread
    loop {
        if let Some(msg) = receiver_from_socket.recv().await {
            let blockchain_request: BlockchainRequest = from_str(&msg).unwrap();
            match blockchain_request.action.as_str() {
                "get_chains" => {
                    let response = get_chains();
                    sender_to_socket.send(to_string(&response).unwrap()).await.unwrap();
                },
                "create_chain" => {
                    // Need to get parameters from socket message here and use them
                    let response = create_chain(blockchain_request.parameters);
                    sender_to_socket.send(to_string(&response).unwrap()).await.unwrap();
                },
                "get_patient_info" => {
                    let response = get_patient_info(blockchain_request.parameters.get("id").unwrap().as_str().unwrap().to_string());
                    sender_to_socket.send(to_string(&response).unwrap()).await.unwrap();
                }
                _ => {}
            }
        }
    }
}

pub fn get_chains() -> BlockchainResponse {
    match fetch_chains() {
        Ok(chains) => {
            let chains_json_string = to_value(&chains).unwrap();
            BlockchainResponse{ok: true, data: chains_json_string}
        },
        Err(_) => {BlockchainResponse{ok: false, data: Value::Null}}
    }
}

pub fn get_patient_info(id: String) -> BlockchainResponse {
    let shared_key_vec = get_shared_key(id.clone()).unwrap();
    let shared_key = shared_key_vec.as_slice();
    
    match fetch_all_blocks(id){
        Ok(blocks) => {
            let mut data: Map<String, Value> = Map::default();

            // For now, records are of shape: id, date, subject, provider_name
            let mut records:Vec<(i64, String, String, String)> = vec![];

            // For now, providers are of shape: id, name, ip_address
            let mut providers: Vec<(i64, String, String)> = vec![];

            // TODO: Process each block into either a record or provider, or adjust the providers given subsequent providers revoking
            for (_, encrypted_data) in blocks {
                let block_data = decrypt_data(&encrypted_data, shared_key);
                match block_data.action.as_str() {
                    "genesis" => {
                        data.insert("date_of_birth".to_string(), block_data.fields.get("date_of_birth").unwrap().clone());
                    },
                    _ => {}
                }
            }

            let patient_blocks_string = to_value(&data).unwrap();
            BlockchainResponse{ok: true, data: patient_blocks_string}
        },
        Err(_) => {BlockchainResponse{ok: false, data: Value::Null}}
    }
}

pub fn create_chain(parameters: Map<String, Value>) -> BlockchainResponse {
    // Generate a new symmetric key for encryption
    let shared_key = generate_shared_key();
    let shared_key_hash = hash_shared_key(&shared_key);
    let my_key = get_key_pair().unwrap().expect("Expected KeyPair");

    let first_name = parameters.get("first_name").unwrap().as_str().unwrap().to_string();
    let last_name = parameters.get("last_name").unwrap().as_str().unwrap().to_string();
    let date_of_birth = parameters.get("date_of_birth").unwrap().as_str().unwrap().to_string();

    // Generate global id for new chain
    let id = Uuid::new_v4().to_string();

    // TODO: figure out data formats for different types of transactions
    let data = BlockData{ action: "genesis".to_string(), fields: parameters };
    let encrypted_data = encrypt_data(&data, &shared_key);
    let hashed_data = hash_data(&data);

    // dev
    // let decrypted_data = decrypt_data(&encrypted_data, &shared_key);
    // println!("Decrypted data: {}", to_string(&decrypted_data).unwrap());

    let genesis_block = Block{ 
        chain_id: id.clone(), 
        id: 0, 
        timestamp: Utc::now().timestamp(), 
        data: encrypted_data,
        previous_hash: 0.to_string(), 
        hash: "0".to_string(), 
        provider_key: my_key.public_key,
        shared_key_hash: shared_key_hash,
        data_hash: hashed_data
    };

    let new_chain = Chain { first_name: first_name, last_name: last_name, date_of_birth: date_of_birth, id: id.clone() };
    let _ = insert_chain(&new_chain);
    let _ = insert_block(&genesis_block);
    let _ = insert_shared_key(&shared_key, id);
    
    BlockchainResponse{ok: true, data: Value::Null}
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

fn hash_data(data: &BlockData) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(to_string(data).unwrap().as_bytes());
    let result = sha256.finish();
    result.to_hex()
}

fn encrypt_data(data: &BlockData, key: &[u8]) -> String {
    let cipher = Cipher::aes_256_cbc();
    // TODO: Figure out the IV for each block
    let iv = [0; 16];
    let ciphertext = encrypt(cipher, key, Some(&iv), to_string(data).unwrap().as_bytes()).unwrap();
    let hex_encoded_ciphertext = ciphertext.to_hex();
    hex_encoded_ciphertext
}

fn decrypt_data(encrypted_data: &str, key: &[u8]) -> BlockData {
    let cipher = Cipher::aes_256_cbc();
    // TODO: Figure out the IV for each block
    let iv = [0; 16];
    let ciphertext = encrypted_data.from_hex().unwrap();
    let decrypted_data = decrypt(cipher, key, Some(&iv), &ciphertext).expect("Decryption error");
    let decrypted_string = String::from_utf8(decrypted_data).expect("UTF-8 decoding error");

    // Parse the JSON string into BlockData struct
    from_str(&decrypted_string).expect("JSON deserialization error")
}