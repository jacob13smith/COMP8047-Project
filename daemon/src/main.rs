use std::os::unix::net::UnixListener;
use std::io::prelude::*;

const UNIX_SOCKET_DOMAIN: &str = "/tmp/ehr.sock";

fn main() -> std::io::Result<()>{
    let _ = std::fs::remove_file(UNIX_SOCKET_DOMAIN);
    let listener = UnixListener::bind(UNIX_SOCKET_DOMAIN)?;
    println!("waiting for connection from client");
    match listener.accept() {
        Ok((mut socket, addr)) => {
            println!("Got a client: {:?} - {:?}", socket, addr);
            socket.write_all(b"hello world")?;
            let mut response = String::new();
            socket.read_to_string(&mut response)?;
            println!("{}", response);
        },
        Err(e) => println!("accept function failed: {:?}", e),
    }
    Ok(())
}
