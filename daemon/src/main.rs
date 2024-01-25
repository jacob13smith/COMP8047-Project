use internal_lib::{blockchain::create_chain, database, socket };

#[tokio::main]
async fn main() {
    // Connect to local database
    let _ = database::bootstrap();
    let _ = create_chain("Jacob Smith".to_string());

    // Open socket connections with frontend
    let _ = socket::initialize_socket().await;
    
    // Run forever
    tokio::signal::ctrl_c().await.unwrap();
}
