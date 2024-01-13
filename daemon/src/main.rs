use internal_lib::{database, socket };

#[tokio::main]
async fn main() {
    // Connect to local database
    let _ = database::bootstrap();

    // Open socket connections with frontend
    let _ = socket::initialize_socket().await;
    
    // Run forever
    tokio::signal::ctrl_c().await.unwrap();
}
