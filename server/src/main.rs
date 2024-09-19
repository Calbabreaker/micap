#[tokio::main]
async fn main() {
    micap_server::setup_log();
    tokio::spawn(async {
        if let Err(error) = micap_server::start_server().await {
            log::error!("Server error: {error:?}");
        }
    })
    .await
    .unwrap();
}
