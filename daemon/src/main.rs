use internal_lib::blockchain::initialize_blockchain_thread;
use internal_lib::socket::initialize_socket_thread; 
use internal_lib::{database, network::initialize_p2p };
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() {

    // Connect to local database and bootstrap tables if this is first launch
    let _ = database::bootstrap();

    // Create channels for communication between threads
    let (socket_tx, socket_rx) = channel(10);
    let (blockchain_tx, blockchain_rx) = channel(10);

    // Spawn blockchain task
    let blockchain_thread = tokio::spawn(async move {
        initialize_blockchain_thread(blockchain_rx, socket_tx).await;
    });

    // Spawn socket task
    let socket_thread = tokio::spawn(async move {
        initialize_socket_thread(socket_rx, blockchain_tx).await;
    });

    initialize_p2p().await;

    // Wait for both tasks to complete
    if let Err(err) = tokio::try_join!(blockchain_thread, socket_thread) {
        eprintln!("Error running tasks: {:?}", err);
    }
    
    // Run forever
    tokio::signal::ctrl_c().await.unwrap();
}
