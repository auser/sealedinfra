use sealed_cli::exec;
use sealed_common::error;

#[tokio::main]
async fn main() {
    match exec().await {
        Ok(_) => (),
        Err(e) => error!("Error: {}", e),
    }
}
