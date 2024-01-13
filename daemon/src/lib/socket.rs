use tokio::net::{UnixStream, UnixListener};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use serde::{Deserialize, Serialize};

use crate::database::fetch_chains;

const UNIX_SOCKET_DOMAIN: &str = "/tmp/ehr.sock";

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    id: i64,
    action: String,
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    id: i64,
    data: String,
}

pub async fn initialize_socket() -> std::io::Result<()>{
    // Temp POC code for connection with frontend
    let _ = std::fs::remove_file(UNIX_SOCKET_DOMAIN);
    let listener = UnixListener::bind(UNIX_SOCKET_DOMAIN)?;

    let stream = match listener.accept().await {
        Ok((stream, addr)) => {
            println!("Got a client: {:?} - {:?}", stream, addr);
            stream
        }
        Err(e) => return Err(e),
    };

    // Spawn tasks to handle read and write operations concurrently
    tokio::spawn(handle_read(stream));
    Ok(())
}

async fn handle_read(mut stream: UnixStream) {
    loop {
        let mut buffer = vec![0; 1024];

        match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                break;
            }
            Ok(n) => {
                // Process the received data (in this example, simply print it)
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                println!("Request received: {}", received_data);
                let request: Request = serde_json::from_str(&received_data).unwrap();
                let action: &str = &request.action;
                let response = match action {
                    "get_chains" => get_chains(request.id),
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

fn get_chains(request_id: i64) -> Response {
    let mut response = Response{id: request_id, data: "".to_string()};
    match fetch_chains() {
        Ok(chains) => {
            let chains_json_string = serde_json::to_string(&chains).unwrap();
            response.data = chains_json_string;
        },
        Err(err) => {}
    }

    return response;
}
