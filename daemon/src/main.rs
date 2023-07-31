use std::os::unix::net::UnixListener;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Result;

const UNIX_SOCKET_DOMAIN: &str = "/tmp/ehr.sock";


#[derive(Serialize, Deserialize)]
struct Message {
    text: String,
}

fn main() -> std::io::Result<()>{
    let _ = std::fs::remove_file(UNIX_SOCKET_DOMAIN);
    let listener = UnixListener::bind(UNIX_SOCKET_DOMAIN)?;
    println!("waiting for connection from client");
    match listener.accept() {
        Ok((mut socket, addr)) => {
            println!("Got a client: {:?} - {:?}", socket, addr);

            let data = r#"
            {
                "text": "This is a test message."
            }"#;

            let v: Message = serde_json::from_str(data)?;
            let json_string = serde_json::to_string(&v).expect("Failed to serialize to JSON");
            socket.write_all(json_string.as_bytes())?;
            let mut response = String::new();
            socket.read_to_string(&mut response)?;
            println!("{}", response);
        },
        Err(e) => println!("accept function failed: {:?}", e),
    }
    Ok(())
}
