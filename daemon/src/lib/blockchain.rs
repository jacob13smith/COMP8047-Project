// Define the structure for a block
#[derive(Debug, Clone)]
struct Block {
    blockchain_id: String,
    index: u64,
    timestamp: DateTime<Utc>,
    data: String,
    previous_hash: String,
    hash: String,
    provider_pub_key: String,
    shared_key_hash: String,
    unencrypt_checksum: u64,
    data: String
}

