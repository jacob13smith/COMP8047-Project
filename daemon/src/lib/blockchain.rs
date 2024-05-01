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
use crate::database::{fetch_all_blocks, fetch_chains, fetch_last_block, fetch_record, get_key_pair, get_shared_key, insert_block, insert_chain, insert_shared_key};
use crate::network::P2PRequest;

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
    pub sender: String,
    pub action: String,
    pub parameters: Map<String, Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainResponse {
    pub ok: bool,
    pub data: Value,
}

pub async fn initialize_blockchain_thread(mut receiver: Receiver<String>, sender_to_socket: Sender<String>, sender_to_p2p: Sender<String>){
    // Receive messages from the socket thread
    loop {
        if let Some(msg) = receiver.recv().await {
            let blockchain_request: BlockchainRequest = from_str(&msg).unwrap();
            if blockchain_request.sender == "socket" {
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
                    },
                    "get_record" => {
                        let response = get_record(blockchain_request.parameters.get("id").unwrap().as_str().unwrap().to_string(), blockchain_request.parameters.get("block_id").unwrap().as_i64().unwrap()).await;
                        sender_to_socket.send(to_string(&response).unwrap()).await.unwrap();
                    }
                    "add_provider" => {
                        let response = add_provider(blockchain_request.parameters, &sender_to_p2p).await;
                        sender_to_socket.send(to_string(&response).unwrap()).await.unwrap();
                    },
                    "add_record" => {
                        let response = add_record(blockchain_request.parameters, &sender_to_p2p).await;
                        sender_to_socket.send(to_string(&response).unwrap()).await.unwrap();
                    }
                    "remove_provider" => {
                        let response = remove_provider(blockchain_request.parameters, &sender_to_p2p).await;
                        sender_to_socket.send(to_string(&response).unwrap()).await.unwrap();
                    }
                    _ => {}
                }
            } else if blockchain_request.sender == "p2p" {
                // TODO: Add p2p request/responses
                match blockchain_request.action.as_str() {
                    _ => {}
                }
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

            // For now, records are of shape: date, subject, record_id
            let mut records:Vec<(Value, Value, Value)> = vec![];

            // For now, providers are of shape: name, ip_address
            let mut providers: Vec<(Value, Value)> = vec![];
            
            for (timestamp, block_id, encrypted_data) in blocks {
                let block_data = decrypt_data(&encrypted_data, shared_key);
                match block_data.action.as_str() {
                    "genesis" => {
                        data.insert("date_of_birth".to_string(), block_data.fields.get("date_of_birth").unwrap().clone());
                    },
                    "add-provider" => {
                        providers.push((block_data.fields.get("name").unwrap().clone(), block_data.fields.get("ip").unwrap().clone()));
                    }
                    "add-record" => {
                        records.push((to_value(timestamp).unwrap(), block_data.fields.get("subject").unwrap().clone(), to_value(block_id).unwrap()));
                    }
                    "remove-provider" => {
                        providers.retain(|(_, ip)| ip.as_str().unwrap().to_string() != block_data.fields.get("ip").unwrap().as_str().unwrap().to_string())
                    }
                    _ => {}
                }
            }

            data.insert("providers".to_string(), to_value(providers).unwrap());
            data.insert("records".to_string(), to_value(records).unwrap());

            let patient_blocks_string = to_value(&data).unwrap();
            BlockchainResponse{ok: true, data: patient_blocks_string}
        },
        Err(_) => {BlockchainResponse{ok: false, data: Value::Null}}
    }
}

pub async fn get_record(chain_id: String, block_id: i64) -> BlockchainResponse {
    let shared_key_vec = get_shared_key(chain_id.clone()).unwrap();
    let shared_key = shared_key_vec.as_slice();

    match fetch_record(chain_id, block_id) {
        Ok(record) => {
            let mut block_data = decrypt_data(&record.1, shared_key);
            block_data.fields.insert("timestamp".to_string(), to_value(record.0).unwrap());
            return BlockchainResponse{ok: true, data: to_value(block_data.fields).unwrap()}
        },
        Err(_) => {return BlockchainResponse{ok: false, data: Value::Null};}
    }
}

pub async fn add_record(parameters: Map<String, Value>, sender_to_p2p: &Sender<String>) -> BlockchainResponse {
    let chain_id = parameters.get("chain_id").unwrap().as_str().unwrap().to_string();
    let shared_key_vec = get_shared_key(chain_id.clone()).unwrap();
    let shared_key = shared_key_vec.as_slice();
    let my_key = get_key_pair().unwrap().expect("Expected KeyPair");

    let data = BlockData{action:"add-record".to_string(), fields: parameters.clone()};
    let encrypted_data = encrypt_data(&data, &shared_key);
    let hashed_data = hash_data(&data);

    let last_block = get_last_block(chain_id.clone());
    let mut add_record_block = Block{
        chain_id: chain_id,
        id: last_block.id + 1,
        timestamp: Utc::now().timestamp(), 
        data: encrypted_data,
        previous_hash: last_block.hash,
        hash: "".to_string(),
        provider_key: my_key.public_key,
        data_hash: hashed_data
    };

    let hash = hash_block(&add_record_block);
    add_record_block.hash = hash;

    let _ = insert_block(&add_record_block);
    
    let _ = sender_to_p2p.send(to_string(&P2PRequest{action: "add-record".to_string(), parameters}).unwrap()).await;

    BlockchainResponse{ok: true, data: Value::Null}
}

