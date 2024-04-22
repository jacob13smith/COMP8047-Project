use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::{Cursor, Read, Write}, net::{TcpListener, TcpStream}, sync::Arc, time::Duration};
use native_tls::{Identity, TlsAcceptor, TlsConnector};
use openssl::pkey::PKey;
use rcgen::Certificate;
use serde_json::{from_str, to_string, to_value, Map, Value};
use serde::{Deserialize, Serialize};
use tokio::{io::{self, BufReader}, select};
use tokio::sync::mpsc::{Receiver, Sender};
use rsa::{RsaPrivateKey};
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
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let identity = Identity::from_pkcs8(cert.cert.pem().as_bytes(), &rsa_pkey_bytes).unwrap();
        
        let identity_clone = identity.clone();
        
        let sender_clone = sender_to_blockchain.clone();

        let blockchain_listener = tokio::spawn(async move {
            handle_request_from_blockchain(receiver_from_blockchain, sender_clone, identity_clone).await;
        });

        let network_listener = tokio::spawn(async move {
            handle_request_from_network(sender_to_blockchain, identity).await;
        });
    }

}

async fn handle_request_from_network(mut sender_to_blockchain: Sender<String>, identity: Identity){
    
    let acceptor = TlsAcceptor::builder(identity).build().unwrap();
    let listener = TcpListener::bind("0.0.0.0:8047").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                let mut stream = acceptor.accept(stream).unwrap();
                
                let mut buf = [0u8; 1024];
                let bytes_read = stream.read(&mut buf).unwrap();
            
                let data = &buf[..bytes_read];
                println!("Received data: {:?}", data);
            }
            Err(e) => { /* connection failed */ }
        }

    }
}

async fn handle_request_from_blockchain(mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>, identity: Identity) {

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            println!("Recieved message from blockchain to network");
            let blockchain_request: P2PRequest = from_str(&msg).unwrap();

            let stream = TcpStream::connect("192.168.2.128:8047").unwrap();

            let connector = TlsConnector::builder()
                .identity(identity.clone())
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .disable_built_in_roots(true)
                .build().unwrap();

            let mut stream = connector.connect("localhost", stream).unwrap();

            stream.write_all(b"Hi there!").unwrap();
        }
    }
}