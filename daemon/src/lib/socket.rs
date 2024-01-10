use tokio::net::{UnixStream, UnixListener};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use serde::{Deserialize, Serialize};

const UNIX_SOCKET_DOMAIN_WRITE: &str = "/tmp/ehr_0.sock";
const UNIX_SOCKET_DOMAIN_READ: &str = "/tmp/ehr_1.sock";

#[derive(Serialize, Deserialize)]
struct Message {
    text: String,
}

pub async fn initialize_socket() -> std::io::Result<()>{
    // Temp POC code for connection with frontend
    let _ = std::fs::remove_file(UNIX_SOCKET_DOMAIN_READ);
    let _ = std::fs::remove_file(UNIX_SOCKET_DOMAIN_WRITE);
    let read_listener = UnixListener::bind(UNIX_SOCKET_DOMAIN_READ)?;
    let write_listener = UnixListener::bind(UNIX_SOCKET_DOMAIN_WRITE)?;

    let read_stream = match read_listener.accept().await {
        Ok((stream, addr)) => {
            println!("Got a client for reading: {:?} - {:?}", stream, addr);
            stream
        }
        Err(e) => return Err(e),
    };

    let write_stream = match write_listener.accept().await {
        Ok((stream, addr)) => {
            println!("Got a client for writing: {:?} - {:?}", stream, addr);
            stream
        }
        Err(e) => return Err(e),
    };
    
    // Spawn tasks to handle read and write operations concurrently
    let read_handle = tokio::spawn(handle_read(read_stream));
    let write_handle = tokio::spawn(handle_write(write_stream));

    
    // Run forever
    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}

async fn handle_read(mut read_stream: UnixStream) {
    loop {
        let mut buffer = vec![0; 1024];

        match read_stream.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                // End of stream, client disconnected
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                // Process the received data (in this example, simply print it)
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                println!("Received in handle_read: {}", received_data);

                // Add specific processing logic here
            }
            Err(err) => {
                eprintln!("Error reading from client: {:?}", err);
                break;
            }
        }
    }
}

async fn handle_write(mut write_stream: UnixStream) {
    // For example, you can write a message periodically
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    let message = Message { text: "Hello from handle_write!".to_string() };
    let serialized_message = serde_json::to_string(&message).unwrap();
    write_stream.write_all(serialized_message.as_bytes()).await.expect("Failed to write data");
}