use chrono::Utc;
use rusqlite::Result;
use serde::{Deserialize, Serialize};

use crate::database::{insert_chain, insert_block, get_next_chain_id};

pub fn create_chain(name: String) -> Result<Chain> {
    let id = get_next_chain_id()?;
    let genesis_block = Block{ 
        chain_id: id, 
        id: 0, 
        timestamp: Utc::now().timestamp(), 
        data: String::from("genesis"), 
        previous_hash: String::from("genesis"), 
        hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(), 
        provider_key: String::from("0"),
        shared_key_hash: String::from("0"),
        unencrypted_data_hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string() };

    let new_chain = Chain { name: name, id: id };
    let _ = insert_chain(&new_chain);
    let _ = insert_block(&genesis_block);
    Ok(new_chain)
}

// Define the structure for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub chain_id: i64,
    pub id: i64,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub provider_key: String,
    pub shared_key_hash: String,
    pub unencrypted_data_hash: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub id: i64,
    pub name: String,
}

