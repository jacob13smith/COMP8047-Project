use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::Cursor, net::{TcpListener, TcpStream}, time::Duration};
use openssl::{pkey::{PKey, Private}, ssl::{SslAcceptor, SslConnector, SslFiletype, SslMethod, SslVerifyMode}};
use rustls_pemfile::pkcs8_private_keys;
use serde_json::{from_str, to_string, to_value, Map, Value};
use serde::{Deserialize, Serialize};
use tokio::{io::{self, BufReader}, select};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::{blockchain::BlockchainRequest, database::get_key_pair};

const DEFAULT_PORT: i32 = 24195;

#[derive(Debug, Serialize, Deserialize)]
pub struct P2PRequest {
    pub action: String,
    pub parameters: Map<String, Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2PResponse {
    pub ok: bool,
    pub data: Value,
}

pub async fn initialize_p2p_thread(mut receiver_from_blockchain: Receiver<String>, mut sender_to_blockchain: Sender<String>) {
    
    let keys = get_key_pair().unwrap();
    
    if let Some(key_pair) = keys {
        let rsa_pkey_bytes = key_pair.private_key.clone();
        let private_key = PKey::private_key_from_pkcs8(&rsa_pkey_bytes).unwrap();
        
        tokio::spawn(handle_request_from_blockchain(receiver_from_blockchain, sender_to_blockchain.clone(), private_key.clone()));

        // Build SSL Acceptor with saved RSA key and no cert verification
        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        acceptor.set_private_key(&private_key).unwrap();
        acceptor.set_verify(SslVerifyMode::NONE);
        let acceptor = acceptor.build();

        let listener = TcpListener::bind("127.0.0.1:8047").unwrap();

        println!("Waiting on peer-to-peer connections...");
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let mut ssl_stream = acceptor.accept(stream).unwrap();

            let mut buf = [0u8; 1024];
            let bytes_read = ssl_stream.ssl_read(&mut buf).unwrap();

            let data = &buf[..bytes_read];
            println!("Received data: {:?}", data);
        }
    }

}

async fn handle_request_from_blockchain(mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>, private_key: PKey<Private>) {
    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            let blockchain_request: BlockchainRequest = from_str(&msg).unwrap();
            
            let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
            connector.set_private_key(&private_key).unwrap(); // Set the private key
            let connector = connector.build();

                        // Connect to the server
            let stream = TcpStream::connect("192.168.2.128:8047").unwrap();
            let mut stream = connector.connect("localhost", stream).unwrap();

            stream.ssl_write(b"hi").unwrap();

            break;
        }
    }
}