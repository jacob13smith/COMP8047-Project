use libp2p::{SwarmBuilder, identity::{self, Keypair}, PeerId, floodsub::Topic};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

const DEFAULT_PORT: u32 = 8081;

pub static KEYS: Lazy<Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blocks"));

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

pub fn initialize_p2p() {
    
}