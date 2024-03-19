use libp2p::{SwarmBuilder, identity::{self, Keypair}, PeerId, floodsub::Topic};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

pub fn initialize_p2p() {
    // TODO: Figure out libp2p swarm initialization
}