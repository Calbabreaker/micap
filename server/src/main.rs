#[tokio::main]
async fn main() {
    mycap_server::setup_log();
    if let Err(error) = mycap_server::start_server().await {
        log::error!("{error}");
    }
}
