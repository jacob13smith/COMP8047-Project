use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{UnixListener, UnixStream}};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::blockchain::BlockchainAction;

const UNIX_SOCKET_DOMAIN: &str = "/tmp/ehr.sock";

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    id: i64,
    action: String,
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
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
                let request: Request = serde_json::from_str(&received_data).unwrap();
                let action: &str = &request.action;
                // TODO: Describe enum or strings for actions from client
                let response = match action {
                    "get_chains" => request_chains(request.id, &mut receiver_from_blockchain, sender_to_blockchain.clone()).await,
                    _ => Response{id: request.id, data: "".to_string()}
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

async fn request_chains(request_id: i64, receiver_from_blockchain: &mut Receiver<String>, sender_to_blockchain: Sender<String>) -> Response {
    sender_to_blockchain.send(BlockchainAction::GetChains.val()).await.unwrap();
    let mut response = Response{id: request_id, data: "".to_string()};

    loop {
        if let Some(msg) = receiver_from_blockchain.recv().await {
            response.data = msg;
            break;
        }
    }

    return response;
}
