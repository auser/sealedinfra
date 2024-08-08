use log::error;
use sealedinfra::exec;

#[tokio::main]
async fn main() {
    match exec().await {
        Ok(_) => (),
        Err(e) => error!("Error: {}", e),
    }
}
