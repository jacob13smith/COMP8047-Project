use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::{Cursor, Read, Write}, net::{TcpListener, TcpStream}, sync::Arc, time::Duration};
use openssl::pkey::PKey;
use rustls::{pki_types::PrivateKeyDer, ClientConfig, ClientConnection, ServerConfig};
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
        let rsa_pkey_pem = key_pair.private_key.clone();
        let ssl_pkey = PKey::private_key_from_pkcs8(&rsa_pkey_pem).unwrap();
        let pem = String::from_utf8(ssl_pkey.private_key_to_pem_pkcs8().unwrap()).unwrap();
        let mut cursor = Cursor::new(pem);
        
        let private_key = rustls_pemfile::private_key(&mut cursor).unwrap().unwrap();
        let private_key_clone = private_key.clone_key();
        
        let sender_clone = sender_to_blockchain.clone();

        let blockchain_listener = tokio::spawn(async move {
            handle_request_from_blockchain(receiver_from_blockchain, sender_clone, private_key_clone).await;
        });

        let network_listener = tokio::spawn(async move {
            handle_request_from_network(sender_to_blockchain, private_key).await;
        });
    }

}

async fn handle_request_from_network(mut sender_to_blockchain: Sender<String>, private_key: PrivateKeyDer<'static>){
    
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![], private_key).unwrap();

    let listener = TcpListener::bind("0.0.0.0:8047").unwrap();

    loop {
        let (mut stream, _) = listener.accept().unwrap();
        let mut conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();
        conn.complete_io(&mut stream).unwrap();
    
        let mut buf = [0u8; 1024];
        let bytes_read = conn.reader().read(&mut buf).unwrap();
    
        let data = &buf[..bytes_read];
        println!("Received data: {:?}", data);
    }
}

async fn handle_request_from_blockchain(mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>, private_key: PrivateKeyDer<'static>) {

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            println!("Recieved message from blockchain to network");
            let blockchain_request: P2PRequest = from_str(&msg).unwrap();

            let root_store = rustls::RootCertStore::empty();
            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_client_auth_cert(vec![], private_key.clone_key())
                .unwrap();

            let server_name = "blockchain-ehr.com".try_into().unwrap();
            let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
            let mut sock = TcpStream::connect("192.168.2.128:443").unwrap();
            let mut tls = rustls::Stream::new(&mut conn, &mut sock);

            tls.write_all(b"Hi there!").unwrap();
        }
    }
}