pub async fn add_provider(mut parameters: Map<String, Value>, sender_to_p2p: &Sender<String>) -> BlockchainResponse {
    let chain_id = parameters.get("chain_id").unwrap().as_str().unwrap().to_string();

    let shared_key_vec = get_shared_key(chain_id.clone()).unwrap();
    let shared_key = shared_key_vec.as_slice();
    let my_key = get_key_pair().unwrap().expect("Expected KeyPair");

    let data = BlockData{action:"add-provider".to_string(), fields: parameters.clone()};
    let encrypted_data = encrypt_data(&data, &shared_key);
    let hashed_data = hash_data(&data);

    let last_block = get_last_block(chain_id.clone());
    let mut add_provider_block = Block{
        chain_id: chain_id,
        id: last_block.id + 1,
        timestamp: Utc::now().timestamp(), 
        data: encrypted_data,
        previous_hash: last_block.hash,
        hash: "".to_string(),
        provider_key: my_key.public_key,
        data_hash: hashed_data
    };

    let hash = hash_block(&add_provider_block);
    add_provider_block.hash = hash;

    let _ = insert_block(&add_provider_block);
    parameters.insert("shared_key".to_string(), from_str(format!("\"{}\"", shared_key_vec.to_hex().as_str()).as_str()).unwrap());
    let _ = sender_to_p2p.send(to_string(&P2PRequest{action: "add-provider".to_string(), parameters}).unwrap()).await;

    BlockchainResponse{ok: true, data: Value::Null}
}

// TODO: Fill this in
pub async fn remove_provider(parameters: Map<String, Value>, sender_to_p2p: &Sender<String>) -> BlockchainResponse {
    let chain_id = parameters.get("chain_id").unwrap().as_str().unwrap().to_string();

    let shared_key_vec = get_shared_key(chain_id.clone()).unwrap();
    let shared_key = shared_key_vec.as_slice();
    let my_key = get_key_pair().unwrap().expect("Expected KeyPair");

    let data = BlockData{action:"remove-provider".to_string(), fields: parameters.clone()};
    let encrypted_data = encrypt_data(&data, &shared_key);
    let hashed_data = hash_data(&data);

    let last_block = get_last_block(chain_id.clone());
    let mut remove_provider_block = Block{
        chain_id: chain_id,
        id: last_block.id + 1,
        timestamp: Utc::now().timestamp(), 
        data: encrypted_data,
        previous_hash: last_block.hash,
        hash: "".to_string(),
        provider_key: my_key.public_key,
        data_hash: hashed_data
    };

    let hash = hash_block(&remove_provider_block);
    remove_provider_block.hash = hash;

    let _ = insert_block(&remove_provider_block);
    let _ = sender_to_p2p.send(to_string(&P2PRequest{action: "add-provider".to_string(), parameters}).unwrap()).await;

    BlockchainResponse{
        ok: true,
        data: Value::Null,
    }
}

pub fn create_chain(parameters: Map<String, Value>) -> BlockchainResponse {
    // Generate a new symmetric key for encryption
    let shared_key = generate_shared_key();
    let my_key = get_key_pair().unwrap().expect("Expected KeyPair");

    let first_name = parameters.get("first_name").unwrap().as_str().unwrap().to_string();
    let last_name = parameters.get("last_name").unwrap().as_str().unwrap().to_string();
    let date_of_birth = parameters.get("date_of_birth").unwrap().as_str().unwrap().to_string();

    // Generate global id for new chain
    let id = Uuid::new_v4().to_string();
    let data = BlockData{ action: "genesis".to_string(), fields: parameters };
    let encrypted_data = encrypt_data(&data, &shared_key);
    let hashed_data = hash_data(&data);

    let mut genesis_block = Block{ 
        chain_id: id.clone(), 
        id: 0, 
        timestamp: Utc::now().timestamp(), 
        data: encrypted_data,
        previous_hash: 0.to_string(), 
        hash: "".to_string(), 
        provider_key: my_key.public_key,
        data_hash: hashed_data
    };

    let hash = hash_block(&genesis_block);
    genesis_block.hash = hash;

    let new_chain = Chain { first_name: first_name, last_name: last_name, date_of_birth: date_of_birth, id: id.clone() };
    let _ = insert_chain(&new_chain);
    let _ = insert_block(&genesis_block);
    let _ = insert_shared_key(&shared_key, id);
    
    BlockchainResponse{ok: true, data: Value::Null}
}

pub fn get_last_block(chain_id: String) -> Block {
    match fetch_last_block(chain_id) {
        Ok(block) => { return block },
        Err(_) => { panic!("Expected a block for this chain") }
    }
}

fn generate_shared_key() -> [u8; 32] {
    let mut key: [u8; 32] = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

fn hash_block(block: &Block) -> String {
    let mut block_clone = block.clone();
    block_clone.hash = "".to_string();
    let serialized = format!(
        "{}{}{}{}{}{}{}",
        block.chain_id,
        block.id,
        block.timestamp,
        block.data,
        block.previous_hash,
        block.provider_key,
        block.data_hash
    );

    // Compute the SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    let result = hasher.finish();

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
    let iv = [0; 16];
    let ciphertext = encrypt(cipher, key, Some(&iv), to_string(data).unwrap().as_bytes()).unwrap();
    let hex_encoded_ciphertext = ciphertext.to_hex();
    hex_encoded_ciphertext
}

fn decrypt_data(encrypted_data: &str, key: &[u8]) -> BlockData {
    let cipher = Cipher::aes_256_cbc();
    let iv = [0; 16];
    let ciphertext = encrypted_data.from_hex().unwrap();
    let decrypted_data = decrypt(cipher, key, Some(&iv), &ciphertext).expect("Decryption error");
    let decrypted_string = String::from_utf8(decrypted_data).expect("UTF-8 decoding error");

    // Parse the JSON string into BlockData struct
    from_str(&decrypted_string).expect("JSON deserialization error")
}