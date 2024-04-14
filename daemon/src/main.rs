use internal_lib::blockchain::initialize_blockchain_thread;
use internal_lib::socket::initialize_socket_thread; 
use internal_lib::{database, network::initialize_p2p_thread };
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() {

    // Connect to local database and bootstrap tables if this is first launch
    let _ = database::bootstrap();

    // Create channels for communication between threads
    let (socket_tx, socket_rx) = channel(10);
    let (blockchain_tx, blockchain_rx) = channel(10);
    let (p2p_tx, p2p_rx) = channel(10);
    let blockchain_tx_clone = blockchain_tx.clone();

    // Spawn blockchain task
    let blockchain_thread = tokio::spawn(async move {
        initialize_blockchain_thread(blockchain_rx, socket_tx, p2p_tx).await;
    });

    // Spawn socket task
    let socket_thread = tokio::spawn(async move {
        initialize_socket_thread(socket_rx, blockchain_tx).await;
    });

    let p2p_thread = tokio::spawn(async move {
        initialize_p2p_thread(p2p_rx, blockchain_tx_clone).await;
    });

    // Wait for threads
    if let Err(err) = tokio::try_join!(blockchain_thread, socket_thread, p2p_thread) {
        eprintln!("Error running tasks: {:?}", err);
    }
    
    // Run forever
    tokio::signal::ctrl_c().await.unwrap();
}
