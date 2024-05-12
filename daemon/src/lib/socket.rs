use serde_json::{from_str, to_string, Map, Value};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{UnixListener, UnixStream}};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::blockchain::{BlockchainRequest, BlockchainResponse};

const UNIX_SOCKET_DOMAIN: &str = "/tmp/ehr.sock";

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketRequest {
    id: i64,
    action: String,
    parameters: Map<String, Value>,
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
        Ok((stream, _)) => {
            stream
        }
        Err(_) => todo!(),
    };

    // Spawn tasks to handle read operations concurrently (to allow push updates from blockchain later)
    let handle = tokio::spawn(handle_read_from_client(stream, receiver_from_blockchain, sender_to_blockchain, listener));

    // Wait for threads
    if let Err(err) = tokio::try_join!(handle) {
        eprintln!("Error running tasks: {:?}", err);
    }
        
}

async fn handle_read_from_client(mut stream: UnixStream, mut receiver_from_blockchain: Receiver<String>, sender_to_blockchain: Sender<String>, listener: UnixListener) {
    loop {
        let mut buffer = vec![0; 1024];

        match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                match listener.accept().await {
                    Ok((new_stream, _)) => {
                        stream = new_stream;
                        continue;
                    }
                    Err(_) => {},
                };
            }
            Ok(n) => {
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                println!("{}", received_data);
                let request: SocketRequest = from_str(&received_data).unwrap();
                let action: &str = &request.action;
                let parameters = &request.parameters;
                let response = request_blockchain(request.id, action.to_string(), parameters, &mut receiver_from_blockchain, sender_to_blockchain.clone()).await;
                let response_json = to_string(&response).unwrap();
                stream.write_all(response_json.as_bytes()).await.unwrap();
            }
            Err(_) => {}
        }
    }
}

async fn request_blockchain(request_id: i64, action: String, parameters: &Map<String, Value>, receiver_from_blockchain: &mut Receiver<String>, sender_to_blockchain: Sender<String>) -> SocketResponse {
    sender_to_blockchain.send(to_string(&BlockchainRequest{action: action, parameters: parameters.clone(), sender: "socket".to_string() }).unwrap()).await.unwrap();
    let mut response = SocketResponse{id: request_id, data: "".to_string()};

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            let blockchain_response: BlockchainResponse = from_str(&msg).unwrap();
            if blockchain_response.ok {
                response.data = blockchain_response.data.to_string();
            }
            break;
        }
    }
    response
}
