use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::{Cursor, Read, Write}, net::{TcpListener, TcpStream}, sync::Arc, time::Duration};
use openssl::pkey::PKey;
use rustls::{pki_types::{CertificateDer, PrivateKeyDer}, ServerConfig};
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
        let ssl_pkey = PKey::private_key_from_pkcs8(&rsa_pkey_bytes).unwrap();
        let pem = String::from_utf8(ssl_pkey.private_key_to_pem_pkcs8().unwrap()).unwrap();
        let mut cursor = Cursor::new(pem);
        
        let private_key = rustls_pemfile::private_key(&mut cursor).unwrap().unwrap();
        let private_key_clone = private_key.clone_key();
        
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert = cert.cert;
        let cert_der = cert.der().to_owned();
        let cert_clone = cert_der.clone();
        
        let sender_clone = sender_to_blockchain.clone();

        let blockchain_listener = tokio::spawn(async move {
            handle_request_from_blockchain(receiver_from_blockchain, sender_clone, private_key, cert_der).await;
        });

        let network_listener = tokio::spawn(async move {
            handle_request_from_network(sender_to_blockchain, private_key_clone, cert_clone).await;
        });
    }

}

async fn handle_request_from_network(mut sender_to_blockchain: Sender<String>, key: PrivateKeyDer<'static>, cert: CertificateDer<'static>){
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key);

    let listener = TcpListener::bind("0.0.0.0:8047").unwrap();

}

async fn handle_request_from_blockchain(mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>, key: PrivateKeyDer<'static>, cert: CertificateDer<'static>) {

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            println!("Recieved message from blockchain to network");
            let blockchain_request: P2PRequest = from_str(&msg).unwrap();

        }
    }
}