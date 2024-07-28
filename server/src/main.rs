#[tokio::main]
async fn main() {
    micap_server::setup_log();
    if let Err(error) = micap_server::start_server().await {
        log::error!("Server error: {error:?}");
    }
}
