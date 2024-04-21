use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::Cursor, net::TcpListener, time::Duration};
use openssl::{pkey::PKey, ssl::{SslAcceptor, SslFiletype, SslMethod, SslVerifyMode}};
use rustls_pemfile::pkcs8_private_keys;
use serde_json::{from_str, to_string, to_value, Map, Value};
use serde::{Deserialize, Serialize};
use tokio::{io::{self, BufReader}, select};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::database::get_key_pair;

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

        // Build SSL Acceptor with saved RSA key and no cert verification
        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        acceptor.set_private_key(&private_key).unwrap();
        acceptor.set_verify(SslVerifyMode::NONE);
        let acceptor = acceptor.build();

        let listener = TcpListener::bind("127.0.0.1:8047").unwrap();

        match listener.accept() {
            Ok((stream, addr)) => {
                println!("new client: {:?}", addr);
                acceptor.accept(stream).unwrap();
            },
            Err(e) => println!("couldn't get client: {:?}", e),
        }
    }

}

