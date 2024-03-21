use serde_json::{json};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{UnixListener, UnixStream}};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::blockchain::{self, BlockchainRequest, BlockchainResponse};

const UNIX_SOCKET_DOMAIN: &str = "/tmp/ehr.sock";

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketRequest {
    id: i64,
    action: String,
    parameters: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketResponse {
    id: i64,
    data: String,
}

pub async fn initialize_socket_thread(receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>){
    // Temp POC code for connection with frontend
    let _ = std::fs::remove_file(UNIX_SOCKET_DOMAIN);
    let listener = UnixListener::bind(UNIX_SOCKET_DOMAIN).unwrap();

    let stream = match listener.accept().await {
        Ok((stream, addr)) => {
            println!("Got a client: {:?} - {:?}", stream, addr);
            stream
        }
        Err(_) => todo!(),
    };


    // Spawn tasks to handle read operations concurrently
    tokio::spawn(handle_read_from_client(stream, receiver_from_blockchain, sender_to_blockchain));
}

async fn handle_read_from_client(mut stream: UnixStream, mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>) {
    loop {
        let mut buffer = vec![0; 1024];

        match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                break;
            }
            Ok(n) => {
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                println!("{}", received_data);
                let request: SocketRequest = serde_json::from_str(&received_data).unwrap();
                let action: &str = &request.action;
                let parameters = &request.parameters;
                // TODO: Describe enum or strings for actions from client
                let response = match action {
                    "get_chains" => request_chains(request.id, &mut receiver_from_blockchain, sender_to_blockchain.clone()).await,
                    "create_chain" => create_chain(request.id, parameters["name"].as_str().unwrap().to_string(), &mut receiver_from_blockchain, sender_to_blockchain.clone()).await,
                    _ => SocketResponse{id: request.id, data: "".to_string()}
                };
                let response_json = serde_json::to_string(&response).unwrap();
                stream.write_all(response_json.as_bytes()).await.unwrap();
            }
            Err(err) => {
                eprintln!("Error reading from client: {:?}", err);
                break;
            }
        }
    }
}

async fn request_chains(request_id: i64, receiver_from_blockchain: &mut Receiver<String>, sender_to_blockchain: Sender<String>) -> SocketResponse {
    sender_to_blockchain.send(serde_json::to_string(&BlockchainRequest{action: "get_chains".to_string(), parameters: serde_json::Map::default()}).unwrap()).await.unwrap();
    let mut response = SocketResponse{id: request_id, data: "".to_string()};

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            let blockchain_response: BlockchainResponse = serde_json::from_str(&msg).unwrap();
            if blockchain_response.ok {
                response.data = blockchain_response.data.to_string();
            }
            break;
        }
    }

    response
}

async fn create_chain(request_id: i64, name: String, receiver_from_blockchain: &mut Receiver<String>, sender_to_blockchain: Sender<String>) -> SocketResponse {
    let mut parameters = serde_json::Map::new();
    parameters.insert("name".to_string(), serde_json::Value::String(name));
    sender_to_blockchain.send(serde_json::to_string(&BlockchainRequest{action: "create_chain".to_string(), parameters: parameters }).unwrap()).await.unwrap();
    let mut response = SocketResponse{id: request_id, data: "".to_string()};
    
    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            let blockchain_response: BlockchainResponse = serde_json::from_str(&msg).unwrap();
            if blockchain_response.ok {
                response.data = blockchain_response.data.to_string();
            }
            break;
        }
    }
    response
}
