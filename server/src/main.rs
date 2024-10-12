#[tokio::main]
async fn main() {
    micap_server::setup_log();
    if let Err(err) = micap_server::start_server().await {
        log::error!("Server error: {err:?}");
        std::process::exit(1);
    }
}
