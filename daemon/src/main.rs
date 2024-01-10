use internal_lib::{database, socket};

#[tokio::main]
async fn main() {
    // Bootstrap server daemon
    // Connect to local database
    let _ = database::bootstrap();
    let _ = socket::initialize_socket().await;

}

