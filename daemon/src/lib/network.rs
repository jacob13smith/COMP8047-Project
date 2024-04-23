use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::{Cursor, Read, Write}, net::{TcpListener, TcpStream}, os::linux::net, str::from_utf8, sync::Arc, time::Duration};
use openssl::pkey::PKey;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::{client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier}, crypto::aws_lc_rs::sign::any_supported_type, pki_types::{CertificateDer, PrivateKeyDer}, server::{danger::{ClientCertVerified, ClientCertVerifier}, ResolvesServerCert}, sign::SigningKey, RootCertStore, ServerConfig, Stream};
use serde_json::{from_str, to_string, to_value, Map, Value};
use serde::{Deserialize, Serialize};
use tokio::{io::{self, BufReader}, select};
use tokio::sync::mpsc::{Receiver, Sender};
use rsa::{traits::SignatureScheme, RsaPrivateKey};
use crate::{blockchain::BlockchainRequest, database::{get_key_pair, get_shared_key, insert_shared_key}};

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

pub async fn initialize_p2p_thread(mut receiver_from_blockchain: Receiver<String>, mut sender_to_blockchain: Sender<String>) {

    let sender_clone = sender_to_blockchain.clone();
    let blockchain_listener = tokio::spawn(async move {
        handle_request_from_blockchain(receiver_from_blockchain, sender_clone).await;
    });

    let network_listener = tokio::spawn(async move {
        handle_request_from_network(sender_to_blockchain).await;
    });

}

#[derive(Debug)]
struct AllowAnyCertVerifier;

impl ServerCertVerifier for AllowAnyCertVerifier {
    fn verify_server_cert(
            &self,
            end_entity: &CertificateDer<'_>,
            intermediates: &[CertificateDer<'_>],
            server_name: &rustls::pki_types::ServerName<'_>,
            ocsp_response: &[u8],
            now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
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
    fn resolve(&self, client_hello: rustls::server::ClientHello) -> Option<Arc<rustls::sign::CertifiedKey>> {
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
        let (mut stream, _) = listener.accept().unwrap();
        let mut conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();
        conn.complete_io(&mut stream).unwrap();
        let mut buf = [0; 65536];
        loop {
            if let Ok(io_state) = conn.process_new_packets(){
                if io_state.plaintext_bytes_to_read() > 0 {
                    let len =  conn.reader().read(&mut buf).unwrap();
                    let network_request_str = from_utf8(&buf[0..len]).unwrap();
                    let request: P2PRequest = from_str(network_request_str).unwrap();
                    
                    match request.action.as_str() {
                        "add-provider" => {
                            add_provider_from_remote(request);
                            let _ = conn.writer().write_all(to_string(&P2PResponse{ok: true, data: Value::Null}).unwrap().as_bytes());
                        },
                        _ => {}
                    }
                }

            }
        }
            
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
        let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
        let mut sock = TcpStream::connect(format!("{}:8047", ip)).unwrap();
        let mut tls = rustls::StreamOwned::new(conn, sock);
        Some(tls)
    } else {
        None
    }
}

fn add_remote_provider(ip: String, chain_id: String) {
    let mut tls = connect_to_host(ip).unwrap();
    let shared_key = get_shared_key(chain_id.clone()).unwrap();

    let mut parameters = Map::new();
    parameters.insert("chain_id".to_string(), to_value(chain_id).unwrap());
    parameters.insert("shared_key".to_string(), to_value(shared_key).unwrap());

    let network_request = P2PRequest{
        action: "add-provider".to_string(),
        parameters
    };

    let serialized_request = to_string(&network_request).unwrap();

    let _ = tls.write_all(serialized_request.as_bytes());

    let mut buf = [0; 65536];
    tls.read(&mut buf).unwrap();
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