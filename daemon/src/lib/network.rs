use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::{Cursor, Read, Write}, net::{TcpListener, TcpStream}, sync::Arc, time::Duration};
use openssl::pkey::PKey;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::{client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier}, crypto::aws_lc_rs::sign::any_supported_type, pki_types::{CertificateDer, PrivateKeyDer}, server::{danger::{ClientCertVerified, ClientCertVerifier}, ResolvesServerCert}, sign::SigningKey, RootCertStore, ServerConfig};
use serde_json::{from_str, to_string, to_value, Map, Value};
use serde::{Deserialize, Serialize};
use tokio::{io::{self, BufReader}, select};
use tokio::sync::mpsc::{Receiver, Sender};
use rsa::{traits::SignatureScheme, RsaPrivateKey};
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

impl ResolvesServerCert for AllowAnyCertVerifier {
    fn resolve(&self, client_hello: rustls::server::ClientHello) -> Option<Arc<rustls::sign::CertifiedKey>> {
        // Create a dummy server certificate and private key
        
        let cert_key = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let cert_der = cert_key.cert.der().to_owned();
    
        // Return the dummy server certificate and private key as a CertifiedKey
        Some(Arc::new(rustls::sign::CertifiedKey::new(vec![cert_der], any_supported_type(&PrivateKeyDer::Pkcs8(cert_key.key_pair.serialized_der().into())).unwrap())))
    }
}

async fn handle_request_from_network(mut sender_to_blockchain: Sender<String>, key: PrivateKeyDer<'static>, cert: CertificateDer<'static>){
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(AllowAnyCertVerifier));

    let listener = TcpListener::bind("0.0.0.0:8047").unwrap();

    loop {
        let (mut stream, _) = listener.accept().unwrap();
        let mut conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();
        conn.complete_io(&mut stream).unwrap();
        let mut buf = [0; 64];
        let len = conn.reader().read(&mut buf).unwrap();
        println!("Received message from client: {:?}", &buf[..len]);
    }

}

async fn handle_request_from_blockchain(mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>, key: PrivateKeyDer<'static>, cert: CertificateDer<'static>) {

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            println!("Recieved message from blockchain to network");
            let blockchain_request: P2PRequest = from_str(&msg).unwrap();

            let mut root_store = RootCertStore::empty();
            let _ = root_store.add(cert.clone());
            let mut config = rustls::ClientConfig::builder()
                .dangerous().with_custom_certificate_verifier(Arc::new(AllowAnyCertVerifier))
                .with_no_client_auth();

            let server_name = "localhost".try_into().unwrap();
            let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
            let mut sock = TcpStream::connect("192.168.2.128:8047").unwrap();
            let mut tls = rustls::Stream::new(&mut conn, &mut sock);

            tls.write_all(b"Hi there!").unwrap();
        }
    }
}