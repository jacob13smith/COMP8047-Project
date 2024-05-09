use std::{io::{Cursor, Read, Write}, net::{TcpListener, TcpStream}, str::from_utf8, sync::Arc};
use openssl::pkey::PKey;
use rcgen::generate_simple_self_signed;
use rustls::{client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier}, crypto::aws_lc_rs::sign::any_supported_type, pki_types::{CertificateDer, PrivateKeyDer}, server::ResolvesServerCert, ServerConfig};
use serde_json::{from_str, from_value, to_string, to_value, to_vec, Map, Value};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::{blockchain::Block, database::{fetch_all_blocks, get_key_pair, get_shared_key, insert_shared_key}};

const DEFAULT_PORT: i32 = 8047;

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

pub async fn initialize_p2p_thread(receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>) {

    let sender_clone = sender_to_blockchain.clone();
    let blockchain_listener = tokio::spawn(async move {
        handle_request_from_blockchain(receiver_from_blockchain, sender_clone).await;
    });

    let network_listener = tokio::spawn(async move {
        handle_request_from_network(sender_to_blockchain).await;
    });

    // Wait for threads
    if let Err(err) = tokio::try_join!(blockchain_listener, network_listener) {
        eprintln!("Error running tasks: {:?}", err);
    }

}

#[derive(Debug)]
struct AllowAnyCertVerifier;

impl ServerCertVerifier for AllowAnyCertVerifier {
    fn verify_server_cert(
            &self,
            _: &CertificateDer<'_>,
            _: &[CertificateDer<'_>],
            _: &rustls::pki_types::ServerName<'_>,
            _: &[u8],
            _: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![rustls::SignatureScheme::RSA_PKCS1_SHA1,
        rustls::SignatureScheme::ECDSA_SHA1_Legacy,
        rustls::SignatureScheme::RSA_PKCS1_SHA256,
        rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
        rustls::SignatureScheme::RSA_PKCS1_SHA384,
        rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
        rustls::SignatureScheme::RSA_PKCS1_SHA512,
        rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
        rustls::SignatureScheme::RSA_PSS_SHA256,
        rustls::SignatureScheme::RSA_PSS_SHA384,
        rustls::SignatureScheme::RSA_PSS_SHA512,
        rustls::SignatureScheme::ED25519,
        rustls::SignatureScheme::ED448]
    }
}

// Need to allow any cert to get around certificate authorities for now
impl ResolvesServerCert for AllowAnyCertVerifier {
    fn resolve(&self, _: rustls::server::ClientHello) -> Option<Arc<rustls::sign::CertifiedKey>> {
        // Create a dummy server certificate and private key
        let cert_key = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let cert_der = cert_key.cert.der().to_owned();
    
        // Return the dummy server certificate and private key as a CertifiedKey
        Some(Arc::new(rustls::sign::CertifiedKey::new(vec![cert_der], any_supported_type(&PrivateKeyDer::Pkcs8(cert_key.key_pair.serialized_der().into())).unwrap())))
    }
}

async fn handle_request_from_network(mut sender_to_blockchain: Sender<String>){
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(AllowAnyCertVerifier));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", DEFAULT_PORT)).unwrap();

    loop {
        let (stream, _) = listener.accept().unwrap();
        let conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();
        let mut tls = rustls::StreamOwned::new(conn, stream);
        // conn.complete_io(&mut stream).unwrap();
        let mut buf = [0; 32896];

        // let len =  conn.reader().read(&mut buf).unwrap();

        let len = tls.read(&mut buf).unwrap();

        let network_request_str = from_utf8(&buf[0..len]).unwrap();
        let request: P2PRequest = from_str(network_request_str).unwrap();
        let response = handle_request(request);
        let _ = tls.write_all(to_string(&response).unwrap().as_bytes());
    }
}

async fn handle_request_from_blockchain(mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>) {

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            let blockchain_request: P2PRequest = from_str(&msg).unwrap();

            match blockchain_request.action.as_str() {
                "add-provider" => add_remote_provider( blockchain_request.parameters.get("ip").unwrap().as_str().unwrap().to_string(), blockchain_request.parameters.get("chain_id").unwrap().as_str().unwrap().to_string()),
                _ => {}
            }
        }
    }
}

fn connect_to_host(ip: String) -> Option<rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream>>{
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

        let config = rustls::ClientConfig::builder()
        .dangerous().with_custom_certificate_verifier(Arc::new(AllowAnyCertVerifier))
        .with_client_auth_cert(vec![cert_clone], private_key_clone)
        .unwrap();
    
        let server_name = "localhost".try_into().unwrap();
        let conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
        let sock = TcpStream::connect(format!("{}:8047", ip)).unwrap();
        let tls = rustls::StreamOwned::new(conn, sock);
        Some(tls)
    } else {
        None
    }
}

fn request_remote(ip: String, request: P2PRequest) -> P2PResponse {
    let mut tls = connect_to_host(ip.clone()).unwrap();
    let serialized_request = to_string(&request).unwrap();

    let _ = tls.write_all(serialized_request.as_bytes());
    tls.flush().unwrap();

    let mut buf = [0; 32896];
    tls.read(&mut buf).unwrap();
    println!("Response: {}", from_utf8(&buf).unwrap());
    P2PResponse{ ok: true, data: Value::Null }
}

fn add_remote_provider(ip: String, chain_id: String) {
    let shared_key = get_shared_key(chain_id.clone()).unwrap();
    let mut parameters = Map::new();
    parameters.insert("chain_id".to_string(), to_value(chain_id.clone()).unwrap());
    parameters.insert("shared_key".to_string(), to_value(shared_key).unwrap());
    let share_key_message = P2PRequest{
        action: "add-provider".to_string(),
        parameters
    };
    let response = request_remote(ip.clone(), share_key_message);

    // Send all the blocks
    let mut parameters = Map::new();
    let blocks = fetch_all_blocks(chain_id).unwrap();
    let json_blocks = to_value(blocks).unwrap();
    parameters.insert("blocks".to_string(), json_blocks);
    let update_chain_message = P2PRequest{
        action: "update-chain".to_string(),
        parameters
    };

    let response = request_remote(ip.clone(), update_chain_message);
}



// --------- REMOTE REQUEST HANDLERS ------------ //

fn handle_request(request: P2PRequest) -> P2PResponse {
    match request.action.as_str() {
        "add-provider" => {
            add_provider_from_remote(request);
        },
        "update-chain" => {
            update_chain_from_remote(request);
            println!("UPDATE CHAIN YES");
        }
        _ => {}
    }
    P2PResponse{ ok: true, data: Value::Null }
}


fn add_provider_from_remote(request: P2PRequest){
    let chain_id = request.parameters.get("chain_id").unwrap().as_str().unwrap().to_string();
    let shared_key_value = request.parameters.get("shared_key").unwrap();

    // Convert the shared_key_value to a Vec<u8>
    let shared_key:Vec<u8> = match shared_key_value {
        Value::Array(array) => array.iter().map(|v| v.as_u64().unwrap() as u8).collect(),
        _ => panic!("shared_key field is not an array"),
    };
    insert_shared_key(&shared_key, chain_id).unwrap();
}

fn update_chain_from_remote(request: P2PRequest) {
    let json_blocks_value = request.parameters.get("blocks").unwrap();
    let blocks: Vec<Block> = from_value(json_blocks_value.clone()).unwrap();

    for block in blocks {
        println!("{:?}", block);
    }
}